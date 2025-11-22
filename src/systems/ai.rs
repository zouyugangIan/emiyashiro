use bevy::prelude::*;
use crate::components::player::PlayerInputState;
use crate::components::ai::BotController;

/// Controller trait - 输入源抽象接口
/// 允许键盘、网络或 AI 脚本控制角色
pub trait Controller {
    /// 获取当前帧的输入状态
    fn get_input(&mut self, transform: &Transform, time: f32) -> PlayerInputState;
}

/// 简单的 Bot 控制器 - 自动巡逻
/// 实现基本的左右移动和随机跳跃行为
impl Controller for BotController {
    fn get_input(&mut self, transform: &Transform, time: f32) -> PlayerInputState {
        let mut input = PlayerInputState::default();
        
        // 更新巡逻方向
        if transform.translation.x > self.patrol_max_x {
            self.direction = -1.0;
        } else if transform.translation.x < self.patrol_min_x {
            self.direction = 1.0;
        }
        
        // 设置移动输入
        input.move_x = self.direction;
        
        // 随机跳跃
        self.jump_timer -= time;
        if self.jump_timer <= 0.0 {
            input.jump_pressed = true;
            self.jump_timer = self.jump_interval;
        }
        
        input
    }
}

/// Bot 控制系统
/// 每帧更新 Bot 的输入状态
pub fn bot_control_system(
    mut query: Query<(&Transform, &mut PlayerInputState, &mut BotController)>,
    time: Res<Time>,
) {
    let delta_time = time.delta_secs();
    
    for (transform, mut input, mut bot) in query.iter_mut() {
        *input = bot.get_input(transform, delta_time);
    }
}
