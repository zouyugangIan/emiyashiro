//! 鏍稿績娓告垙绯荤粺
//!
//! 鍖呭惈娓告垙鍦烘櫙鐨勮缃€佹竻鐞嗗拰鏍稿績娓告垙閫昏緫绠＄悊銆?
use crate::{components::*, resources::*, states::*};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

const PLAYER_RENDER_SIZE: Vec2 = Vec2::new(96.0, 144.0);

#[derive(SystemParam)]
pub struct SetupGameParams<'w, 's> {
    pub character_selection: ResMut<'w, CharacterSelection>,
    pub game_assets: Res<'w, GameAssets>,
    pub anim_data_map: Res<'w, crate::components::animation_data::AnimationDataMap>,
    pub camera_query: Query<'w, 's, Entity, With<Camera>>,
    pub player_query: Query<'w, 's, Entity, With<Player>>,
    pub ground_query: Query<'w, 's, Entity, With<Ground>>,
    pub loaded_game_state: Res<'w, crate::systems::ui::LoadedGameState>,
}

/// Sets up gameplay entities for a fresh start or loaded run.
pub fn setup_game(mut commands: Commands, mut params: SetupGameParams) {
    if params.camera_query.is_empty() {
        commands.spawn(Camera2d);
        crate::debug_log!("Created gameplay camera");
    }

    if params.ground_query.is_empty() {
        commands.spawn((
            Sprite {
                color: GameConfig::GROUND_COLOR,
                custom_size: Some(GameConfig::GROUND_SIZE),
                ..default()
            },
            Transform::from_translation(GameConfig::GROUND_POS),
            Ground,
            crate::systems::collision::CollisionBox::new(GameConfig::GROUND_SIZE),
        ));
    }

    if params.loaded_game_state.should_restore
        && let Some(state) = &params.loaded_game_state.state
    {
        params.character_selection.selected_character = state.selected_character.clone();
    }

    if !params.player_query.is_empty() {
        crate::debug_log!("Player already exists, continuing game");
        return;
    }

    let texture = match params.character_selection.selected_character {
        CharacterType::Shirou1 => params.game_assets.get_current_shirou_frame(),
        CharacterType::Shirou2 => params.game_assets.get_current_sakura_frame(),
    };

    crate::debug_log!(
        "Selected character: {:?}",
        params.character_selection.selected_character
    );

    let character_name = match params.character_selection.selected_character {
        CharacterType::Shirou1 => "hf_shirou",
        CharacterType::Shirou2 => "sakura",
    };

    let player_common = (
        Transform::from_translation(GameConfig::PLAYER_START_POS),
        Player,
        Velocity { x: 0.0, y: 0.0 },
        PlayerState::default(),
        crate::systems::collision::CollisionBox::new(GameConfig::PLAYER_SIZE),
        Health::default(),
        ShroudState::default(),
    );

    if let Some(atlas_layout) = &params.game_assets.shirou_atlas
        && params.character_selection.selected_character == CharacterType::Shirou1
        && let Some(texture) = &params.game_assets.shirou_spritesheet
    {
        let anim_component = crate::systems::sprite_animation::create_character_animation(
            &params.anim_data_map,
            character_name,
        );

        commands.spawn((
            Sprite {
                image: texture.clone(),
                custom_size: Some(PLAYER_RENDER_SIZE),
                texture_atlas: Some(TextureAtlas {
                    layout: atlas_layout.clone(),
                    index: 0,
                }),
                ..default()
            },
            player_common,
            anim_component,
        ));

        crate::debug_log!("HF Shirou spawned in atlas mode");
        return;
    }

    commands.spawn((
        Sprite {
            image: texture,
            custom_size: Some(PLAYER_RENDER_SIZE),
            ..default()
        },
        player_common,
    ));

    crate::debug_log!("Character spawned in frame fallback mode");
}
/// 鎭㈠鍔犺浇鐨勬父鎴忕姸鎬佷腑鐨勫疄浣撲綅缃?
pub fn restore_loaded_game_entities(
    mut loaded_game_state: ResMut<crate::systems::ui::LoadedGameState>,
    mut player_query: Query<(&mut Transform, &mut Velocity, &mut PlayerState), With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    mut game_stats: ResMut<GameStats>,
    mut character_selection: ResMut<CharacterSelection>,
    mut audio_state_manager: ResMut<AudioStateManager>,
) {
    use crate::systems::text_constants::SaveLoadText;

    if !loaded_game_state.should_restore {
        return;
    }
    let Some(state) = &loaded_game_state.state else {
        return;
    };

    crate::debug_log!("Loading Game...");
    let mut player_restored = false;

    if let Ok((mut player_transform, mut player_velocity, mut player_state)) =
        player_query.single_mut()
    {
        player_transform.translation = state.player_position;
        *player_velocity = state.player_velocity.clone();
        player_state.is_grounded = state.player_grounded;
        player_state.is_crouching = state.player_crouching;
        player_restored = true;
    } else {
        warn!("Player entity not ready yet, retrying save restore next frame");
    }

    if let Ok(mut camera_transform) = camera_query.single_mut() {
        camera_transform.translation = state.camera_position;
    }

    game_stats.distance_traveled = state.distance_traveled;
    game_stats.jump_count = state.jump_count;
    game_stats.play_time = state.play_time;

    character_selection.selected_character = state.selected_character.clone();

    audio_state_manager.music_playing = state.music_playing;
    audio_state_manager.music_volume = state.audio_volume;

    crate::debug_log!("{}", SaveLoadText::LOAD_SUCCESS);

    if player_restored {
        loaded_game_state.should_restore = false;
        loaded_game_state.previous_state = None;
    }
}
pub fn cleanup_game(
    mut commands: Commands,
    player_query: Query<Entity, With<Player>>,
    ground_query: Query<Entity, With<Ground>>,
) {
    for entity in player_query.iter() {
        commands.entity(entity).despawn();
        crate::debug_log!("Cleaned player entity");
    }

    for entity in ground_query.iter() {
        commands.entity(entity).despawn();
        crate::debug_log!("Cleaned ground entity");
    }
}
