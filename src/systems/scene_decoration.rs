//! 场景装饰系统
//!
//! 为游戏场景添加更丰富的视觉层次：多层天空、远景轮廓、云层与地面装饰。

use crate::resources::GameConfig;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

const MAX_ACTIVE_CLOUDS: usize = 9;
const CLOUD_SPAWN_INTERVAL_SECS: f32 = 3.4;
const CLOUD_IMAGES: [&str; 2] = ["images/cloud/cloud01.png", "images/cloud/cloud02.png"];

/// 场景装饰组件标记
#[derive(Component)]
pub struct SceneDecoration {
    pub layer: DecorationLayer,
    pub speed_multiplier: f32,
}

#[derive(Component)]
pub struct CloudStrata {
    pub lane: u8,
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

/// 装饰层级
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DecorationLayer {
    FarBackground,  // 远景（最慢）z = -10.0
    MidBackground,  // 中景 z = -7.0
    NearBackground, // 近景 z = -3.0
    Ground,         // 地面装饰 z = 0.5
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

#[derive(Clone, Copy)]
struct CloudProfile {
    lane: u8,
    layer: DecorationLayer,
    y: f32,
    scale: f32,
    alpha: f32,
    speed_multiplier: f32,
}

#[cfg(test)]
fn cloud_profile(seed: f32, window_height: f32) -> CloudProfile {
    let lane = (pseudo01(seed + 0.13) * 3.0).floor() as u8;
    cloud_profile_for_lane(lane, seed, window_height)
}

fn cloud_profile_for_lane(lane: u8, seed: f32, window_height: f32) -> CloudProfile {
    let lane = lane.min(2);
    let jitter = pseudo01(seed + 3.7) - 0.5;
    let min_y = GameConfig::GROUND_LEVEL + 250.0;
    let max_y = window_height * 0.42;

    let (layer, base_y, scale, alpha, speed_multiplier) = match lane {
        0 => (
            DecorationLayer::MidBackground,
            window_height * 0.34,
            0.72 + pseudo01(seed + 5.1) * 0.18,
            0.20 + pseudo01(seed + 7.9) * 0.08,
            0.22,
        ),
        1 => (
            DecorationLayer::MidBackground,
            window_height * 0.23,
            0.86 + pseudo01(seed + 5.1) * 0.20,
            0.22 + pseudo01(seed + 7.9) * 0.08,
            0.30,
        ),
        _ => (
            DecorationLayer::NearBackground,
            window_height * 0.12,
            1.02 + pseudo01(seed + 5.1) * 0.22,
            0.20 + pseudo01(seed + 7.9) * 0.07,
            0.42,
        ),
    };

    CloudProfile {
        lane,
        layer,
        y: (base_y + jitter * window_height * 0.055).clamp(min_y, max_y),
        scale,
        alpha,
        speed_multiplier,
    }
}

fn spawn_stratified_cloud(
    commands: &mut Commands,
    asset_server: &AssetServer,
    image_path: &'static str,
    x: f32,
    profile: CloudProfile,
) {
    commands.spawn((
        Sprite {
            image: asset_server.load(image_path),
            custom_size: Some(Vec2::new(190.0 * profile.scale, 112.0 * profile.scale)),
            color: Color::srgba(1.0, 1.0, 1.0, profile.alpha),
            ..default()
        },
        Transform::from_xyz(x, profile.y, profile.layer.z_index()),
        SceneDecoration {
            layer: profile.layer,
            speed_multiplier: profile.speed_multiplier,
        },
        CloudStrata { lane: profile.lane },
    ));
}

/// 设置多层视差背景
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

    // 1) 远景天空底色 + 缓慢呼吸。
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

    // 2) 保留封面图作为超低透明纹理，降低重复感。
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

    // 3) 中景轮廓（山脊/废墟块）提供空间深度。
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

    // 4) 星点：小尺寸 + 轻微闪烁。
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

    // 5) 预铺分层云带，避免开局空白，同时不侵入玩家动作判读区。
    for i in 0..6 {
        let seed = 91.0 + i as f32 * 11.83;
        let profile = cloud_profile_for_lane((i % 3) as u8, seed, height);
        let image_path = CLOUD_IMAGES
            [(pseudo01(seed + 1.4) * CLOUD_IMAGES.len() as f32) as usize % CLOUD_IMAGES.len()];
        let x = -width * 0.28 + i as f32 * width * 0.46 + pseudo01(seed + 2.6) * width * 0.12;
        spawn_stratified_cloud(&mut commands, &asset_server, image_path, x, profile);
    }

    crate::debug_log!("🎨 设置视差背景完成（多层天空 + 轮廓 + 星点）");
}

/// 清理场景装饰
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

