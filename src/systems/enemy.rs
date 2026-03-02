//! 敵人系統

use crate::asset_paths;
use crate::components::*;
use crate::resources::GameConfig;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

const MAX_ACTIVE_ENEMIES: usize = 16;
const ENEMY_SPAWN_INTERVAL_MIN: f32 = 1.6;
const ENEMY_SPAWN_INTERVAL_MAX: f32 = 3.4;

const SLIME_RENDER_SIZE: Vec2 = Vec2::new(56.0, 44.0);
const SLIME_COLLISION_SIZE: Vec2 = Vec2::new(42.0, 30.0);
const FAMILIAR_RENDER_SIZE: Vec2 = Vec2::new(72.0, 40.0);
const FAMILIAR_COLLISION_SIZE: Vec2 = Vec2::new(48.0, 28.0);
const HEROIC_SPIRIT_RENDER_SIZE: Vec2 = Vec2::new(52.0, 110.0);
const HEROIC_SPIRIT_COLLISION_SIZE: Vec2 = Vec2::new(40.0, 78.0);

#[derive(Clone, Copy)]
struct EnemyArchetype {
    enemy_type: EnemyType,
    health: i32,
    patrol_range: f32,
    base_speed: f32,
    contact_damage: f32,
    spawn_y_offset: f32,
}

fn enemy_archetype_for_roll(roll: f32) -> EnemyArchetype {
    if roll < 0.45 {
        EnemyArchetype {
            enemy_type: EnemyType::Slime,
            health: 4,
            patrol_range: 160.0,
            base_speed: 56.0,
            contact_damage: 10.0,
            spawn_y_offset: 18.0,
        }
    } else if roll < 0.78 {
        EnemyArchetype {
            enemy_type: EnemyType::Familiar,
            health: 5,
            patrol_range: 260.0,
            base_speed: 88.0,
            contact_damage: 14.0,
            spawn_y_offset: 124.0,
        }
    } else {
        EnemyArchetype {
            enemy_type: EnemyType::EnemyHeroicSpirit,
            health: 12,
            patrol_range: 320.0,
            base_speed: 124.0,
            contact_damage: 18.0,
            spawn_y_offset: 30.0,
        }
    }
}

fn spawn_slime(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    spawn_x: f32,
    spawn_y: f32,
    enemy_state: EnemyState,
) {
    let slime_texture = asset_server.load(asset_paths::IMAGE_CLOUD_01);

    commands
        .spawn((
            Sprite {
                image: slime_texture,
                color: Color::srgba(0.45, 0.95, 0.58, 0.96),
                custom_size: Some(SLIME_RENDER_SIZE),
                ..default()
            },
            Transform::from_xyz(spawn_x, spawn_y, 1.0),
            Enemy,
            EnemyType::Slime,
            enemy_state,
            Velocity { x: 0.0, y: 0.0 },
            crate::systems::collision::CollisionBox::new(SLIME_COLLISION_SIZE),
        ))
        .with_children(|parent| {
            parent.spawn((
                Sprite {
                    color: Color::srgba(1.0, 1.0, 1.0, 0.42),
                    custom_size: Some(Vec2::new(14.0, 8.0)),
                    ..default()
                },
                Transform::from_xyz(-8.0, 10.0, 0.2),
            ));
            parent.spawn((
                Sprite {
                    color: Color::srgba(0.06, 0.16, 0.12, 0.95),
                    custom_size: Some(Vec2::new(4.0, 5.0)),
                    ..default()
                },
                Transform::from_xyz(-8.0, 3.0, 0.2),
            ));
            parent.spawn((
                Sprite {
                    color: Color::srgba(0.06, 0.16, 0.12, 0.95),
                    custom_size: Some(Vec2::new(4.0, 5.0)),
                    ..default()
                },
                Transform::from_xyz(6.0, 3.0, 0.2),
            ));
            parent.spawn((
                Sprite {
                    color: Color::srgba(0.08, 0.20, 0.16, 0.88),
                    custom_size: Some(Vec2::new(12.0, 2.5)),
                    ..default()
                },
                Transform::from_xyz(-1.0, -5.0, 0.2),
            ));
        });
}

