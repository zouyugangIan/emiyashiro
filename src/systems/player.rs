use bevy::prelude::*;
use crate::{
    components::*,
    resources::*,
};

/// 玩家移动系统
pub fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut Transform, &PlayerState), With<Player>>,
    time: Res<Time>,
) {
    if let Ok((mut transform, player_state)) = player_query.single_mut() {
        let mut direction = 0.0;
        
        // 趴下时不能移动
        if !player_state.is_crouching {
            if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
                direction -= 1.0;
            }
            if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
                direction += 1.0;
            }
        }
        
        if direction != 0.0 {
            transform.translation.x += direction * GameConfig::MOVE_SPEED * time.delta_secs();
        }
    }
}

/// 玩家跳跃系统
pub fn player_jump(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut Transform, &mut Velocity, &PlayerState), With<Player>>,
    time: Res<Time>,
    mut game_stats: ResMut<GameStats>,
) {
    if let Ok((mut transform, mut velocity, player_state)) = player_query.single_mut() {
        // 跳跃输入（趴下时不能跳跃）
        if (keyboard_input.just_pressed(KeyCode::KeyW) || keyboard_input.just_pressed(KeyCode::ArrowUp))
            && player_state.is_grounded 
            && !player_state.is_crouching {
            velocity.y = GameConfig::JUMP_VELOCITY;
            game_stats.jump_count += 1;
            println!("士郎跳跃！(第{}次)", game_stats.jump_count);
        }
        
        // 重力
        velocity.y -= GameConfig::GRAVITY * time.delta_secs();
        
        // 更新位置
        transform.translation.y += velocity.y * time.delta_secs();
        
        // 地面碰撞检测
        if transform.translation.y < GameConfig::GROUND_LEVEL {
            transform.translation.y = GameConfig::GROUND_LEVEL;
            velocity.y = 0.0;
        }
    }
}

/// 玩家趴下系统
pub fn player_crouch(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&mut Transform, &mut PlayerState), With<Player>>,
) {
    if let Ok((mut transform, mut player_state)) = player_query.single_mut() {
        let is_crouch_pressed = keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown);
        
        if is_crouch_pressed && !player_state.is_crouching && player_state.is_grounded {
            // 开始趴下
            player_state.is_crouching = true;
            transform.scale.y = 0.5; // 压缩高度
            transform.translation.y -= 15.0; // 调整位置
            println!("士郎趴下！");
        } else if !is_crouch_pressed && player_state.is_crouching {
            // 停止趴下
            player_state.is_crouching = false;
            transform.scale.y = 0.2; // 恢复原始缩放
            transform.translation.y += 15.0; // 恢复位置
            println!("士郎站起！");
        }
    }
}

/// 更新玩家状态
pub fn update_player_state(
    mut player_query: Query<(&Transform, &mut PlayerState), With<Player>>,
) {
    if let Ok((transform, mut player_state)) = player_query.single_mut() {
        // 更新是否在地面上
        player_state.is_grounded = transform.translation.y <= GameConfig::GROUND_LEVEL + 1.0;
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