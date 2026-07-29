use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::tasks::{ComputeTaskPool, Task};
use futures_lite::future;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::{
    events::{StartLoadGame, StartSaveGame},
    resources::{CompleteGameState, PauseManager, SaveFileManager, SaveFileMetadata},
    states::GameState,
    systems::{
        error_handling::SaveSystemError,
        ui::{LoadedGameState, SaveLoadUiState},
    },
};

use super::io;

#[derive(Component)]
pub(crate) struct SaveTask(Task<Result<(), SaveSystemError>>);

#[derive(Component)]
pub(crate) struct LoadTask(Task<Result<(CompleteGameState, SaveFileMetadata), SaveSystemError>>);

#[derive(SystemParam)]
pub(crate) struct PollSaveTasks<'w> {
    next_state: ResMut<'w, NextState<GameState>>,
    loaded_game_state: ResMut<'w, LoadedGameState>,
    pause_manager: ResMut<'w, PauseManager>,
    save_load_ui_state: ResMut<'w, SaveLoadUiState>,
}

fn resolve_save_target(
    save_directory: &Path,
    requested_name: &str,
    existing_saves: &[SaveFileMetadata],
) -> (String, PathBuf, bool) {
    if let Some(existing) = existing_saves
        .iter()
        .find(|save| save.name == requested_name)
    {
        let existing_path = PathBuf::from(&existing.file_path);
        let target_path =
            if existing_path.as_os_str().is_empty() || !is_valid_save_path(&existing_path) {
                save_directory.join(format!("{requested_name}.json"))
            } else {
                existing_path
            };
        return (requested_name.to_string(), target_path, true);
    }

    let existing_names: HashSet<&str> = existing_saves
        .iter()
        .map(|save| save.name.as_str())
        .collect();
    let mut resolved_name = requested_name.to_string();
    let mut candidate_path = save_directory.join(format!("{resolved_name}.json"));
    let mut suffix = 2u32;

    while existing_names.contains(resolved_name.as_str()) || candidate_path.exists() {
        resolved_name = format!("{requested_name}_{suffix}");
        candidate_path = save_directory.join(format!("{resolved_name}.json"));
        suffix += 1;
    }

    (resolved_name, candidate_path, false)
}

fn is_valid_save_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| !name.is_empty())
        && path.extension().and_then(|extension| extension.to_str()) == Some("json")
}

/// Starts one background save task for each accepted request.
pub(crate) fn handle_save_requests(
    mut commands: Commands,
    mut requests: MessageReader<StartSaveGame>,
    save_file_manager: Res<SaveFileManager>,
    active_tasks: Query<(), With<SaveTask>>,
    mut ui_state: ResMut<SaveLoadUiState>,
) {
    let mut task_spawned = false;

    for request in requests.read() {
        if !active_tasks.is_empty() || task_spawned {
            ui_state.is_busy = true;
            ui_state.error_message.clear();
            ui_state.status_message = "A save operation is already running...".to_string();
            continue;
        }

        let state = request.state.clone();
        let save_directory = PathBuf::from(&save_file_manager.save_directory);
        let (resolved_name, file_path, is_overwrite) = resolve_save_target(
            &save_directory,
            &request.save_name,
            &save_file_manager.save_files,
        );
        let metadata = SaveFileMetadata {
            name: resolved_name.clone(),
            score: state.score,
            distance: state.distance_traveled,
            play_time: state.play_time,
            save_timestamp: state.save_timestamp,
            file_path: file_path.to_string_lossy().to_string(),
            selected_character: state.selected_character.clone(),
        };

        let task = ComputeTaskPool::get()
            .spawn(async move { io::save_game_state(file_path, state, metadata) });
        commands.spawn(SaveTask(task));
        task_spawned = true;

        ui_state.is_busy = true;
        ui_state.pending_load_index = None;
        ui_state.error_message.clear();
        ui_state.status_message = if is_overwrite {
            format!("Saving '{resolved_name}' (overwriting existing save)...")
        } else {
            format!("Saving '{resolved_name}'...")
        };
    }
}

/// Starts one background load task for each accepted request.
pub(crate) fn handle_load_requests(
    mut commands: Commands,
    mut requests: MessageReader<StartLoadGame>,
    active_tasks: Query<(), With<LoadTask>>,
    mut ui_state: ResMut<SaveLoadUiState>,
) {
    let mut task_spawned = false;

    for request in requests.read() {
        if !active_tasks.is_empty() || task_spawned {
            ui_state.is_busy = true;
            ui_state.error_message.clear();
            ui_state.status_message = "A load operation is already running...".to_string();
            continue;
        }

        let file_path = PathBuf::from(&request.file_path);
        let task = ComputeTaskPool::get().spawn(async move { io::load_game_state(file_path) });
        commands.spawn(LoadTask(task));
        task_spawned = true;

        ui_state.is_busy = true;
        ui_state.error_message.clear();
        ui_state.status_message = "Loading save data...".to_string();
    }
}

