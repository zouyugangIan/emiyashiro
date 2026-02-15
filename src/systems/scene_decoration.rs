//! 場景裝飾系統
//!
//! 為遊戲場景添加豐富的視覺元素，包括多層背景、裝飾物等

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

/// 場景裝飾組件標記
#[derive(Component)]
pub struct SceneDecoration {
    pub layer: DecorationLayer,
    pub speed_multiplier: f32,
}

/// 裝飾層級
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DecorationLayer {
    FarBackground,  // 遠景（最慢）z = -10.0
    MidBackground,  // 中景 z = -7.0
    NearBackground, // 近景 z = -3.0
    Ground,         // 地面裝飾 z = 0.5
}

impl DecorationLayer {
    pub fn z_index(&self) -> f32 {
        match self {
            DecorationLayer::FarBackground => -10.0,
            DecorationLayer::MidBackground => -7.0,
            DecorationLayer::NearBackground => -3.0,
            DecorationLayer::Ground => 0.5,
        }
    }

    pub fn speed_multiplier(&self) -> f32 {
        match self {
            DecorationLayer::FarBackground => 0.2,  // 20% 速度
            DecorationLayer::MidBackground => 0.5,  // 50% 速度
            DecorationLayer::NearBackground => 0.8, // 80% 速度
            DecorationLayer::Ground => 1.0,         // 100% 速度
        }
    }
}

/// 設置多層視差背景
pub fn setup_parallax_background(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let Some(window) = window_query.iter().next() else {
        return;
    };

    // 遠景層 - 使用封面圖片作為遠景
    let far_bg_images = [
        "images/ui/cover10.jpg",
        "images/ui/cover11.jpg",
        "images/ui/cover12.jpg",
    ];

    for (i, image_path) in far_bg_images.iter().enumerate() {
        let x_offset = (i as f32) * window.width();
        commands.spawn((
            Sprite {
                image: asset_server.load(*image_path),
                custom_size: Some(Vec2::new(window.width(), window.height())),
                color: Color::srgba(1.0, 1.0, 1.0, 0.3), // 半透明
                ..default()
            },
            Transform::from_xyz(x_offset, 0.0, DecorationLayer::FarBackground.z_index()),
            SceneDecoration {
                layer: DecorationLayer::FarBackground,
                speed_multiplier: DecorationLayer::FarBackground.speed_multiplier(),
            },
        ));
    }

    println!("🎨 設置視差背景完成");
}

/// 生成地面裝飾物
pub fn spawn_ground_decorations(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
    mut spawn_timer: Local<f32>,
) {
    let Some(window) = window_query.iter().next() else {
        return;
    };

    *spawn_timer += time.delta_secs();

    // 每 2 秒生成一個裝飾物
    if *spawn_timer > 2.0 {
        *spawn_timer = 0.0;

        let pseudo_random = (time.elapsed_secs() * 100.0) as u32;

        // 隨機選擇裝飾物類型
        let decoration_type = pseudo_random % 3;
        let (size, color) = match decoration_type {
            0 => (Vec2::new(20.0, 30.0), Color::srgb(0.2, 0.6, 0.2)), // 草
            1 => (Vec2::new(15.0, 15.0), Color::srgb(0.5, 0.5, 0.5)), // 石頭
            _ => (Vec2::new(10.0, 40.0), Color::srgb(0.3, 0.5, 0.2)), // 小樹
        };

        commands.spawn((
            Sprite {
                color,
                custom_size: Some(size),
                ..default()
            },
            Transform::from_xyz(
                window.width() + 50.0,
                -240.0,
                DecorationLayer::Ground.z_index(),
            ),
            SceneDecoration {
                layer: DecorationLayer::Ground,
                speed_multiplier: 1.0,
            },
        ));
    }
}

/// 更新場景裝飾物的移動（視差效果）
pub fn move_scene_decorations(
    mut decoration_query: Query<(&mut Transform, &SceneDecoration)>,
    time: Res<Time>,
) {
    const BASE_SPEED: f32 = 50.0; // 基礎移動速度

    for (mut transform, decoration) in decoration_query.iter_mut() {
        // 根據層級應用不同的速度
        let speed = BASE_SPEED * decoration.speed_multiplier;
        transform.translation.x -= speed * time.delta_secs();
    }
}

