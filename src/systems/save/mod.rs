//! Local save subsystem.
//!
//! This is the only public entry point for save serialization, file
//! management, and background save/load tasks.

mod codec;
mod io;
mod tasks;

pub(crate) use codec::calculate_checksum;
pub use io::{delete_save_file, rename_save_file, scan_save_files};
pub(crate) use tasks::{handle_load_requests, handle_save_requests, poll_save_tasks};

use bevy::prelude::*;
use std::path::PathBuf;

use crate::{
    resources::{CompleteGameState, GameStats, SaveFileData, SaveFileMetadata, SaveManager},
    states::{CharacterSelection, GameState},
    systems::error_handling::SaveSystemError,
};

const AUTOSAVE_NAME: &str = "autosave";

/// Writes the lightweight automatic save used at startup.
pub fn save_game(
    game_stats: Res<GameStats>,
    character_selection: Res<CharacterSelection>,
    mut save_manager: ResMut<SaveManager>,
) {
    let previous_state = save_manager
        .current_save
        .as_ref()
        .map(|save| &save.game_state);
    let save_timestamp = chrono::Utc::now();
    let best_distance = game_stats.distance_traveled.max(
        previous_state
            .map(|state| state.distance_traveled)
            .unwrap_or(0.0),
    );
    let total_jumps =
        game_stats.jump_count + previous_state.map(|state| state.jump_count).unwrap_or(0);
    let total_play_time =
        game_stats.play_time + previous_state.map(|state| state.play_time).unwrap_or(0.0);

    let save_path = PathBuf::from(&save_manager.save_file_path);
    let mut metadata = save_manager
        .current_save
        .as_ref()
        .map(|save| save.metadata.clone())
        .unwrap_or_else(|| SaveFileMetadata {
            name: AUTOSAVE_NAME.to_string(),
            score: 0,
            distance: 0.0,
            play_time: 0.0,
            save_timestamp,
            file_path: save_path.to_string_lossy().to_string(),
            selected_character: character_selection.selected_character.clone(),
        });

    let mut state = CompleteGameState {
        selected_character: character_selection.selected_character.clone(),
        distance_traveled: best_distance,
        jump_count: total_jumps,
        play_time: total_play_time,
        save_timestamp,
        ..Default::default()
    };
    state.score = (state.distance_traveled * 10.0) as u32 + state.jump_count * 50;

    metadata.score = state.score;
    metadata.distance = state.distance_traveled;
    metadata.play_time = state.play_time;
    metadata.save_timestamp = save_timestamp;
    metadata.file_path = save_path.to_string_lossy().to_string();
    metadata.selected_character = state.selected_character.clone();

    let save_data = SaveFileData::new(metadata, state);
    match io::write_save_file(&save_path, &save_data) {
        Ok(()) => {
            save_manager.current_save = Some(save_data);
            crate::debug_log!("Automatic save completed");
        }
        Err(error) => crate::debug_log!("Automatic save failed: {error}"),
    }
}

/// Loads the automatic v2 save used at startup.
pub fn load_game(
    mut save_manager: ResMut<SaveManager>,
    mut character_selection: ResMut<CharacterSelection>,
) {
    let save_path = PathBuf::from(&save_manager.save_file_path);
    match io::read_save_file(&save_path) {
        Ok(save_data) => {
            character_selection.selected_character =
                save_data.game_state.selected_character.clone();
            save_manager.current_save = Some(save_data);
            crate::debug_log!("Loaded automatic save: {}", save_path.display());
        }
        Err(SaveSystemError::FileNotFound(_)) => {
            crate::debug_log!("No automatic save found; a new one will be created");
        }
        Err(error) => {
            crate::debug_log!(
                "Failed to load automatic save {}: {}",
                save_path.display(),
                error
            );
        }
    }
}

/// Saves every 30 seconds while gameplay is active.
pub fn auto_save_system(
    mut timer: Local<Timer>,
    time: Res<Time>,
    game_stats: Res<GameStats>,
    character_selection: Res<CharacterSelection>,
    save_manager: ResMut<SaveManager>,
    current_state: Res<State<GameState>>,
) {
    if timer.duration().is_zero() {
        timer.set_duration(std::time::Duration::from_secs(30));
        timer.set_mode(bevy::time::TimerMode::Repeating);
    }
    timer.tick(time.delta());

    if timer.just_finished() && *current_state.get() == GameState::Playing {
        save_game(game_stats, character_selection, save_manager);
    }
}
