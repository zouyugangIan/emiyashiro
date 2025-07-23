use bevy::prelude::*;
use crate::resources::GameAssets;

/// 加载游戏资源
pub fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let game_assets = GameAssets {
        shirou_idle1: asset_server.load("images/characters/shirou_idle1.jpg"),
        shirou_idle2: asset_server.load("images/characters/shirou_idle2.jpg"),
        cover1: asset_server.load("images/ui/cover1.jpg"),
        cover2: asset_server.load("images/ui/cover2.jpg"),
        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
    };
    
    commands.insert_resource(game_assets);
}

/// 设置摄像机
pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}