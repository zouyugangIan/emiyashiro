//! 摄像机控制系统
//!
//! 包含摄像机跟随、视角控制和场景渲染相关功能。
//! 提供平滑的摄像机跟随、预测性移动和边界限制。

use crate::{components::*, events::CameraImpulseEvent, resources::*};
use bevy::prelude::*;

type PlayerMotionQuery<'w, 's> =
    Query<'w, 's, (&'static Transform, &'static Velocity), (With<Player>, Without<Camera>)>;

/// 轻量镜头震动状态。
#[derive(Resource, Debug, Clone)]
pub struct CameraShakeState {
    pub remaining: f32,
    pub duration: f32,
    pub intensity: f32,
    pub phase: f32,
}

impl Default for CameraShakeState {
    fn default() -> Self {
        Self {
            remaining: 0.0,
            duration: 0.0,
            intensity: 0.0,
            phase: 0.0,
        }
    }
}

impl CameraShakeState {
    pub fn trigger(&mut self, intensity: f32, duration: f32, max_intensity: f32, stack_blend: f32) {
        if intensity <= 0.0 || duration <= 0.0 {
            return;
        }

        let stack_blend = stack_blend.clamp(0.0, 1.0);
        let stacked = self.intensity + intensity * stack_blend.max(0.05);
        self.intensity = stacked.clamp(0.0, max_intensity.max(0.5));

        if duration > self.remaining {
            self.remaining = duration;
            self.duration = duration;
        }
        self.phase = (self.phase + 1.618_034) % std::f32::consts::TAU;
    }

    pub fn tick(&mut self, delta_secs: f32) {
        self.remaining = (self.remaining - delta_secs).max(0.0);
        if self.remaining <= 0.0 {
            self.intensity = 0.0;
            self.duration = 0.0;
        }
    }
}

/// 摄像机配置资源
///
/// 存储摄像机的各种设置参数，允许运行时调整。
#[derive(Resource)]
pub struct CameraConfig {
    /// 跟随速度
    pub follow_speed: f32,
    /// 水平偏移
    pub horizontal_offset: f32,
    /// 垂直跟随强度
    pub vertical_follow_strength: f32,
    /// 预测移动强度
    pub prediction_strength: f32,
    /// 死区大小（摄像机不移动的区域）
    pub dead_zone_width: f32,
    pub dead_zone_height: f32,
    /// 摄像机边界
    pub min_x: f32,
    pub max_x: f32,
    pub min_y: f32,
    pub max_y: f32,
    /// 摇晃效果
    pub shake_intensity: f32,
    pub shake_duration: f32,
    pub shake_timer: f32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            follow_speed: GameConfig::CAMERA_FOLLOW_SPEED,
            horizontal_offset: GameConfig::CAMERA_OFFSET,
            vertical_follow_strength: 0.3,
            prediction_strength: 0.5,
            dead_zone_width: 100.0,
            dead_zone_height: 50.0,
            min_x: -1000.0,
            max_x: 10000.0,
            min_y: -500.0,
            max_y: 500.0,
            shake_intensity: 0.0,
            shake_duration: 0.0,
            shake_timer: 0.0,
        }
    }
}

impl CameraConfig {
    /// 触发摄像机摇晃效果
    pub fn trigger_shake(&mut self, intensity: f32, duration: f32) {
        self.shake_intensity = intensity;
        self.shake_duration = duration;
        self.shake_timer = duration;
    }

    /// 更新摇晃效果
    pub fn update_shake(&mut self, delta_time: f32) {
        if self.shake_timer > 0.0 {
            self.shake_timer -= delta_time;
            if self.shake_timer <= 0.0 {
                self.shake_intensity = 0.0;
                self.shake_timer = 0.0;
            }
        }
    }

    /// 获取当前摇晃偏移
    pub fn get_shake_offset(&self, time: f32) -> Vec2 {
        if self.shake_timer > 0.0 {
            let shake_factor = self.shake_timer / self.shake_duration;
            let intensity = self.shake_intensity * shake_factor;

            Vec2::new(
                (time * 50.0).sin() * intensity,
                (time * 60.0).cos() * intensity,
            )
        } else {
            Vec2::ZERO
        }
    }
}

