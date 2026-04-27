//! 敌人系统

use crate::asset_paths;
use crate::components::*;
use crate::resources::{EnemyArchetypeTuning, EnemyDirectorTuning, GameConfig, GameplayTuning};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

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

fn enemy_base_color(enemy_type: EnemyType) -> Color {
    match enemy_type {
        EnemyType::Slime => Color::srgba(0.45, 0.95, 0.58, 0.96),
        EnemyType::Familiar => Color::srgba(0.62, 0.64, 0.97, 0.92),
        EnemyType::EnemyHeroicSpirit => Color::srgba(0.38, 0.42, 0.52, 0.96),
    }
}

fn enemy_type_for_roll(roll: f32, tuning: &EnemyDirectorTuning) -> EnemyType {
    let slime = tuning.spawn_weights.slime.max(0.0);
    let familiar = tuning.spawn_weights.familiar.max(0.0);
    let heroic = tuning.spawn_weights.heroic_spirit.max(0.0);
    let sum = (slime + familiar + heroic).max(f32::EPSILON);

    let slime_threshold = slime / sum;
    let familiar_threshold = slime_threshold + familiar / sum;

    if roll < slime_threshold {
        EnemyType::Slime
    } else if roll < familiar_threshold {
        EnemyType::Familiar
    } else {
        EnemyType::EnemyHeroicSpirit
    }
}

fn enemy_archetype(enemy_type: EnemyType, tuning: &EnemyDirectorTuning) -> EnemyArchetype {
    let profile: &EnemyArchetypeTuning = match enemy_type {
        EnemyType::Slime => &tuning.slime,
        EnemyType::Familiar => &tuning.familiar,
        EnemyType::EnemyHeroicSpirit => &tuning.heroic_spirit,
    };

    EnemyArchetype {
        enemy_type,
        health: profile.health.max(1),
        patrol_range: profile.patrol_range.max(40.0),
        base_speed: profile.base_speed.max(10.0),
        contact_damage: profile.contact_damage.max(1.0),
        spawn_y_offset: profile.spawn_y_offset,
    }
}

