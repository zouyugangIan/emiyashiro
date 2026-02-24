//! 敵人系統

use crate::asset_paths;
use crate::components::*;
use crate::resources::GameConfig;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

const SLIME_RENDER_SIZE: Vec2 = Vec2::new(56.0, 44.0);
const SLIME_COLLISION_SIZE: Vec2 = Vec2::new(42.0, 30.0);

/// 生成史莱姆敌人（保留原系统名以兼容已有调度）
pub fn spawn_mushroom_enemies(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut spawn_timer: Local<f32>,
) {
    let Some(window) = window_query.iter().next() else {
        return;
    };

    *spawn_timer += time.delta_secs();

    // 每 5 秒生成一个史莱姆敌人
    if *spawn_timer > 5.0 {
        *spawn_timer = 0.0;

        let pseudo_random = (time.elapsed_secs() * 100.0) as u32;
        let spawn_x = window.width() + 100.0;
        let spawn_y = GameConfig::GROUND_LEVEL + 18.0;

        // 随机巡逻范围
        let patrol_range = 80.0 + ((pseudo_random % 120) as f32);
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
                EnemyState::new(4, patrol_range),
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

        crate::debug_log!("🟢 生成史莱姆敌人 at x={:.1}", spawn_x);
    }
}

/// 敵人 AI - 巡邏移動
pub fn enemy_patrol_ai(
    mut enemy_query: Query<(&mut Transform, &mut EnemyState, &mut Velocity), With<Enemy>>,
    time: Res<Time>,
) {
    const ENEMY_SPEED: f32 = 50.0;

    for (mut transform, mut state, mut velocity) in enemy_query.iter_mut() {
        if !state.is_alive {
            velocity.x = 0.0;
            continue;
        }

        // 計算相對於生成點的位置
        let relative_x = transform.translation.x;

        // 巡邏邏輯：在範圍內來回移動
        if state.move_direction > 0.0 && relative_x > state.patrol_right {
            state.move_direction = -1.0; // 轉向左
        } else if state.move_direction < 0.0 && relative_x < state.patrol_left {
            state.move_direction = 1.0; // 轉向右
        }

        velocity.x = ENEMY_SPEED * state.move_direction;
        transform.translation.x += velocity.x * time.delta_secs();
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
) {
    for (entity, transform) in enemy_query.iter() {
        if transform.translation.x < -300.0 {
            commands.entity(entity).despawn();
            crate::debug_log!("🗑️ 清理離屏敵人");
        }
    }
}
