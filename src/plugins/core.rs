use bevy::prelude::*;

use crate::{
    asset_paths,
    events::DamageEvent,
    events::{StartLoadGame, StartSaveGame},
    resources::{
        AudioSettings, AudioStateManager, GameAssets, GameStats, PauseManager, SaveFileManager,
        SaveManager,
    },
    states::CharacterSelection,
    states::GameState,
    systems,
};

/// Initializes global resources, schedules, and startup setup systems.
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        crate::systems::interfaces::SystemConfig::configure_all_systems(app);
        crate::systems::system_sets::configure_save_system_scheduling(app);
        crate::systems::system_sets::configure_performance_scheduling(app);

        app.init_state::<GameState>()
            .insert_resource(Time::<Fixed>::from_hz(60.0))
            .add_message::<StartSaveGame>()
            .add_message::<StartLoadGame>()
            .add_message::<DamageEvent>()
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
            .init_resource::<systems::ui::SaveNameInput>()
            .init_resource::<systems::ui::SaveLoadUiState>()
            .init_resource::<systems::ui::LoadedGameState>()
            .init_resource::<systems::ui::RenameInput>()
            .insert_resource(systems::text_input::TextInputState::new(25))
            .init_resource::<systems::text_input::KeyboardInputHandler>()
            .init_resource::<systems::error_handling::ErrorRecoveryManager>()
            .init_resource::<systems::async_file_ops::AsyncFileManager>()
            .init_resource::<systems::async_file_ops::OperationProgress>()
            .init_resource::<systems::sprite_animation::AnimationRuntimeConfig>()
            .add_systems(
                Startup,
                (
                    setup_game_resources,
                    setup_animation_data,
                    systems::save::load_game,
                    systems::background::setup_cloud_spawner,
                ),
            );
    }
}

/// Load Game Resources.
fn setup_game_resources(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    let cover_textures: Vec<Handle<Image>> = asset_paths::UI_COVER_IMAGES
        .iter()
        .map(|path| asset_server.load(*path))
        .collect();

    let shirou_animation_frames: Vec<Handle<Image>> = asset_paths::SHIROU_ANIMATION_FRAMES
        .iter()
        .map(|path| asset_server.load(*path))
        .collect();

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
        shirou_spritesheet_run: None,
        shirou_spritesheet_attack: None,
        sakura_spritesheet: None,
        shirou_atlas: None,
        shirou_atlas_run: None,
        shirou_atlas_attack: None,
        sakura_atlas: None,
        jump_sound: asset_server.load(asset_paths::SOUND_JUMP),
        land_sound: asset_server.load(asset_paths::SOUND_LAND),
        menu_music: asset_server.load(asset_paths::SOUND_MENU_MUSIC),
        game_music: asset_server.load(asset_paths::SOUND_GAME_MUSIC),
        game_whyifight_music: asset_server.load(asset_paths::SOUND_GAME_WHY_I_FIGHT_MUSIC),
        footstep_sound: asset_server.load(asset_paths::SOUND_FOOTSTEP),
        background_music: asset_server.load(asset_paths::SOUND_BACKGROUND_MUSIC),
    };

    let core_texture_handle = asset_server.load(asset_paths::IMAGE_HF_SHIROU_CORE_SHEET);
    let run_texture_handle = asset_server.load(asset_paths::IMAGE_HF_SHIROU_RUN_SHEET);
    let attack_texture_handle = asset_server.load(asset_paths::IMAGE_HF_SHIROU_ATTACK_SHEET);

    let core_layout = TextureAtlasLayout::from_grid(UVec2::new(256, 256), 4, 2, None, None);
    let run_layout = TextureAtlasLayout::from_grid(UVec2::new(256, 256), 5, 1, None, None);
    let attack_layout = TextureAtlasLayout::from_grid(UVec2::new(256, 256), 4, 1, None, None);

    let mut assets = game_assets;
    assets.shirou_spritesheet = Some(core_texture_handle);
    assets.shirou_spritesheet_run = Some(run_texture_handle);
    assets.shirou_spritesheet_attack = Some(attack_texture_handle);
    assets.shirou_atlas = Some(texture_atlases.add(core_layout));
    assets.shirou_atlas_run = Some(texture_atlases.add(run_layout));
    assets.shirou_atlas_attack = Some(texture_atlases.add(attack_layout));

    commands.insert_resource(assets);
}

/// Load Animation Data.
fn setup_animation_data(mut commands: Commands) {
    let animation_data = systems::sprite_animation::load_animation_data();
    commands.insert_resource(animation_data);
}