fn lerp_color(base: Color, target: Color, t: f32, alpha: f32) -> Color {
    let base = base.to_srgba();
    let target = target.to_srgba();
    let ratio = t.clamp(0.0, 1.0);

    Color::srgba(
        base.red + (target.red - base.red) * ratio,
        base.green + (target.green - base.green) * ratio,
        base.blue + (target.blue - base.blue) * ratio,
        alpha.clamp(0.0, 1.0),
    )
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
                color: enemy_base_color(EnemyType::Slime),
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
                color: enemy_base_color(EnemyType::Familiar),
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
                color: enemy_base_color(EnemyType::EnemyHeroicSpirit),
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
#[allow(clippy::too_many_arguments)]
pub fn spawn_mushroom_enemies(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
    player_query: Query<&Transform, With<Player>>,
    enemy_query: Query<Entity, With<Enemy>>,
    tuning: Option<Res<GameplayTuning>>,
    time: Res<Time>,
    mut spawn_cooldown: Local<f32>,
    mut rng_state: Local<Option<StdRng>>,
) {
    let default_tuning = GameplayTuning::default();
    let enemy_tuning = &tuning.as_deref().unwrap_or(&default_tuning).enemies;

    if rng_state.is_none() {
        *rng_state = Some(StdRng::seed_from_u64(0x2026_5EED_A11E));
        *spawn_cooldown = enemy_tuning.spawn_interval_min_secs.max(0.2);
    }

    *spawn_cooldown -= time.delta_secs();
    if *spawn_cooldown > 0.0 || enemy_query.iter().count() >= enemy_tuning.max_active_enemies {
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
    let enemy_type = enemy_type_for_roll(rng.random_range(0.0..1.0), enemy_tuning);
    let archetype = enemy_archetype(enemy_type, enemy_tuning);

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

    let min_spawn_interval = enemy_tuning.spawn_interval_min_secs.max(0.15);
    let max_spawn_interval = enemy_tuning
        .spawn_interval_max_secs
        .max(min_spawn_interval + 0.05);
    *spawn_cooldown = rng.random_range(min_spawn_interval..max_spawn_interval);
}

/// 敌人 AI - 巡逻与战斗行为。
pub fn enemy_patrol_ai(
    mut enemy_query: Query<
        (&EnemyType, &mut Transform, &mut EnemyState, &mut Velocity),
        With<Enemy>,
    >,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    tuning: Option<Res<GameplayTuning>>,
    time: Res<Time>,
) {
    let default_tuning = GameplayTuning::default();
    let enemy_tuning = &tuning.as_deref().unwrap_or(&default_tuning).enemies;
    let slime_behavior = &enemy_tuning.slime_behavior;
    let heroic_behavior = &enemy_tuning.heroic_spirit_behavior;

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

        let was_charging = state.dash_charge_timer > 0.0;
        state.tick_timers(delta);

        if was_charging && state.dash_charge_timer <= f32::EPSILON {
            state.dash_active_timer = heroic_behavior.dash_active_secs.max(0.08);
            velocity.x = state.base_speed
                * heroic_behavior.dash_speed_multiplier.max(1.2)
                * state.dash_direction;
            state.attack_cooldown = heroic_behavior.dash_cooldown_secs.max(0.6);
        }

        if state.hit_stun_timer > 0.0 {
            transform.translation.x += velocity.x * delta;
            transform.translation.y += velocity.y * delta;
            velocity.x *= 0.84;
            velocity.y *= 0.80;
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

                let player_delta = player_x - transform.translation.x;
                if player_delta.abs() < slime_behavior.engage_distance.max(1.0)
                    && state.attack_cooldown <= 0.0
                {
                    state.move_direction = player_delta.signum();
                    velocity.x = state.base_speed
                        * slime_behavior.burst_speed_multiplier
                        * state.move_direction;
                    state.attack_cooldown = slime_behavior.burst_cooldown_secs.max(0.1);
                } else {
                    velocity.x = state.base_speed * state.move_direction;
                }

                transform.translation.x += velocity.x * delta;
                transform.translation.y =
                    GameConfig::GROUND_LEVEL + enemy_tuning.slime.spawn_y_offset;
            }
            EnemyType::Familiar => {
                let (patrol_left, patrol_right) = state.patrol_world_bounds();
                let desired_x = (player_x + 140.0).clamp(patrol_left, patrol_right);
                let x_delta = desired_x - transform.translation.x;

                if x_delta.abs() > 8.0 {
                    state.move_direction = x_delta.signum();
                }

                let speed_factor = if x_delta.abs() > 120.0 { 1.2 } else { 0.8 };
                velocity.x = state.base_speed * speed_factor * state.move_direction;
                if state.pending_ranged_shot {
                    velocity.x *= 0.22;
                }

                let target_y = GameConfig::GROUND_LEVEL
                    + enemy_tuning.familiar.spawn_y_offset
                    + (elapsed * 2.7 + state.hover_phase).sin() * 30.0;
                velocity.y = ((target_y - transform.translation.y) * 4.2).clamp(-140.0, 140.0);
                if state.pending_ranged_shot {
                    velocity.y *= 0.55;
                }

                transform.translation.x += velocity.x * delta;
                transform.translation.y += velocity.y * delta;
            }
            EnemyType::EnemyHeroicSpirit => {
                let player_delta = player_x - transform.translation.x;
                let base_y = GameConfig::GROUND_LEVEL
                    + enemy_tuning.heroic_spirit.spawn_y_offset
                    + (elapsed * 8.0 + state.hover_phase).sin() * 3.0;

                if state.dash_charge_timer > 0.0 {
                    velocity.x = 0.0;
                    transform.translation.y = base_y + (elapsed * 45.0).sin() * 2.5;
                    continue;
                }

                if state.dash_active_timer > 0.0 {
                    transform.translation.x += velocity.x * delta;
                    transform.translation.y = base_y;
                    continue;
                }

                if player_delta.abs() > 4.0 {
                    state.move_direction = player_delta.signum();
                }

                let chase_speed = if player_delta.abs() > 240.0 {
                    state.base_speed * 1.2
                } else {
                    state.base_speed * 0.92
                };
                velocity.x = chase_speed * state.move_direction;

                transform.translation.x += velocity.x * delta;
                transform.translation.y = base_y;

                if state.attack_cooldown <= 0.0
                    && player_delta.abs() > heroic_behavior.dash_trigger_distance.max(1.0)
                {
                    state.dash_direction = player_delta.signum();
                    state.dash_charge_timer = heroic_behavior.dash_charge_secs.max(0.05);
                    state.attack_cooldown = heroic_behavior.dash_cooldown_secs.max(0.6);
                }
            }
        }
    }
}