/// 高级摄像机跟随系统
///
/// 提供平滑的摄像机跟随、预测性移动、死区检测和摇晃效果。
/// 包含边界限制和多种跟随模式。
///
/// # 参数
/// * `camera_query` - 摄像机实体查询
/// * `player_query` - 玩家实体查询（包含速度信息用于预测）
/// * `camera_config` - 摄像机配置资源
/// * `time` - 时间资源
pub fn advanced_camera_follow(
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    player_query: PlayerMotionQuery,
    mut camera_config: ResMut<CameraConfig>,
    time: Res<Time>,
) {
    let delta_time = time.delta_secs();
    let current_time = time.elapsed_secs();

    // 更新摇晃效果
    camera_config.update_shake(delta_time);

    for mut camera_transform in camera_query.iter_mut() {
        if let Ok((player_transform, player_velocity)) = player_query.single() {
            // 计算基础目标位置
            let mut target_x = player_transform.translation.x + camera_config.horizontal_offset;
            let mut target_y =
                player_transform.translation.y * camera_config.vertical_follow_strength;

            // 预测性移动 - 根据玩家速度预测未来位置
            if camera_config.prediction_strength > 0.0 {
                let prediction_time = 0.5; // 预测0.5秒后的位置
                target_x += player_velocity.x * prediction_time * camera_config.prediction_strength;
                target_y +=
                    player_velocity.y * prediction_time * camera_config.prediction_strength * 0.3;
            }

            // 死区检测 - 只有当玩家离开死区时才移动摄像机
            let camera_center = camera_transform.translation;
            let distance_x = target_x - camera_center.x;
            let distance_y = target_y - camera_center.y;

            let should_move_x = distance_x.abs() > camera_config.dead_zone_width * 0.5;
            let should_move_y = distance_y.abs() > camera_config.dead_zone_height * 0.5;

            // 计算移动速度（基于距离的动态速度）
            let dynamic_speed_x = if should_move_x {
                let speed_multiplier = (distance_x.abs() / 100.0).clamp(0.5, 3.0);
                camera_config.follow_speed * speed_multiplier
            } else {
                0.0
            };

            let dynamic_speed_y = if should_move_y {
                let speed_multiplier = (distance_y.abs() / 50.0).clamp(0.5, 2.0);
                camera_config.follow_speed * speed_multiplier * 0.5
            } else {
                0.0
            };

            // 应用平滑移动
            if should_move_x {
                let movement_x = distance_x * dynamic_speed_x * delta_time;
                camera_transform.translation.x += movement_x;
            }

            if should_move_y {
                let movement_y = distance_y * dynamic_speed_y * delta_time;
                camera_transform.translation.y += movement_y;
            }

            // 应用边界限制
            camera_transform.translation.x = camera_transform
                .translation
                .x
                .clamp(camera_config.min_x, camera_config.max_x);
            camera_transform.translation.y = camera_transform
                .translation
                .y
                .clamp(camera_config.min_y, camera_config.max_y);

            // 应用摇晃效果
            let shake_offset = camera_config.get_shake_offset(current_time);
            camera_transform.translation.x += shake_offset.x;
            camera_transform.translation.y += shake_offset.y;
        } else {
            // 没有玩家时的摄像机行为
            idle_camera_behavior(&mut camera_transform, &camera_config, delta_time);
        }
    }
}

