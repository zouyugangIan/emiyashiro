use bevy::prelude::*;
use std::time::Duration;

/// 圣骸布状态
///
/// 控制卫宫士郎的"圣骸布解放"机制。
/// 解放后攻击切换为 Overedge，持续固定时间后自动恢复。
///
/// # 字段
/// * `is_released` - 圣骸布是否已解开
/// * `overedge_timer` - Overedge 持续计时器
/// * `toggle_health_cost` - 每次切换时扣除的生命值
#[derive(Component, Debug, Clone)] // derive Clone for simple fields
pub struct ShroudState {
    pub is_released: bool,
    pub overedge_timer: Timer,
    pub toggle_health_cost: f32,
}

impl Default for ShroudState {
    fn default() -> Self {
        let mut overedge_timer = Timer::from_seconds(Self::OVEREDGE_DURATION_SECS, TimerMode::Once);
        overedge_timer.pause();

        Self {
            is_released: false,
            // 每次解放持续 10 秒
            overedge_timer,
            // 每次切换扣 2HP
            toggle_health_cost: 2.0,
        }
    }
}

impl ShroudState {
    pub const OVEREDGE_DURATION_SECS: f32 = 3.0;

    pub fn toggle(&mut self) -> bool {
        if self.is_released {
            self.disable_release();
        } else {
            self.enable_release();
        }
        self.is_released
    }

    pub fn enable_release(&mut self) {
        self.is_released = true;
        self.overedge_timer.reset();
        self.overedge_timer.unpause();
    }

    pub fn disable_release(&mut self) {
        self.is_released = false;
        self.overedge_timer.pause();
        self.overedge_timer.reset();
    }

    /// 仅在解放状态下推进计时，返回值表示本帧是否刚过期。
    pub fn tick(&mut self, delta: Duration) -> bool {
        if !self.is_released {
            return false;
        }

        self.overedge_timer.tick(delta);
        if self.overedge_timer.just_finished() || self.overedge_timer.is_finished() {
            self.disable_release();
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_enters_release_state_with_fresh_timer() {
        let mut shroud = ShroudState::default();
        assert!(!shroud.is_released);

        shroud.toggle();
        assert!(shroud.is_released);
        assert!(!shroud.overedge_timer.is_finished());
    }

    #[test]
    fn tick_expires_release_after_duration() {
        let mut shroud = ShroudState::default();
        shroud.enable_release();

        let expired = shroud.tick(Duration::from_secs_f32(ShroudState::OVEREDGE_DURATION_SECS));
        assert!(expired);
        assert!(!shroud.is_released);
    }
}
