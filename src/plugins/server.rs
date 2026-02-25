use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use crate::components::ai::BotController;
use crate::components::network::NetworkId;
use crate::components::physics::Velocity;
use crate::components::player::{Player, PlayerInputState};
use crate::protocol::{GamePacket, InputEventKind, PlayerAction};
use crate::resources::GameConfig;
use crate::systems::ai::bot_control_system;
#[cfg(feature = "server")]
use crate::systems::sync_redis::sync_transform_to_redis;

type ActionReceiver = mpsc::UnboundedReceiver<(u64, PlayerAction)>;

/// Cross-runtime channels used by the server:
/// Tokio network tasks push input actions into ECS and receive snapshots from ECS.
#[derive(Resource, Clone)]
pub struct NetworkChannels {
    pub action_rx: Arc<Mutex<ActionReceiver>>,
    pub broadcast_tx: mpsc::UnboundedSender<GamePacket>,
}

#[derive(Resource, Default)]
pub struct ClientEntityMap(pub HashMap<u64, Entity>);

#[derive(Resource, Default)]
pub struct ServerTick(pub u64);

#[derive(Resource, Default)]
pub struct ClientInputSequence(pub HashMap<u64, u32>);

#[derive(Resource, Default)]
pub struct SnapshotStateCache {
    pub last_players: HashMap<u64, crate::protocol::PlayerState>,
    pub last_full_tick: u64,
}

#[derive(Resource, Default)]
pub struct SnapshotBandwidthMetrics {
    pub full_snapshot_bytes: u64,
    pub delta_snapshot_bytes: u64,
    pub full_snapshot_count: u64,
    pub delta_snapshot_count: u64,
}

pub struct ServerRuntimePlugin {
    pub channels: NetworkChannels,
}

impl Plugin for ServerRuntimePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.channels.clone())
            .insert_resource(Time::<Fixed>::from_hz(60.0))
            .init_resource::<ClientEntityMap>()
            .init_resource::<ServerTick>()
            .init_resource::<ClientInputSequence>()
            .init_resource::<SnapshotStateCache>()
            .init_resource::<SnapshotBandwidthMetrics>()
            .add_systems(Startup, setup_bots)
            .add_systems(
                FixedUpdate,
                (
                    increment_tick,
                    process_network_events,
                    bot_control_system,
                    server_physics_system,
                    broadcast_snapshot_system,
                )
                    .chain(),
            );

        #[cfg(feature = "server")]
        app.add_systems(FixedUpdate, sync_transform_to_redis);
    }
}

fn increment_tick(mut tick: ResMut<ServerTick>) {
    tick.0 = tick.0.wrapping_add(1);
}

fn setup_bots(mut commands: Commands) {
    commands.spawn((
        Transform::from_xyz(100.0, GameConfig::GROUND_LEVEL, 0.0),
        Velocity::zero(),
        Player,
        PlayerInputState::default(),
        BotController::default(),
        NetworkId(9999),
    ));
}

fn process_network_events(
    mut commands: Commands,
    channels: Res<NetworkChannels>,
    mut client_map: ResMut<ClientEntityMap>,
    mut input_query: Query<&mut PlayerInputState>,
    mut net_id_query: Query<&mut NetworkId>,
    mut sequence_state: ResMut<ClientInputSequence>,
) {
    let mut rx = match channels.action_rx.lock() {
        Ok(receiver) => receiver,
        Err(_) => return,
    };

    while let Ok((client_id, action)) = rx.try_recv() {
        match action {
            PlayerAction::Ping(id) => {
                let _ = channels.broadcast_tx.send(GamePacket::Pong(id));
            }
            PlayerAction::ResumeSession { previous_id } => {
                if previous_id == client_id {
                    continue;
                }

                if let Some(resumed_entity) = client_map.0.remove(&previous_id) {
                    if let Some(existing_new_entity) =
                        client_map.0.insert(client_id, resumed_entity)
                        && existing_new_entity != resumed_entity
                    {
                        commands.entity(existing_new_entity).despawn();
                    }

                    if let Ok(mut net_id) = net_id_query.get_mut(resumed_entity) {
                        net_id.0 = client_id;
                    }

                    sequence_state.0.remove(&previous_id);
                    continue;
                }
            }
            PlayerAction::InputState { sequence, x, y } => {
                if !accept_sequence(&mut sequence_state, client_id, sequence) {
                    continue;
                }

                let entity = ensure_entity(&mut commands, &mut client_map, client_id);
                if let Ok(mut input) = input_query.get_mut(entity) {
                    input.move_x = x;
                    input.move_y = y;
                }
            }
            PlayerAction::InputEvent { sequence, kind } => {
                if !accept_sequence(&mut sequence_state, client_id, sequence) {
                    continue;
                }

                let entity = ensure_entity(&mut commands, &mut client_map, client_id);
                match kind {
                    InputEventKind::Jump => {
                        if let Ok(mut input) = input_query.get_mut(entity) {
                            input.jump_pressed = true;
                        }
                    }
                    InputEventKind::Attack => {}
                }
            }
        }
    }
}

