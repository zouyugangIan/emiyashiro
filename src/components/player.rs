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
/// use emiyashiro::components::Player;
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
/// use emiyashiro::components::PlayerState;
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

/// 玩家朝向状态
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FacingDirection {
    Left,
    #[default]
    Right,
}

impl FacingDirection {
    pub fn sign(self) -> f32 {
        match self {
            Self::Left => -1.0,
            Self::Right => 1.0,
        }
    }

    pub fn from_horizontal_input(move_left: bool, move_right: bool, fallback: Self) -> Self {
        if move_left && !move_right {
            Self::Left
        } else if move_right && !move_left {
            Self::Right
        } else {
            fallback
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

/// 玩家受击无敌帧
///
/// 用于降低连续接触伤害的挫败感。
#[derive(Component, Debug, Clone)]
pub struct DamageInvulnerability {
    pub remaining: f32,
}

impl Default for DamageInvulnerability {
    fn default() -> Self {
        Self { remaining: 0.0 }
    }
}

impl DamageInvulnerability {
    pub fn trigger(&mut self, duration: f32) {
        self.remaining = self.remaining.max(duration);
    }

    pub fn tick(&mut self, delta_secs: f32) {
        self.remaining = (self.remaining - delta_secs).max(0.0);
    }

    pub fn is_active(&self) -> bool {
        self.remaining > 0.0
    }
}

/// 玩家输入状态（服务端使用）
#[derive(Component, Default, Debug, Clone)]
pub struct PlayerInputState {
    pub move_x: f32,
    pub move_y: f32,
    pub jump_pressed: bool,
}
