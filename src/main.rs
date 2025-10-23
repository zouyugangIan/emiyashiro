use bevy::prelude::*;

mod components;
mod database;
mod resources;
mod states;
mod systems;
mod tools;

#[cfg(test)]
mod tests;

// use components::*;
use resources::*;
use states::*;
use systems::background::*;
use systems::*;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Fate/stay night Heaven's Feel - Shirou Runner".into(),
            resolution: (1024, 768).into(),
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
    .init_resource::<ui::RenameInput>()
    .insert_resource(systems::text_input::TextInputState::new(25))
    .init_resource::<systems::text_input::KeyboardInputHandler>()
    .init_resource::<systems::error_handling::ErrorRecoveryManager>()
    .init_resource::<systems::async_file_ops::AsyncFileManager>()
    .init_resource::<systems::async_file_ops::OperationProgress>()
    .add_systems(
        Startup,
        (
            setup_game_resources,
            systems::save::load_game,
            setup_cloud_spawner,
        ),
    )
    .add_systems(OnEnter(GameState::Menu), menu::setup_menu)
    .add_systems(
        Update,
        systems::audio::play_menu_music.run_if(in_state(GameState::Menu)),
    )
    .add_systems(
        Update,
        (
            menu::handle_start_button,
            menu::handle_load_button,
            menu::handle_character_select,
            systems::save::handle_save_button_click,
            menu::cover_fade_animation,
            systems::visual_effects::button_hover_effect,
        )
            .run_if(in_state(GameState::Menu)),
    )
    .add_systems(
        OnExit(GameState::Menu),
        (menu::cleanup_menu, systems::audio::stop_menu_music),
    )
    .add_systems(
        OnEnter(GameState::Playing),
        (
            game::setup_game,
            ui::setup_game_hud,
            systems::audio::play_game_music_and_stop_menu,
        ),
    )
    .add_systems(
        Update,
        game::restore_loaded_game_entities.run_if(in_state(GameState::Playing)),
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
        )
            .run_if(in_state(GameState::Playing)),
    )
    .add_systems(
        Update,
        (
            // 动画和视觉效果系统
            animation::animate_character,
            animation::setup_character_animation,
            animation::trigger_audio_effects,
            systems::frame_animation::update_frame_animations,
            systems::frame_animation::update_character_animations,
            systems::frame_animation::setup_player_animation,
            systems::frame_animation::debug_animations,
            // 音乐切换系统
            systems::audio::handle_music_transitions,
        )
            .run_if(in_state(GameState::Playing)),
    )
    .add_systems(
        Update,
        (
            spawn_clouds_system,
            move_clouds_system,
            despawn_offscreen_clouds_system,
        )
            .run_if(in_state(GameState::Playing)),
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
        )
            .run_if(in_state(GameState::Playing)),
    )
    .add_systems(
        Update,
        (
            // 数据和存档系统
            systems::save::auto_save_system,
            systems::database_service::database_stats_system,
            systems::database_service::cleanup_old_sessions,
            // 异步文件操作和进度显示
            systems::async_file_ops::update_operation_progress,
            systems::async_file_ops::display_progress_indicator,
        )
            .run_if(in_state(GameState::Playing)),
    )
    .add_systems(
        Update,
        (
            systems::input::update_game_input,
            systems::sprite_animation::update_sprite_animations,
            systems::sprite_animation::update_character_animation_state,
            systems::input::debug_input_system,
        )
            .run_if(in_state(GameState::Playing)),
    )
    .add_systems(
        Update,
        systems::pause_save::handle_pause_input
            .run_if(in_state(GameState::Playing).or(in_state(GameState::Paused))),
    )
    .add_systems(
        OnExit(GameState::Playing),
        (
            game::cleanup_game,
            ui::cleanup_game_hud,
            systems::audio::stop_game_music,
        ),
    )
    .add_systems(
        OnEnter(GameState::Paused),
        (systems::pause_save::scan_save_files, ui::setup_pause_menu).chain(),
    )
    .add_systems(
        Update,
        (
            ui::handle_pause_menu_interactions,
            systems::pause_save::restore_paused_state,
            systems::audio::maintain_audio_during_pause,
        )
            .run_if(in_state(GameState::Paused)),
    )
    .add_systems(OnExit(GameState::Paused), ui::cleanup_pause_menu)
    .add_systems(OnEnter(GameState::SaveDialog), ui::setup_save_dialog)
    .add_systems(
        Update,
        (
            systems::text_input::handle_keyboard_input,
            ui::handle_save_name_input,
            ui::handle_save_dialog_interactions,
            ui::update_text_cursor,
        )
            .run_if(in_state(GameState::SaveDialog)),
    )
    .add_systems(OnExit(GameState::SaveDialog), ui::cleanup_save_dialog)
    .add_systems(
        OnEnter(GameState::LoadTable),
        (systems::pause_save::scan_save_files, ui::setup_load_table).chain(),
    )
    .add_systems(
        Update,
        ui::handle_load_table_interactions.run_if(in_state(GameState::LoadTable)),
    )
    .add_systems(OnExit(GameState::LoadTable), ui::cleanup_load_table)
    .add_systems(OnEnter(GameState::RenameDialog), ui::setup_rename_dialog)
    .add_systems(
        Update,
        (
            systems::text_input::handle_keyboard_input,
            ui::handle_rename_input,
            ui::handle_rename_dialog_interactions,
        )
            .run_if(in_state(GameState::RenameDialog)),
    )
    .add_systems(OnExit(GameState::RenameDialog), ui::cleanup_rename_dialog);

    // 配置最终集成
    systems::final_integration::configure_final_integration(&mut app);

    app.run();
}

/// 加载游戏资源
fn setup_game_resources(mut commands: Commands, asset_server: Res<AssetServer>) {
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
        game_whyifight_music: asset_server.load("sounds/game-whyIfight.ogg"),
        footstep_sound: asset_server.load("sounds/footstep.ogg"),
        background_music: asset_server.load("sounds/background.ogg"),
    };

    commands.insert_resource(game_assets);
}
