//! 游戏组件模块
//! 本模块包含游戏中所有的组件定义，按功能分类组织。

pub mod ai;
pub mod animation;
pub mod animation_data;
pub mod audio;
pub mod enemy;
pub mod health;
pub mod level;
pub mod network;
pub mod physics;
pub mod player;
pub mod projectile;
pub mod shirou;
pub mod ui;

// 对外导出常用组件。
pub use animation::*;
pub use animation_data::*;
pub use audio::*;
pub use enemy::*;
pub use health::*;
pub use level::*;
pub use physics::*;
pub use player::*;
pub use projectile::*;
pub use shirou::*;
pub use ui::*;
