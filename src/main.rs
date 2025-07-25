use bevy::prelude::*;

mod components;
mod resources;
mod states;
mod systems;
mod database;
mod tools;

#[cfg(test)]
mod tests;

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
        .init_resource::<AudioSettings>()
        .init_resource::<systems::audio::AudioManager>()
        .init_resource::<SaveManager>()
        .init_resource::<SaveFileManager>()
        .init_resource::<PauseManager>()
        .init_resource::<AudioStateManager>()
        .init_resource::<systems::database_service::DatabaseService>()
        .init_resource::<systems::input::GameInput>()
        .init_resource::<ui::SaveNameInput>()
        .init_resource::<ui::LoadedGameState>()
        .add_systems(Startup, (setup_game_resources, systems::save::load_game))
        .add_systems(OnEnter(GameState::Menu), menu::setup_menu)
        .add_systems(Update, systems::audio::play_menu_music.run_if(in_state(GameState::Menu)))
        .add_systems(
            Update,
            (
                menu::handle_start_button,
                menu::handle_load_button,
                menu::handle_character_select,
                systems::save::handle_save_button_click,
                menu::cover_fade_animation,
                systems::visual_effects::button_hover_effect,
            ).run_if(in_state(GameState::Menu))
        )
        .add_systems(OnExit(GameState::Menu), (menu::cleanup_menu, systems::audio::stop_menu_music))
        .add_systems(OnEnter(GameState::Playing), (game::setup_game, ui::setup_game_hud, systems::audio::play_game_music_and_stop_menu))
        .add_systems(
            Update,
            game::restore_loaded_game_entities.run_if(in_state(GameState::Playing))
        )
        .add_systems(
            Update,
            (
                // 核心游戏系统
                player::player_movement,
                player::player_jump,
                player::player_crouch,
                player::update_player_state,
                player::update_game_stats,
                camera::camera_follow,
                ui::update_game_hud,
            ).run_if(in_state(GameState::Playing))
        )
        .add_systems(
            Update,
            (
                // 动画和视觉效果系统
                animation::animate_character,
                animation::setup_character_animation,
                animation::trigger_audio_effects,
                animation::manage_background_music,
                systems::frame_animation::update_frame_animations,
                systems::frame_animation::update_character_animations,
                systems::frame_animation::setup_player_animation,
                systems::frame_animation::debug_animations,
            ).run_if(in_state(GameState::Playing))
        )
        .add_systems(
            Update,
            (
                // 视觉效果系统
                systems::visual_effects::trigger_jump_effect,
                systems::visual_effects::trigger_land_effect,
                systems::visual_effects::trigger_run_effect,
                systems::visual_effects::trigger_crouch_effect,
                systems::visual_effects::update_visual_effects,
                systems::visual_effects::update_blinking_text,
            ).run_if(in_state(GameState::Playing))
        )
        .add_systems(
            Update,
            (
                // 数据和存档系统
                systems::save::auto_save_system,
                systems::database_service::database_stats_system,
                systems::database_service::cleanup_old_sessions,
            ).run_if(in_state(GameState::Playing))
        )
        .add_systems(
            Update,
            (
                systems::input::update_game_input,
                systems::sprite_animation::update_sprite_animations,
                systems::sprite_animation::update_character_animation_state,
                systems::input::debug_input_system,
            ).run_if(in_state(GameState::Playing))
        )
        .add_systems(
            Update,
            systems::pause_save::handle_pause_input.run_if(in_state(GameState::Playing).or(in_state(GameState::Paused)))
        )
        .add_systems(OnExit(GameState::Playing), (game::cleanup_game, ui::cleanup_game_hud, systems::audio::stop_game_music))
        .add_systems(OnEnter(GameState::Paused), (ui::setup_pause_menu, systems::pause_save::scan_save_files))
        .add_systems(
            Update,
            (
                ui::handle_pause_menu_interactions,
                systems::pause_save::restore_paused_state,
                systems::audio::maintain_audio_during_pause,
            ).run_if(in_state(GameState::Paused))
        )
        .add_systems(OnExit(GameState::Paused), ui::cleanup_pause_menu)
        .add_systems(OnEnter(GameState::SaveDialog), ui::setup_save_dialog)
        .add_systems(
            Update,
            (
                ui::handle_save_name_input,
                ui::handle_save_dialog_interactions,
            ).run_if(in_state(GameState::SaveDialog))
        )
        .add_systems(OnExit(GameState::SaveDialog), ui::cleanup_save_dialog)
        .add_systems(OnEnter(GameState::LoadTable), (ui::setup_load_table, systems::pause_save::scan_save_files))
        .add_systems(
            Update,
            ui::handle_load_table_interactions.run_if(in_state(GameState::LoadTable))
        )
        .add_systems(OnExit(GameState::LoadTable), ui::cleanup_load_table)
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
        shirou2_texture: asset_server.load("images/characters/sakura_idle1.jpg"),
        font: asset_server.load("fonts/FiraSans-Bold.ttf"), // 使用现有字体，暂时不支持中文
        // 精灵表资源 - 初始化为空，稍后设置
        shirou_spritesheet: None,
        sakura_spritesheet: None,
        shirou_atlas: None,
        sakura_atlas: None,
        // 音效资源 - 暂时使用占位符，后续可以替换为实际音效文件
        jump_sound: asset_server.load("sounds/jump.ogg"),
        land_sound: asset_server.load("sounds/land.ogg"),
        menu_music: asset_server.load("sounds/menu.ogg"),
        game_music: asset_server.load("sounds/game.ogg"),
        footstep_sound: asset_server.load("sounds/footstep.ogg"),
        background_music: asset_server.load("sounds/background.ogg"),
    };
    
    commands.insert_resource(game_assets);
}