fn ensure_entity(
    commands: &mut Commands,
    client_map: &mut ClientEntityMap,
    target_client_id: u64,
) -> Entity {
    *client_map.0.entry(target_client_id).or_insert_with(|| {
        commands
            .spawn((
                Transform::from_xyz(
                    GameConfig::PLAYER_START_POS.x,
                    GameConfig::GROUND_LEVEL,
                    0.0,
                ),
                Velocity::zero(),
                Player,
                NetworkId(target_client_id),
                PlayerInputState::default(),
            ))
            .id()
    })
}

fn accept_sequence(
    sequence_state: &mut ClientInputSequence,
    target_client_id: u64,
    sequence: u32,
) -> bool {
    let entry = sequence_state.0.entry(target_client_id).or_insert(0);
    if sequence <= *entry {
        return false;
    }
    *entry = sequence;
    true
}

fn server_physics_system(
    mut query: Query<(&mut Transform, &mut Velocity, &mut PlayerInputState)>,
    time: Res<Time<Fixed>>,
) {
    let delta_seconds = time.delta_secs();

    for (mut transform, mut velocity, mut input) in query.iter_mut() {
        velocity.x = input.move_x * GameConfig::MOVE_SPEED;

        if input.jump_pressed {
            velocity.y = GameConfig::JUMP_VELOCITY;
            input.jump_pressed = false;
        }

        velocity.y -= GameConfig::GRAVITY * delta_seconds;
        transform.translation.x += velocity.x * delta_seconds;
        transform.translation.y += velocity.y * delta_seconds;

        if transform.translation.y < GameConfig::GROUND_LEVEL {
            transform.translation.y = GameConfig::GROUND_LEVEL;
            if velocity.y < 0.0 {
                velocity.y = 0.0;
            }
        }
    }
}

