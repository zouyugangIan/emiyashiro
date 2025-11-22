//! 戰鬥系統 - 攻擊、投射物、傷害

use bevy::prelude::*;
use crate::components::*;

/// 玩家發射法波
pub fn player_shoot_projectile(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    player_query: Query<&Transform, With<Player>>,
    mut cooldown: Local<f32>,
    time: Res<Time>,
) {
    *cooldown -= time.delta_secs();
    
    // J 鍵發射法波
    if keyboard.just_pressed(KeyCode::KeyJ) && *cooldown <= 0.0 {
        if let Some(player_transform) = player_query.iter().next() {
            *cooldown = 0.5; // 0.5 秒冷卻
            
            // 在玩家前方生成法波
            let projectile_x = player_transform.translation.x + 50.0;
            let projectile_y = player_transform.translation.y;
            
            commands.spawn((
                Sprite {
                    color: Color::srgb(0.3, 0.6, 1.0), // 藍色法波
                    custom_size: Some(Vec2::new(20.0, 10.0)),
                    ..default()
                },
                Transform::from_xyz(projectile_x, projectile_y, 2.0),
                Projectile,
                ProjectileType::MagicWave,
                ProjectileData::new(1, 300.0, 3.0), // 1 點傷害，300 速度，3 秒存活
                Velocity { x: 300.0, y: 0.0 },
                crate::systems::collision::CollisionBox::new(Vec2::new(20.0, 10.0)),
            ));
            
            println!("✨ 發射法波！");
        }
    }
}

/// 更新投射物移動
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

/// 清理過期投射物
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

/// 投射物與敵人碰撞檢測
pub fn projectile_enemy_collision(
    mut commands: Commands,
    projectile_query: Query<(Entity, &Transform, &ProjectileData, &crate::systems::collision::CollisionBox), With<Projectile>>,
    mut enemy_query: Query<(Entity, &Transform, &mut EnemyState, &crate::systems::collision::CollisionBox), With<Enemy>>,
) {
    for (proj_entity, proj_transform, proj_data, proj_box) in projectile_query.iter() {
        for (_enemy_entity, enemy_transform, mut enemy_state, enemy_box) in enemy_query.iter_mut() {
            if !enemy_state.is_alive {
                continue;
            }
            
            // 簡單的 AABB 碰撞檢測
            let proj_pos = proj_transform.translation;
            let enemy_pos = enemy_transform.translation;
            
            let dx = (proj_pos.x - enemy_pos.x).abs();
            let dy = (proj_pos.y - enemy_pos.y).abs();
            
            let collision_x = dx < (proj_box.size.x + enemy_box.size.x) / 2.0;
            let collision_y = dy < (proj_box.size.y + enemy_box.size.y) / 2.0;
            
            if collision_x && collision_y {
                // 造成傷害
                enemy_state.take_damage(proj_data.damage);
                println!("💥 擊中敵人！剩餘血量: {}", enemy_state.health);
                
                // 銷毀投射物
                commands.entity(proj_entity).despawn();
                
                // 如果敵人死亡，改變顏色
                if !enemy_state.is_alive {
                    println!("☠️ 敵人被擊敗！");
                }
                
                break;
            }
        }
    }
}

/// 玩家與敵人碰撞檢測（受傷）
pub fn player_enemy_collision(
    player_query: Query<(&Transform, &crate::systems::collision::CollisionBox), With<Player>>,
    enemy_query: Query<(&Transform, &EnemyState, &crate::systems::collision::CollisionBox), With<Enemy>>,
    mut last_damage_time: Local<f32>,
    time: Res<Time>,
) {
    *last_damage_time += time.delta_secs();
    
    if let Some((player_transform, player_box)) = player_query.iter().next() {
        for (enemy_transform, enemy_state, enemy_box) in enemy_query.iter() {
            if !enemy_state.is_alive {
                continue;
            }
            
            let player_pos = player_transform.translation;
            let enemy_pos = enemy_transform.translation;
            
            let dx = (player_pos.x - enemy_pos.x).abs();
            let dy = (player_pos.y - enemy_pos.y).abs();
            
            let collision_x = dx < (player_box.size.x + enemy_box.size.x) / 2.0;
            let collision_y = dy < (player_box.size.y + enemy_box.size.y) / 2.0;
            
            if collision_x && collision_y && *last_damage_time > 1.0 {
                println!("❤️ 玩家受傷！");
                *last_damage_time = 0.0;
                // TODO: 減少玩家血量
            }
        }
    }
}
