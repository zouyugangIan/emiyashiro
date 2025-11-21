//! 游戏组件模块
//!
//! 本模块包含游戏中所有的组件定义，按功能分类组织。

pub mod animation;
pub mod audio;
pub mod physics;
pub mod player;
pub mod ui;
pub mod network;
pub mod ai;

// 重新导出所有组件，保持向后兼容性
pub use animation::*;
pub use audio::*;
pub use physics::*;
pub use player::*;
pub use ui::*;
pub mod background;
pub use background::*;
