//! 場景裝飾系統
//!
//! 為遊戲場景添加更豐富的視覺層次：多層天空、遠景輪廓、雲層與地面裝飾。

use crate::resources::GameConfig;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

/// 場景裝飾組件標記
#[derive(Component)]
pub struct SceneDecoration {
    pub layer: DecorationLayer,
    pub speed_multiplier: f32,
}

#[derive(Component)]
pub struct SkyPulse {
    base_rgb: Vec3,
    pulse_amplitude: f32,
    pulse_speed: f32,
    alpha: f32,
}

#[derive(Component)]
pub struct StarPulse {
    base_alpha: f32,
    pulse_speed: f32,
    phase: f32,
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

fn pseudo01(seed: f32) -> f32 {
    ((seed.sin() * 43_758.547).abs()).fract()
}

/// 設置多層視差背景
pub fn setup_parallax_background(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    existing_decorations: Query<Entity, With<SceneDecoration>>,
) {
    // Keep this setup idempotent when re-entering Playing from pause/load flows.
    if !existing_decorations.is_empty() {
        return;
    }

    let Some(window) = window_query.iter().next() else {
        return;
    };

    let width = window.width();
    let height = window.height();

    // 1) 遠景天空底色 + 緩慢呼吸。
    for i in 0..3 {
        let x_offset = (i as f32) * width;

        commands.spawn((
            Sprite {
                color: Color::srgba(0.08, 0.12, 0.23, 0.98),
                custom_size: Some(Vec2::new(width, height * 1.05)),
                ..default()
            },
            Transform::from_xyz(
                x_offset,
                height * 0.02,
                DecorationLayer::FarBackground.z_index() - 0.3,
            ),
            SceneDecoration {
                layer: DecorationLayer::FarBackground,
                speed_multiplier: 0.12,
            },
            SkyPulse {
                base_rgb: Vec3::new(0.08, 0.12, 0.23),
                pulse_amplitude: 0.03,
                pulse_speed: 0.10,
                alpha: 0.98,
            },
        ));

        commands.spawn((
            Sprite {
                color: Color::srgba(0.22, 0.30, 0.46, 0.40),
                custom_size: Some(Vec2::new(width, height * 0.62)),
                ..default()
            },
            Transform::from_xyz(
                x_offset,
                -height * 0.12,
                DecorationLayer::FarBackground.z_index() - 0.1,
            ),
            SceneDecoration {
                layer: DecorationLayer::FarBackground,
                speed_multiplier: 0.15,
            },
            SkyPulse {
                base_rgb: Vec3::new(0.22, 0.30, 0.46),
                pulse_amplitude: 0.04,
                pulse_speed: 0.14,
                alpha: 0.40,
            },
        ));
    }

    // 2) 保留封面圖作為超低透明紋理，降低重複感。
    let far_bg_images = [
        "images/ui/cover10.jpg",
        "images/ui/cover11.jpg",
        "images/ui/cover12.jpg",
    ];
    for (i, image_path) in far_bg_images.iter().enumerate() {
        let x_offset = (i as f32) * width;
        commands.spawn((
            Sprite {
                image: asset_server.load(*image_path),
                custom_size: Some(Vec2::new(width, height)),
                color: Color::srgba(1.0, 1.0, 1.0, 0.13),
                ..default()
            },
            Transform::from_xyz(x_offset, 0.0, DecorationLayer::FarBackground.z_index()),
            SceneDecoration {
                layer: DecorationLayer::FarBackground,
                speed_multiplier: DecorationLayer::FarBackground.speed_multiplier(),
            },
        ));
    }

    // 3) 中景輪廓（山脊/廢墟塊）提供空間深度。
    for segment in 0..3 {
        for ridge in 0..5 {
            let seed = segment as f32 * 13.0 + ridge as f32 * 7.0;
            let width_factor = 0.18 + pseudo01(seed + 2.4) * 0.22;
            let ridge_width = width * width_factor;
            let ridge_height = height * (0.18 + pseudo01(seed + 4.7) * 0.20);
            let base_x = (segment as f32) * width - width * 0.44 + ridge as f32 * width * 0.22;
            let base_y = GameConfig::GROUND_LEVEL + ridge_height * 0.35;

            commands.spawn((
                Sprite {
                    color: Color::srgba(0.11, 0.14, 0.20, 0.92),
                    custom_size: Some(Vec2::new(ridge_width, ridge_height)),
                    ..default()
                },
                Transform::from_xyz(base_x, base_y, DecorationLayer::MidBackground.z_index()),
                SceneDecoration {
                    layer: DecorationLayer::MidBackground,
                    speed_multiplier: 0.42,
                },
            ));
        }
    }

    // 4) 星點：小尺寸 + 輕微閃爍。
    for i in 0..56 {
        let seed = i as f32 * 17.171;
        let segment = (i % 3) as f32;
        let x = segment * width + pseudo01(seed + 1.7) * width;
        let y = -height * 0.10 + pseudo01(seed + 8.3) * height * 0.62;
        let size = 1.5 + pseudo01(seed + 3.2) * 2.8;
        let alpha = 0.24 + pseudo01(seed + 5.9) * 0.44;

        commands.spawn((
            Sprite {
                color: Color::srgba(1.0, 1.0, 1.0, alpha),
                custom_size: Some(Vec2::new(size, size)),
                ..default()
            },
            Transform::from_xyz(x, y, DecorationLayer::FarBackground.z_index() + 0.35),
            SceneDecoration {
                layer: DecorationLayer::FarBackground,
                speed_multiplier: 0.14,
            },
            StarPulse {
                base_alpha: alpha,
                pulse_speed: 1.2 + pseudo01(seed + 9.1) * 2.4,
                phase: pseudo01(seed + 11.3) * std::f32::consts::TAU,
            },
        ));
    }

    crate::debug_log!("🎨 設置視差背景完成（多層天空 + 輪廓 + 星點）");
}

/// 清理場景裝飾
pub fn cleanup_scene_decorations(
    mut commands: Commands,
    decorations: Query<Entity, With<SceneDecoration>>,
) {
    for entity in decorations.iter() {
        commands.entity(entity).despawn();
    }
}

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

