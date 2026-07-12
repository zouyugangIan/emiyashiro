use bevy::prelude::*;

use crate::{
    asset_paths,
    components::SpriteAnimationSheets,
    events::{CameraImpulseEvent, DamageEvent},
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
            .add_message::<CameraImpulseEvent>()
            .init_resource::<CharacterSelection>()
            .init_resource::<GameStats>()
            .init_resource::<AudioSettings>()
            .init_resource::<systems::settings_ui::VolumeControlState>()
            .init_resource::<systems::audio::AudioManager>()
            .init_resource::<SaveManager>()
            .init_resource::<SaveFileManager>()
            .init_resource::<PauseManager>()
            .init_resource::<AudioStateManager>()
            .init_resource::<systems::input::GameInput>()
            .init_resource::<systems::input::NetworkInputSyncState>()
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
            .init_resource::<systems::camera::CameraShakeState>()
            .init_resource::<systems::combat::HitStopState>()
            .add_systems(
                Startup,
                (
                    systems::setup::load_gameplay_tuning,
                    setup_game_resources,
                    setup_animation_data,
                    systems::save::load_game,
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

    let mut game_assets = GameAssets {
        cover_textures,
        current_cover_index: 0,
        shirou_initial_image: asset_server.load(asset_paths::IMAGE_HF_SHIROU_IDLE),
        sakura_initial_image: asset_server.load(asset_paths::IMAGE_CHAR_SAKURA_IDLE01),
        hf_shirou_animation: None,
        font: asset_server.load(asset_paths::FONT_FIRA_SANS),
        volume_icon: asset_server.load(asset_paths::IMAGE_UI_VOLUME_ICON),
        volume_muted_icon: asset_server.load(asset_paths::IMAGE_UI_VOLUME_MUTED_ICON),
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
    let overedge_light_attack_texture_handle =
        asset_server.load(asset_paths::IMAGE_HF_SHIROU_OVEREDGE_LIGHT_ATTACK_SHEET);
    let overedge_heavy_attack_texture_handle =
        asset_server.load(asset_paths::IMAGE_HF_SHIROU_OVEREDGE_HEAVY_ATTACK_SHEET);

    let core_layout = TextureAtlasLayout::from_grid(
        UVec2::new(256, 256),
        asset_paths::HF_SHIROU_CORE_COLS,
        asset_paths::HF_SHIROU_CORE_ROWS,
        None,
        None,
    );
    let run_layout = TextureAtlasLayout::from_grid(
        UVec2::new(256, 256),
        asset_paths::HF_SHIROU_RUN_COLS,
        1,
        None,
        None,
    );
    let attack_layout = TextureAtlasLayout::from_grid(
        UVec2::new(256, 256),
        asset_paths::HF_SHIROU_ATTACK_COLS,
        1,
        None,
        None,
    );
    // Overedge light sheet: 2816x256, 11帧 1行
    let overedge_light_attack_layout =
        TextureAtlasLayout::from_grid(UVec2::new(256, 256), 11, 1, None, None);
    // Overedge heavy sheet: 4352x256, 17帧 1行
    let overedge_heavy_attack_layout =
        TextureAtlasLayout::from_grid(UVec2::new(256, 256), 17, 1, None, None);

    // Reference Board 精灵表布局（Shift+V 未激活时使用）
    let ref_ground_light_layout = TextureAtlasLayout::from_grid(
        UVec2::from(asset_paths::REFERENCE_BOARD_GROUND_LIGHT_CELL),
        asset_paths::REFERENCE_BOARD_GROUND_LIGHT_COLS,
        1,
        None,
        Some(UVec2::from(
            asset_paths::REFERENCE_BOARD_GROUND_LIGHT_OFFSET,
        )),
    );
    let ref_air_combo_layout = TextureAtlasLayout::from_grid(
        UVec2::from(asset_paths::REFERENCE_BOARD_AIR_COMBO_CELL),
        asset_paths::REFERENCE_BOARD_AIR_COMBO_COLS,
        asset_paths::REFERENCE_BOARD_AIR_COMBO_ROWS,
        None,
        Some(UVec2::from(asset_paths::REFERENCE_BOARD_AIR_COMBO_OFFSET)),
    );
    let ref_heavy_layout = TextureAtlasLayout::from_grid(
        UVec2::from(asset_paths::REFERENCE_BOARD_HEAVY_CELL),
        asset_paths::REFERENCE_BOARD_HEAVY_COLS,
        asset_paths::REFERENCE_BOARD_HEAVY_ROWS,
        None,
        Some(UVec2::from(asset_paths::REFERENCE_BOARD_HEAVY_OFFSET)),
    );
    let ref_ultimate_layout = TextureAtlasLayout::from_grid(
        UVec2::from(asset_paths::REFERENCE_BOARD_ULTIMATE_CELL),
        asset_paths::REFERENCE_BOARD_ULTIMATE_COLS,
        asset_paths::REFERENCE_BOARD_ULTIMATE_ROWS,
        None,
        Some(UVec2::from(asset_paths::REFERENCE_BOARD_ULTIMATE_OFFSET)),
    );
    let ref_mobility_layout = TextureAtlasLayout::from_grid(
        UVec2::from(asset_paths::REFERENCE_BOARD_MOBILITY_CELL),
        asset_paths::REFERENCE_BOARD_MOBILITY_COLS,
        asset_paths::REFERENCE_BOARD_MOBILITY_ROWS,
        None,
        Some(UVec2::from(asset_paths::REFERENCE_BOARD_MOBILITY_OFFSET)),
    );
    let ref_ninjutsu_layout = TextureAtlasLayout::from_grid(
        UVec2::from(asset_paths::REFERENCE_BOARD_NINJUTSU_CELL),
        asset_paths::REFERENCE_BOARD_NINJUTSU_COLS,
        asset_paths::REFERENCE_BOARD_NINJUTSU_ROWS,
        None,
        Some(UVec2::from(asset_paths::REFERENCE_BOARD_NINJUTSU_OFFSET)),
    );
    let ref_weapon_proj_layout = TextureAtlasLayout::from_grid(
        UVec2::from(asset_paths::REFERENCE_BOARD_WEAPON_PROJ_CELL),
        asset_paths::REFERENCE_BOARD_WEAPON_PROJ_COLS,
        asset_paths::REFERENCE_BOARD_WEAPON_PROJ_ROWS,
        None,
        Some(UVec2::from(asset_paths::REFERENCE_BOARD_WEAPON_PROJ_OFFSET)),
    );
    let ref_advance_layout = TextureAtlasLayout::from_grid(
        UVec2::from(asset_paths::REFERENCE_BOARD_ADVANCED_OVERVIEW_CELL),
        asset_paths::REFERENCE_BOARD_ADVANCED_OVERVIEW_COLS,
        asset_paths::REFERENCE_BOARD_ADVANCED_OVERVIEW_ROWS,
        None,
        None,
    );

    // Reference Board 精灵表图片加载
    let ref_ground_light_handle =
        asset_server.load(asset_paths::IMAGE_HF_SHIROU_ATTACK_GROUND_LIGHT_REFERENCE);
    let ref_air_combo_handle =
        asset_server.load(asset_paths::IMAGE_HF_SHIROU_ATTACK_AIR_COMBO_REFERENCE);
    let ref_heavy_handle = asset_server.load(asset_paths::IMAGE_HF_SHIROU_ATTACK_HEAVY_REFERENCE);
    let ref_ultimate_handle =
        asset_server.load(asset_paths::IMAGE_HF_SHIROU_ATTACK_ULTIMATE_REFERENCE);
    let ref_mobility_handle =
        asset_server.load(asset_paths::IMAGE_HF_SHIROU_ATTACK_MOBILITY_REFERENCE);
    let ref_ninjutsu_handle =
        asset_server.load(asset_paths::IMAGE_HF_SHIROU_ATTACK_NINJUTSU_PROJECTILES_REFERENCE);
    let ref_weapon_proj_handle =
        asset_server.load(asset_paths::IMAGE_HF_SHIROU_ATTACK_WEAPON_PROJECTION_REFERENCE);
    let ref_advance_handle =
        asset_server.load(asset_paths::IMAGE_HF_SHIROU_ADVANCED_ATTACK_MODULES_OVERVIEW);
    let ref_ground_light_row_handles = asset_paths::IMAGE_HF_SHIROU_ATTACK_GROUND_LIGHT_ROW_SHEETS
        .iter()
        .map(|path| asset_server.load(*path))
        .collect();

    game_assets.hf_shirou_animation = Some(SpriteAnimationSheets {
        core_texture: core_texture_handle,
        core_layout: texture_atlases.add(core_layout),
        running_texture: run_texture_handle,
        running_layout: texture_atlases.add(run_layout),
        attacking_texture: attack_texture_handle,
        attacking_layout: texture_atlases.add(attack_layout),
        overedge_light_attacking_texture: Some(overedge_light_attack_texture_handle),
        overedge_light_attacking_layout: Some(texture_atlases.add(overedge_light_attack_layout)),
        overedge_light_attacking_frame_count:
            asset_paths::HF_SHIROU_OVEREDGE_LIGHT_ATTACK_FRAME_COUNT,
        overedge_heavy_attacking_texture: Some(overedge_heavy_attack_texture_handle),
        overedge_heavy_attacking_layout: Some(texture_atlases.add(overedge_heavy_attack_layout)),
        overedge_heavy_attacking_frame_count:
            asset_paths::HF_SHIROU_OVEREDGE_HEAVY_ATTACK_FRAME_COUNT,
        reference_ground_light_texture: Some(ref_ground_light_handle),
        reference_ground_light_row_textures: ref_ground_light_row_handles,
        reference_ground_light_layout: Some(texture_atlases.add(ref_ground_light_layout)),
        reference_ground_light_frame_count: asset_paths::REFERENCE_BOARD_GROUND_LIGHT_COLS as usize,
        reference_air_combo_texture: Some(ref_air_combo_handle),
        reference_air_combo_row_textures: Vec::new(),
        reference_air_combo_layout: Some(texture_atlases.add(ref_air_combo_layout)),
        reference_air_combo_frame_count: (asset_paths::REFERENCE_BOARD_AIR_COMBO_COLS
            * asset_paths::REFERENCE_BOARD_AIR_COMBO_ROWS)
            as usize,
        reference_heavy_texture: Some(ref_heavy_handle),
        reference_heavy_row_textures: Vec::new(),
        reference_heavy_layout: Some(texture_atlases.add(ref_heavy_layout)),
        reference_heavy_frame_count: (asset_paths::REFERENCE_BOARD_HEAVY_COLS
            * asset_paths::REFERENCE_BOARD_HEAVY_ROWS)
            as usize,
        reference_ultimate_texture: Some(ref_ultimate_handle),
        reference_ultimate_row_textures: Vec::new(),
        reference_ultimate_layout: Some(texture_atlases.add(ref_ultimate_layout)),
        reference_ultimate_frame_count: (asset_paths::REFERENCE_BOARD_ULTIMATE_COLS
            * asset_paths::REFERENCE_BOARD_ULTIMATE_ROWS)
            as usize,
        reference_mobility_texture: Some(ref_mobility_handle),
        reference_mobility_row_textures: Vec::new(),
        reference_mobility_layout: Some(texture_atlases.add(ref_mobility_layout)),
        reference_mobility_frame_count: (asset_paths::REFERENCE_BOARD_MOBILITY_COLS
            * asset_paths::REFERENCE_BOARD_MOBILITY_ROWS)
            as usize,
        reference_ninjutsu_texture: Some(ref_ninjutsu_handle),
        reference_ninjutsu_row_textures: Vec::new(),
        reference_ninjutsu_layout: Some(texture_atlases.add(ref_ninjutsu_layout)),
        reference_ninjutsu_frame_count: (asset_paths::REFERENCE_BOARD_NINJUTSU_COLS
            * asset_paths::REFERENCE_BOARD_NINJUTSU_ROWS)
            as usize,
        reference_weapon_proj_texture: Some(ref_weapon_proj_handle),
        reference_weapon_proj_row_textures: Vec::new(),
        reference_weapon_proj_layout: Some(texture_atlases.add(ref_weapon_proj_layout)),
        reference_weapon_proj_frame_count: (asset_paths::REFERENCE_BOARD_WEAPON_PROJ_COLS
            * asset_paths::REFERENCE_BOARD_WEAPON_PROJ_ROWS)
            as usize,
        reference_advance_texture: Some(ref_advance_handle),
        reference_advance_layout: Some(texture_atlases.add(ref_advance_layout)),
        reference_advance_frame_count: (asset_paths::REFERENCE_BOARD_ADVANCED_OVERVIEW_COLS
            * asset_paths::REFERENCE_BOARD_ADVANCED_OVERVIEW_ROWS)
            as usize,
    });

    commands.insert_resource(game_assets);
}

/// Load Animation Data.
fn setup_animation_data(mut commands: Commands) {
    let animation_data = systems::sprite_animation::load_animation_data();
    commands.insert_resource(animation_data);
}
