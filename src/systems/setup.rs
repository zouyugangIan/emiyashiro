//! 游戏初始化系统
//!
//! 包含游戏启动时的资源加载和基础设置。

use crate::{
    asset_paths,
    resources::{GameAssets, GameplayTuning},
};
use bevy::prelude::*;

/// 加载游戏资源
///
/// 加载游戏所需的所有资源，包括图片、音频、字体等。
/// 创建 GameAssets 资源并插入到世界中供其他系统使用。
pub fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 加载所有UI封面图片
    let cover_textures: Vec<Handle<Image>> = asset_paths::UI_COVER_IMAGES
        .iter()
        .map(|path| asset_server.load(*path))
        .collect();

    // 加载所有Shirou动画帧
    let shirou_animation_frames: Vec<Handle<Image>> = asset_paths::SHIROU_ANIMATION_FRAMES
        .iter()
        .map(|path| asset_server.load(*path))
        .collect();

    // 加载所有Sakura动画帧
    let sakura_animation_frames: Vec<Handle<Image>> = asset_paths::SAKURA_ANIMATION_FRAMES
        .iter()
        .map(|path| asset_server.load(*path))
        .collect();

    crate::debug_log!("📦 加载资源:");
    crate::debug_log!("  - UI封面图片: {} 张", cover_textures.len());
    crate::debug_log!("  - Shirou动画帧: {} 帧", shirou_animation_frames.len());
    crate::debug_log!("  - Sakura动画帧: {} 帧", sakura_animation_frames.len());

    let game_assets = GameAssets {
        cover_textures,
        current_cover_index: 0,
        shirou_animation_frames,
        sakura_animation_frames,
        current_shirou_frame: 0,
        current_sakura_frame: 0,
        font: asset_server.load(asset_paths::FONT_FIRA_SANS),
        volume_icon: asset_server.load(asset_paths::IMAGE_UI_VOLUME_ICON),
        volume_muted_icon: asset_server.load(asset_paths::IMAGE_UI_VOLUME_MUTED_ICON),

        // 精灵表资源（可选）
        shirou_spritesheet: None,
        shirou_spritesheet_run: None,
        shirou_spritesheet_attack: None,
        shirou_spritesheet_overedge_light_attack: None,
        shirou_spritesheet_overedge_heavy_attack: None,
        sakura_spritesheet: None,
        shirou_atlas: None,
        shirou_atlas_run: None,
        shirou_atlas_attack: None,
        shirou_atlas_overedge_light_attack: None,
        shirou_atlas_overedge_heavy_attack: None,
        sakura_atlas: None,
        // Reference Board 精灵表
        shirou_ref_ground_light: None,
        shirou_ref_air_combo: None,
        shirou_ref_heavy: None,
        shirou_ref_ultimate: None,
        shirou_ref_mobility: None,
        shirou_ref_ninjutsu: None,
        shirou_ref_weapon_proj: None,
        shirou_ref_advance: None,
        shirou_ref_ground_light_rows: Vec::new(),
        shirou_ref_air_combo_rows: Vec::new(),
        shirou_ref_heavy_rows: Vec::new(),
        shirou_ref_ultimate_rows: Vec::new(),
        shirou_ref_mobility_rows: Vec::new(),
        shirou_ref_ninjutsu_rows: Vec::new(),
        shirou_ref_weapon_proj_rows: Vec::new(),
        shirou_atlas_ref_ground_light: None,
        shirou_atlas_ref_air_combo: None,
        shirou_atlas_ref_heavy: None,
        shirou_atlas_ref_ultimate: None,
        shirou_atlas_ref_mobility: None,
        shirou_atlas_ref_ninjutsu: None,
        shirou_atlas_ref_weapon_proj: None,
        shirou_atlas_ref_advance: None,

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
    crate::debug_log!("摄像机设置完成");
}

/// 加载可调节的玩法参数。
pub fn load_gameplay_tuning(mut commands: Commands) {
    commands.insert_resource(GameplayTuning::load_from_disk());
}
