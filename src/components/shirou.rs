use bevy::prelude::*;

/// 圣骸布状态
/// 
/// 控制卫宫士郎的"圣骸布解放"机制。
/// 解放后攻击范围增加（Overedge），但会持续扣血。
/// 
/// # 字段
/// * `is_released` - 圣骸布是否已解开
/// * `health_drain_timer` - 扣血计时器
/// * `health_drain_amount` - 每次扣血量
#[derive(Component, Debug, Clone)] // derive Clone for simple fields
pub struct ShroudState {
    pub is_released: bool,
    pub health_drain_timer: Timer,
    pub health_drain_amount: f32,
}

impl Default for ShroudState {
    fn default() -> Self {
        Self {
            is_released: false,
            // 每 0.5 秒扣一次血
            health_drain_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
            // 每次扣 2 点
            health_drain_amount: 2.0,
        }
    }
}

impl ShroudState {
    pub fn toggle(&mut self) -> bool {
        self.is_released = !self.is_released;
        self.is_released
    }
}
