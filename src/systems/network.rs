use crate::protocol::{GamePacket, PlayerAction};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkRuntimeEvent {
    Connected,
    Disconnected,
    ConnectFailed,
}

#[derive(Resource)]
pub struct NetworkResource {
    pub action_tx: Option<mpsc::UnboundedSender<PlayerAction>>,
    pub packet_rx: Arc<Mutex<VecDeque<GamePacket>>>,
    pub runtime_events: Arc<Mutex<VecDeque<NetworkRuntimeEvent>>>,
    pub status: NetworkStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkStatus {
    Disconnected,
    Connecting,
    Connected,
}

#[derive(Resource, Debug, Clone)]
pub struct NetworkConfig {
    pub server_url: String,
    pub reconnect_enabled: bool,
    pub reconnect_interval_secs: f32,
    pub heartbeat_interval_secs: f32,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            server_url: "ws://127.0.0.1:8080".to_string(),
            reconnect_enabled: true,
            reconnect_interval_secs: 2.0,
            heartbeat_interval_secs: 5.0,
        }
    }
}

#[derive(Resource, Debug, Default)]
pub struct NetworkReconnectState {
    pub cooldown_remaining_secs: f32,
    pub attempt_count: u32,
}

impl Default for NetworkResource {
    fn default() -> Self {
        Self {
            action_tx: None,
            packet_rx: Arc::new(Mutex::new(VecDeque::new())),
            runtime_events: Arc::new(Mutex::new(VecDeque::new())),
            status: NetworkStatus::Disconnected,
        }
    }
}

