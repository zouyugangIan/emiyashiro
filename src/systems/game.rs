//! 镙稿绩娓告垙绯荤粺
//!
//! 鍖呭惈娓告垙鍦烘櫙镄勮缃€佹竻鐞嗗拰镙稿绩娓告垙阃昏緫绠＄悊銆?
use crate::{components::*, resources::*, states::*};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::sprite::Anchor;

pub const PLAYER_RENDER_SIZE: Vec2 = Vec2::new(96.0, 144.0);
pub const SAKURA_RENDER_SIZE: Vec2 = Vec2::splat(192.0);
pub const PLAYER_VISUAL_BASELINE_ANCHOR_Y: f32 = -0.22;

/// The camera's exact first-frame position for a newly spawned player.
pub fn initial_gameplay_camera_position(player_position: Vec3) -> Vec3 {
    Vec3::new(
        player_position.x + GameConfig::CAMERA_OFFSET,
        (player_position.y * 0.2).clamp(-80.0, 80.0),
        0.0,
    )
}

#[derive(SystemParam)]
pub struct SetupGameParams<'w, 's> {
    pub character_selection: ResMut<'w, CharacterSelection>,
    pub game_assets: Res<'w, GameAssets>,
    pub anim_data_map: Res<'w, crate::components::animation_data::AnimationDataMap>,
    pub camera_query: Query<'w, 's, &'static mut Transform, (With<Camera>, Without<Player>)>,
    pub player_query: Query<'w, 's, (Entity, &'static Transform), With<Player>>,
    pub ground_query: Query<'w, 's, Entity, With<Ground>>,
    pub loaded_game_state: Res<'w, crate::systems::ui::LoadedGameState>,
    pub sky_level: Option<ResMut<'w, crate::components::SkyLevelRuntime>>,
}

/// Sets up gameplay entities for a fresh start or loaded run.
pub fn setup_game(mut commands: Commands, mut params: SetupGameParams) {
    let sky_level_active = params
        .sky_level
        .as_deref()
        .is_some_and(|level| level.active);
    if params.ground_query.is_empty() && !sky_level_active {
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

    let mut should_rebuild_player = false;
    if params.loaded_game_state.should_restore
        && let Some(state) = &params.loaded_game_state.state
    {
        params.character_selection.selected_character = state.selected_character.clone();
        if let Some(level) = params.sky_level.as_deref_mut().filter(|level| level.active) {
            // The LDtk start initializer runs asynchronously after the save
            // handoff. Marking this position initialized prevents it from
            // overwriting a successfully restored player on a later frame.
            level.player_initialized = true;
            level.checkpoint_position = state.player_position;
            level.checkpoint_needs_reconciliation = true;
        }
        should_rebuild_player = true;
    }

    if !params.player_query.is_empty() {
        if should_rebuild_player {
            for (entity, _) in params.player_query.iter() {
                commands.entity(entity).despawn();
            }
            crate::debug_log!("Rebuilding player entity from loaded save");
        } else {
            if params.camera_query.is_empty()
                && let Some((_, player_transform)) = params.player_query.iter().next()
            {
                commands.spawn((
                    Camera2d,
                    Transform::from_translation(initial_gameplay_camera_position(
                        player_transform.translation,
                    )),
                ));
                crate::debug_log!("Recreated gameplay camera at the current player");
            }
            crate::debug_log!("Player already exists, continuing game");
            return;
        }
    }

    let player_start = params
        .sky_level
        .as_deref()
        .filter(|level| level.active)
        .map(|level| level.checkpoint_position)
        .unwrap_or(GameConfig::PLAYER_START_POS);

    let camera_position = params
        .loaded_game_state
        .state
        .as_ref()
        .filter(|_| params.loaded_game_state.should_restore)
        .map(|state| state.camera_position)
        .unwrap_or_else(|| initial_gameplay_camera_position(player_start));

    if params.camera_query.is_empty() {
        commands.spawn((Camera2d, Transform::from_translation(camera_position)));
        crate::debug_log!("Created gameplay camera at the starting position");
    } else {
        for mut camera_transform in params.camera_query.iter_mut() {
            camera_transform.translation = camera_position;
        }
        crate::debug_log!("Snapped gameplay camera to the starting position");
    }

    let texture = match params.character_selection.selected_character {
        CharacterType::Shirou => params.game_assets.shirou_initial_image.clone(),
        CharacterType::Sakura => params.game_assets.sakura_initial_image.clone(),
    };
    let render_size = match params.character_selection.selected_character {
        CharacterType::Shirou => PLAYER_RENDER_SIZE,
        CharacterType::Sakura => SAKURA_RENDER_SIZE,
    };

    crate::debug_log!(
        "Selected character: {:?}",
        params.character_selection.selected_character
    );

    let character_name = match params.character_selection.selected_character {
        CharacterType::Shirou => "hf_shirou",
        CharacterType::Sakura => "sakura",
    };

    let player_common = (
        Transform::from_translation(player_start),
        Player,
        crate::systems::network::LocalPlayer,
        crate::components::network::NetworkId(0),
        Velocity { x: 0.0, y: 0.0 },
        PlayerState::default(),
        AttackAnimationState::default(),
        FacingDirection::default(),
        DamageInvulnerability::default(),
        crate::systems::collision::CollisionBox::new(GameConfig::PLAYER_SIZE),
        Health::default(),
        ShroudState::default(),
    );

    if params.character_selection.selected_character == CharacterType::Shirou
        && let Some(sprite_sheets) = params.game_assets.hf_shirou_sprite_animation_sheets()
    {
        let anim_component = crate::systems::sprite_animation::create_character_animation(
            &params.anim_data_map,
            character_name,
        );

        commands.spawn((
            Sprite {
                image: sprite_sheets.core_texture.clone(),
                custom_size: Some(PLAYER_RENDER_SIZE),
                texture_atlas: Some(TextureAtlas {
                    layout: sprite_sheets.core_layout.clone(),
                    index: 0,
                }),
                ..default()
            },
            Anchor(Vec2::new(0.0, PLAYER_VISUAL_BASELINE_ANCHOR_Y)),
            player_common,
            anim_component,
            sprite_sheets,
        ));

        crate::debug_log!("HF Shirou spawned in atlas mode");
        return;
    }

    commands.spawn((
        Sprite {
            image: texture,
            custom_size: Some(render_size),
            ..default()
        },
        Anchor(Vec2::new(0.0, PLAYER_VISUAL_BASELINE_ANCHOR_Y)),
        player_common,
    ));

    crate::debug_log!("Character spawned in image sequence mode");
}
/// 鎭㈠锷犺浇镄勬父鎴忕姸镐佷腑镄勫疄浣扑綅缃?
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