fn broadcast_snapshot_system(
    channels: Res<NetworkChannels>,
    tick: Res<ServerTick>,
    mut snapshot_cache: ResMut<SnapshotStateCache>,
    mut bandwidth_metrics: ResMut<SnapshotBandwidthMetrics>,
    query: Query<(&Transform, &Velocity, &NetworkId, &PlayerInputState)>,
) {
    let mut players = Vec::new();
    let mut current_map = HashMap::new();
    for (transform, velocity, net_id, input) in query.iter() {
        let animation_state = determine_animation_state(velocity, input, transform);
        let state = crate::protocol::PlayerState {
            id: net_id.0,
            position: transform.translation,
            velocity: Vec3::new(velocity.x, velocity.y, 0.0),
            facing_right: velocity.x >= 0.0,
            animation_state,
        };
        current_map.insert(net_id.0, state.clone());
        players.push(state);
    }

    if players.is_empty() {
        snapshot_cache.last_players.clear();
        return;
    }

    const FULL_SNAPSHOT_INTERVAL_TICKS: u64 = 30;
    let should_send_full = snapshot_cache.last_players.is_empty()
        || tick.0.saturating_sub(snapshot_cache.last_full_tick) >= FULL_SNAPSHOT_INTERVAL_TICKS;

    if should_send_full {
        let packet = GamePacket::WorldSnapshot {
            tick: tick.0,
            players,
        };
        if let Ok(bytes) = bincode::serde::encode_to_vec(&packet, bincode::config::standard()) {
            bandwidth_metrics.full_snapshot_bytes = bandwidth_metrics
                .full_snapshot_bytes
                .wrapping_add(bytes.len() as u64);
            bandwidth_metrics.full_snapshot_count =
                bandwidth_metrics.full_snapshot_count.wrapping_add(1);
        }
        let _ = channels.broadcast_tx.send(packet);
        snapshot_cache.last_full_tick = tick.0;
        snapshot_cache.last_players = current_map;
        return;
    }

    let mut changed_players = Vec::new();
    for (id, state) in &current_map {
        let changed = snapshot_cache
            .last_players
            .get(id)
            .map(|old| has_meaningful_delta(old, state))
            .unwrap_or(true);
        if changed {
            changed_players.push(state.clone());
        }
    }

    let removed_player_ids: Vec<u64> = snapshot_cache
        .last_players
        .keys()
        .copied()
        .filter(|id| !current_map.contains_key(id))
        .collect();

    if !changed_players.is_empty() || !removed_player_ids.is_empty() {
        let packet = GamePacket::WorldSnapshotDelta {
            tick: tick.0,
            changed_players,
            removed_player_ids,
        };
        if let Ok(bytes) = bincode::serde::encode_to_vec(&packet, bincode::config::standard()) {
            bandwidth_metrics.delta_snapshot_bytes = bandwidth_metrics
                .delta_snapshot_bytes
                .wrapping_add(bytes.len() as u64);
            bandwidth_metrics.delta_snapshot_count =
                bandwidth_metrics.delta_snapshot_count.wrapping_add(1);
        }
        let _ = channels.broadcast_tx.send(packet);
    }

    snapshot_cache.last_players = current_map;
}

fn has_meaningful_delta(
    previous: &crate::protocol::PlayerState,
    current: &crate::protocol::PlayerState,
) -> bool {
    const POSITION_EPSILON: f32 = 0.01;
    const VELOCITY_EPSILON: f32 = 0.01;

    previous.position.distance(current.position) > POSITION_EPSILON
        || previous.velocity.distance(current.velocity) > VELOCITY_EPSILON
        || previous.facing_right != current.facing_right
        || previous.animation_state != current.animation_state
}

