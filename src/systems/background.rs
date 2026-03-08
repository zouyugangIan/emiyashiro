use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::components::Cloud;

const CLOUD_SPEED: f32 = 50.0;
const CLOUD_SPAWN_TIME: f32 = 5.0;

#[derive(Resource)]
pub struct CloudSpawnTimer(pub Timer);

pub fn setup_cloud_spawner(mut commands: Commands) {
    commands.insert_resource(CloudSpawnTimer(Timer::from_seconds(
        CLOUD_SPAWN_TIME,
        TimerMode::Repeating,
    )));
}

pub fn spawn_clouds_system(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
    mut cloud_spawn_timer: ResMut<CloudSpawnTimer>,
    asset_server: Res<AssetServer>,
) {
    if cloud_spawn_timer.0.tick(time.delta()).just_finished() {
        let Some(window) = window_query.iter().next() else {
            crate::debug_log!("⚠️ 无法获取窗口，跳过云彩生成");
            return;
        };

        // HACK: Using time for pseudo-randomness to avoid rand dependency issue.
        let pseudo_random = (time.elapsed_secs() * 100.0) as u32;
        let cloud_y =
            (pseudo_random % (window.height() * 0.4) as u32) as f32 + window.height() * 0.5;

        // 随机选择云彩图片（使用伪随机）
        let cloud_images = ["images/cloud/cloud01.png", "images/cloud/cloud02.png"];
        let cloud_index = (pseudo_random % cloud_images.len() as u32) as usize;
        let cloud_image = asset_server.load(cloud_images[cloud_index]);

        // 随机缩放（0.8 到 1.2 倍）
        let scale_factor = 0.8 + ((pseudo_random % 40) as f32 / 100.0);

        // 使用真实的云彩图片
        commands.spawn((
            Sprite {
                image: cloud_image,
                custom_size: Some(Vec2::new(150.0 * scale_factor, 100.0 * scale_factor)),
                ..default()
            },
            Transform::from_xyz(window.width() + 100.0, cloud_y, -5.0), // z = -5.0 确保在背景
            Cloud,
        ));

        crate::debug_log!(
            "☁️ 生成云彩 at x={}, y={}, scale={:.2}",
            window.width() + 100.0,
            cloud_y,
            scale_factor
        );
    }
}

pub fn move_clouds_system(mut cloud_query: Query<&mut Transform, With<Cloud>>, time: Res<Time>) {
    for mut transform in cloud_query.iter_mut() {
        transform.translation.x -= CLOUD_SPEED * time.delta_secs();
    }
}

pub fn despawn_offscreen_clouds_system(
    mut commands: Commands,
    cloud_query: Query<(Entity, &Transform), With<Cloud>>,
) {
    for (entity, transform) in cloud_query.iter() {
        if transform.translation.x < -200.0 {
            // Despawn when off-screen
            commands.entity(entity).despawn();
            crate::debug_log!("🗑️ 清理离屏云彩 at x={:.1}", transform.translation.x);
        }
    }
}

/// 调试系统：定期报告云彩数量
pub fn debug_cloud_count(
    cloud_query: Query<&Transform, With<Cloud>>,
    time: Res<Time>,
    mut last_report: Local<f32>,
) {
    let current_time = time.elapsed_secs();

    // 每 10 秒报告一次
    if current_time - *last_report > 10.0 {
        let count = cloud_query.iter().count();
        crate::debug_log!("☁️ 当前云彩数量: {}", count);

        // 显示所有云彩的位置
        for (i, transform) in cloud_query.iter().enumerate() {
            crate::debug_log!(
                "  云彩 #{}: x={:.1}, y={:.1}",
                i + 1,
                transform.translation.x,
                transform.translation.y
            );
        }

        *last_report = current_time;
    }
}
