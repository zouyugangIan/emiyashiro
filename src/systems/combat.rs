//! 战斗系统 - 攻击、投射物、命中与伤害结算

use crate::{
    components::*,
    events::{DamageEvent, DamageSource},
    states::GameState,
};
use bevy::prelude::*;

const PLAYER_CONTACT_DAMAGE: f32 = 12.0;
const PLAYER_CONTACT_DAMAGE_COOLDOWN: f32 = 1.0;

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
        let is_shroud_released = shroud_query
            .iter()
            .next()
            .map(|state| state.is_released)
            .unwrap_or(false);

        let (projectile_type, damage, speed, lifetime, size, color, cd) = if is_shroud_released {
            (
                ProjectileType::Overedge,
                10,
                400.0,
                0.5,
                Vec2::new(80.0, 60.0),
                Color::srgb(1.0, 0.0, 0.0),
                0.8,
            )
        } else {
            (
                ProjectileType::MagicWave,
                1,
                300.0,
                3.0,
                Vec2::new(20.0, 10.0),
                Color::srgb(0.3, 0.6, 1.0),
                0.3,
            )
        };

        *cooldown = cd;

        let projectile_x = player_transform.translation.x + 50.0;
        let projectile_y = player_transform.translation.y;

        commands.spawn((
            Sprite {
                color,
                custom_size: Some(size),
                ..default()
            },
            Transform::from_xyz(projectile_x, projectile_y, 2.0),
            Projectile,
            projectile_type,
            ProjectileData::new(damage, speed, lifetime),
            Velocity { x: speed, y: 0.0 },
            crate::systems::collision::CollisionBox::new(size),
        ));
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
