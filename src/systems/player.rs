//! 玩家控制系统
//!
//! 包含玩家移动、跳跃、蹲下等核心玩法系统。

use crate::{components::*, resources::*};
use bevy::prelude::*;

/// 玩家移动系统
///
/// 处理玩家的水平移动，根据输入更新玩家的速度和位置。
/// 支持左右移动，包含改进的移动物理和加速度系统。
///
/// # 参数
/// * `game_input` - 游戏输入状态
/// * `player_query` - 玩家实体查询
/// * `time` - 时间资源
/// * `game_stats` - 游戏统计资源
pub fn player_movement(
    game_input: Res<crate::systems::input::GameInput>,
    mut player_query: Query<(&mut Transform, &mut Velocity, &PlayerState), With<Player>>,
    time: Res<Time>,
    mut game_stats: ResMut<GameStats>,
) {
    if let Ok((mut transform, mut velocity, player_state)) = player_query.single_mut() {
        let delta_time = time.delta_secs();

        // 获取水平输入方向
        let input_direction = if !player_state.is_crouching {
            game_input.get_horizontal_input()
        } else {
            game_input.get_horizontal_input() * 0.5 // 趴下时移动速度减半
        };

        // 计算目标速度
        let target_speed = input_direction * GameConfig::MOVE_SPEED;

        // 应用加速度和减速度（更平滑的移动）
        let acceleration = if input_direction != 0.0 {
            GameConfig::MOVE_SPEED * 8.0 // 加速度
        } else {
            GameConfig::MOVE_SPEED * 12.0 // 减速度（更快停下）
        };

        // 平滑地改变水平速度
        if (target_speed - velocity.x).abs() > 0.1 {
            let speed_diff = target_speed - velocity.x;
            let max_change = acceleration * delta_time;

            if speed_diff.abs() <= max_change {
                velocity.x = target_speed;
            } else {
                velocity.x += speed_diff.signum() * max_change;
            }
        } else {
            velocity.x = target_speed;
        }

        // 记录移动前的位置
        let old_x = transform.translation.x;

        // 应用水平移动
        transform.translation.x += velocity.x * delta_time;

        // 更新移动距离统计
        let distance_moved = (transform.translation.x - old_x).abs();
        if distance_moved > 0.01 {
            game_stats.distance_traveled += distance_moved;
        }

        // 边界检查（防止玩家移动到屏幕外太远）
        let max_distance = 10000.0; // 最大允许距离
        if transform.translation.x.abs() > max_distance {
            transform.translation.x = transform.translation.x.signum() * max_distance;
        }
    }
}

/// 玩家跳跃和重力系统
///
/// 处理玩家的跳跃输入、重力应用和地面碰撞检测。
/// 包含改进的物理计算和更精确的碰撞处理。
pub fn player_jump(
    mut commands: Commands,
    game_input: Res<crate::systems::input::GameInput>,
    mut player_query: Query<(&mut Transform, &mut Velocity, &PlayerState), With<Player>>,
    time: Res<Time>,
    mut game_stats: ResMut<GameStats>,
) {
    if let Ok((mut transform, mut velocity, player_state)) = player_query.single_mut() {
        let was_grounded = player_state.is_grounded;
        let delta_time = time.delta_secs();

        // 跳跃输入处理（改进的跳跃检测）
        if game_input.jump && player_state.can_jump() {
            velocity.y = GameConfig::JUMP_VELOCITY;
            game_stats.jump_count += 1;

            // 触发跳跃音效
            commands.spawn(AudioTrigger {
                sound_type: SoundType::Jump,
                should_play: true,
            });

            println!("🗡️ 士郎跳跃！(第{}次)", game_stats.jump_count);
        }

        // 可变跳跃高度 - 如果松开跳跃键，减少向上速度
        if !game_input.jump && velocity.y > 0.0 {
            velocity.y *= 0.5; // 减少50%的向上速度，实现可变跳跃高度
        }

        // 应用重力（改进的重力系统）
        apply_gravity(&mut velocity, player_state, delta_time);

        // 更新垂直位置（使用改进的物理积分）
        let old_y = transform.translation.y;
        transform.translation.y += velocity.y * delta_time;

        // 终端速度限制（防止无限加速下落）
        velocity.y = velocity.y.max(-GameConfig::GRAVITY * 2.0);

        // 死亡检测 - 如果掉到地面以下太远
        if transform.translation.y < GameConfig::GROUND_LEVEL - 200.0 {
            handle_player_death(&mut transform, &mut velocity, &mut game_stats);
            return;
        }

        // 地面碰撞检测和处理（改进的碰撞系统）
        handle_ground_collision(
            &mut commands,
            &mut transform,
            &mut velocity,
            old_y,
            was_grounded,
        );
    }
}