/// 使魔的远程飞弹攻击。
pub fn enemy_ranged_attack(
    mut commands: Commands,
    player_query: Query<&Transform, (With<Player>, Without<Enemy>)>,
    tuning: Option<Res<GameplayTuning>>,
    mut enemy_query: Query<(&EnemyType, &Transform, &mut EnemyState), With<Enemy>>,
) {
    let default_tuning = GameplayTuning::default();
    let familiar_tuning = &tuning
        .as_deref()
        .unwrap_or(&default_tuning)
        .enemies
        .familiar_behavior;

    let Some(player_transform) = player_query.iter().next() else {
        return;
    };

    for (enemy_type, enemy_transform, mut enemy_state) in enemy_query.iter_mut() {
        if *enemy_type != EnemyType::Familiar || !enemy_state.is_alive {
            continue;
        }

        if enemy_state.hit_stun_timer > 0.0 {
            enemy_state.pending_ranged_shot = false;
            enemy_state.ranged_windup_timer = 0.0;
            continue;
        }

        let to_player = player_transform.translation - enemy_transform.translation;
        let distance = to_player.length();
        let direction_to_player = to_player.truncate().normalize_or_zero();

        if enemy_state.pending_ranged_shot {
            if enemy_state.ranged_windup_timer > 0.0 {
                continue;
            }

            let fire_direction = if enemy_state.ranged_shot_direction == Vec2::ZERO {
                direction_to_player
            } else {
                enemy_state.ranged_shot_direction.normalize_or_zero()
            };

            if fire_direction == Vec2::ZERO {
                enemy_state.pending_ranged_shot = false;
                enemy_state.ranged_shot_direction = Vec2::ZERO;
                continue;
            }

            let spawn_position = enemy_transform.translation
                + Vec3::new(fire_direction.x * 24.0, fire_direction.y * 18.0, 0.0);
            crate::systems::combat::spawn_enemy_projectile(
                &mut commands,
                spawn_position,
                fire_direction,
                familiar_tuning.projectile_speed.max(20.0),
                enemy_state.contact_damage * 0.85,
                familiar_tuning.projectile_lifetime_secs.max(0.2),
            );

            enemy_state.pending_ranged_shot = false;
            enemy_state.ranged_shot_direction = Vec2::ZERO;
            enemy_state.ranged_cooldown = familiar_tuning.attack_cooldown_secs.max(0.2);
            enemy_state.attack_cooldown = enemy_state.attack_cooldown.max(0.24);
            continue;
        }

        if enemy_state.ranged_cooldown > 0.0 {
            continue;
        }

        let min_distance = familiar_tuning.attack_min_distance.max(0.0);
        let max_distance = familiar_tuning.attack_max_distance.max(min_distance + 1.0);
        if !(min_distance..=max_distance).contains(&distance) {
            continue;
        }

        let direction = direction_to_player;
        if direction == Vec2::ZERO {
            continue;
        }

        enemy_state.pending_ranged_shot = true;
        enemy_state.ranged_shot_direction = direction;
        enemy_state.ranged_windup_timer = familiar_tuning.cast_windup_secs.max(0.01);
        enemy_state.attack_cooldown = enemy_state.attack_cooldown.max(0.32);
    }
}

