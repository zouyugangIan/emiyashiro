use bevy::prelude::*;
use s__emiyashiro::systems::animation; // Explicit import to disambiguate from components::animation
use s__emiyashiro::systems::background::*;
use s__emiyashiro::systems::player; // Explicit import to disambiguate from components::player
use s__emiyashiro::systems::ui; // Explicit import to disambiguate from components::ui
use s__emiyashiro::systems::*;
use s__emiyashiro::*; // Import from our lib

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
    .init_resource::<ui::SaveLoadUiState>()
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
    .init_resource::<systems::sprite_animation::AnimationRuntimeConfig>()
    .add_systems(
        Startup,
        (
            setup_game_resources,
            setup_animation_data,
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
            systems::network::send_player_input,
            systems::network::interpolate_positions,
        ),
    )
    .add_systems(
        OnEnter(GameState::Menu),
        (
            game::cleanup_game,
            ui::cleanup_game_hud,
            systems::audio::stop_game_music,
            menu::setup_menu,
        )
            .chain(),
    )
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
            systems::scene_decoration::setup_parallax_background, // 設置視差背景
        ),
    )
    .add_systems(
        Update,
        game::restore_loaded_game_entities.run_if(in_state(GameState::Playing)),
    )
    .add_systems(
        Update,
        (
            // Core Game Systems - Local physics as fallback when server is not connected
            player::player_movement,
            player::player_jump,
            player::player_crouch,
            player::update_player_state,
            player::physics_update_system,
            player::update_game_stats,
            camera::camera_follow,
            ui::update_game_hud,
            // Shirou Mechanics
            systems::shirou::handle_shroud_input,
            systems::shirou::shroud_health_drain,
        )
            .run_if(in_state(GameState::Playing)),
    )
    .add_systems(
        Update,
        (
            // Animation (single source of truth: sprite atlas / frame animation)
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
            // 場景裝飾系統
            systems::scene_decoration::spawn_enhanced_clouds,
            systems::scene_decoration::spawn_ground_decorations,
            systems::scene_decoration::move_scene_decorations,
            systems::scene_decoration::cleanup_offscreen_decorations,
            systems::scene_decoration::loop_far_background,
            systems::scene_decoration::dynamic_lighting,
        )
            .run_if(in_state(GameState::Playing)),
    )
    .add_systems(
        Update,
        (
            // 敵人系統
            systems::enemy::spawn_mushroom_enemies,
            systems::enemy::enemy_patrol_ai,
            systems::enemy::cleanup_dead_enemies,
            systems::enemy::cleanup_offscreen_enemies,
        )
            .run_if(in_state(GameState::Playing)),
    )
    .add_systems(
        Update,
        (
            // 戰鬥系統
            systems::combat::player_shoot_projectile,
            systems::combat::update_projectiles,
            systems::combat::cleanup_expired_projectiles,
            systems::combat::projectile_enemy_collision,
            systems::combat::player_enemy_collision,
        )
            .run_if(in_state(GameState::Playing)),
    )
    .add_systems(
        Update,
        (
            // 死亡系統
            systems::death::check_player_fall_death,
            systems::death::respawn_player,
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
            // Save/Load (legacy gameplay autosave + database)
            systems::save::auto_save_system,
            systems::database_service::database_stats_system,
            systems::database_service::cleanup_old_sessions,
        )
            .run_if(in_state(GameState::Playing)),
    )
    .add_systems(
        Update,
        (
            // Save/Load UX feedback
            systems::async_file_ops::update_operation_progress,
            systems::async_file_ops::display_progress_indicator,
            ui::update_save_load_status_text,
        )
            .run_if(
                in_state(GameState::Playing)
                    .or(in_state(GameState::Paused))
                    .or(in_state(GameState::SaveDialog))
                    .or(in_state(GameState::LoadTable)),
            ),
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
            // debug_input_system removed - enable only when debugging input issues
        )
            .run_if(in_state(GameState::Playing)),
    )
    .add_systems(
        Update,
        systems::pause_save::handle_pause_input
            .run_if(in_state(GameState::Playing).or(in_state(GameState::Paused))),
    )
    .add_systems(
        OnEnter(GameState::Paused),
        (systems::pause_save::scan_save_files, ui::setup_pause_menu).chain(),
    )
    .add_systems(
        Update,
        (
            ui::handle_pause_menu_interactions,
            systems::audio::maintain_audio_during_pause,
        )
            .run_if(in_state(GameState::Paused)),
    )
    .add_systems(
        Update,
        systems::pause_save::restore_paused_state.run_if(in_state(GameState::Playing)),
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

    app.run();
}

/// Load Game Resources
fn setup_game_resources(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
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

    let game_assets = GameAssets {
        cover_textures,
        current_cover_index: 0,
        shirou_animation_frames,
        sakura_animation_frames,
        current_shirou_frame: 0,
        current_sakura_frame: 0,
        font: asset_server.load(asset_paths::FONT_FIRA_SANS),
        shirou_spritesheet: None,
        sakura_spritesheet: None,
        shirou_atlas: None,
        shirou_atlas_run: None,
        sakura_atlas: None,
        jump_sound: asset_server.load(asset_paths::SOUND_JUMP),
        land_sound: asset_server.load(asset_paths::SOUND_LAND),
        menu_music: asset_server.load(asset_paths::SOUND_MENU_MUSIC),
        game_music: asset_server.load(asset_paths::SOUND_GAME_MUSIC),
        game_whyifight_music: asset_server.load(asset_paths::SOUND_GAME_WHY_I_FIGHT_MUSIC),
        footstep_sound: asset_server.load(asset_paths::SOUND_FOOTSTEP),
        background_music: asset_server.load(asset_paths::SOUND_BACKGROUND_MUSIC),
    };

    // Load HF Shirou Atlas
    // Layout 1: 4x4 Grid (Standard) - for Idle, Jump, Attack
    // Width 1024 / 4 = 256. Height 1024 / 4 = 256.
    let texture_handle = asset_server.load(asset_paths::IMAGE_HF_SHIROU_SPRITESHEET);
    let layout_4x4 = TextureAtlasLayout::from_grid(UVec2::new(256, 256), 4, 4, None, None);
    let layout_4x4_handle = texture_atlases.add(layout_4x4);

    // Layout 2: 5x4 Grid (Run) - for Run animation (Row 1 has 5 frames)
    // Width ~204. Height 256.
    let layout_5x4 = TextureAtlasLayout::from_grid(UVec2::new(204, 256), 5, 4, None, None);
    let layout_5x4_handle = texture_atlases.add(layout_5x4);

    // Assign to assets
    let mut assets = game_assets;
    assets.shirou_spritesheet = Some(texture_handle);
    assets.shirou_atlas = Some(layout_4x4_handle);
    assets.shirou_atlas_run = Some(layout_5x4_handle);

    commands.insert_resource(assets);
}

/// Load Animation Data
fn setup_animation_data(mut commands: Commands) {
    let animation_data = systems::sprite_animation::load_animation_data();
    commands.insert_resource(animation_data);
}
