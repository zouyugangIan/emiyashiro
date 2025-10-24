//! 游戏初始化系统
//!
//! 包含游戏启动时的资源加载和基础设置。

use crate::{asset_paths, resources::GameAssets};
use bevy::prelude::*;

/// 加载游戏资源
///
/// 加载游戏所需的所有资源，包括图片、音频、字体等。
/// 创建 GameAssets 资源并插入到世界中供其他系统使用。
pub fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let game_assets = GameAssets {
        cover_texture: asset_server.load(asset_paths::IMAGE_UI_COVER1),
        cover2_texture: asset_server.load(asset_paths::IMAGE_UI_COVER2),
        shirou1_texture: asset_server.load(asset_paths::IMAGE_CHAR_SHIROU_IDLE1),
        shirou2_texture: asset_server.load(asset_paths::IMAGE_CHAR_SHIROU_IDLE2),
        font: asset_server.load(asset_paths::FONT_FIRA_SANS),

        // 精灵表资源（可选）
        shirou_spritesheet: None,
        sakura_spritesheet: None,
        shirou_atlas: None,
        sakura_atlas: None,

        // 音效资源
        jump_sound: asset_server.load(asset_paths::SOUND_JUMP),
        land_sound: asset_server.load(asset_paths::SOUND_LAND),
        footstep_sound: asset_server.load(asset_paths::SOUND_FOOTSTEP),

        // 背景音乐
        menu_music: asset_server.load(asset_paths::SOUND_MENU_MUSIC),
        game_music: asset_server.load(asset_paths::SOUND_GAME_MUSIC),
        game_whyifight_music: asset_server.load(asset_paths::SOUND_GAME_WHY_I_FIGHT_MUSIC),
        background_music: asset_server.load(asset_paths::SOUND_BACKGROUND_MUSIC),
    };

    commands.insert_resource(game_assets);
}

/// 设置摄像机
///
/// 创建游戏的主摄像机，用于渲染游戏场景。
/// 使用 2D 摄像机配置，适合横版游戏。
pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
    println!("摄像机设置完成");
}