fn spawn_familiar(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    spawn_x: f32,
    spawn_y: f32,
    enemy_state: EnemyState,
) {
    let familiar_texture = asset_server.load(asset_paths::IMAGE_CLOUD_02);

    commands
        .spawn((
            Sprite {
                image: familiar_texture,
                color: Color::srgba(0.62, 0.64, 0.97, 0.92),
                custom_size: Some(FAMILIAR_RENDER_SIZE),
                ..default()
            },
            Transform::from_xyz(spawn_x, spawn_y, 1.2),
            Enemy,
            EnemyType::Familiar,
            enemy_state,
            Velocity { x: 0.0, y: 0.0 },
            crate::systems::collision::CollisionBox::new(FAMILIAR_COLLISION_SIZE),
        ))
        .with_children(|parent| {
            parent.spawn((
                Sprite {
                    color: Color::srgba(0.90, 0.92, 1.0, 0.76),
                    custom_size: Some(Vec2::new(18.0, 10.0)),
                    ..default()
                },
                Transform::from_xyz(-16.0, 0.0, 0.2),
            ));
            parent.spawn((
                Sprite {
                    color: Color::srgba(0.90, 0.92, 1.0, 0.76),
                    custom_size: Some(Vec2::new(18.0, 10.0)),
                    ..default()
                },
                Transform::from_xyz(16.0, 0.0, 0.2),
            ));
            parent.spawn((
                Sprite {
                    color: Color::srgba(0.15, 0.12, 0.26, 0.96),
                    custom_size: Some(Vec2::new(6.0, 8.0)),
                    ..default()
                },
                Transform::from_xyz(-8.0, -2.0, 0.3),
            ));
            parent.spawn((
                Sprite {
                    color: Color::srgba(0.15, 0.12, 0.26, 0.96),
                    custom_size: Some(Vec2::new(6.0, 8.0)),
                    ..default()
                },
                Transform::from_xyz(8.0, -2.0, 0.3),
            ));
        });
}

