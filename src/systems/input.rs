//! 输入处理系统
//!
//! 包含游戏输入的捕获、处理和状态管理。
//! 支持键盘输入、连招检测和输入历史记录。

use bevy::prelude::*;

/// 游戏输入资源
///
/// 存储当前的输入状态和输入历史，用于游戏逻辑处理。
/// 包含移动、跳跃、蹲下等基本操作，以及连招检测功能。
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
    pub action1: bool, // 投影魔术
    pub action2: bool, // 特殊技能
    pub action1_pressed_this_frame: bool,
    pub jump_pressed_this_frame: bool,
    pub jump_buffer_seconds: f32,

    // 输入历史（用于连招检测）
    pub input_history: Vec<InputEvent>,

    // 输入过滤器
    pub input_filter: InputFilter,
}

/// 输入事件
#[derive(Debug, Clone)]
pub struct InputEvent {
    pub input_type: InputType,
    pub timestamp: f32,
    pub pressed: bool,
}

/// 输入类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum InputType {
    MoveLeft,
    MoveRight,
    Jump,
    Crouch,
    Action1,
    Action2,
    Confirm,
    Cancel,
    Pause,
}

/// 更新游戏输入系统
///
/// 从键盘输入更新 GameInput 资源，并进行输入验证和错误处理。
pub fn update_game_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut game_input: ResMut<GameInput>,
    time: Res<Time>,
    net: Res<crate::systems::network::NetworkResource>,
) {
    const JUMP_BUFFER_DURATION: f32 = 0.15;

    let current_time = time.elapsed_secs();
    let delta_seconds = time.delta_secs();

    // 验证当前输入状态（静默处理错误，避免日志污染）
    if let Err(error) = game_input.validate_input_state() {
        match error {
            InputValidationError::HistoryTooLong => {
                game_input.cleanup_history(current_time, 1.0);
            }
            InputValidationError::InvalidTimestamp => {
                game_input.input_history.clear();
            }
            InputValidationError::InputTooFrequent => {
                return;
            }
            _ => {}
        }
    }

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
    let new_action2 =
        keyboard_input.pressed(KeyCode::KeyK) || keyboard_input.pressed(KeyCode::KeyX);

    // 更新菜单输入
    let new_confirm =
        keyboard_input.pressed(KeyCode::Enter) || keyboard_input.pressed(KeyCode::Space);
    let new_cancel =
        keyboard_input.pressed(KeyCode::KeyQ) || keyboard_input.pressed(KeyCode::Backspace);
    let new_pause = keyboard_input.just_pressed(KeyCode::Escape);

    // 记录输入变化到历史（先保存旧值）
    let old_move_left = game_input.move_left;
    let old_move_right = game_input.move_right;
    let old_jump = game_input.jump;
    let old_crouch = game_input.crouch;
    let old_action1 = game_input.action1;
    let old_action2 = game_input.action2;

    game_input.action1_pressed_this_frame = new_action1 && !old_action1;
    game_input.jump_pressed_this_frame = new_jump_just_pressed || (new_jump && !old_jump);
    if game_input.jump_pressed_this_frame {
        game_input.jump_buffer_seconds = JUMP_BUFFER_DURATION;
    } else {
        game_input.jump_buffer_seconds = (game_input.jump_buffer_seconds - delta_seconds).max(0.0);
    }

    record_input_change(
        &mut game_input,
        InputType::MoveLeft,
        old_move_left,
        resolved_move_left,
        current_time,
    );
    record_input_change(
        &mut game_input,
        InputType::MoveRight,
        old_move_right,
        resolved_move_right,
        current_time,
    );
    record_input_change(
        &mut game_input,
        InputType::Jump,
        old_jump,
        new_jump,
        current_time,
    );
    record_input_change(
        &mut game_input,
        InputType::Crouch,
        old_crouch,
        new_crouch,
        current_time,
    );
    record_input_change(
        &mut game_input,
        InputType::Action1,
        old_action1,
        new_action1,
        current_time,
    );
    record_input_change(
        &mut game_input,
        InputType::Action2,
        old_action2,
        new_action2,
        current_time,
    );

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

    // 注意：不再对持续输入进行过滤
    // 输入过滤器会导致正常的游戏输入被错误地过滤
    // 只在输入历史中记录变化，不对持续按住的按键进行限制

    // 定期清理输入历史（保留最近2秒）
    game_input.cleanup_history(current_time, 2.0);

    // Send actions to server
    if net.status == crate::systems::network::NetworkStatus::Connected
        && let Some(tx) = &net.action_tx
    {
        // Jump
        if new_jump && !old_jump {
            let _ = tx.send(crate::protocol::PlayerAction::Jump);
        }

        // Attack
        if new_action1 && !old_action1 {
            let _ = tx.send(crate::protocol::PlayerAction::Attack);
        }

        // Move
        if (resolved_move_left != old_move_left)
            || (resolved_move_right != old_move_right)
            || (new_crouch != old_crouch)
        {
            let x = if resolved_move_right {
                1.0
            } else if resolved_move_left {
                -1.0
            } else {
                0.0
            };
            let y = if new_crouch { -1.0 } else { 0.0 };
            let _ = tx.send(crate::protocol::PlayerAction::Move { x, y });
        }
    }
}

