use crate::protocol::{GamePacket, PlayerAction};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};
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

#[derive(Resource, Debug)]
pub struct NetworkLifecycleState {
    pub last_status: NetworkStatus,
    pub transition_history: Vec<NetworkStatus>,
    pub last_disconnect_time_secs: Option<f32>,
    pub reconnect_durations_secs: Vec<f32>,
}

impl Default for NetworkLifecycleState {
    fn default() -> Self {
        Self {
            last_status: NetworkStatus::Disconnected,
            transition_history: vec![NetworkStatus::Disconnected],
            last_disconnect_time_secs: None,
            reconnect_durations_secs: Vec::new(),
        }
    }
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

fn apply_status_transition(
    net: &mut NetworkResource,
    lifecycle: &mut NetworkLifecycleState,
    new_status: NetworkStatus,
    now_secs: f32,
) {
    if lifecycle.last_status == new_status {
        return;
    }

    net.status = new_status;
    lifecycle.last_status = new_status;
    lifecycle.transition_history.push(new_status);
    if lifecycle.transition_history.len() > 128 {
        let drop_count = lifecycle.transition_history.len() - 128;
        lifecycle.transition_history.drain(0..drop_count);
    }

    match new_status {
        NetworkStatus::Disconnected => {
            lifecycle.last_disconnect_time_secs = Some(now_secs);
        }
        NetworkStatus::Connected => {
            if let Some(disconnect_at) = lifecycle.last_disconnect_time_secs.take() {
                lifecycle
                    .reconnect_durations_secs
                    .push((now_secs - disconnect_at).max(0.0));
                if lifecycle.reconnect_durations_secs.len() > 64 {
                    let drop_count = lifecycle.reconnect_durations_secs.len() - 64;
                    lifecycle.reconnect_durations_secs.drain(0..drop_count);
                }
            }
        }
        NetworkStatus::Connecting => {}
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
                                match bincode::serde::encode_to_vec(&action, bincode::config::standard()) {
                                    Ok(bin) => {
                                        if let Err(error) = write.send(tokio_tungstenite::tungstenite::Message::Binary(bin.into())).await {
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
                                        if let Ok((packet, _)) = bincode::serde::decode_from_slice::<GamePacket, _>(&bin, bincode::config::standard())
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
                                    if let Ok(bin) = bincode::serde::encode_to_vec(&action, bincode::config::standard()) {
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
                                    if let Ok((packet, _)) = bincode::serde::decode_from_slice::<GamePacket, _>(&bin, bincode::config::standard()) {
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

pub fn setup_network(
    mut net: ResMut<NetworkResource>,
    config: Res<NetworkConfig>,
    mut lifecycle: ResMut<NetworkLifecycleState>,
    time: Res<Time>,
) {
    start_network_connection(&mut net, &config.server_url);
    if net.status == NetworkStatus::Connecting {
        apply_status_transition(
            &mut net,
            &mut lifecycle,
            NetworkStatus::Connecting,
            time.elapsed_secs(),
        );
    }
}

pub fn update_network_status(
    mut net: ResMut<NetworkResource>,
    mut lifecycle: ResMut<NetworkLifecycleState>,
    time: Res<Time>,
) {
    let now_secs = time.elapsed_secs();
    let runtime_events = net.runtime_events.clone();
    let mut pending_events = Vec::new();

    if let Ok(mut events) = runtime_events.lock() {
        pending_events.extend(events.drain(..));
    }

    for event in pending_events {
        match event {
            NetworkRuntimeEvent::Connected => {
                if net.status != NetworkStatus::Connected {
                    info!("Network status: Connected");
                }
                apply_status_transition(
                    &mut net,
                    &mut lifecycle,
                    NetworkStatus::Connected,
                    now_secs,
                );
            }
            NetworkRuntimeEvent::Disconnected => {
                if net.status != NetworkStatus::Disconnected {
                    info!("Network status: Disconnected");
                }
                apply_status_transition(
                    &mut net,
                    &mut lifecycle,
                    NetworkStatus::Disconnected,
                    now_secs,
                );
                net.action_tx = None;
            }
            NetworkRuntimeEvent::ConnectFailed => {
                warn!("Network status: ConnectFailed");
                apply_status_transition(
                    &mut net,
                    &mut lifecycle,
                    NetworkStatus::Disconnected,
                    now_secs,
                );
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
    mut lifecycle: ResMut<NetworkLifecycleState>,
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
            if net.status == NetworkStatus::Connecting {
                apply_status_transition(
                    &mut net,
                    &mut lifecycle,
                    NetworkStatus::Connecting,
                    time.elapsed_secs(),
                );
            }
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

#[derive(Resource, Debug, Clone)]
pub struct ClientPredictionConfig {
    pub correction_deadzone: f32,
    pub snap_threshold: f32,
    pub min_correction_secs: f32,
    pub max_correction_secs: f32,
}

impl Default for ClientPredictionConfig {
    fn default() -> Self {
        Self {
            correction_deadzone: 6.0,
            snap_threshold: 140.0,
            min_correction_secs: 0.05,
            max_correction_secs: 0.2,
        }
    }
}

#[derive(Resource, Debug, Default)]
pub struct NetworkSnapshotState {
    pub last_server_tick: u64,
}

#[derive(Component)]
pub struct InterpolationState {
    pub start_pos: Vec3,
    pub target_pos: Vec3,
    pub start_time: f32,
    pub duration: f32,
}

#[derive(Component)]
pub struct ServerCorrectionState {
    pub start_pos: Vec3,
    pub target_pos: Vec3,
    pub start_time: f32,
    pub duration: f32,
}

#[derive(Component)]
pub struct LocalPlayer;

#[derive(Debug, Clone, Copy, PartialEq)]
enum LocalCorrectionMode {
    Noop,
    Smooth(f32),
    Snap,
}

fn correction_duration(error_distance: f32, config: &ClientPredictionConfig) -> f32 {
    let normalized = (error_distance / config.snap_threshold.max(1.0)).clamp(0.0, 1.0);
    config.min_correction_secs
        + (config.max_correction_secs - config.min_correction_secs) * normalized
}

fn correction_mode(error_distance: f32, config: &ClientPredictionConfig) -> LocalCorrectionMode {
    if error_distance <= config.correction_deadzone {
        LocalCorrectionMode::Noop
    } else if error_distance >= config.snap_threshold {
        LocalCorrectionMode::Snap
    } else {
        LocalCorrectionMode::Smooth(correction_duration(error_distance, config))
    }
}

fn smoothstep(t: f32) -> f32 {
    let x = t.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

#[derive(SystemParam)]
pub struct NetworkEventParams<'w, 's> {
    net: ResMut<'w, NetworkResource>,
    entity_map: ResMut<'w, NetworkEntityMap>,
    my_id: ResMut<'w, MyNetworkId>,
    prediction_config: Res<'w, ClientPredictionConfig>,
    snapshot_state: ResMut<'w, NetworkSnapshotState>,
    remote_query: Query<
        'w,
        's,
        (
            &'static mut Transform,
            Option<&'static mut InterpolationState>,
        ),
        Without<LocalPlayer>,
    >,
    local_player_query: Query<
        'w,
        's,
        (
            Entity,
            &'static mut Transform,
            &'static mut crate::components::network::NetworkId,
            Option<&'static mut ServerCorrectionState>,
        ),
        With<LocalPlayer>,
    >,
    asset_server: Option<Res<'w, AssetServer>>,
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
                let previous_id = params.my_id.0;
                params.my_id.0 = Some(id);

                if let Some(previous_id) = previous_id
                    && previous_id != id
                    && let Some(tx) = &params.net.action_tx
                {
                    let _ = tx.send(PlayerAction::ResumeSession { previous_id });
                }

                if let Ok((entity, _, mut net_id, _)) = params.local_player_query.single_mut() {
                    net_id.0 = id;
                    params
                        .entity_map
                        .0
                        .retain(|_, mapped_entity| *mapped_entity != entity);
                    params.entity_map.0.insert(id, entity);
                    info!("Updated local player NetworkId to {}", id);
                }
            }
            GamePacket::WorldSnapshot { tick, players } => {
                if tick <= params.snapshot_state.last_server_tick {
                    continue;
                }
                params.snapshot_state.last_server_tick = tick;

                let current_time = params.time.elapsed_secs();
                let mut snapshot_ids = HashSet::new();

                for player_state in players {
                    snapshot_ids.insert(player_state.id);
                    let is_local = Some(player_state.id) == params.my_id.0;

                    if is_local {
                        if let Ok((entity, mut local_transform, mut net_id, correction_state)) =
                            params.local_player_query.single_mut()
                        {
                            net_id.0 = player_state.id;
                            params
                                .entity_map
                                .0
                                .retain(|_, mapped_entity| *mapped_entity != entity);
                            params.entity_map.0.insert(player_state.id, entity);

                            let target_position = Vec3::new(
                                player_state.position.x,
                                player_state.position.y,
                                local_transform.translation.z,
                            );
                            let error_distance =
                                local_transform.translation.distance(target_position);
                            match correction_mode(error_distance, &params.prediction_config) {
                                LocalCorrectionMode::Noop => {
                                    commands.entity(entity).remove::<ServerCorrectionState>();
                                }
                                LocalCorrectionMode::Snap => {
                                    local_transform.translation = target_position;
                                    commands.entity(entity).remove::<ServerCorrectionState>();
                                }
                                LocalCorrectionMode::Smooth(duration) => {
                                    if let Some(mut correction) = correction_state {
                                        correction.start_pos = local_transform.translation;
                                        correction.target_pos = target_position;
                                        correction.start_time = current_time;
                                        correction.duration = duration;
                                    } else {
                                        commands.entity(entity).insert(ServerCorrectionState {
                                            start_pos: local_transform.translation,
                                            target_pos: target_position,
                                            start_time: current_time,
                                            duration,
                                        });
                                    }
                                }
                            }
                        }
                        continue;
                    }

                    if let Some(entity) = params.entity_map.0.get(&player_state.id).copied() {
                        if let Ok((transform, interp_state)) = params.remote_query.get_mut(entity) {
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
                            continue;
                        }
                        params.entity_map.0.remove(&player_state.id);
                    }

                    let sprite_image = params
                        .asset_server
                        .as_ref()
                        .map(|asset_server| asset_server.load("images/characters/shirou_idle1.jpg"))
                        .unwrap_or_default();

                    let entity = commands
                        .spawn((
                            Sprite {
                                image: sprite_image,
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

                let to_remove: Vec<(u64, Entity)> = params
                    .entity_map
                    .0
                    .iter()
                    .filter_map(|(id, entity)| {
                        if Some(*id) == params.my_id.0 || snapshot_ids.contains(id) {
                            None
                        } else {
                            Some((*id, *entity))
                        }
                    })
                    .collect();

                for (removed_id, removed_entity) in to_remove {
                    params.entity_map.0.remove(&removed_id);
                    commands.entity(removed_entity).despawn();
                }
            }
            GamePacket::WorldSnapshotDelta {
                tick,
                changed_players,
                removed_player_ids,
            } => {
                if tick <= params.snapshot_state.last_server_tick {
                    continue;
                }
                params.snapshot_state.last_server_tick = tick;

                let current_time = params.time.elapsed_secs();
                for player_state in changed_players {
                    let is_local = Some(player_state.id) == params.my_id.0;

                    if is_local {
                        if let Ok((entity, mut local_transform, mut net_id, correction_state)) =
                            params.local_player_query.single_mut()
                        {
                            net_id.0 = player_state.id;
                            params
                                .entity_map
                                .0
                                .retain(|_, mapped_entity| *mapped_entity != entity);
                            params.entity_map.0.insert(player_state.id, entity);

                            let target_position = Vec3::new(
                                player_state.position.x,
                                player_state.position.y,
                                local_transform.translation.z,
                            );
                            let error_distance =
                                local_transform.translation.distance(target_position);
                            match correction_mode(error_distance, &params.prediction_config) {
                                LocalCorrectionMode::Noop => {
                                    commands.entity(entity).remove::<ServerCorrectionState>();
                                }
                                LocalCorrectionMode::Snap => {
                                    local_transform.translation = target_position;
                                    commands.entity(entity).remove::<ServerCorrectionState>();
                                }
                                LocalCorrectionMode::Smooth(duration) => {
                                    if let Some(mut correction) = correction_state {
                                        correction.start_pos = local_transform.translation;
                                        correction.target_pos = target_position;
                                        correction.start_time = current_time;
                                        correction.duration = duration;
                                    } else {
                                        commands.entity(entity).insert(ServerCorrectionState {
                                            start_pos: local_transform.translation,
                                            target_pos: target_position,
                                            start_time: current_time,
                                            duration,
                                        });
                                    }
                                }
                            }
                        }
                        continue;
                    }

                    if let Some(entity) = params.entity_map.0.get(&player_state.id).copied() {
                        if let Ok((transform, interp_state)) = params.remote_query.get_mut(entity) {
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
                            continue;
                        }
                        params.entity_map.0.remove(&player_state.id);
                    }

                    let sprite_image = params
                        .asset_server
                        .as_ref()
                        .map(|asset_server| asset_server.load("images/characters/shirou_idle1.jpg"))
                        .unwrap_or_default();

                    let entity = commands
                        .spawn((
                            Sprite {
                                image: sprite_image,
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

                for removed_id in removed_player_ids {
                    if Some(removed_id) == params.my_id.0 {
                        continue;
                    }
                    if let Some(removed_entity) = params.entity_map.0.remove(&removed_id) {
                        commands.entity(removed_entity).despawn();
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

pub fn apply_server_corrections(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &ServerCorrectionState), With<LocalPlayer>>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();

    for (entity, mut transform, correction) in query.iter_mut() {
        let elapsed = current_time - correction.start_time;
        let t = (elapsed / correction.duration).clamp(0.0, 1.0);
        transform.translation = correction
            .start_pos
            .lerp(correction.target_pos, smoothstep(t));

        if t >= 1.0 {
            commands.entity(entity).remove::<ServerCorrectionState>();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::network::NetworkId;

    fn test_player_state(id: u64, position: Vec3) -> crate::protocol::PlayerState {
        crate::protocol::PlayerState {
            id,
            position,
            velocity: Vec3::ZERO,
            facing_right: true,
            animation_state: "Idle".to_string(),
        }
    }

    fn setup_network_event_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<NetworkResource>()
            .init_resource::<NetworkEntityMap>()
            .init_resource::<MyNetworkId>()
            .init_resource::<ClientPredictionConfig>()
            .init_resource::<NetworkSnapshotState>()
            .add_systems(Update, handle_network_events);
        app
    }

    #[test]
    fn welcome_with_new_id_sends_resume_session_and_updates_local_mapping() {
        let mut app = setup_network_event_app();
        let local_entity = app
            .world_mut()
            .spawn((
                LocalPlayer,
                NetworkId(7),
                Transform::from_xyz(0.0, 0.0, 1.0),
            ))
            .id();
        app.world_mut().resource_mut::<MyNetworkId>().0 = Some(7);

        let (action_tx, mut action_rx) = tokio::sync::mpsc::unbounded_channel::<PlayerAction>();
        {
            let mut net = app.world_mut().resource_mut::<NetworkResource>();
            net.status = NetworkStatus::Connected;
            net.action_tx = Some(action_tx);
        }

        let packet_rx = app.world().resource::<NetworkResource>().packet_rx.clone();
        if let Ok(mut queue) = packet_rx.lock() {
            queue.push_back(GamePacket::Welcome {
                id: 9,
                message: "welcome".to_string(),
            });
        }

        app.update();

        let resume_action = action_rx
            .try_recv()
            .expect("welcome with changed id should trigger resume request");
        assert_eq!(
            resume_action,
            PlayerAction::ResumeSession { previous_id: 7 }
        );

        let my_id = app.world().resource::<MyNetworkId>();
        assert_eq!(my_id.0, Some(9));

        let local_net_id = app
            .world()
            .entity(local_entity)
            .get::<NetworkId>()
            .expect("local player should keep NetworkId");
        assert_eq!(local_net_id.0, 9);

        let entity_map = app.world().resource::<NetworkEntityMap>();
        assert_eq!(entity_map.0.get(&9), Some(&local_entity));
    }

    #[test]
    fn world_snapshot_delta_updates_and_removes_remote_entities() {
        let mut app = setup_network_event_app();
        app.world_mut().resource_mut::<MyNetworkId>().0 = Some(1);

        let local_entity = app
            .world_mut()
            .spawn((
                LocalPlayer,
                NetworkId(1),
                Transform::from_xyz(0.0, 0.0, 1.0),
            ))
            .id();
        let remote_entity = app
            .world_mut()
            .spawn((NetworkId(2), Transform::from_xyz(10.0, 0.0, 1.0)))
            .id();
        {
            let mut map = app.world_mut().resource_mut::<NetworkEntityMap>();
            map.0.insert(1, local_entity);
            map.0.insert(2, remote_entity);
        }

        let packet_rx = app.world().resource::<NetworkResource>().packet_rx.clone();
        if let Ok(mut queue) = packet_rx.lock() {
            queue.push_back(GamePacket::WorldSnapshot {
                tick: 1,
                players: vec![
                    test_player_state(1, Vec3::new(0.0, 0.0, 1.0)),
                    test_player_state(2, Vec3::new(10.0, 0.0, 1.0)),
                ],
            });
            queue.push_back(GamePacket::WorldSnapshotDelta {
                tick: 2,
                changed_players: vec![test_player_state(2, Vec3::new(45.0, 0.0, 1.0))],
                removed_player_ids: Vec::new(),
            });
        }

        app.update();

        let interp = app
            .world()
            .entity(remote_entity)
            .get::<InterpolationState>()
            .expect("delta update should create interpolation state on remote entity");
        assert_eq!(interp.target_pos, Vec3::new(45.0, 0.0, 1.0));

        let packet_rx = app.world().resource::<NetworkResource>().packet_rx.clone();
        if let Ok(mut queue) = packet_rx.lock() {
            queue.push_back(GamePacket::WorldSnapshotDelta {
                tick: 3,
                changed_players: Vec::new(),
                removed_player_ids: vec![2],
            });
        }

        app.update();

        let entity_map = app.world().resource::<NetworkEntityMap>();
        assert!(
            !entity_map.0.contains_key(&2),
            "delta remove should clear remote entity mapping"
        );
        assert!(
            app.world().get_entity(remote_entity).is_err(),
            "delta remove should despawn remote entity"
        );
    }

    #[test]
    fn update_network_status_tracks_runtime_event_order_and_reconnect_duration() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<NetworkResource>()
            .init_resource::<NetworkLifecycleState>()
            .add_systems(Update, update_network_status);

        let runtime_events = app
            .world()
            .resource::<NetworkResource>()
            .runtime_events
            .clone();
        if let Ok(mut queue) = runtime_events.lock() {
            queue.push_back(NetworkRuntimeEvent::Connected);
            queue.push_back(NetworkRuntimeEvent::Disconnected);
            queue.push_back(NetworkRuntimeEvent::Connected);
        }

        app.update();

        let net = app.world().resource::<NetworkResource>();
        assert_eq!(net.status, NetworkStatus::Connected);

        let lifecycle = app.world().resource::<NetworkLifecycleState>();
        assert_eq!(
            lifecycle.transition_history,
            vec![
                NetworkStatus::Disconnected,
                NetworkStatus::Connected,
                NetworkStatus::Disconnected,
                NetworkStatus::Connected
            ]
        );
        assert_eq!(
            lifecycle.reconnect_durations_secs.len(),
            1,
            "disconnect -> reconnect should record reconnect duration"
        );
    }

    #[test]
    fn correction_mode_respects_thresholds() {
        let config = ClientPredictionConfig::default();

        assert_eq!(
            correction_mode(config.correction_deadzone * 0.5, &config),
            LocalCorrectionMode::Noop
        );
        assert_eq!(
            correction_mode(config.snap_threshold + 1.0, &config),
            LocalCorrectionMode::Snap
        );
        assert!(
            matches!(
                correction_mode(config.correction_deadzone + 10.0, &config),
                LocalCorrectionMode::Smooth(_)
            ),
            "medium error should use smooth reconciliation"
        );
    }

    #[test]
    fn local_snapshot_creates_smooth_correction_for_medium_error() {
        let mut app = setup_network_event_app();
        let local_entity = app.world_mut().spawn((
            LocalPlayer,
            NetworkId(0),
            Transform::from_xyz(0.0, 0.0, 1.0),
        ));
        let local_entity = local_entity.id();
        app.world_mut().resource_mut::<MyNetworkId>().0 = Some(7);

        let snapshot = GamePacket::WorldSnapshot {
            tick: 1,
            players: vec![test_player_state(7, Vec3::new(40.0, 0.0, 0.0))],
        };

        let packet_rx = app.world().resource::<NetworkResource>().packet_rx.clone();
        if let Ok(mut queue) = packet_rx.lock() {
            queue.push_back(snapshot);
        }

        app.update();

        let entity_ref = app.world().entity(local_entity);
        let correction = entity_ref
            .get::<ServerCorrectionState>()
            .expect("local player should enter smooth correction");
        assert_eq!(correction.target_pos, Vec3::new(40.0, 0.0, 1.0));
    }

    #[test]
    fn local_snapshot_snaps_when_error_is_large() {
        let mut app = setup_network_event_app();
        let local_entity = app.world_mut().spawn((
            LocalPlayer,
            NetworkId(0),
            Transform::from_xyz(0.0, 0.0, 1.0),
        ));
        let local_entity = local_entity.id();
        app.world_mut().resource_mut::<MyNetworkId>().0 = Some(9);

        let snapshot = GamePacket::WorldSnapshot {
            tick: 1,
            players: vec![test_player_state(9, Vec3::new(1000.0, 0.0, 0.0))],
        };

        let packet_rx = app.world().resource::<NetworkResource>().packet_rx.clone();
        if let Ok(mut queue) = packet_rx.lock() {
            queue.push_back(snapshot);
        }

        app.update();

        let entity_ref = app.world().entity(local_entity);
        assert!(
            !entity_ref.contains::<ServerCorrectionState>(),
            "large error should snap directly without correction component"
        );
        let transform = entity_ref
            .get::<Transform>()
            .expect("local player transform should exist");
        assert_eq!(transform.translation, Vec3::new(1000.0, 0.0, 1.0));
    }

    #[test]
    fn apply_server_corrections_completes_and_removes_component() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Update, apply_server_corrections);
        let entity = app.world_mut().spawn((
            LocalPlayer,
            Transform::from_xyz(0.0, 0.0, 1.0),
            ServerCorrectionState {
                start_pos: Vec3::new(0.0, 0.0, 1.0),
                target_pos: Vec3::new(50.0, 0.0, 1.0),
                start_time: -1.0,
                duration: 0.2,
            },
        ));
        let entity = entity.id();

        app.update();

        let entity_ref = app.world().entity(entity);
        assert!(
            !entity_ref.contains::<ServerCorrectionState>(),
            "completed correction should clean up component"
        );
        let transform = entity_ref
            .get::<Transform>()
            .expect("entity should still have transform");
        assert_eq!(transform.translation, Vec3::new(50.0, 0.0, 1.0));
    }
}