/// 优化的摄像机跟随系统
///
/// 实现更平滑的摄像机移动和边界限制，满足需求 3.3 和 3.4。
/// 包含动态跟随速度、预测性移动和完整的边界限制。
pub fn camera_follow(
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    player_query: PlayerMotionQuery,
    time: Res<Time>,
) {
    let delta_time = time.delta_secs();

    for mut camera_transform in camera_query.iter_mut() {
        if let Ok((player_transform, player_velocity)) = player_query.single() {
            // 计算基础目标位置 - 满足需求 3.3：在角色前方保持适当的偏移距离
            let base_offset = GameConfig::CAMERA_OFFSET;
            let dynamic_offset = if player_velocity.x > 0.0 {
                // 向右移动时增加前方偏移
                base_offset + (player_velocity.x * 0.3).min(100.0)
            } else if player_velocity.x < 0.0 {
                // 向左移动时减少偏移
                base_offset + (player_velocity.x * 0.3).max(-100.0)
            } else {
                base_offset
            };

            let target_x = player_transform.translation.x + dynamic_offset;

            // 计算距离和动态跟随速度 - 满足需求 3.4：使用平滑插值减缓移动速度
            let distance_x = target_x - camera_transform.translation.x;
            let distance_abs = distance_x.abs();

            // 动态跟随速度：距离越远速度越快，但有上限
            let base_speed = GameConfig::CAMERA_FOLLOW_SPEED;
            let dynamic_speed = if distance_abs > 200.0 {
                // 距离很远时加速跟随
                base_speed * 2.0
            } else if distance_abs > 100.0 {
                // 中等距离时正常速度
                base_speed * 1.5
            } else if distance_abs > 50.0 {
                // 近距离时减速
                base_speed
            } else {
                // 很近时进一步减速，实现平滑效果
                base_speed * 0.5
            };

            // 应用平滑插值移动
            let follow_speed = dynamic_speed * delta_time;
            let movement_x = distance_x * follow_speed;

            // 限制单帧最大移动距离，防止移动过快
            let max_movement_per_frame = 300.0 * delta_time;
            let clamped_movement_x =
                movement_x.clamp(-max_movement_per_frame, max_movement_per_frame);

            camera_transform.translation.x += clamped_movement_x;

            // 垂直跟随 - 更平滑的垂直移动
            let target_y = (player_transform.translation.y * 0.2).clamp(-80.0, 80.0);
            let distance_y = target_y - camera_transform.translation.y;
            let movement_y = distance_y * follow_speed * 0.3;
            camera_transform.translation.y += movement_y;

            // 摄像机边界限制 - 扩展边界范围
            let left_boundary = -800.0;
            let right_boundary = player_transform.translation.x.max(2000.0);
            let bottom_boundary = -300.0;
            let top_boundary = 200.0;

            camera_transform.translation.x = camera_transform
                .translation
                .x
                .clamp(left_boundary, right_boundary);
            camera_transform.translation.y = camera_transform
                .translation
                .y
                .clamp(bottom_boundary, top_boundary);
        } else {
            // 没有玩家时的摄像机行为 - 更平滑的空闲移动
            let idle_speed = GameConfig::CAMERA_IDLE_SPEED * delta_time;
            camera_transform.translation.x += idle_speed;

            // 空闲状态下的边界限制
            camera_transform.translation.x = camera_transform.translation.x.max(-500.0);

            // 轻微的垂直摆动效果
            let time_factor = time.elapsed_secs() * 0.5;
            let vertical_sway = (time_factor).sin() * 20.0 * delta_time;
            camera_transform.translation.y += vertical_sway;
            camera_transform.translation.y = camera_transform.translation.y.clamp(-100.0, 100.0);
        }
    }
}

/// 读取镜头冲击事件。
pub fn consume_camera_impulse_events(
    mut events: MessageReader<CameraImpulseEvent>,
    mut shake_state: ResMut<CameraShakeState>,
    tuning: Option<Res<crate::resources::GameplayTuning>>,
) {
    let default_tuning = crate::resources::GameplayTuning::default();
    let feedback = &tuning.as_deref().unwrap_or(&default_tuning).camera_feedback;

    for event in events.read() {
        shake_state.trigger(
            event.intensity,
            event.duration,
            feedback.max_shake_intensity,
            feedback.stack_blend,
        );
    }
}

/// 在跟随逻辑之后叠加镜头震动偏移。
pub fn apply_camera_shake(
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    mut shake_state: ResMut<CameraShakeState>,
    tuning: Option<Res<crate::resources::GameplayTuning>>,
    time: Res<Time>,
    mut last_offset: Local<Vec2>,
) {
    if camera_query.is_empty() {
        *last_offset = Vec2::ZERO;
        return;
    }

    shake_state.tick(time.delta_secs());
    let default_tuning = crate::resources::GameplayTuning::default();
    let feedback = &tuning.as_deref().unwrap_or(&default_tuning).camera_feedback;

    let next_offset = if shake_state.remaining > 0.0 && shake_state.duration > 0.0 {
        let progress = (shake_state.remaining / shake_state.duration).clamp(0.0, 1.0);
        let amplitude = shake_state.intensity * progress.powf(feedback.decay_power.clamp(0.2, 1.5));
        let t = time.elapsed_secs() * 58.0 + shake_state.phase;
        Vec2::new(t.sin() * amplitude, (t * 1.31).cos() * amplitude * 0.72)
    } else {
        Vec2::ZERO
    };

    for mut camera_transform in camera_query.iter_mut() {
        camera_transform.translation.x -= last_offset.x;
        camera_transform.translation.y -= last_offset.y;
        camera_transform.translation.x += next_offset.x;
        camera_transform.translation.y += next_offset.y;
    }

    *last_offset = next_offset;
}

