//! 游戏组件模块
//! 
//! 本模块包含游戏中所有的组件定义，按功能分类组织。

pub mod player;
pub mod physics;
pub mod ui;
pub mod animation;
pub mod audio;

// 重新导出所有组件，保持向后兼容性
pub use player::*;
pub use physics::*;
pub use ui::*;
pub use animation::*;
pub use audio::*;