/// 应用重力效果
///
/// 根据玩家状态应用不同的重力效果。
fn apply_gravity(velocity: &mut Velocity, player_state: &PlayerState, delta_time: f32) {
    // 只有在空中或有向上速度时才应用重力
    if !player_state.is_grounded || velocity.y > 0.0 {
        // 根据玩家状态调整重力
        let gravity_multiplier = if player_state.is_crouching && velocity.y < 0.0 {
            1.5 // 趴下时下落更快
        } else if velocity.y > 0.0 {
            0.8 // 上升时重力稍小，让跳跃感觉更好
        } else {
            1.0 // 正常重力
        };

        velocity.y -= GameConfig::GRAVITY * gravity_multiplier * delta_time;
    }
}

/// 处理地面碰撞
///
/// 检测和处理玩家与地面的碰撞，包括着陆音效和状态更新。
fn handle_ground_collision(
    commands: &mut Commands,
    transform: &mut Transform,
    velocity: &mut Velocity,
    _old_y: f32,
    was_grounded: bool,
) {
    if transform.translation.y <= GameConfig::GROUND_LEVEL {
        // 精确的地面位置设置
        transform.translation.y = GameConfig::GROUND_LEVEL;

        // 计算着陆冲击力（用于音效和视觉效果）
        let impact_velocity = velocity.y.abs();

        // 如果刚着地且有足够的冲击力，触发着地音效
        if !was_grounded && impact_velocity > 50.0 {
            commands.spawn(AudioTrigger {
                sound_type: SoundType::Land,
                should_play: true,
            });

            // 根据冲击力输出不同的着陆消息
            if impact_velocity > 300.0 {
                println!("🗡️ 士郎重重着陆！冲击力: {:.1}", impact_velocity);
            } else {
                println!("🗡️ 士郎轻巧着陆！");
            }
        }

        // 重置垂直速度（只有向下的速度才重置）
        if velocity.y < 0.0 {
            velocity.y = 0.0;
        }
    }
}

/// 处理玩家死亡
///
/// 当玩家掉入深渊时重置游戏状态。
fn handle_player_death(
    transform: &mut Transform,
    velocity: &mut Velocity,
    game_stats: &mut GameStats,
) {
    println!("💀 士郎掉入深渊！游戏结束！");

    // 重置玩家位置和速度
    transform.translation = GameConfig::PLAYER_START_POS;
    velocity.y = 0.0;
    velocity.x = 0.0;

    // 保存最佳记录，然后重置当前统计
    let current_distance = game_stats.distance_traveled;
    let current_jumps = game_stats.jump_count;
    let current_time = game_stats.play_time;

    // 重置统计数据
    game_stats.jump_count = 0;
    game_stats.distance_traveled = 0.0;
    game_stats.play_time = 0.0;

    println!("📊 本次游戏统计:");
    println!("   距离: {:.1}m", current_distance);
    println!("   跳跃次数: {}", current_jumps);
    println!("   游戏时间: {:.1}s", current_time);
}