/// 记录输入变化
fn record_input_change(
    game_input: &mut GameInput,
    input_type: InputType,
    old_state: bool,
    new_state: bool,
    timestamp: f32,
) {
    if old_state != new_state {
        game_input.input_history.push(InputEvent {
            input_type,
            timestamp,
            pressed: new_state,
        });
    }
}

/// 检测连招输入
pub fn detect_combo_input(game_input: &GameInput) -> Option<ComboType> {
    let recent_inputs: Vec<&InputEvent> = game_input
        .input_history
        .iter()
        .filter(|event| event.pressed) // 只考虑按下事件
        .collect();

    if recent_inputs.len() < 2 {
        return None;
    }

    // 检测双击跳跃（空中冲刺）
    if recent_inputs.len() >= 2 {
        let last_two = &recent_inputs[recent_inputs.len() - 2..];
        if matches!(last_two[0].input_type, InputType::Jump)
            && matches!(last_two[1].input_type, InputType::Jump)
            && last_two[1].timestamp - last_two[0].timestamp < 0.3
        {
            return Some(ComboType::DoubleJump);
        }
    }

    // 检测投影魔术连招 (下+动作1)
    if recent_inputs.len() >= 2 {
        let last_two = &recent_inputs[recent_inputs.len() - 2..];
        if matches!(last_two[0].input_type, InputType::Crouch)
            && matches!(last_two[1].input_type, InputType::Action1)
            && last_two[1].timestamp - last_two[0].timestamp < 0.5
        {
            return Some(ComboType::ProjectionMagic);
        }
    }

    None
}

/// 连招类型
#[derive(Debug, Clone, PartialEq)]
pub enum ComboType {
    DoubleJump,      // 双击跳跃
    ProjectionMagic, // 投影魔术
    SwordThrow,      // 剑投掷
}

/// 输入辅助函数
impl GameInput {
    /// 检查是否刚按下某个输入
    pub fn just_pressed(&self, input_type: InputType, current_time: f32) -> bool {
        self.input_history.iter().any(|event| {
            event.input_type == input_type && event.pressed && current_time - event.timestamp < 0.1
        })
    }

    /// 检查是否刚释放某个输入
    pub fn just_released(&self, input_type: InputType, current_time: f32) -> bool {
        self.input_history.iter().any(|event| {
            event.input_type == input_type && !event.pressed && current_time - event.timestamp < 0.1
        })
    }

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

    /// 验证输入状态的一致性
    ///
    /// 检查输入状态是否有效，用于调试和错误检测。
    pub fn validate_input_state(&self) -> Result<(), InputValidationError> {
        // 检查输入历史是否过长
        if self.input_history.len() > 1000 {
            return Err(InputValidationError::HistoryTooLong);
        }

        // 检查时间戳是否单调递增
        for window in self.input_history.windows(2) {
            if window[0].timestamp > window[1].timestamp {
                return Err(InputValidationError::InvalidTimestamp);
            }
        }

        // 检查无效的输入组合
        if self.validate_input_combinations().is_err() {
            return Err(InputValidationError::InvalidInputCombination);
        }

        Ok(())
    }

    /// 验证输入组合的有效性
    ///
    /// 检查当前输入组合是否合理，防止无效的输入状态。
    fn validate_input_combinations(&self) -> Result<(), InputValidationError> {
        // 检查相互冲突的输入
        if self.move_left && self.move_right {
            return Err(InputValidationError::StateConflict);
        }

        // 检查逻辑上不合理的组合
        // 例如：同时确认和取消（在某些上下文中可能不合理）
        if self.confirm && self.cancel {
            return Err(InputValidationError::StateConflict);
        }

        Ok(())
    }

    /// 过滤输入
    ///
    /// 根据输入过滤器的设置过滤输入，防止过于频繁的输入。
    pub fn filter_input(&mut self, current_time: f32) -> Result<(), InputValidationError> {
        // 只有在有输入时才进行过滤检查
        if self.has_any_input() {
            self.input_filter.should_accept_input(current_time)?;
        }

        Ok(())
    }

    /// 清理过期的输入历史
    ///
    /// 移除超过指定时间的输入事件，防止内存泄漏。
    pub fn cleanup_history(&mut self, current_time: f32, max_age: f32) {
        self.input_history
            .retain(|event| current_time - event.timestamp < max_age);
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
        self.jump_pressed_this_frame = false;
        self.jump_buffer_seconds = 0.0;
        self.input_history.clear();
        self.input_filter.reset();
    }

