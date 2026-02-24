use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use crate::components::ai::BotController;
use crate::components::network::NetworkId;
use crate::components::physics::Velocity;
use crate::components::player::{Player, PlayerInputState};
use crate::protocol::{GamePacket, PlayerAction};
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

pub struct ServerRuntimePlugin {
    pub channels: NetworkChannels,
}

impl Plugin for ServerRuntimePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.channels.clone())
            .insert_resource(Time::<Fixed>::from_hz(60.0))
            .init_resource::<ClientEntityMap>()
            .init_resource::<ServerTick>()
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
        Transform::from_xyz(100.0, 0.0, 0.0),
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
    mut query: Query<&mut PlayerInputState>,
) {
    let mut rx = match channels.action_rx.lock() {
        Ok(receiver) => receiver,
        Err(_) => return,
    };

    while let Ok((client_id, action)) = rx.try_recv() {
        let entity = *client_map.0.entry(client_id).or_insert_with(|| {
            commands
                .spawn((
                    Transform::from_xyz(0.0, 0.0, 0.0),
                    Velocity::zero(),
                    Player,
                    NetworkId(client_id),
                    PlayerInputState::default(),
                ))
                .id()
        });

        match action {
            PlayerAction::Ping(id) => {
                let _ = channels.broadcast_tx.send(GamePacket::Pong(id));
            }
            PlayerAction::Move { x, y } => {
                if let Ok(mut input) = query.get_mut(entity) {
                    input.move_x = x;
                    input.move_y = y;
                }
            }
            PlayerAction::Jump => {
                if let Ok(mut input) = query.get_mut(entity) {
                    input.jump_pressed = true;
                }
            }
            PlayerAction::Attack => {}
        }
    }
}

fn server_physics_system(
    mut query: Query<(&mut Transform, &mut Velocity, &mut PlayerInputState)>,
    time: Res<Time<Fixed>>,
) {
    let delta_seconds = time.delta_secs();

    for (mut transform, mut velocity, mut input) in query.iter_mut() {
        velocity.x = input.move_x * 200.0;

        if input.jump_pressed {
            velocity.y = 500.0;
            input.jump_pressed = false;
        }

        velocity.y -= 980.0 * delta_seconds;
        transform.translation.x += velocity.x * delta_seconds;
        transform.translation.y += velocity.y * delta_seconds;

        if transform.translation.y < 0.0 {
            transform.translation.y = 0.0;
            if velocity.y < 0.0 {
                velocity.y = 0.0;
            }
        }
    }
}

fn broadcast_snapshot_system(
    channels: Res<NetworkChannels>,
    tick: Res<ServerTick>,
    query: Query<(&Transform, &Velocity, &NetworkId, &PlayerInputState)>,
) {
    let mut players = Vec::new();
    for (transform, velocity, net_id, input) in query.iter() {
        let animation_state = determine_animation_state(velocity, input, transform);
        players.push(crate::protocol::PlayerState {
            id: net_id.0,
            position: transform.translation,
            velocity: Vec3::new(velocity.x, velocity.y, 0.0),
            facing_right: velocity.x >= 0.0,
            animation_state,
        });
    }

    if !players.is_empty() {
        let snapshot = GamePacket::WorldSnapshot {
            tick: tick.0,
            players,
        };
        let _ = channels.broadcast_tx.send(snapshot);
    }
}

fn determine_animation_state(
    velocity: &Velocity,
    input: &PlayerInputState,
    transform: &Transform,
) -> String {
    let is_grounded = transform.translation.y <= 0.5;

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
