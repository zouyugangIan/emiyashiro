//! 输入处理系统
//!
//! 包含游戏输入的捕获、状态管理与网络同步。

use bevy::prelude::*;

#[derive(Resource, Debug, Clone)]
pub struct NetworkInputSyncState {
    pub next_sequence: u32,
    pub last_sent_move_x: f32,
    pub last_sent_move_y: f32,
    pub last_state_send_time: f32,
    pub state_send_interval_secs: f32,
    pub sent_state_count: u64,
    pub sent_event_count: u64,
}

impl Default for NetworkInputSyncState {
    fn default() -> Self {
        Self {
            next_sequence: 1,
            last_sent_move_x: 0.0,
            last_sent_move_y: 0.0,
            last_state_send_time: f32::NEG_INFINITY,
            state_send_interval_secs: 0.1,
            sent_state_count: 0,
            sent_event_count: 0,
        }
    }
}

/// 游戏输入资源
///
/// 存储当前持续输入、本帧边沿输入和跳跃缓冲。
#[derive(Resource, Default)]
pub struct GameInput {
    // 移动输入
    pub move_left: bool,
    pub move_right: bool,
    pub jump: bool,
    pub crouch: bool,

    // 菜单输入
    pub confirm: bool,
    pub cancel: bool,
    pub pause: bool,

    // 特殊输入
    pub action1: bool, // 默认平A / 近战挥刀
    pub action2: bool, // 投影魔术（远程）
    pub action1_pressed_this_frame: bool,
    pub action2_pressed_this_frame: bool,
    pub jump_pressed_this_frame: bool,
    pub jump_buffer_seconds: f32,
}

/// 更新游戏输入系统
///
/// 从键盘输入更新 [`GameInput`]，并将边沿事件与持续状态发送到服务器。
pub fn update_game_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut game_input: ResMut<GameInput>,
    time: Res<Time>,
    net: Res<crate::systems::network::NetworkResource>,
    mut net_sync: ResMut<NetworkInputSyncState>,
) {
    const JUMP_BUFFER_DURATION: f32 = 0.15;

    let current_time = time.elapsed_secs();
    let delta_seconds = time.delta_secs();

    // 更新移动输入
    let new_move_left =
        keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft);
    let new_move_right =
        keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight);
    let new_jump = keyboard_input.pressed(KeyCode::KeyW)
        || keyboard_input.pressed(KeyCode::ArrowUp)
        || keyboard_input.pressed(KeyCode::Space);
    let new_jump_just_pressed = keyboard_input.just_pressed(KeyCode::KeyW)
        || keyboard_input.just_pressed(KeyCode::ArrowUp)
        || keyboard_input.just_pressed(KeyCode::Space);
    let new_crouch =
        keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown);

    // 同时按下左右时仅清空水平移动，不中断本帧其它输入更新（避免丢失跳跃/起身）。
    let (resolved_move_left, resolved_move_right) = if new_move_left && new_move_right {
        (false, false)
    } else {
        (new_move_left, new_move_right)
    };

    // 更新动作输入
    let new_action1 =
        keyboard_input.pressed(KeyCode::KeyJ) || keyboard_input.pressed(KeyCode::KeyZ);
    let new_action2 = keyboard_input.pressed(KeyCode::KeyX);

    // 更新菜单输入
    let new_confirm =
        keyboard_input.pressed(KeyCode::Enter) || keyboard_input.pressed(KeyCode::Space);
    let new_cancel =
        keyboard_input.pressed(KeyCode::KeyQ) || keyboard_input.pressed(KeyCode::Backspace);
    let new_pause = keyboard_input.just_pressed(KeyCode::Escape);

    // 边沿状态需要与上一帧比较。
    let old_jump = game_input.jump;
    let old_action1 = game_input.action1;
    let old_action2 = game_input.action2;

    game_input.action1_pressed_this_frame = new_action1 && !old_action1;
    game_input.action2_pressed_this_frame = new_action2 && !old_action2;
    game_input.jump_pressed_this_frame = new_jump_just_pressed || (new_jump && !old_jump);
    if game_input.jump_pressed_this_frame {
        game_input.jump_buffer_seconds = JUMP_BUFFER_DURATION;
    } else {
        game_input.jump_buffer_seconds = (game_input.jump_buffer_seconds - delta_seconds).max(0.0);
    }

    // 更新输入状态
    game_input.move_left = resolved_move_left;
    game_input.move_right = resolved_move_right;
    game_input.jump = new_jump;
    game_input.crouch = new_crouch;
    game_input.action1 = new_action1;
    game_input.action2 = new_action2;
    game_input.confirm = new_confirm;
    game_input.cancel = new_cancel;
    game_input.pause = new_pause;

    // Send actions to server
    if net.status == crate::systems::network::NetworkStatus::Connected
        && let Some(tx) = &net.action_tx
    {
        // Instant events use edge-triggered stream.
        if new_jump && !old_jump {
            let sequence = net_sync.next_sequence;
            net_sync.next_sequence = net_sync.next_sequence.wrapping_add(1);
            let _ = tx.send(crate::protocol::PlayerAction::InputEvent {
                sequence,
                kind: crate::protocol::InputEventKind::Jump,
            });
            net_sync.sent_event_count = net_sync.sent_event_count.wrapping_add(1);
        }

        if new_action1 && !old_action1 {
            let sequence = net_sync.next_sequence;
            net_sync.next_sequence = net_sync.next_sequence.wrapping_add(1);
            let _ = tx.send(crate::protocol::PlayerAction::InputEvent {
                sequence,
                kind: crate::protocol::InputEventKind::Attack,
            });
            net_sync.sent_event_count = net_sync.sent_event_count.wrapping_add(1);
        }

        // Continuous movement uses state stream with delta + throttle.
        let x = if resolved_move_right {
            1.0
        } else if resolved_move_left {
            -1.0
        } else {
            0.0
        };
        let y = if new_crouch { -1.0 } else { 0.0 };

        let state_changed = (x - net_sync.last_sent_move_x).abs() > f32::EPSILON
            || (y - net_sync.last_sent_move_y).abs() > f32::EPSILON;
        let throttle_expired =
            current_time - net_sync.last_state_send_time >= net_sync.state_send_interval_secs;
        if state_changed || throttle_expired {
            let sequence = net_sync.next_sequence;
            net_sync.next_sequence = net_sync.next_sequence.wrapping_add(1);
            let _ = tx.send(crate::protocol::PlayerAction::InputState { sequence, x, y });
            net_sync.last_sent_move_x = x;
            net_sync.last_sent_move_y = y;
            net_sync.last_state_send_time = current_time;
            net_sync.sent_state_count = net_sync.sent_state_count.wrapping_add(1);
        }
    }
}

/// 输入辅助函数
impl GameInput {
    /// 获取水平移动输入
    pub fn get_horizontal_input(&self) -> f32 {
        let mut input = 0.0;
        if self.move_left {
            input -= 1.0;
        }
        if self.move_right {
            input += 1.0;
        }
        input
    }

    /// 重置所有输入状态
    ///
    /// 用于状态切换时清理输入状态。
    pub fn reset(&mut self) {
        self.move_left = false;
        self.move_right = false;
        self.jump = false;
        self.crouch = false;
        self.confirm = false;
        self.cancel = false;
        self.pause = false;
        self.action1 = false;
        self.action2 = false;
        self.action1_pressed_this_frame = false;
        self.action2_pressed_this_frame = false;
        self.jump_pressed_this_frame = false;
        self.jump_buffer_seconds = 0.0;
    }
}