    // 每 1.7 秒生成一个装饰物
    if *spawn_timer > 1.7 {
        *spawn_timer = 0.0;

        let seed = time.elapsed_secs() * 7.31;
        let decoration_type = (pseudo01(seed) * 4.0) as u32;

        let (size, color, y_offset) = match decoration_type {
            0 => (Vec2::new(18.0, 28.0), Color::srgb(0.18, 0.56, 0.23), 12.0), // 草丛
            1 => (Vec2::new(26.0, 14.0), Color::srgb(0.46, 0.46, 0.52), 4.0),  // 碎石
            2 => (Vec2::new(14.0, 42.0), Color::srgb(0.27, 0.42, 0.24), 18.0), // 灌木
            _ => (Vec2::new(10.0, 48.0), Color::srgb(0.32, 0.40, 0.33), 22.0), // 残柱
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

/// 更新场景装饰物的移动（视差效果）
pub fn move_scene_decorations(
    mut decoration_query: Query<(&mut Transform, &SceneDecoration)>,
    time: Res<Time>,
) {
    const BASE_SPEED: f32 = 58.0;

    for (mut transform, decoration) in decoration_query.iter_mut() {
        // 根据层级应用不同的速度
        let speed = BASE_SPEED * decoration.speed_multiplier;
        transform.translation.x -= speed * time.delta_secs();
    }
}

/// 清理离屏的装饰物
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
        // 远景背景需要循环，不清理
        if decoration.layer == DecorationLayer::FarBackground {
            continue;
        }

        // 其他装饰物离开屏幕后清理
        if transform.translation.x < -260.0 {
            to_despawn.push(entity);
        }
    }

    for entity in to_despawn {
        commands.entity(entity).despawn();
    }
}

/// 增强云彩系统 - 添加更多变化
pub fn spawn_enhanced_clouds(
    mut commands: Commands,
    window_query: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
    mut spawn_timer: Local<f32>,
    mut lane_cursor: Local<u8>,
    asset_server: Res<AssetServer>,
    active_clouds: Query<&CloudStrata>,
) {
    let Some(window) = window_query.iter().next() else {
        return;
    };

    *spawn_timer += time.delta_secs();

    // 分层云带：低密度、低透明、避开角色与攻击特效判读区。
    let mut lane_counts = [0usize; 3];
    let mut active_count = 0usize;
    for strata in active_clouds.iter() {
        lane_counts[strata.lane.min(2) as usize] += 1;
        active_count += 1;
    }

    if *spawn_timer > CLOUD_SPAWN_INTERVAL_SECS && active_count < MAX_ACTIVE_CLOUDS {
        *spawn_timer = 0.0;

        let seed = time.elapsed_secs() * 9.17;
        let lane = (0..3)
            .map(|offset| ((*lane_cursor as usize + offset) % 3) as u8)
            .min_by_key(|lane| lane_counts[*lane as usize])
            .unwrap_or(0);
        if lane_counts[lane as usize] >= MAX_ACTIVE_CLOUDS.div_ceil(3) {
            return;
        }

        *lane_cursor = (lane + 1) % 3;
        let profile = cloud_profile_for_lane(lane, seed, window.height());
        let image_path = CLOUD_IMAGES
            [(pseudo01(seed + 1.4) * CLOUD_IMAGES.len() as f32) as usize % CLOUD_IMAGES.len()];
        let x = window.width() + 150.0 + pseudo01(seed + 2.6) * 240.0;
        spawn_stratified_cloud(&mut commands, &asset_server, image_path, x, profile);
    }
}

/// 远景背景循环系统
pub fn loop_far_background(
    mut decoration_query: Query<(&mut Transform, &SceneDecoration)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let Some(window) = window_query.iter().next() else {
        return;
    };

    for (mut transform, decoration) in decoration_query.iter_mut() {
        if decoration.layer == DecorationLayer::FarBackground {
            // 如果移出左侧，移到右侧
            if transform.translation.x < -window.width() {
                transform.translation.x += window.width() * 3.0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cloud_profile_keeps_clouds_in_readable_upper_bands() {
        let window_height = 720.0;

        for index in 0..128 {
            let profile = cloud_profile(index as f32 * 4.31, window_height);
            assert!(
                profile.y >= GameConfig::GROUND_LEVEL + 250.0,
                "cloud should stay above the action readability band"
            );
            assert!(
                profile.y <= window_height * 0.42,
                "cloud should stay out of the extreme top crop"
            );
            assert!(
                profile.alpha <= 0.30,
                "cloud opacity should stay muted behind combat"
            );
            assert!(
                profile.speed_multiplier <= 0.42,
                "cloud speed should remain slower than foreground action"
            );
        }
    }

    #[test]
    fn cloud_profiles_cover_three_readable_lanes() {
        let window_height = 720.0;
        let low = cloud_profile_for_lane(0, 11.0, window_height);
        let mid = cloud_profile_for_lane(1, 11.0, window_height);
        let high = cloud_profile_for_lane(2, 11.0, window_height);

        assert_eq!(low.lane, 0);
        assert_eq!(mid.lane, 1);
        assert_eq!(high.lane, 2);
        assert!(
            low.y > mid.y,
            "lower cloud lane should sit below the mid lane"
        );
        assert!(
            mid.y > high.y,
            "mid cloud lane should sit below the high lane"
        );
        assert!(
            [low.lane, mid.lane, high.lane].iter().all(|lane| *lane < 3),
            "cloud lanes should stay in the controlled three-band layout"
        );
    }
}

/// 添加动态光照效果（天空呼吸 + 星点闪烁）
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
