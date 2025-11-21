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

pub fn handle_network_events(
    mut commands: Commands,
    mut net: ResMut<NetworkResource>,
    mut entity_map: ResMut<NetworkEntityMap>,
    mut my_id: ResMut<MyNetworkId>,
    mut query: Query<&mut Transform>,
    asset_server: Res<AssetServer>,
) {
    let mut rx = net.packet_rx.lock().unwrap();
    while let Some(packet) = rx.pop_front() {
        match packet {
            GamePacket::Welcome { id, message } => {
                println!("Server says: {} (My ID: {})", message, id);
                my_id.0 = Some(id);
            }
            GamePacket::WorldSnapshot { tick: _, players } => {
                for player_state in players {
                    if let Some(&entity) = entity_map.0.get(&player_state.id) {
                        // Update existing entity
                        if let Ok(mut transform) = query.get_mut(entity) {
                            // Simple interpolation (lerp) could be added here
                            // For now, direct snap
                            transform.translation = player_state.position;
                        }
                    } else {
                        // Check if this is us (local player)
                        // If we already have a local player entity (spawned by game setup), we should attach NetworkId to it.
                        // But here we might not know which entity is local player easily unless we tagged it.
                        // For simplicity, let's assume we spawn new entities for everyone for now, 
                        // or we need to find the local player entity.
                        
                        // If it's us, we might want to find the existing Player entity
                        if Some(player_state.id) == my_id.0 {
                             // Find entity with Player component
                             // This requires a query. But we can't query inside loop easily if we didn't pass it.
                             // Let's just spawn a visual representation for now.
                        }

                        println!("Spawning network player {}", player_state.id);
                        let entity = commands.spawn((
                            Sprite {
                                image: asset_server.load("images/characters/shirou/idle/shirou_idle1.png"), // Placeholder path
                                ..default()
                            },
                            Transform::from_translation(player_state.position).with_scale(Vec3::splat(0.5)),
                            crate::components::network::NetworkId(player_state.id),
                        )).id();
                        entity_map.0.insert(player_state.id, entity);
                    }
                }
            }
            GamePacket::Pong(id) => {
                println!("Pong from server: {}", id);
            }
            _ => {}
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