/// 敌人攻击预警视觉，提升读招公平性。
pub fn update_enemy_telegraph_visuals(
    mut enemy_query: Query<(&EnemyType, &EnemyState, &mut Sprite, &mut Transform), With<Enemy>>,
    tuning: Option<Res<GameplayTuning>>,
    time: Res<Time>,
) {
    let default_tuning = GameplayTuning::default();
    let enemy_tuning = &tuning.as_deref().unwrap_or(&default_tuning).enemies;
    let elapsed = time.elapsed_secs();

    for (enemy_type, enemy_state, mut sprite, mut transform) in enemy_query.iter_mut() {
        let base_color = enemy_base_color(*enemy_type);
        transform.scale = Vec3::ONE;

        if !enemy_state.is_alive {
            sprite.color = lerp_color(base_color, Color::BLACK, 0.55, 0.42);
            continue;
        }

        match enemy_type {
            EnemyType::Slime => {
                let cooldown = enemy_tuning.slime_behavior.burst_cooldown_secs.max(0.1);
                let warning_window = (cooldown * 0.22).clamp(0.08, 0.35);
                if enemy_state.attack_cooldown > 0.0
                    && enemy_state.attack_cooldown <= warning_window
                {
                    let progress = 1.0 - enemy_state.attack_cooldown / warning_window;
                    let pulse = (elapsed * 20.0).sin() * 0.5 + 0.5;
                    let blend = (progress * 0.68 + pulse * 0.32).clamp(0.0, 1.0);
                    sprite.color =
                        lerp_color(base_color, Color::srgba(0.90, 1.0, 0.92, 1.0), blend, 0.98);
                    transform.scale = Vec3::splat(1.0 + 0.06 * blend);
                } else {
                    sprite.color = base_color;
                }
            }
            EnemyType::Familiar => {
                if enemy_state.pending_ranged_shot {
                    let windup = enemy_tuning.familiar_behavior.cast_windup_secs.max(0.01);
                    let progress = 1.0 - (enemy_state.ranged_windup_timer / windup).clamp(0.0, 1.0);
                    let pulse = (elapsed * 24.0).sin() * 0.5 + 0.5;
                    let blend = (progress * 0.65 + pulse * 0.35).clamp(0.0, 1.0);
                    sprite.color =
                        lerp_color(base_color, Color::srgba(0.92, 0.84, 1.0, 1.0), blend, 0.98);
                    transform.scale = Vec3::splat(1.0 + 0.08 * blend);
                } else {
                    let warning_window = enemy_tuning
                        .familiar_behavior
                        .telegraph_window_secs
                        .max(0.05);
                    if enemy_state.ranged_cooldown > 0.0
                        && enemy_state.ranged_cooldown <= warning_window
                    {
                        let progress = 1.0 - enemy_state.ranged_cooldown / warning_window;
                        let pulse = (elapsed * 16.0).sin() * 0.5 + 0.5;
                        let blend = (progress * 0.52 + pulse * 0.48).clamp(0.0, 1.0);
                        sprite.color =
                            lerp_color(base_color, Color::srgba(0.86, 0.88, 1.0, 1.0), blend, 0.95);
                    } else {
                        sprite.color = base_color;
                    }
                }
            }
            EnemyType::EnemyHeroicSpirit => {
                if enemy_state.dash_charge_timer > 0.0 {
                    let charge = enemy_tuning
                        .heroic_spirit_behavior
                        .dash_charge_secs
                        .max(0.05);
                    let progress = 1.0 - (enemy_state.dash_charge_timer / charge).clamp(0.0, 1.0);
                    let pulse = (elapsed * 34.0).sin() * 0.5 + 0.5;
                    let blend = (progress * 0.7 + pulse * 0.3).clamp(0.0, 1.0);
                    sprite.color =
                        lerp_color(base_color, Color::srgba(1.0, 0.34, 0.27, 1.0), blend, 0.99);
                    transform.scale = Vec3::splat(1.0 + 0.11 * blend);
                } else if enemy_state.dash_active_timer > 0.0 {
                    let active = enemy_tuning
                        .heroic_spirit_behavior
                        .dash_active_secs
                        .max(0.05);
                    let progress = (enemy_state.dash_active_timer / active).clamp(0.0, 1.0);
                    let blend = progress.powf(0.45);
                    sprite.color =
                        lerp_color(base_color, Color::srgba(1.0, 0.50, 0.4, 1.0), blend, 0.98);
                    transform.scale = Vec3::splat(1.0 + 0.05 * blend);
                } else {
                    sprite.color = base_color;
                }
            }
        }
    }
}

/// 清理死亡的敌人
pub fn cleanup_dead_enemies(
    mut commands: Commands,
    enemy_query: Query<(Entity, &EnemyState), With<Enemy>>,
    mut death_timer: Local<std::collections::HashMap<Entity, f32>>,
    time: Res<Time>,
) {
    for (entity, state) in enemy_query.iter() {
        if !state.is_alive {
            // 记录死亡时间
            let timer = death_timer.entry(entity).or_insert(0.0);
            *timer += time.delta_secs();

            // 1 秒后清理
            if *timer > 1.0 {
                commands.entity(entity).despawn();
                death_timer.remove(&entity);
                crate::debug_log!("💀 清理死亡敌人");
            }
        }
    }
}

/// 清理离屏敌人
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
            crate::debug_log!("🗑️ 清理离屏敌人");
        }
    }
}