    // 每 1.7 秒生成一個裝飾物
    if *spawn_timer > 1.7 {
        *spawn_timer = 0.0;

        let seed = time.elapsed_secs() * 7.31;
        let decoration_type = (pseudo01(seed) * 4.0) as u32;

        let (size, color, y_offset) = match decoration_type {
            0 => (Vec2::new(18.0, 28.0), Color::srgb(0.18, 0.56, 0.23), 12.0), // 草叢
            1 => (Vec2::new(26.0, 14.0), Color::srgb(0.46, 0.46, 0.52), 4.0),  // 碎石
            2 => (Vec2::new(14.0, 42.0), Color::srgb(0.27, 0.42, 0.24), 18.0), // 灌木
            _ => (Vec2::new(10.0, 48.0), Color::srgb(0.32, 0.40, 0.33), 22.0), // 殘柱
        };

        commands.spawn((
            Sprite {
                color,
                custom_size: Some(size),
                ..default()
            },
            Transform::from_xyz(
                window.width() + 90.0,
                GameConfig::GROUND_LEVEL + y_offset,
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
    const BASE_SPEED: f32 = 58.0;

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
    let Some(_window) = window_query.iter().next() else {
        return;
    };

    let mut to_despawn = Vec::new();

    for (entity, transform, decoration) in decoration_query.iter() {
        // 遠景背景需要循環，不清理
        if decoration.layer == DecorationLayer::FarBackground {
            continue;
        }

        // 其他裝飾物離開屏幕後清理
        if transform.translation.x < -260.0 {
            to_despawn.push(entity);
        }
    }

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

    // 每 2.6 秒生成一朵雲
    if *spawn_timer > 2.6 {
        *spawn_timer = 0.0;

        let seed = time.elapsed_secs() * 9.17;

        // 隨機選擇雲彩圖片
        let cloud_images = ["images/cloud/cloud01.png", "images/cloud/cloud02.png"];
        let cloud_index = if pseudo01(seed + 0.4) > 0.5 { 1 } else { 0 };
        let cloud_image = asset_server.load(cloud_images[cloud_index]);

        // 隨機高度（中上半部分屏幕）
        let cloud_y = -window.height() * 0.08 + pseudo01(seed + 1.1) * window.height() * 0.56;

        // 隨機大小和透明度
        let scale = 0.65 + pseudo01(seed + 2.2) * 0.75;
        let alpha = 0.35 + pseudo01(seed + 3.3) * 0.50;

        // 隨機選擇層級（近景或中景）
        let layer = if pseudo01(seed + 4.4) > 0.5 {
            DecorationLayer::NearBackground
        } else {
            DecorationLayer::MidBackground
        };

        commands.spawn((
            Sprite {
                image: cloud_image,
                custom_size: Some(Vec2::new(170.0 * scale, 110.0 * scale)),
                color: Color::srgba(1.0, 1.0, 1.0, alpha),
                ..default()
            },
            Transform::from_xyz(window.width() + 120.0, cloud_y, layer.z_index()),
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

/// 添加動態光照效果（天空呼吸 + 星點閃爍）
pub fn dynamic_lighting(
    mut decoration_query: Query<(&mut Sprite, Option<&SkyPulse>, Option<&StarPulse>)>,
    time: Res<Time>,
) {
    let t = time.elapsed_secs();

    for (mut sprite, sky_pulse, star_pulse) in decoration_query.iter_mut() {
        if let Some(sky) = sky_pulse {
            let wave = (t * sky.pulse_speed).sin() * sky.pulse_amplitude;
            let r = (sky.base_rgb.x + wave * 0.7).clamp(0.0, 1.0);
            let g = (sky.base_rgb.y + wave * 0.85).clamp(0.0, 1.0);
            let b = (sky.base_rgb.z + wave).clamp(0.0, 1.0);
            sprite.color = Color::srgba(r, g, b, sky.alpha);
        }

        if let Some(star) = star_pulse {
            let twinkle = 0.62 + 0.38 * (t * star.pulse_speed + star.phase).sin().abs();
            let alpha = (star.base_alpha * twinkle).clamp(0.08, 0.95);
            sprite.color = Color::srgba(1.0, 1.0, 1.0, alpha);
        }
    }
}
