//! 游戏系统模块
//!
//! 本模块包含游戏中所有的系统定义，按功能分类组织。

// 系统接口和配置
pub mod interfaces;

// 核心游戏系统
pub mod game;
pub mod setup;

// 玩家相关系统
pub mod input;
pub mod player;

// 渲染和视觉系统
pub mod animation;
pub mod camera;
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
pub mod async_file_ops;
#[cfg(feature = "server")]
pub mod server_file_ops;
pub mod async_tasks;
pub mod database_service;
pub mod pause_save;
pub mod save;
pub mod shared_utils;

// 资源生成系统
pub mod background;
pub mod procedural_assets;

// 视觉效果系统
pub mod visual_effects;

// 文本常量系统
pub mod text_constants;

// 文本输入系统
pub mod text_input;

// 错误处理系统
pub mod error_handling;

// 系统调度配置
pub mod system_sets;

// 网络系统
pub mod network;
#[cfg(feature = "server")]
pub mod sync_redis;
#[cfg(feature = "server")]
pub mod save_worker;
pub mod ai;

// 最终集成
pub mod final_integration;
