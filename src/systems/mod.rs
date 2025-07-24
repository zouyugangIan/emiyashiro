//! 游戏系统模块
//! 
//! 本模块包含游戏中所有的系统定义，按功能分类组织。

// 系统接口和配置
pub mod interfaces;

// 核心游戏系统
pub mod game;
pub mod setup;

// 玩家相关系统
pub mod player;
pub mod input;

// 渲染和视觉系统
pub mod camera;
pub mod animation;
pub mod frame_animation;
pub mod sprite_animation;

// 物理和碰撞系统
pub mod collision;

// UI系统
pub mod menu;
pub mod ui;

// 音频系统
pub mod audio;

// 数据持久化系统
pub mod save;
pub mod database_service;

// 资源生成系统
pub mod procedural_assets;

// 重新导出常用系统，保持向后兼容性
pub use interfaces::*;
pub use game::*;
pub use setup::*;
pub use player::*;
pub use input::*;
pub use camera::*;
pub use animation::*;
pub use collision::*;
pub use menu::*;
pub use ui::*;
pub use audio::*;
pub use save::*;