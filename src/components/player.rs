//! 玩家相关组件
//!
//! 包含玩家实体的标记组件和状态组件。

use bevy::prelude::*;

/// 玩家组件标记
///
/// 用于标识玩家实体，使系统能够查询和操作玩家相关的实体。
/// 这是一个标记组件，不包含数据，仅用于实体识别。
///
/// # 示例
///
/// ```rust
/// use bevy::prelude::*;
/// use s_emiyashiro::components::Player;
///
/// fn spawn_player(mut commands: Commands) {
///     commands.spawn((
///         Player,
///         // 其他组件...
///     ));
/// }
/// ```
#[derive(Component, Debug)]
pub struct Player;

/// 玩家状态组件
///
/// 跟踪玩家的当前状态，用于控制动画和物理行为。
///
/// # 字段
/// * `is_grounded` - 玩家是否在地面上
/// * `is_crouching` - 玩家是否在蹲下
///
/// # 示例
///
/// ```rust
/// use s_emiyashiro::components::PlayerState;
///
/// let mut player_state = PlayerState::default();
/// if player_state.can_jump() {
///     println!("玩家可以跳跃");
/// }
/// ```
#[derive(Component, Debug, Clone)]
pub struct PlayerState {
    pub is_grounded: bool,
    pub is_crouching: bool,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            is_grounded: true,
            is_crouching: false,
        }
    }
}

impl PlayerState {
    /// 创建新的玩家状态
    ///
    /// # 参数
    /// * `is_grounded` - 是否在地面上
    /// * `is_crouching` - 是否在蹲下
    ///
    /// # 返回
    /// 新的 PlayerState 实例
    pub fn new(is_grounded: bool, is_crouching: bool) -> Self {
        Self {
            is_grounded,
            is_crouching,
        }
    }

    /// 检查玩家是否可以跳跃
    ///
    /// 玩家只有在地面上且不在蹲下状态时才能跳跃。
    ///
    /// # 返回
    /// 如果玩家可以跳跃返回 true，否则返回 false
    pub fn can_jump(&self) -> bool {
        self.is_grounded && !self.is_crouching
    }

    /// 设置玩家为地面状态
    pub fn set_grounded(&mut self, grounded: bool) {
        self.is_grounded = grounded;
    }

    /// 设置玩家蹲下状态
    pub fn set_crouching(&mut self, crouching: bool) {
        self.is_crouching = crouching;
    }
}

/// 玩家输入状态（服务端使用）
#[derive(Component, Default, Debug, Clone)]
pub struct PlayerInputState {
    pub move_x: f32,
    pub move_y: f32,
    pub jump_pressed: bool,
}
