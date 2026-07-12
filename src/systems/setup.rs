//! 游戏初始化系统
//!
//! 包含游戏启动时的资源加载和基础设置。

use crate::resources::GameplayTuning;
use bevy::prelude::*;

/// 加载可调节的玩法参数。
pub fn load_gameplay_tuning(mut commands: Commands) {
    commands.insert_resource(GameplayTuning::load_from_disk());
}