    /// 检查是否有任何输入激活
    pub fn has_any_input(&self) -> bool {
        self.move_left
            || self.move_right
            || self.jump
            || self.crouch
            || self.confirm
            || self.cancel
            || self.pause
            || self.action1
            || self.action2
    }
}

/// 输入验证错误类型
#[derive(Debug, Clone, PartialEq)]
pub enum InputValidationError {
    /// 输入历史过长
    HistoryTooLong,
    /// 无效的时间戳
    InvalidTimestamp,
    /// 输入状态冲突
    StateConflict,
    /// 输入频率过高
    InputTooFrequent,
    /// 无效的输入组合
    InvalidInputCombination,
}

/// 输入过滤器
///
/// 用于过滤和验证输入，防止无效或恶意输入。
#[derive(Debug, Clone)]
pub struct InputFilter {
    /// 最小输入间隔（秒）
    pub min_input_interval: f32,
    /// 最大输入频率（每秒）
    pub max_input_frequency: f32,
    /// 上次输入时间
    pub last_input_time: f32,
    /// 输入计数器（用于频率检测）
    pub input_count: u32,
    /// 计数器重置时间
    pub counter_reset_time: f32,
}

impl Default for InputFilter {
    fn default() -> Self {
        Self {
            min_input_interval: 0.01,   // 10ms 最小间隔
            max_input_frequency: 100.0, // 每秒最多100次输入
            last_input_time: 0.0,
            input_count: 0,
            counter_reset_time: 0.0,
        }
    }
}

impl InputFilter {
    /// 创建新的输入过滤器
    pub fn new(min_interval: f32, max_frequency: f32) -> Self {
        Self {
            min_input_interval: min_interval,
            max_input_frequency: max_frequency,
            ..Default::default()
        }
    }

    /// 检查输入是否应该被接受
    pub fn should_accept_input(&mut self, current_time: f32) -> Result<(), InputValidationError> {
        // 检查输入间隔
        if current_time - self.last_input_time < self.min_input_interval {
            return Err(InputValidationError::InputTooFrequent);
        }

        // 重置计数器（每秒重置一次）
        if current_time - self.counter_reset_time >= 1.0 {
            self.input_count = 0;
            self.counter_reset_time = current_time;
        }

        // 检查输入频率
        if self.input_count as f32 >= self.max_input_frequency {
            return Err(InputValidationError::InputTooFrequent);
        }

        // 更新状态
        self.last_input_time = current_time;
        self.input_count += 1;

        Ok(())
    }

    /// 重置过滤器状态
    pub fn reset(&mut self) {
        self.last_input_time = 0.0;
        self.input_count = 0;
        self.counter_reset_time = 0.0;
    }
}

/// 显示输入调试信息（仅在需要调试时启用）
#[allow(dead_code)]
pub fn debug_input_system(game_input: Res<GameInput>, mut timer: Local<Timer>, time: Res<Time>) {
    if timer.duration().is_zero() {
        timer.set_duration(std::time::Duration::from_secs(5)); // 降低频率到5秒
        timer.set_mode(bevy::time::TimerMode::Repeating);
    }
    timer.tick(time.delta());

    if timer.just_finished() {
        let horizontal = game_input.get_horizontal_input();
        if horizontal != 0.0
            || game_input.jump
            || game_input.crouch
            || game_input.action1
            || game_input.action2
        {
            crate::debug_log!(
                "🎮 输入状态: 水平={:.1}, 跳跃={}, 蹲下={}, 动作1={}, 动作2={}",
                horizontal,
                game_input.jump,
                game_input.crouch,
                game_input.action1,
                game_input.action2
            );
        }

        // 检测连招
        if let Some(combo) = detect_combo_input(&game_input) {
            crate::debug_log!("⚡ 连招检测: {:?}", combo);
        }
    }
}

/// 输入健康检查系统
///
/// 定期检查输入系统的健康状态，确保输入处理正常。
pub fn input_health_check_system(
    mut game_input: ResMut<GameInput>,
    mut timer: Local<Timer>,
    time: Res<Time>,
) {
    // 每5秒进行一次健康检查
    if timer.duration().is_zero() {
        timer.set_duration(std::time::Duration::from_secs(5));
        timer.set_mode(bevy::time::TimerMode::Repeating);
    }
    timer.tick(time.delta());

    if timer.just_finished() {
        let current_time = time.elapsed_secs();

        // 验证输入状态（静默修复）
        if let Err(error) = game_input.validate_input_state() {
            match error {
                InputValidationError::HistoryTooLong => {
                    game_input.cleanup_history(current_time, 2.0);
                }
                InputValidationError::InvalidTimestamp => {
                    game_input.input_history.clear();
                }
                InputValidationError::StateConflict => {
                    // 重置冲突的输入
                    if game_input.move_left && game_input.move_right {
                        game_input.move_left = false;
                        game_input.move_right = false;
                    }
                    if game_input.confirm && game_input.cancel {
                        game_input.confirm = false;
                        game_input.cancel = false;
                    }
                }
                _ => {}
            }
        }
    }
}