/// Applies completed background save/load operations to ECS resources.
pub(crate) fn poll_save_tasks(
    mut commands: Commands,
    mut save_tasks: Query<(Entity, &mut SaveTask)>,
    mut load_tasks: Query<(Entity, &mut LoadTask)>,
    mut state: PollSaveTasks,
) {
    for (entity, mut task) in &mut save_tasks {
        if let Some(result) = future::block_on(future::poll_once(&mut task.0)) {
            state.save_load_ui_state.is_busy = false;
            match result {
                Ok(()) => {
                    state.save_load_ui_state.error_message.clear();
                    state.save_load_ui_state.status_message =
                        "Save completed successfully".to_string();
                }
                Err(error) => {
                    state.save_load_ui_state.status_message.clear();
                    state.save_load_ui_state.error_message = error.to_user_message().to_string();
                    crate::debug_log!("Background save failed: {error}");
                }
            }
            commands.entity(entity).despawn();
        }
    }

    for (entity, mut task) in &mut load_tasks {
        if let Some(result) = future::block_on(future::poll_once(&mut task.0)) {
            state.save_load_ui_state.is_busy = false;
            match result {
                Ok((game_state, metadata)) => {
                    crate::debug_log!("Loaded save '{}'", metadata.name);
                    state.pause_manager.clear_pause_state();
                    state.loaded_game_state.state = Some(game_state);
                    state.loaded_game_state.should_restore = true;
                    state.loaded_game_state.previous_state = None;
                    state.save_load_ui_state.error_message.clear();
                    state.save_load_ui_state.pending_load_index = None;
                    state.save_load_ui_state.status_message =
                        "Load completed, restoring scene...".to_string();
                    NextState::set_if_neq(&mut state.next_state, GameState::Playing);
                }
                Err(error) => {
                    state.save_load_ui_state.status_message.clear();
                    state.save_load_ui_state.error_message = error.to_user_message().to_string();
                    crate::debug_log!("Background load failed: {error}");
                }
            }
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn metadata(name: &str, file_path: &Path) -> SaveFileMetadata {
        SaveFileMetadata {
            name: name.to_string(),
            score: 0,
            distance: 0.0,
            play_time: 0.0,
            save_timestamp: chrono::Utc::now(),
            file_path: file_path.to_string_lossy().to_string(),
            selected_character: crate::states::CharacterType::Shirou,
        }
    }

    fn temp_save_directory() -> PathBuf {
        let directory =
            std::env::temp_dir().join(format!("emiyashiro-save-target-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&directory).expect("create temp save directory");
        directory
    }

    #[test]
    fn available_name_is_reused() {
        let directory = temp_save_directory();
        let (name, path, overwrite) = resolve_save_target(&directory, "slot", &[]);

        assert_eq!(name, "slot");
        assert_eq!(
            path.file_name().and_then(|name| name.to_str()),
            Some("slot.json")
        );
        assert!(!overwrite);
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn existing_named_save_is_overwritten() {
        let directory = temp_save_directory();
        let existing_path = directory.join("slot.json");
        fs::write(&existing_path, b"{}").expect("write existing slot");
        let saves = vec![metadata("slot", &existing_path)];

        let (name, path, overwrite) = resolve_save_target(&directory, "slot", &saves);

        assert_eq!(name, "slot");
        assert_eq!(path, existing_path);
        assert!(overwrite);
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn filesystem_conflict_gets_a_suffix() {
        let directory = temp_save_directory();
        fs::write(directory.join("slot.json"), b"{}").expect("write existing slot");

        let (name, path, overwrite) = resolve_save_target(&directory, "slot", &[]);

        assert_eq!(name, "slot_2");
        assert_eq!(
            path.file_name().and_then(|name| name.to_str()),
            Some("slot_2.json")
        );
        assert!(!overwrite);
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn invalid_existing_path_falls_back_to_save_directory() {
        let directory = temp_save_directory();
        let saves = vec![metadata("slot", Path::new("."))];

        let (name, path, overwrite) = resolve_save_target(&directory, "slot", &saves);

        assert_eq!(name, "slot");
        assert_eq!(path, directory.join("slot.json"));
        assert!(overwrite);
        let _ = fs::remove_dir_all(directory);
    }
}