fn push_runtime_event(
    runtime_events: &Arc<Mutex<VecDeque<NetworkRuntimeEvent>>>,
    event: NetworkRuntimeEvent,
) {
    if let Ok(mut events) = runtime_events.lock() {
        events.push_back(event);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn start_network_connection(net: &mut NetworkResource, server_url: &str) {
    if net.status != NetworkStatus::Disconnected {
        return;
    }

    info!("Connecting to server: {}", server_url);
    net.status = NetworkStatus::Connecting;

    let (action_tx, mut action_rx) = mpsc::unbounded_channel::<PlayerAction>();
    let packet_rx = net.packet_rx.clone();
    let runtime_events = net.runtime_events.clone();
    let url = server_url.to_string();
    net.action_tx = Some(action_tx);

    std::thread::spawn(move || {
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(runtime) => runtime,
            Err(error) => {
                error!("Failed to build network runtime: {}", error);
                push_runtime_event(&runtime_events, NetworkRuntimeEvent::ConnectFailed);
                push_runtime_event(&runtime_events, NetworkRuntimeEvent::Disconnected);
                return;
            }
        };

        rt.block_on(async move {
            use futures_util::{SinkExt, StreamExt};

            match tokio_tungstenite::connect_async(&url).await {
                Ok((ws_stream, _)) => {
                    info!("Connected to server: {}", url);
                    push_runtime_event(&runtime_events, NetworkRuntimeEvent::Connected);

                    let (mut write, mut read) = ws_stream.split();

                    loop {
                        tokio::select! {
                            Some(action) = action_rx.recv() => {
                                match bincode::serialize(&action) {
                                    Ok(bin) => {
                                        if let Err(error) = write.send(tokio_tungstenite::tungstenite::Message::Binary(bin)).await {
                                            warn!("Network send error: {}", error);
                                            break;
                                        }
                                    }
                                    Err(error) => {
                                        warn!("Serialize PlayerAction failed: {}", error);
                                    }
                                }
                            }
                            Some(msg) = read.next() => {
                                match msg {
                                    Ok(tokio_tungstenite::tungstenite::Message::Binary(bin)) => {
                                        if let Ok(packet) = bincode::deserialize::<GamePacket>(&bin)
                                            && let Ok(mut queue) = packet_rx.lock() {
                                                queue.push_back(packet);
                                            }
                                    }
                                    Ok(tokio_tungstenite::tungstenite::Message::Close(_)) => {
                                        info!("Server closed WebSocket connection");
                                        break;
                                    }
                                    Err(error) => {
                                        warn!("Network read error: {}", error);
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                            else => {
                                break;
                            }
                        }
                    }

                    push_runtime_event(&runtime_events, NetworkRuntimeEvent::Disconnected);
                }
                Err(error) => {
                    warn!("Failed to connect to {}: {}", url, error);
                    push_runtime_event(&runtime_events, NetworkRuntimeEvent::ConnectFailed);
                    push_runtime_event(&runtime_events, NetworkRuntimeEvent::Disconnected);
                }
            }
        });
    });
}

#[cfg(target_arch = "wasm32")]
fn start_network_connection(net: &mut NetworkResource, server_url: &str) {
    use futures_util::{FutureExt, SinkExt, StreamExt, select};
    use gloo_net::websocket::{Message, futures::WebSocket};
    use wasm_bindgen_futures::spawn_local;

    if net.status != NetworkStatus::Disconnected {
        return;
    }

    net.status = NetworkStatus::Connecting;

    let (action_tx, mut action_rx) = mpsc::unbounded_channel::<PlayerAction>();
    let packet_rx = net.packet_rx.clone();
    let runtime_events = net.runtime_events.clone();
    let url = server_url.to_string();
    net.action_tx = Some(action_tx);

    spawn_local(async move {
        match WebSocket::open(&url) {
            Ok(socket) => {
                push_runtime_event(&runtime_events, NetworkRuntimeEvent::Connected);
                let (mut write, mut read) = socket.split();

                loop {
                    let action_future = action_rx.recv().fuse();
                    let read_future = read.next().fuse();
                    futures_util::pin_mut!(action_future, read_future);

                    select! {
                        action = action_future => {
                            match action {
                                Some(action) => {
                                    if let Ok(bin) = bincode::serialize(&action) {
                                        if write.send(Message::Bytes(bin)).await.is_err() {
                                            break;
                                        }
                                    }
                                }
                                None => break,
                            }
                        }
                        msg = read_future => {
                            match msg {
                                Some(Ok(Message::Bytes(bin))) => {
                                    if let Ok(packet) = bincode::deserialize::<GamePacket>(&bin) {
                                        if let Ok(mut queue) = packet_rx.lock() {
                                            queue.push_back(packet);
                                        }
                                    }
                                }
                                Some(Ok(Message::Text(_))) => {}
                                Some(Err(_)) | None => break,
                            }
                        }
                    }
                }

                push_runtime_event(&runtime_events, NetworkRuntimeEvent::Disconnected);
            }
            Err(_) => {
                push_runtime_event(&runtime_events, NetworkRuntimeEvent::ConnectFailed);
                push_runtime_event(&runtime_events, NetworkRuntimeEvent::Disconnected);
            }
        }
    });
}

pub fn setup_network(mut net: ResMut<NetworkResource>, config: Res<NetworkConfig>) {
    start_network_connection(&mut net, &config.server_url);
}

pub fn update_network_status(mut net: ResMut<NetworkResource>) {
    let mut last_event = None;
    if let Ok(mut events) = net.runtime_events.lock() {
        while let Some(event) = events.pop_front() {
            last_event = Some(event);
        }
    }

    if let Some(event) = last_event {
        match event {
            NetworkRuntimeEvent::Connected => {
                if net.status != NetworkStatus::Connected {
                    info!("Network status: Connected");
                }
                net.status = NetworkStatus::Connected;
            }
            NetworkRuntimeEvent::Disconnected => {
                if net.status != NetworkStatus::Disconnected {
                    info!("Network status: Disconnected");
                }
                net.status = NetworkStatus::Disconnected;
                net.action_tx = None;
            }
            NetworkRuntimeEvent::ConnectFailed => {
                warn!("Network status: ConnectFailed");
                net.status = NetworkStatus::Disconnected;
                net.action_tx = None;
            }
        }
    }
}

pub fn auto_reconnect_network(
    time: Res<Time>,
    config: Res<NetworkConfig>,
    mut reconnect_state: ResMut<NetworkReconnectState>,
    mut net: ResMut<NetworkResource>,
) {
    if !config.reconnect_enabled {
        return;
    }

    match net.status {
        NetworkStatus::Connected | NetworkStatus::Connecting => {
            reconnect_state.cooldown_remaining_secs = 0.0;
        }
        NetworkStatus::Disconnected => {
            reconnect_state.cooldown_remaining_secs =
                (reconnect_state.cooldown_remaining_secs - time.delta_secs()).max(0.0);

            if reconnect_state.cooldown_remaining_secs > 0.0 {
                return;
            }

            reconnect_state.attempt_count = reconnect_state.attempt_count.wrapping_add(1);
            reconnect_state.cooldown_remaining_secs = config.reconnect_interval_secs.max(0.2);
            start_network_connection(&mut net, &config.server_url);
        }
    }
}

pub fn send_heartbeat_ping_system(
    time: Res<Time>,
    config: Res<NetworkConfig>,
    net: Res<NetworkResource>,
    mut heartbeat_timer: Local<Option<Timer>>,
    mut next_ping_id: Local<u64>,
) {
    if heartbeat_timer.is_none() {
        *heartbeat_timer = Some(Timer::from_seconds(
            config.heartbeat_interval_secs.max(0.5),
            TimerMode::Repeating,
        ));
    }

    let Some(timer) = heartbeat_timer.as_mut() else {
        return;
    };

    if net.status != NetworkStatus::Connected {
        timer.reset();
        return;
    }

    timer.tick(time.delta());
    if !timer.just_finished() {
        return;
    }

    let Some(tx) = &net.action_tx else {
        return;
    };

    *next_ping_id = next_ping_id.wrapping_add(1);
    let _ = tx.send(PlayerAction::Ping(*next_ping_id));
}

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

#[derive(SystemParam)]
pub struct NetworkEventParams<'w, 's> {
    net: ResMut<'w, NetworkResource>,
    entity_map: ResMut<'w, NetworkEntityMap>,
    my_id: ResMut<'w, MyNetworkId>,
    query: Query<
        'w,
        's,
        (
            &'static mut Transform,
            Option<&'static mut InterpolationState>,
        ),
    >,
    local_player_query: Query<
        'w,
        's,
        (Entity, &'static mut crate::components::network::NetworkId),
        With<LocalPlayer>,
    >,
    asset_server: Res<'w, AssetServer>,
    time: Res<'w, Time>,
}

pub fn handle_network_events(mut commands: Commands, mut params: NetworkEventParams) {
    let mut rx = match params.net.packet_rx.lock() {
        Ok(guard) => guard,
        Err(_) => return,
    };

    while let Some(packet) = rx.pop_front() {
        match packet {
            GamePacket::Welcome { id, message } => {
                info!("Server says: {} (My ID: {})", message, id);
                params.my_id.0 = Some(id);

                if let Ok((entity, mut net_id)) = params.local_player_query.single_mut() {
                    net_id.0 = id;
                    params.entity_map.0.insert(id, entity);
                    info!("Updated local player NetworkId to {}", id);
                }
            }
            GamePacket::WorldSnapshot { tick: _, players } => {
                let current_time = params.time.elapsed_secs();

                for player_state in players {
                    let is_local = Some(player_state.id) == params.my_id.0;

                    if let Some(&entity) = params.entity_map.0.get(&player_state.id) {
                        if let Ok((transform, interp_state)) = params.query.get_mut(entity) {
                            if let Some(mut interp) = interp_state {
                                interp.start_pos = transform.translation;
                                interp.target_pos = player_state.position;
                                interp.start_time = current_time;
                                interp.duration = 0.1;
                            } else {
                                commands.entity(entity).insert(InterpolationState {
                                    start_pos: transform.translation,
                                    target_pos: player_state.position,
                                    start_time: current_time,
                                    duration: 0.1,
                                });
                            }
                        }
                    } else if !is_local {
                        let entity = commands
                            .spawn((
                                Sprite {
                                    image: params
                                        .asset_server
                                        .load("images/characters/shirou_idle1.jpg"),
                                    ..default()
                                },
                                Transform::from_translation(player_state.position)
                                    .with_scale(Vec3::splat(0.5)),
                                crate::components::network::NetworkId(player_state.id),
                                InterpolationState {
                                    start_pos: player_state.position,
                                    target_pos: player_state.position,
                                    start_time: current_time,
                                    duration: 0.1,
                                },
                            ))
                            .id();
                        params.entity_map.0.insert(player_state.id, entity);
                    }
                }
            }
            GamePacket::Pong(id) => {
                info!("Pong from server: {}", id);
            }
            _ => {}
        }
    }
}

pub fn interpolate_positions(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut InterpolationState)>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();

    for (entity, mut transform, interp) in query.iter_mut() {
        let elapsed = current_time - interp.start_time;
        let t = (elapsed / interp.duration).min(1.0);

        transform.translation = interp.start_pos.lerp(interp.target_pos, t);

        if t >= 1.0 {
            commands.entity(entity).remove::<InterpolationState>();
        }
    }
}

pub fn send_ping_system(input: Res<ButtonInput<KeyCode>>, net: Res<NetworkResource>) {
    if net.status != NetworkStatus::Connected {
        return;
    }

    if input.just_pressed(KeyCode::KeyP)
        && let Some(tx) = &net.action_tx
    {
        let _ = tx.send(PlayerAction::Ping(0));
        info!("Sent Ping");
    }
}
