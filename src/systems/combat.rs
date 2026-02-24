//! 战斗系统 - 攻击、投射物、命中与伤害结算

use crate::{
    components::*,
    events::{DamageEvent, DamageSource},
    states::GameState,
};
use bevy::prelude::*;

const PLAYER_CONTACT_DAMAGE: f32 = 12.0;
const PLAYER_CONTACT_DAMAGE_COOLDOWN: f32 = 1.0;
const PROJECTILE_MUZZLE_X_OFFSET: f32 = 54.0;
const PROJECTILE_MUZZLE_Y_OFFSET: f32 = 18.0;

#[derive(Clone, Copy)]
struct ProjectileConfig {
    projectile_type: ProjectileType,
    damage: i32,
    speed: f32,
    lifetime: f32,
    collision_size: Vec2,
    core_size: Vec2,
    core_color: Color,
    aura_size: Vec2,
    aura_color: Color,
    accent_size: Vec2,
    accent_color: Color,
    aura_offset: Vec3,
    accent_offset: Vec3,
    initial_rotation: f32,
    accent_rotation: f32,
    cooldown: f32,
    pulse_speed: f32,
    pulse_amount: f32,
    spin_speed: f32,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct ProjectileVisualMotion {
    base_scale: Vec3,
    pulse_speed: f32,
    pulse_amount: f32,
    spin_speed: f32,
}

fn projectile_config(is_overedge: bool) -> ProjectileConfig {
    if is_overedge {
        ProjectileConfig {
            projectile_type: ProjectileType::Overedge,
            damage: 9,
            speed: 420.0,
            lifetime: 0.9,
            collision_size: Vec2::new(98.0, 52.0),
            core_size: Vec2::new(96.0, 22.0),
            core_color: Color::srgba(0.92, 0.20, 0.25, 0.96),
            aura_size: Vec2::new(128.0, 38.0),
            aura_color: Color::srgba(1.0, 0.30, 0.25, 0.32),
            accent_size: Vec2::new(102.0, 5.0),
            accent_color: Color::srgba(1.0, 0.9, 0.85, 0.75),
            aura_offset: Vec3::new(8.0, 0.0, -0.05),
            accent_offset: Vec3::new(0.0, 4.0, 0.05),
            initial_rotation: 0.04,
            accent_rotation: 0.03,
            cooldown: 0.55,
            pulse_speed: 12.0,
            pulse_amount: 0.05,
            spin_speed: 0.4,
        }
    } else {
        ProjectileConfig {
            projectile_type: ProjectileType::MagicWave,
            damage: 2,
            speed: 330.0,
            lifetime: 2.8,
            collision_size: Vec2::new(30.0, 24.0),
            core_size: Vec2::new(18.0, 18.0),
            core_color: Color::srgba(0.70, 0.92, 1.0, 0.98),
            aura_size: Vec2::new(34.0, 24.0),
            aura_color: Color::srgba(0.22, 0.56, 1.0, 0.34),
            accent_size: Vec2::new(20.0, 3.0),
            accent_color: Color::srgba(0.84, 0.95, 1.0, 0.8),
            aura_offset: Vec3::new(-2.0, 0.0, -0.05),
            accent_offset: Vec3::new(-12.0, 0.0, 0.05),
            initial_rotation: std::f32::consts::FRAC_PI_4,
            accent_rotation: 0.0,
            cooldown: 0.25,
            pulse_speed: 18.0,
            pulse_amount: 0.08,
            spin_speed: 3.0,
        }
    }
}

fn spawn_projectile_with_style(
    commands: &mut Commands,
    spawn_position: Vec3,
    config: ProjectileConfig,
) {
    commands
        .spawn((
            Sprite {
                color: config.core_color,
                custom_size: Some(config.core_size),
                ..default()
            },
            Transform::from_xyz(spawn_position.x, spawn_position.y, 2.0)
                .with_rotation(Quat::from_rotation_z(config.initial_rotation)),
            Projectile,
            config.projectile_type,
            ProjectileData::new(config.damage, config.speed, config.lifetime),
            Velocity {
                x: config.speed,
                y: 0.0,
            },
            crate::systems::collision::CollisionBox::new(config.collision_size),
            ProjectileVisualMotion {
                base_scale: Vec3::ONE,
                pulse_speed: config.pulse_speed,
                pulse_amount: config.pulse_amount,
                spin_speed: config.spin_speed,
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Sprite {
                    color: config.aura_color,
                    custom_size: Some(config.aura_size),
                    ..default()
                },
                Transform::from_translation(config.aura_offset),
            ));

            parent.spawn((
                Sprite {
                    color: config.accent_color,
                    custom_size: Some(config.accent_size),
                    ..default()
                },
                Transform::from_translation(config.accent_offset)
                    .with_rotation(Quat::from_rotation_z(config.accent_rotation)),
            ));
        });
}

/// 玩家发射投射物。
pub fn player_shoot_projectile(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    player_query: Query<&Transform, With<Player>>,
    mut cooldown: Local<f32>,
    time: Res<Time>,
    shroud_query: Query<&ShroudState, With<Player>>,
) {
    *cooldown -= time.delta_secs();

    if keyboard.just_pressed(KeyCode::KeyJ)
        && *cooldown <= 0.0
        && let Some(player_transform) = player_query.iter().next()
    {
        let use_overedge = shroud_query
            .iter()
            .next()
            .map(|state| state.is_released)
            .unwrap_or(false);

        let config = projectile_config(use_overedge);
        *cooldown = config.cooldown;

        let spawn_position = Vec3::new(
            player_transform.translation.x + PROJECTILE_MUZZLE_X_OFFSET,
            player_transform.translation.y + PROJECTILE_MUZZLE_Y_OFFSET,
            2.0,
        );

        spawn_projectile_with_style(&mut commands, spawn_position, config);
    }
}

