//! 游戏初始化系统
//! 
//! 包含游戏启动时的资源加载和基础设置。

use bevy::prelude::*;
use crate::resources::GameAssets;

/// 加载游戏资源
/// 
/// 加载游戏所需的所有资源，包括图片、音频、字体等。
/// 创建 GameAssets 资源并插入到世界中供其他系统使用。
pub fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let game_assets = GameAssets {
        cover_texture: asset_server.load("images/ui/cover1.jpg"),
        cover2_texture: asset_server.load("images/ui/cover2.jpg"),
        shirou1_texture: asset_server.load("images/characters/shirou_idle1.jpg"),
        shirou2_texture: asset_server.load("images/characters/shirou_idle2.jpg"),
        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
        
        // 精灵表资源（可选）
        shirou_spritesheet: None,
        sakura_spritesheet: None,
        shirou_atlas: None,
        sakura_atlas: None,
        
        // 音效资源
        jump_sound: asset_server.load("sounds/jump.ogg"),
        land_sound: asset_server.load("sounds/land.ogg"),
        footstep_sound: asset_server.load("sounds/footstep.ogg"),
        
        // 背景音乐
        menu_music: asset_server.load("sounds/menu_music.ogg"),
        game_music: asset_server.load("sounds/game_music.ogg"),
        background_music: asset_server.load("sounds/background_music.ogg"),
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