/// 清理離屏的裝飾物
pub fn cleanup_offscreen_decorations(
    mut commands: Commands,
    decoration_query: Query<(Entity, &Transform, &SceneDecoration)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let Some(window) = window_query.iter().next() else {
        return;
    };

    // 分兩次查詢：一次用於清理，一次用於循環
    let mut to_despawn = Vec::new();

    for (entity, transform, decoration) in decoration_query.iter() {
        // 遠景背景需要循環，不清理
        if decoration.layer == DecorationLayer::FarBackground {
            continue;
        }

        // 其他裝飾物離開屏幕後清理
        if transform.translation.x < -200.0 {
            to_despawn.push(entity);
        }
    }

    // 執行清理
    for entity in to_despawn {
        commands.entity(entity).despawn();
    }
}

/// 增強雲彩系統 - 添加更多變化
pub fn spawn_enhanced_clouds(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
    mut spawn_timer: Local<f32>,
    asset_server: Res<AssetServer>,
) {
    let Some(window) = window_query.iter().next() else {
        return;
    };

    *spawn_timer += time.delta_secs();

    // 每 3 秒生成一朵雲
    if *spawn_timer > 3.0 {
        *spawn_timer = 0.0;

        let pseudo_random = (time.elapsed_secs() * 100.0) as u32;

        // 隨機選擇雲彩圖片
        let cloud_images = ["images/cloud/cloud01.png", "images/cloud/cloud02.png"];
        let cloud_index = (pseudo_random % cloud_images.len() as u32) as usize;
        let cloud_image = asset_server.load(cloud_images[cloud_index]);

        // 隨機高度（上半部分屏幕）
        let cloud_y =
            (pseudo_random % (window.height() * 0.5) as u32) as f32 + window.height() * 0.3;

        // 隨機大小和透明度
        let scale = 0.6 + ((pseudo_random % 60) as f32 / 100.0); // 0.6 - 1.2
        let alpha = 0.5 + ((pseudo_random % 50) as f32 / 100.0); // 0.5 - 1.0

        // 隨機選擇層級（近景或中景）
        let layer = if pseudo_random % 2 == 0 {
            DecorationLayer::NearBackground
        } else {
            DecorationLayer::MidBackground
        };

        commands.spawn((
            Sprite {
                image: cloud_image,
                custom_size: Some(Vec2::new(150.0 * scale, 100.0 * scale)),
                color: Color::srgba(1.0, 1.0, 1.0, alpha),
                ..default()
            },
            Transform::from_xyz(window.width() + 100.0, cloud_y, layer.z_index()),
            SceneDecoration {
                layer,
                speed_multiplier: layer.speed_multiplier(),
            },
        ));
    }
}

/// 遠景背景循環系統
pub fn loop_far_background(
    mut decoration_query: Query<(&mut Transform, &SceneDecoration)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let Some(window) = window_query.iter().next() else {
        return;
    };

    for (mut transform, decoration) in decoration_query.iter_mut() {
        if decoration.layer == DecorationLayer::FarBackground {
            // 如果移出左側，移到右側
            if transform.translation.x < -window.width() {
                transform.translation.x += window.width() * 3.0;
            }
        }
    }
}

/// 添加動態光照效果（簡單的顏色變化）
pub fn dynamic_lighting(
    mut decoration_query: Query<(&mut Sprite, &SceneDecoration)>,
    time: Res<Time>,
) {
    let time_factor = (time.elapsed_secs() * 0.1).sin() * 0.1 + 0.9; // 0.8 - 1.0

    for (mut sprite, decoration) in decoration_query.iter_mut() {
        // 只對遠景應用光照變化
        if decoration.layer == DecorationLayer::FarBackground {
            let current_alpha = sprite.color.alpha();
            sprite.color = Color::srgba(time_factor, time_factor, time_factor, current_alpha);
        }
    }
}