/// 更新投射物移动。
pub fn update_projectiles(
    mut projectile_query: Query<(&mut Transform, &Velocity, &mut ProjectileData), With<Projectile>>,
    time: Res<Time>,
) {
    for (mut transform, velocity, mut data) in projectile_query.iter_mut() {
        transform.translation.x += velocity.x * time.delta_secs();
        transform.translation.y += velocity.y * time.delta_secs();
        data.elapsed += time.delta_secs();
    }
}

/// 投射物视觉动画：轻微脉冲 + 旋转，让法术弹更有魔术感。
pub fn animate_projectile_visuals(
    time: Res<Time>,
    mut query: Query<(&ProjectileVisualMotion, &mut Transform), With<Projectile>>,
) {
    let elapsed = time.elapsed_secs();
    let delta = time.delta_secs();

    for (motion, mut transform) in query.iter_mut() {
        let pulse = 1.0 + motion.pulse_amount * (elapsed * motion.pulse_speed).sin();
        transform.scale = motion.base_scale * pulse;
        transform.rotate_z(motion.spin_speed * delta);
    }
}

/// 清理超时投射物。
pub fn cleanup_expired_projectiles(
    mut commands: Commands,
    projectile_query: Query<(Entity, &ProjectileData), With<Projectile>>,
) {
    for (entity, data) in projectile_query.iter() {
        if data.is_expired() {
            commands.entity(entity).despawn();
        }
    }
}

/// 投射物命中敌人后只发伤害事件，统一交给伤害结算系统处理。
pub fn projectile_enemy_collision(
    mut commands: Commands,
    mut damage_writer: MessageWriter<DamageEvent>,
    projectile_query: Query<
        (
            Entity,
            &Transform,
            &ProjectileData,
            &crate::systems::collision::CollisionBox,
        ),
        With<Projectile>,
    >,
    enemy_query: Query<
        (
            Entity,
            &Transform,
            &EnemyState,
            &crate::systems::collision::CollisionBox,
        ),
        With<Enemy>,
    >,
) {
    for (projectile_entity, projectile_transform, projectile_data, projectile_box) in
        projectile_query.iter()
    {
        let mut hit_target = None;

        for (enemy_entity, enemy_transform, enemy_state, enemy_box) in enemy_query.iter() {
            if !enemy_state.is_alive {
                continue;
            }

            let dx = (projectile_transform.translation.x - enemy_transform.translation.x).abs();
            let dy = (projectile_transform.translation.y - enemy_transform.translation.y).abs();

            let collision_x = dx < (projectile_box.size.x + enemy_box.size.x) / 2.0;
            let collision_y = dy < (projectile_box.size.y + enemy_box.size.y) / 2.0;

            if collision_x && collision_y {
                hit_target = Some(enemy_entity);
                break;
            }
        }

        if let Some(enemy_entity) = hit_target {
            damage_writer.write(DamageEvent {
                target: enemy_entity,
                amount: projectile_data.damage as f32,
                source: DamageSource::Projectile,
            });
            commands.entity(projectile_entity).despawn();
        }
    }
}

/// 玩家与敌人接触伤害（带冷却），伤害通过事件统一结算。
pub fn player_enemy_collision(
    player_query: Query<
        (Entity, &Transform, &crate::systems::collision::CollisionBox),
        With<Player>,
    >,
    enemy_query: Query<
        (
            &Transform,
            &EnemyState,
            &crate::systems::collision::CollisionBox,
        ),
        With<Enemy>,
    >,
    mut last_damage_time: Local<f32>,
    time: Res<Time>,
    mut damage_writer: MessageWriter<DamageEvent>,
) {
    *last_damage_time += time.delta_secs();

    if let Some((player_entity, player_transform, player_box)) = player_query.iter().next() {
        for (enemy_transform, enemy_state, enemy_box) in enemy_query.iter() {
            if !enemy_state.is_alive {
                continue;
            }

            let dx = (player_transform.translation.x - enemy_transform.translation.x).abs();
            let dy = (player_transform.translation.y - enemy_transform.translation.y).abs();

            let collision_x = dx < (player_box.size.x + enemy_box.size.x) / 2.0;
            let collision_y = dy < (player_box.size.y + enemy_box.size.y) / 2.0;

            if collision_x && collision_y && *last_damage_time >= PLAYER_CONTACT_DAMAGE_COOLDOWN {
                damage_writer.write(DamageEvent {
                    target: player_entity,
                    amount: PLAYER_CONTACT_DAMAGE,
                    source: DamageSource::EnemyContact,
                });
                *last_damage_time = 0.0;
                break;
            }
        }
    }
}

/// 统一伤害结算管线，处理玩家和敌人的受击逻辑。
pub fn apply_damage_events(
    mut damage_events: MessageReader<DamageEvent>,
    mut player_query: Query<&mut Health, With<Player>>,
    mut enemy_query: Query<&mut EnemyState, With<Enemy>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for event in damage_events.read() {
        if let Ok(mut health) = player_query.get_mut(event.target) {
            if health.is_dead() {
                continue;
            }

            health.take_damage(event.amount);

            if health.is_dead() {
                next_state.set(GameState::GameOver);
            }

            continue;
        }

        if let Ok(mut enemy_state) = enemy_query.get_mut(event.target) {
            if !enemy_state.is_alive {
                continue;
            }

            enemy_state.take_damage(event.amount.ceil() as i32);
        }
    }
}
