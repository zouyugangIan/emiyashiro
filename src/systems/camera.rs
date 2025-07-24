//! 摄像机控制系统
//! 
//! 包含摄像机跟随、视角控制和场景渲染相关功能。
//! 提供平滑的摄像机跟随、预测性移动和边界限制。

use bevy::prelude::*;
use crate::{
    components::*,
    resources::*,
};

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
    player_query: Query<(&Transform, &Velocity), (With<Player>, Without<Camera>)>,
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
            let mut target_y = player_transform.translation.y * camera_config.vertical_follow_strength;
            
            // 预测性移动 - 根据玩家速度预测未来位置
            if camera_config.prediction_strength > 0.0 {
                let prediction_time = 0.5; // 预测0.5秒后的位置
                target_x += player_velocity.x * prediction_time * camera_config.prediction_strength;
                target_y += player_velocity.y * prediction_time * camera_config.prediction_strength * 0.3;
            }
            
            // 死区检测 - 只有当玩家离开死区时才移动摄像机
            let camera_center = camera_transform.translation;
            let distance_x = target_x - camera_center.x;
            let distance_y = target_y - camera_center.y;
            
            let mut should_move_x = distance_x.abs() > camera_config.dead_zone_width * 0.5;
            let mut should_move_y = distance_y.abs() > camera_config.dead_zone_height * 0.5;
            
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
            camera_transform.translation.x = camera_transform.translation.x
                .clamp(camera_config.min_x, camera_config.max_x);
            camera_transform.translation.y = camera_transform.translation.y
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

/// 简化的摄像机跟随系统（向后兼容）
/// 
/// 保持原有的简单跟随行为，用于不需要高级功能的场景。
pub fn camera_follow(
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    player_query: Query<&Transform, (With<Player>, Without<Camera>)>,
    time: Res<Time>,
) {
    let delta_time = time.delta_secs();
    
    for mut camera_transform in camera_query.iter_mut() {
        if let Ok(player_transform) = player_query.single() {
            // 计算目标位置
            let target_x = player_transform.translation.x + GameConfig::CAMERA_OFFSET;
            
            // 平滑跟随计算
            let follow_speed = GameConfig::CAMERA_FOLLOW_SPEED * delta_time;
            let distance = target_x - camera_transform.translation.x;
            let movement = distance * follow_speed;
            
            // 应用移动
            camera_transform.translation.x += movement;
            
            // 摄像机边界限制
            camera_transform.translation.x = camera_transform.translation.x.max(-500.0);
            
            // 垂直跟随
            let target_y = (player_transform.translation.y * 0.3).clamp(-100.0, 100.0);
            camera_transform.translation.y += 
                (target_y - camera_transform.translation.y) * follow_speed * 0.5;
        } else {
            // 没有玩家时摄像机保持静止或缓慢移动
            camera_transform.translation.x += GameConfig::CAMERA_IDLE_SPEED * delta_time;
            camera_transform.translation.x = camera_transform.translation.x.max(-500.0);
        }
    }
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
    camera_transform.translation.x = camera_transform.translation.x
        .clamp(camera_config.min_x, camera_config.max_x);
    
    // 轻微的垂直摆动效果
    let time_factor = delta_time * 0.5;
    camera_transform.translation.y += (time_factor * 2.0).sin() * 10.0 * delta_time;
    camera_transform.translation.y = camera_transform.translation.y
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
        camera_config.follow_speed = GameConfig::CAMERA_FOLLOW_SPEED * (1.0 + progress_factor * 0.5);
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
    
    if timer.just_finished() {
        if let (Ok(camera_transform), Ok(player_transform)) = (camera_query.single(), player_query.single()) {
            let distance = camera_transform.translation.x - player_transform.translation.x;
            println!("📷 摄像机调试信息:");
            println!("   摄像机位置: ({:.1}, {:.1})", 
                camera_transform.translation.x, camera_transform.translation.y);
            println!("   玩家位置: ({:.1}, {:.1})", 
                player_transform.translation.x, player_transform.translation.y);
            println!("   距离差: {:.1}", distance);
            println!("   震动强度: {:.1}", camera_config.shake_intensity);
        }
    }
}