/// 物理系统更新
///
/// 统一处理所有物理相关的计算，确保物理模拟的一致性。
pub fn physics_update_system(
    mut player_query: Query<(&mut Transform, &mut Velocity), With<Player>>,
    time: Res<Time>,
) {
    if let Ok((mut transform, mut velocity)) = player_query.single_mut() {
        let _delta_time = time.delta_secs();

        // 应用空气阻力（轻微的水平减速）
        if velocity.x.abs() > 0.1 {
            let air_resistance = 0.98; // 98% 保留速度，2% 空气阻力
            velocity.x *= air_resistance;
        } else {
            velocity.x = 0.0; // 速度很小时直接停止
        }

        // 限制最大速度
        let max_horizontal_speed = GameConfig::MOVE_SPEED * 1.5;
        let max_vertical_speed = GameConfig::GRAVITY * 2.0;

        velocity.x = velocity
            .x
            .clamp(-max_horizontal_speed, max_horizontal_speed);
        velocity.y = velocity
            .y
            .clamp(-max_vertical_speed, GameConfig::JUMP_VELOCITY * 1.2);

        // 物理积分验证（确保数值稳定性）
        if velocity.x.is_nan() || velocity.y.is_nan() {
            println!("⚠️ 检测到无效速度，重置为零");
            velocity.x = 0.0;
            velocity.y = 0.0;
        }

        if transform.translation.x.is_nan() || transform.translation.y.is_nan() {
            println!("⚠️ 检测到无效位置，重置到起始位置");
            transform.translation = GameConfig::PLAYER_START_POS;
        }
    }
}

/// 游戏时间更新系统
///
/// 更新游戏统计中的游戏时间。
pub fn update_game_time(mut game_stats: ResMut<GameStats>, time: Res<Time>) {
    game_stats.play_time += time.delta_secs();
}

/// 玩家趴下系统
pub fn player_crouch(
    game_input: Res<crate::systems::input::GameInput>,
    mut player_query: Query<(&mut Transform, &mut PlayerState), With<Player>>,
) {
    if let Ok((mut transform, mut player_state)) = player_query.single_mut() {
        let is_crouch_pressed = game_input.crouch;

        if is_crouch_pressed && !player_state.is_crouching && player_state.is_grounded {
            // 开始趴下
            player_state.is_crouching = true;
            // 简单的缩放方法，保持原始X缩放
            let _original_x_scale = transform.scale.x;
            transform.scale.y = 0.5; // 压缩高度
            transform.translation.y -= 15.0; // 向下移动一点
            println!("🗡️ 士郎趴下！");
        } else if !is_crouch_pressed && player_state.is_crouching {
            // 停止趴下
            player_state.is_crouching = false;
            transform.scale.y = transform.scale.x; // 恢复Y缩放与X缩放一致
            transform.translation.y += 15.0; // 向上移动回原位
            println!("🗡️ 士郎站起！");
        }
    }
}

/// 更新玩家状态
///
/// 更新玩家的各种状态，如是否在地面上等。
/// 现在主要由碰撞检测系统处理，这里保留作为备用。
pub fn update_player_state(mut player_query: Query<(&Transform, &mut PlayerState), With<Player>>) {
    if let Ok((transform, mut player_state)) = player_query.single_mut() {
        // 基本的地面检测（作为碰撞系统的备用）
        let ground_threshold = 2.0; // 允许的地面检测阈值
        let distance_to_ground = transform.translation.y - GameConfig::GROUND_LEVEL;

        // 如果玩家非常接近地面，认为在地面上
        if distance_to_ground <= ground_threshold && distance_to_ground >= -1.0 {
            player_state.is_grounded = true;
        } else if distance_to_ground > ground_threshold {
            player_state.is_grounded = false;
        }

        // 边界检查 - 如果玩家超出游戏边界，调整状态
        if transform.translation.y < GameConfig::GROUND_LEVEL - 50.0 {
            player_state.is_grounded = false;
        }
    }
}

/// 更新游戏统计
pub fn update_game_stats(
    player_query: Query<&Transform, (With<Player>, Changed<Transform>)>,
    mut game_stats: ResMut<GameStats>,
    time: Res<Time>,
) {
    // 更新游戏时间
    game_stats.play_time += time.delta_secs();

    // 更新移动距离
    if let Ok(transform) = player_query.single() {
        if transform.translation.x > game_stats.distance_traveled {
            game_stats.distance_traveled = transform.translation.x;
        }
    }
}