fn determine_animation_state(
    velocity: &Velocity,
    input: &PlayerInputState,
    transform: &Transform,
) -> String {
    let is_grounded = transform.translation.y <= GameConfig::GROUND_LEVEL + 0.5;

    if !is_grounded {
        if velocity.y > 0.0 {
            "Jump".to_string()
        } else {
            "Fall".to_string()
        }
    } else if input.move_x.abs() > 0.1 {
        if input.move_y < -0.5 {
            "Crouch".to_string()
        } else {
            "Run".to_string()
        }
    } else {
        "Idle".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spawn_networked_player(app: &mut App, network_id: u64, x: f32) -> Entity {
        app.world_mut()
            .spawn((
                Transform::from_xyz(x, GameConfig::GROUND_LEVEL, 0.0),
                Velocity::zero(),
                Player,
                NetworkId(network_id),
                PlayerInputState::default(),
            ))
            .id()
    }

    #[test]
    fn resume_session_reuses_previous_entity_and_despawns_duplicate() {
        let (action_tx, action_rx) = mpsc::unbounded_channel::<(u64, PlayerAction)>();
        let (broadcast_tx, _broadcast_rx) = mpsc::unbounded_channel::<GamePacket>();
        let channels = NetworkChannels {
            action_rx: Arc::new(Mutex::new(action_rx)),
            broadcast_tx,
        };

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(channels)
            .init_resource::<ClientEntityMap>()
            .init_resource::<ClientInputSequence>()
            .add_systems(Update, process_network_events);

        let resumed_entity = spawn_networked_player(&mut app, 10, 0.0);
        let duplicate_entity = spawn_networked_player(&mut app, 20, 20.0);
        {
            let mut map = app.world_mut().resource_mut::<ClientEntityMap>();
            map.0.insert(10, resumed_entity);
            map.0.insert(20, duplicate_entity);
        }

        action_tx
            .send((20, PlayerAction::ResumeSession { previous_id: 10 }))
            .expect("resume session action should be enqueued");

        app.update();

        let map = app.world().resource::<ClientEntityMap>();
        assert_eq!(map.0.get(&20), Some(&resumed_entity));
        assert!(!map.0.contains_key(&10));
        assert!(
            app.world().get_entity(duplicate_entity).is_err(),
            "pre-existing duplicate entity should be despawned after resume remap"
        );

        let net_id = app
            .world()
            .entity(resumed_entity)
            .get::<NetworkId>()
            .expect("resumed entity must keep NetworkId component");
        assert_eq!(net_id.0, 20);
    }

    #[test]
    fn snapshot_broadcast_uses_full_then_delta_and_records_bandwidth_metrics() {
        let (_action_tx, action_rx) = mpsc::unbounded_channel::<(u64, PlayerAction)>();
        let (broadcast_tx, mut broadcast_rx) = mpsc::unbounded_channel::<GamePacket>();
        let channels = NetworkChannels {
            action_rx: Arc::new(Mutex::new(action_rx)),
            broadcast_tx,
        };

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(channels)
            .insert_resource(ServerTick(1))
            .init_resource::<SnapshotStateCache>()
            .init_resource::<SnapshotBandwidthMetrics>()
            .add_systems(Update, broadcast_snapshot_system);

        let entity = spawn_networked_player(&mut app, 1, 0.0);

        app.update();

        let first_packet = broadcast_rx
            .try_recv()
            .expect("first snapshot should be broadcast as full snapshot");
        assert!(
            matches!(first_packet, GamePacket::WorldSnapshot { .. }),
            "first packet should be full snapshot"
        );

        app.world_mut().resource_mut::<ServerTick>().0 = 2;
        if let Some(mut transform) = app.world_mut().get_mut::<Transform>(entity) {
            transform.translation.x = 50.0;
        }
        app.update();

        let second_packet = broadcast_rx
            .try_recv()
            .expect("changed snapshot should be broadcast as delta");
        match second_packet {
            GamePacket::WorldSnapshotDelta {
                changed_players,
                removed_player_ids,
                ..
            } => {
                assert_eq!(changed_players.len(), 1);
                assert!(removed_player_ids.is_empty());
            }
            _ => panic!("second packet should be delta snapshot"),
        }

        let metrics = app.world().resource::<SnapshotBandwidthMetrics>();
        assert_eq!(metrics.full_snapshot_count, 1);
        assert_eq!(metrics.delta_snapshot_count, 1);
        assert!(metrics.full_snapshot_bytes > 0);
        assert!(metrics.delta_snapshot_bytes > 0);
    }

    #[test]
    fn process_network_events_spawns_player_at_configured_ground_level() {
        let (action_tx, action_rx) = mpsc::unbounded_channel::<(u64, PlayerAction)>();
        let (broadcast_tx, _broadcast_rx) = mpsc::unbounded_channel::<GamePacket>();
        let channels = NetworkChannels {
            action_rx: Arc::new(Mutex::new(action_rx)),
            broadcast_tx,
        };

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(channels)
            .init_resource::<ClientEntityMap>()
            .init_resource::<ClientInputSequence>()
            .add_systems(Update, process_network_events);

        action_tx
            .send((
                42,
                PlayerAction::InputState {
                    sequence: 1,
                    x: 0.0,
                    y: 0.0,
                },
            ))
            .expect("input state should be enqueued");

        app.update();

        let player_entity = app
            .world()
            .resource::<ClientEntityMap>()
            .0
            .get(&42)
            .copied()
            .expect("player entity should be created");

        let transform = app
            .world()
            .entity(player_entity)
            .get::<Transform>()
            .expect("player transform should exist");
        assert!(
            (transform.translation.y - GameConfig::GROUND_LEVEL).abs() <= f32::EPSILON,
            "spawned player y should match configured ground level"
        );
        assert!(
            (transform.translation.x - GameConfig::PLAYER_START_POS.x).abs() <= f32::EPSILON,
            "spawned player x should match configured start position"
        );
    }
}
