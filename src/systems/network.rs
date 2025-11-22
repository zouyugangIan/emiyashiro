use bevy::prelude::*;
use crate::protocol::{GamePacket, PlayerAction};
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use tokio::sync::mpsc;
use futures_util::{SinkExt, StreamExt};

#[derive(Resource)]
pub struct NetworkResource {
    pub action_tx: Option<mpsc::UnboundedSender<PlayerAction>>,
    pub packet_rx: Arc<Mutex<VecDeque<GamePacket>>>,
    pub status: NetworkStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkStatus {
    Disconnected,
    Connecting,
    Connected,
}

impl Default for NetworkResource {
    fn default() -> Self {
        Self {
            action_tx: None,
            packet_rx: Arc::new(Mutex::new(VecDeque::new())),
            status: NetworkStatus::Disconnected,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn setup_network(mut net: ResMut<NetworkResource>) {
    if net.status != NetworkStatus::Disconnected {
        return;
    }

    println!("Connecting to server...");
    net.status = NetworkStatus::Connecting;

    let (action_tx, mut action_rx) = mpsc::unbounded_channel::<PlayerAction>();
    let packet_rx = net.packet_rx.clone();
    net.action_tx = Some(action_tx);

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async move {
            let url = "ws://127.0.0.1:8080";
            match tokio_tungstenite::connect_async(url).await {
                Ok((ws_stream, _)) => {
                    println!("Connected to server!");
                    let (mut write, mut read) = ws_stream.split();

                    loop {
                        tokio::select! {
                            Some(action) = action_rx.recv() => {
                                let bin = bincode::serialize(&action).unwrap();
                                if let Err(e) = write.send(tokio_tungstenite::tungstenite::Message::Binary(bin)).await {
                                    println!("Send error: {}", e);
                                    break;
                                }
                            }
                            Some(msg) = read.next() => {
                                match msg {
                                    Ok(tokio_tungstenite::tungstenite::Message::Binary(bin)) => {
                                        if let Ok(packet) = bincode::deserialize::<GamePacket>(&bin) {
                                            packet_rx.lock().unwrap().push_back(packet);
                                        }
                                    }
                                    Ok(tokio_tungstenite::tungstenite::Message::Close(_)) => break,
                                    Err(e) => {
                                        println!("Read error: {}", e);
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("Failed to connect: {}", e);
                }
            }
        });
    });
    
    net.status = NetworkStatus::Connected;
}

#[cfg(target_arch = "wasm32")]
pub fn setup_network(mut net: ResMut<NetworkResource>) {
    // WASM implementation using gloo-net
}

use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct NetworkEntityMap(pub HashMap<u64, Entity>);

#[derive(Resource, Default)]
pub struct MyNetworkId(pub Option<u64>);

#[derive(Component)]
pub struct InterpolationState {
    pub start_pos: Vec3,
    pub target_pos: Vec3,
    pub start_time: f32,
    pub duration: f32,
}

#[derive(Component)]
pub struct LocalPlayer;

pub fn handle_network_events(
    mut commands: Commands,
    net: ResMut<NetworkResource>,
    mut entity_map: ResMut<NetworkEntityMap>,
    mut my_id: ResMut<MyNetworkId>,
    mut query: Query<(&mut Transform, Option<&mut InterpolationState>)>,
    mut local_player_query: Query<(Entity, &mut crate::components::network::NetworkId), With<LocalPlayer>>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
) {
    let mut rx = net.packet_rx.lock().unwrap();
    while let Some(packet) = rx.pop_front() {
        match packet {
            GamePacket::Welcome { id, message } => {
                println!("Server says: {} (My ID: {})", message, id);
                my_id.0 = Some(id);
                
                // Update local player's NetworkId if it exists
                if let Ok((entity, mut net_id)) = local_player_query.single_mut() {
                    net_id.0 = id;
                    entity_map.0.insert(id, entity);
                    println!("Updated local player NetworkId to {}", id);
                }
            }
            GamePacket::WorldSnapshot { tick: _, players } => {
                let current_time = time.elapsed_secs();
                
                for player_state in players {
                    // Check if this is the local player
                    let is_local = Some(player_state.id) == my_id.0;
                    
                    if let Some(&entity) = entity_map.0.get(&player_state.id) {
                        // Update existing entity with interpolation
                        if let Ok((transform, interp_state)) = query.get_mut(entity) {
                            if let Some(mut interp) = interp_state {
                                // Update interpolation target
                                interp.start_pos = transform.translation;
                                interp.target_pos = player_state.position;
                                interp.start_time = current_time;
                                interp.duration = 0.1; // 100ms interpolation
                            } else {
                                // No interpolation component, add it
                                commands.entity(entity).insert(InterpolationState {
                                    start_pos: transform.translation,
                                    target_pos: player_state.position,
                                    start_time: current_time,
                                    duration: 0.1,
                                });
                            }
                        }
                    } else if !is_local {
                        // Spawn new remote player entity
                        println!("Spawning remote player {}", player_state.id);
                        let entity = commands.spawn((
                            Sprite {
                                image: asset_server.load("images/characters/shirou_idle1.jpg"),
                                ..default()
                            },
                            Transform::from_translation(player_state.position).with_scale(Vec3::splat(0.5)),
                            crate::components::network::NetworkId(player_state.id),
                            InterpolationState {
                                start_pos: player_state.position,
                                target_pos: player_state.position,
                                start_time: current_time,
                                duration: 0.1,
                            },
                        )).id();
                        entity_map.0.insert(player_state.id, entity);
                    }
                    // If is_local and not in entity_map, the local player hasn't been spawned yet
                    // This should be handled by the game setup
                }
            }
            GamePacket::Pong(id) => {
                println!("Pong from server: {}", id);
            }
            _ => {}
        }
    }
}

// Interpolation system
pub fn interpolate_positions(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut InterpolationState)>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();
    
    for (entity, mut transform, interp) in query.iter_mut() {
        let elapsed = current_time - interp.start_time;
        let t = (elapsed / interp.duration).min(1.0);
        
        // Lerp position
        transform.translation = interp.start_pos.lerp(interp.target_pos, t);
        
        // Remove interpolation component when done
        if t >= 1.0 {
            commands.entity(entity).remove::<InterpolationState>();
        }
    }
}

pub fn send_ping_system(
    input: Res<ButtonInput<KeyCode>>,
    net: Res<NetworkResource>,
) {
    if input.just_pressed(KeyCode::KeyP) {
        if let Some(tx) = &net.action_tx {
            let _ = tx.send(PlayerAction::Ping(0));
            println!("Sent Ping");
        }
    }
}