fn spawn_enemy_heroic_spirit(
    commands: &mut Commands,
    spawn_x: f32,
    spawn_y: f32,
    enemy_state: EnemyState,
) {
    commands
        .spawn((
            Sprite {
                color: Color::srgba(0.38, 0.42, 0.52, 0.96),
                custom_size: Some(HEROIC_SPIRIT_RENDER_SIZE),
                ..default()
            },
            Transform::from_xyz(spawn_x, spawn_y, 1.4),
            Enemy,
            EnemyType::EnemyHeroicSpirit,
            enemy_state,
            Velocity { x: 0.0, y: 0.0 },
            crate::systems::collision::CollisionBox::new(HEROIC_SPIRIT_COLLISION_SIZE),
        ))
        .with_children(|parent| {
            parent.spawn((
                Sprite {
                    color: Color::srgba(0.92, 0.94, 0.99, 0.98),
                    custom_size: Some(Vec2::new(18.0, 14.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, 35.0, 0.25),
            ));
            parent.spawn((
                Sprite {
                    color: Color::srgba(0.88, 0.18, 0.24, 0.90),
                    custom_size: Some(Vec2::new(8.0, 60.0)),
                    ..default()
                },
                Transform::from_xyz(26.0, -6.0, 0.2).with_rotation(Quat::from_rotation_z(0.28)),
            ));
        });
}

/// 生成敌人（保留原系统名以兼容已有调度）
pub fn spawn_mushroom_enemies(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
    player_query: Query<&Transform, With<Player>>,
    enemy_query: Query<Entity, With<Enemy>>,
    time: Res<Time>,
    mut spawn_cooldown: Local<f32>,
    mut rng_state: Local<Option<StdRng>>,
) {
    if rng_state.is_none() {
        *rng_state = Some(StdRng::seed_from_u64(0x2026_5EED_A11E));
        *spawn_cooldown = 1.4;
    }

    *spawn_cooldown -= time.delta_secs();
    if *spawn_cooldown > 0.0 || enemy_query.iter().count() >= MAX_ACTIVE_ENEMIES {
        return;
    }

    let Some(window) = window_query.iter().next() else {
        return;
    };
    let Some(rng) = rng_state.as_mut() else {
        return;
    };

    let player_x = player_query
        .iter()
        .next()
        .map(|transform| transform.translation.x)
        .unwrap_or_default();
    let archetype = enemy_archetype_for_roll(rng.random_range(0.0..1.0));

    let spawn_x = player_x + window.width() * 0.65 + rng.random_range(90.0..260.0);
    let spawn_y = GameConfig::GROUND_LEVEL + archetype.spawn_y_offset;
    let patrol_range = archetype.patrol_range * rng.random_range(0.85..1.2);
    let hover_phase = rng.random_range(0.0..std::f32::consts::TAU);

    let enemy_state = EnemyState::new(archetype.health, patrol_range)
        .with_spawn_origin(spawn_x)
        .with_movement(archetype.base_speed, archetype.contact_damage, hover_phase);

    match archetype.enemy_type {
        EnemyType::Slime => {
            spawn_slime(&mut commands, &asset_server, spawn_x, spawn_y, enemy_state);
            crate::debug_log!("🟢 生成史莱姆敌人 at x={:.1}", spawn_x);
        }
        EnemyType::Familiar => {
            spawn_familiar(&mut commands, &asset_server, spawn_x, spawn_y, enemy_state);
            crate::debug_log!("🟣 生成使魔敌人 at x={:.1}", spawn_x);
        }
        EnemyType::EnemyHeroicSpirit => {
            spawn_enemy_heroic_spirit(&mut commands, spawn_x, spawn_y, enemy_state);
            crate::debug_log!("🔴 生成敌方英灵 at x={:.1}", spawn_x);
        }
    }

    *spawn_cooldown = rng.random_range(ENEMY_SPAWN_INTERVAL_MIN..ENEMY_SPAWN_INTERVAL_MAX);
}

/// 敵人 AI - 巡邏移動
pub fn enemy_patrol_ai(
    mut enemy_query: Query<
        (&EnemyType, &mut Transform, &mut EnemyState, &mut Velocity),
        With<Enemy>,
    >,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    time: Res<Time>,
) {
    let player_x = player_query
        .iter()
        .next()
        .map(|transform| transform.translation.x)
        .unwrap_or_default();
    let elapsed = time.elapsed_secs();
    let delta = time.delta_secs();

    for (enemy_type, mut transform, mut state, mut velocity) in enemy_query.iter_mut() {
        if !state.is_alive {
            velocity.x = 0.0;
            velocity.y = 0.0;
            continue;
        }

        match enemy_type {
            EnemyType::Slime => {
                let (patrol_left, patrol_right) = state.patrol_world_bounds();

                if state.move_direction > 0.0 && transform.translation.x > patrol_right {
                    state.move_direction = -1.0;
                } else if state.move_direction < 0.0 && transform.translation.x < patrol_left {
                    state.move_direction = 1.0;
                }

                velocity.x = state.base_speed * state.move_direction;
                velocity.y = 0.0;
                transform.translation.x += velocity.x * delta;
                transform.translation.y = GameConfig::GROUND_LEVEL + 18.0;
            }
            EnemyType::Familiar => {
                let (patrol_left, patrol_right) = state.patrol_world_bounds();
                let desired_x = (player_x + 140.0).clamp(patrol_left, patrol_right);
                let x_delta = desired_x - transform.translation.x;

                if x_delta.abs() > 8.0 {
                    state.move_direction = x_delta.signum();
                }

                velocity.x = state.base_speed * state.move_direction;
                let target_y = GameConfig::GROUND_LEVEL
                    + 124.0
                    + (elapsed * 2.7 + state.hover_phase).sin() * 30.0;
                velocity.y = ((target_y - transform.translation.y) * 4.0).clamp(-120.0, 120.0);

                transform.translation.x += velocity.x * delta;
                transform.translation.y += velocity.y * delta;
            }
            EnemyType::EnemyHeroicSpirit => {
                let player_delta = player_x - transform.translation.x;
                if player_delta.abs() > 4.0 {
                    state.move_direction = player_delta.signum();
                }

                let dash_multiplier = if player_delta.abs() > 260.0 {
                    1.35
                } else if player_delta.abs() > 120.0 {
                    1.15
                } else {
                    0.95
                };

                velocity.x = state.base_speed * dash_multiplier * state.move_direction;
                velocity.y = 0.0;
                transform.translation.x += velocity.x * delta;
                transform.translation.y = GameConfig::GROUND_LEVEL
                    + 30.0
                    + (elapsed * 8.0 + state.hover_phase).sin() * 3.0;
            }
        }
    }
}

/// 清理死亡的敵人
pub fn cleanup_dead_enemies(
    mut commands: Commands,
    enemy_query: Query<(Entity, &EnemyState), With<Enemy>>,
    mut death_timer: Local<std::collections::HashMap<Entity, f32>>,
    time: Res<Time>,
) {
    for (entity, state) in enemy_query.iter() {
        if !state.is_alive {
            // 記錄死亡時間
            let timer = death_timer.entry(entity).or_insert(0.0);
            *timer += time.delta_secs();

            // 1 秒後清理
            if *timer > 1.0 {
                commands.entity(entity).despawn();
                death_timer.remove(&entity);
                crate::debug_log!("💀 清理死亡敵人");
            }
        }
    }
}

/// 清理離屏敵人
pub fn cleanup_offscreen_enemies(
    mut commands: Commands,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
    player_query: Query<&Transform, With<Player>>,
) {
    let player_x = player_query
        .iter()
        .next()
        .map(|transform| transform.translation.x)
        .unwrap_or_default();

    for (entity, transform) in enemy_query.iter() {
        if transform.translation.x < player_x - 900.0 || transform.translation.x > player_x + 2200.0
        {
            commands.entity(entity).despawn();
            crate::debug_log!("🗑️ 清理離屏敵人");
        }
    }
}