/// 空闲状态下的摄像机行为
///
/// 当没有玩家时摄像机的行为模式。
fn idle_camera_behavior(
    camera_transform: &mut Transform,
    camera_config: &CameraConfig,
    delta_time: f32,
) {
    // 缓慢向右移动
    camera_transform.translation.x += GameConfig::CAMERA_IDLE_SPEED * delta_time;

    // 应用边界限制
    camera_transform.translation.x = camera_transform
        .translation
        .x
        .clamp(camera_config.min_x, camera_config.max_x);

    // 轻微的垂直摆动效果
    let time_factor = delta_time * 0.5;
    camera_transform.translation.y += (time_factor * 2.0).sin() * 10.0 * delta_time;
    camera_transform.translation.y = camera_transform
        .translation
        .y
        .clamp(camera_config.min_y * 0.5, camera_config.max_y * 0.5);
}

/// 摄像机震动触发系统
///
/// 在特定事件发生时触发摄像机震动效果。
pub fn camera_shake_trigger_system(
    mut camera_config: ResMut<CameraConfig>,
    player_query: Query<&Velocity, (With<Player>, Changed<Velocity>)>,
) {
    if let Ok(velocity) = player_query.single() {
        // 当玩家着陆时触发轻微震动
        if velocity.y < -300.0 {
            camera_config.trigger_shake(5.0, 0.2);
        }

        // 当玩家高速移动时触发轻微震动
        if velocity.x.abs() > GameConfig::MOVE_SPEED * 1.5 {
            camera_config.trigger_shake(2.0, 0.1);
        }
    }
}

/// 摄像机边界调整系统
///
/// 根据游戏进度动态调整摄像机边界。
pub fn camera_boundary_system(
    mut camera_config: ResMut<CameraConfig>,
    player_query: Query<&Transform, With<Player>>,
    game_stats: Res<GameStats>,
) {
    if let Ok(player_transform) = player_query.single() {
        // 根据玩家位置动态扩展右边界
        let new_max_x = (player_transform.translation.x + 2000.0).max(camera_config.max_x);
        camera_config.max_x = new_max_x;

        // 根据游戏进度调整跟随参数
        let progress_factor = (game_stats.distance_traveled / 1000.0).clamp(0.0, 2.0);
        camera_config.follow_speed =
            GameConfig::CAMERA_FOLLOW_SPEED * (1.0 + progress_factor * 0.5);
    }
}

/// 摄像机调试系统
///
/// 在开发模式下显示摄像机相关信息。
pub fn camera_debug_system(
    camera_query: Query<&Transform, With<Camera>>,
    player_query: Query<&Transform, (With<Player>, Without<Camera>)>,
    camera_config: Res<CameraConfig>,
    mut timer: Local<Timer>,
    time: Res<Time>,
) {
    // 每2秒输出一次调试信息
    if timer.duration().is_zero() {
        timer.set_duration(std::time::Duration::from_secs(2));
        timer.set_mode(bevy::time::TimerMode::Repeating);
    }
    timer.tick(time.delta());

    if timer.just_finished()
        && let (Ok(camera_transform), Ok(player_transform)) =
            (camera_query.single(), player_query.single())
    {
        let distance = camera_transform.translation.x - player_transform.translation.x;
        crate::debug_log!("📷 摄像机调试信息:");
        crate::debug_log!(
            "   摄像机位置: ({:.1}, {:.1})",
            camera_transform.translation.x,
            camera_transform.translation.y
        );
        crate::debug_log!(
            "   玩家位置: ({:.1}, {:.1})",
            player_transform.translation.x,
            player_transform.translation.y
        );
        crate::debug_log!("   距离差: {:.1}", distance);
        crate::debug_log!("   震动强度: {:.1}", camera_config.shake_intensity);
    }
}
