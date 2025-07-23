use bevy::prelude::*;

mod components;
mod resources;
mod states;
mod systems;
mod database;

// use components::*;
use resources::*;
use states::*;
use systems::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Fate/stay night Heaven's Feel - Shirou Runner".into(),
                resolution: (1024.0, 768.0).into(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<GameState>()
        .init_resource::<CharacterSelection>()
        .init_resource::<GameStats>()
        .add_systems(Startup, setup_game_resources)
        .add_systems(OnEnter(GameState::Menu), menu::setup_menu)
        .add_systems(
            Update,
            (
                menu::handle_start_button,
                menu::handle_character_select,
                menu::handle_save_button,
                menu::cover_fade_animation,
            ).run_if(in_state(GameState::Menu))
        )
        .add_systems(OnExit(GameState::Menu), menu::cleanup_menu)
        .add_systems(OnEnter(GameState::Playing), game::setup_game)
        .add_systems(
            Update,
            (
                game::handle_game_input,
                player::player_movement,
                player::player_jump,
                player::player_crouch,
                player::update_player_state,
                player::update_game_stats,
                camera::camera_follow,
            ).run_if(in_state(GameState::Playing))
        )
        .add_systems(OnExit(GameState::Playing), game::cleanup_game)
        .run();
}

/// 加载游戏资源
fn setup_game_resources(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // 加载游戏资源
    let game_assets = GameAssets {
        cover_texture: asset_server.load("images/ui/cover1.jpg"),
        cover2_texture: asset_server.load("images/ui/cover2.jpg"),
        shirou1_texture: asset_server.load("images/characters/shirou_idle1.jpg"),
        shirou2_texture: asset_server.load("images/characters/shirou_idle2.jpg"),
        font: Handle::default(), // 使用默认字体
    };
    
    commands.insert_resource(game_assets);
}