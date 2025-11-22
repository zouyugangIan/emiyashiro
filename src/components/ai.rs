use bevy::prelude::*;

/// Bot 控制器组件
/// 用于自动控制角色的巡逻和跳跃行为
#[derive(Component)]
pub struct BotController {
    /// 巡逻区域最小 X 坐标
    pub patrol_min_x: f32,
    /// 巡逻区域最大 X 坐标
    pub patrol_max_x: f32,
    /// 当前移动方向 (-1.0 或 1.0)
    pub direction: f32,
    /// 跳跃计时器
    pub jump_timer: f32,
    /// 跳跃间隔
    pub jump_interval: f32,
}

impl Default for BotController {
    fn default() -> Self {
        Self {
            patrol_min_x: 0.0,
            patrol_max_x: 500.0,
            direction: 1.0,
            jump_timer: 2.0,
            jump_interval: 3.0, // 每3秒跳一次
        }
    }
}
