//! Pause state capture and restoration.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::{components::*, resources::*, states::*};

#[derive(SystemParam)]
pub struct PauseSnapshotParams<'w, 's> {
    player_query:
        Query<'w, 's, (&'static Transform, &'static Velocity, &'static PlayerState), With<Player>>,
    camera_query: Query<'w, 's, &'static Transform, (With<Camera>, Without<Player>)>,
    game_stats: Res<'w, GameStats>,
    character_selection: Res<'w, CharacterSelection>,
    audio_state_manager: Res<'w, AudioStateManager>,
}

#[derive(SystemParam)]
pub struct RestorePausedParams<'w, 's> {
    player_query: Query<
        'w,
        's,
        (
            &'static mut Transform,
            &'static mut Velocity,
            &'static mut PlayerState,
        ),
        With<Player>,
    >,
    camera_query: Query<'w, 's, &'static mut Transform, (With<Camera>, Without<Player>)>,
    game_stats: ResMut<'w, GameStats>,
    character_selection: ResMut<'w, CharacterSelection>,
    audio_state_manager: ResMut<'w, AudioStateManager>,
}

pub fn capture_game_state(
    player_query: Query<(&Transform, &Velocity, &PlayerState), With<Player>>,
    camera_query: Query<&Transform, (With<Camera>, Without<Player>)>,
    game_stats: Res<GameStats>,
    character_selection: Res<CharacterSelection>,
    audio_state_manager: Res<AudioStateManager>,
) -> CompleteGameState {
    let mut state = CompleteGameState::default();

    if let Ok((transform, velocity, player_state)) = player_query.single() {
        state.player_position = transform.translation;
        state.player_velocity = velocity.clone();
        state.player_grounded = player_state.is_grounded;
        state.player_crouching = player_state.is_crouching;
        state.player_animation_state = if player_state.is_crouching {
            "crouch"
        } else if !player_state.is_grounded {
            "jump"
        } else if velocity.x.abs() > 0.1 {
            "run"
        } else {
            "idle"
        }
        .to_string();
    }

    if let Ok(camera_transform) = camera_query.single() {
        state.camera_position = camera_transform.translation;
        state.camera_target = state.player_position
            + Vec3::new(crate::resources::GameConfig::CAMERA_OFFSET, 0.0, 0.0);
    }

    state.score = (game_stats.distance_traveled * 10.0) as u32 + game_stats.jump_count * 50;
    state.distance_traveled = game_stats.distance_traveled;
    state.jump_count = game_stats.jump_count;
    state.play_time = game_stats.play_time;
    state.selected_character = character_selection.selected_character.clone();
    state.player_count = match character_selection.selected_character {
        CharacterType::Shirou => PlayerCount::Single,
        CharacterType::Sakura => PlayerCount::Double,
    };
    state.music_playing = audio_state_manager.music_playing;
    state.audio_volume = audio_state_manager.music_volume;
    state.music_position = audio_state_manager.music_position;
    state.save_timestamp = chrono::Utc::now();
    state
}

fn restore_game_state(
    state: CompleteGameState,
    mut player_query: Query<(&mut Transform, &mut Velocity, &mut PlayerState), With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    mut game_stats: ResMut<GameStats>,
    mut character_selection: ResMut<CharacterSelection>,
    mut audio_state_manager: ResMut<AudioStateManager>,
) {
    if let Ok((mut transform, mut velocity, mut player_state)) = player_query.single_mut() {
        transform.translation = state.player_position;
        *velocity = state.player_velocity;
        player_state.is_grounded = state.player_grounded;
        player_state.is_crouching = state.player_crouching;
    }

    if let Ok(mut camera_transform) = camera_query.single_mut() {
        camera_transform.translation = state.camera_position;
    }

    game_stats.distance_traveled = state.distance_traveled;
    game_stats.jump_count = state.jump_count;
    game_stats.play_time = state.play_time;
    character_selection.selected_character = state.selected_character;
    audio_state_manager.music_playing = state.music_playing;
    audio_state_manager.music_volume = state.audio_volume;
    audio_state_manager.music_position = state.music_position;
}

pub fn handle_pause_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
    mut pause_manager: ResMut<PauseManager>,
    snapshot: PauseSnapshotParams,
    settings_overlay_query: Query<(), With<crate::systems::settings_ui::SettingsOverlayRoot>>,
    mut last_escape_state: Local<bool>,
) {
    let escape_pressed = keyboard_input.pressed(KeyCode::Escape);
    let escape_just_pressed = escape_pressed && !*last_escape_state;
    let quit_just_pressed = keyboard_input.just_pressed(KeyCode::KeyQ);
    *last_escape_state = escape_pressed;

    if !settings_overlay_query.is_empty() {
        return;
    }

    match current_state.get() {
        GameState::Playing if escape_just_pressed => {
            let state = capture_game_state(
                snapshot.player_query,
                snapshot.camera_query,
                snapshot.game_stats,
                snapshot.character_selection,
                snapshot.audio_state_manager,
            );
            pause_manager.pause_game(state);
            NextState::set_if_neq(&mut next_state, GameState::Paused);
        }
        GameState::Paused if escape_just_pressed => {
            NextState::set_if_neq(&mut next_state, GameState::Playing);
        }
        GameState::Paused if quit_just_pressed => {
            pause_manager.clear_pause_state();
            NextState::set_if_neq(&mut next_state, GameState::Menu);
        }
        _ => {}
    }
}

pub fn restore_paused_state(
    mut pause_manager: ResMut<PauseManager>,
    current_state: Res<State<GameState>>,
    next_state: Res<NextState<GameState>>,
    restore: RestorePausedParams,
) {
    if matches!(next_state.as_ref(), NextState::Pending(GameState::Paused))
        || *current_state.get() != GameState::Playing
        || !pause_manager.is_paused
    {
        return;
    }

    if let Some(state) = pause_manager.resume_game() {
        restore_game_state(
            state,
            restore.player_query,
            restore.camera_query,
            restore.game_stats,
            restore.character_selection,
            restore.audio_state_manager,
        );
    }
}
