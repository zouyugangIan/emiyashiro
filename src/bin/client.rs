use bevy::prelude::*;
use s__emiyashiro::*; // Import from our lib
use s__emiyashiro::systems::background::*;
use s__emiyashiro::systems::*;
use s__emiyashiro::systems::ui; // Explicit import to disambiguate from components::ui
use s__emiyashiro::systems::player; // Explicit import to disambiguate from components::player
use s__emiyashiro::systems::animation; // Explicit import to disambiguate from components::animation
use s__emiyashiro::systems::network; // Explicit import for network systems

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "G-Engine Client (WebGPU)".into(),
            resolution: (1024, 768).into(),
            ..default()
        }),
        ..default()
    }))
    .init_state::<GameState>()
    .add_message::<StartSaveGame>()
    .add_message::<StartLoadGame>()
    .init_resource::<CharacterSelection>()
    .init_resource::<GameStats>()
    .init_resource::<AudioSettings>()
    .init_resource::<systems::audio::AudioManager>()
    .init_resource::<SaveManager>()
    .init_resource::<SaveFileManager>()
    .init_resource::<PauseManager>()
    .init_resource::<AudioStateManager>()
    // DatabaseService is Server-only, but for now we keep it to compile, 
    // we will mock it or remove it in Phase 2
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
    .init_resource::<systems::network::NetworkResource>()
    .init_resource::<systems::network::NetworkEntityMap>()
    .init_resource::<systems::network::MyNetworkId>()
    .add_systems(
        Startup,
        (
            setup_game_resources,
            systems::save::load_game,
            setup_cloud_spawner,
            systems::network::setup_network,
        ),
    )
    .add_systems(
        Update,
        (
            systems::network::handle_network_events,
            systems::network::send_ping_system,
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
            // Core Game Systems - Physics removed (now server-authoritative)
            // Only keep stats tracking and camera
            player::update_game_stats,
            camera::camera_follow,
            ui::update_game_hud,
            // Network systems
            network::handle_network_events,
            network::interpolate_positions,
        )
            .run_if(in_state(GameState::Playing)),
    )
    .add_systems(
        Update,
        (
            // Animation
            animation::animate_character,
            animation::setup_character_animation,
            animation::trigger_audio_effects,
            systems::frame_animation::update_frame_animations,
            systems::frame_animation::update_character_animations,
            systems::frame_animation::setup_player_animation,
            systems::frame_animation::debug_animations,
            // Audio
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
            // VFX
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
            // Save/Load (To be moved to Server)
            systems::save::auto_save_system,
            systems::database_service::database_stats_system,
            systems::database_service::cleanup_old_sessions,
            systems::async_file_ops::update_operation_progress,
            systems::async_file_ops::display_progress_indicator,
        )
            .run_if(in_state(GameState::Playing)),
    )
    .add_systems(
        Update,
        (
            async_tasks::handle_save_requests,
            async_tasks::handle_load_requests,
            async_tasks::poll_async_tasks,
        ),
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

    // Final Integration
    systems::final_integration::configure_final_integration(&mut app);

    app.run();
}

/// Load Game Resources
fn setup_game_resources(mut commands: Commands, asset_server: Res<AssetServer>) {
    let game_assets = GameAssets {
        cover_texture: asset_server.load(asset_paths::IMAGE_UI_COVER1),
        cover2_texture: asset_server.load(asset_paths::IMAGE_UI_COVER2),
        shirou1_texture: asset_server.load(asset_paths::IMAGE_CHAR_SHIROU_IDLE1),
        shirou2_texture: asset_server.load(asset_paths::IMAGE_CHAR_SAKURA_IDLE1),
        font: asset_server.load(asset_paths::FONT_FIRA_SANS),
        shirou_spritesheet: None,
        sakura_spritesheet: None,
        shirou_atlas: None,
        sakura_atlas: None,
        jump_sound: asset_server.load(asset_paths::SOUND_JUMP),
        land_sound: asset_server.load(asset_paths::SOUND_LAND),
        menu_music: asset_server.load(asset_paths::SOUND_MENU_MUSIC),
        game_music: asset_server.load(asset_paths::SOUND_GAME_MUSIC),
        game_whyifight_music: asset_server.load(asset_paths::SOUND_GAME_WHY_I_FIGHT_MUSIC),
        footstep_sound: asset_server.load(asset_paths::SOUND_FOOTSTEP),
        background_music: asset_server.load(asset_paths::SOUND_BACKGROUND_MUSIC),
    };

    commands.insert_resource(game_assets);
}
