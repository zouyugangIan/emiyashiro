use crate::{
    events::{StartLoadGame, StartSaveGame},
    resources::{CompleteGameState, PauseManager, SaveFileManager, SaveFileMetadata},
    states::GameState,
    systems::{
        async_file_ops::{
            AsyncFileManager, OperationProgress, load_game_state_async, save_game_state_async,
        },
        error_handling::SaveSystemError,
        ui::{LoadedGameState, SaveLoadUiState},
    },
};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::tasks::{ComputeTaskPool, Task};
use futures_lite::future;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Component)]
pub struct SaveTask(Task<Result<(), SaveSystemError>>);

#[derive(Component)]
pub struct LoadTask(Task<Result<(CompleteGameState, SaveFileMetadata), SaveSystemError>>);

#[derive(SystemParam)]
pub struct PollAsyncTaskState<'w> {
    next_state: ResMut<'w, NextState<GameState>>,
    loaded_game_state: ResMut<'w, LoadedGameState>,
    pause_manager: ResMut<'w, PauseManager>,
    operation_progress: ResMut<'w, OperationProgress>,
    save_load_ui_state: ResMut<'w, SaveLoadUiState>,
}

fn resolve_unique_save_target(
    save_dir: &Path,
    requested_name: &str,
    existing_names: &HashSet<String>,
) -> (String, PathBuf) {
    let mut resolved_name = requested_name.to_string();
    let mut candidate_path = save_dir.join(format!("{}.json", resolved_name));

    let mut suffix = 2u32;
    while existing_names.contains(&resolved_name) || candidate_path.exists() {
        resolved_name = format!("{}_{}", requested_name, suffix);
        candidate_path = save_dir.join(format!("{}.json", resolved_name));
        suffix += 1;
    }

    (resolved_name, candidate_path)
}

/// System to handle save game requests by spawning them on the async compute pool.
pub fn handle_save_requests(
    mut commands: Commands,
    mut ev_save: MessageReader<StartSaveGame>,
    file_manager: Res<AsyncFileManager>,
    save_file_manager: Res<SaveFileManager>,
    active_save_tasks: Query<(), With<SaveTask>>,
    mut operation_progress: ResMut<OperationProgress>,
    mut save_load_ui_state: ResMut<SaveLoadUiState>,
) {
    let mut save_task_spawned = false;

    for ev in ev_save.read() {
        if !active_save_tasks.is_empty() || save_task_spawned {
            save_load_ui_state.is_busy = true;
            save_load_ui_state.error_message.clear();
            save_load_ui_state.status_message =
                "A save operation is already running...".to_string();
            continue;
        }

        let state = ev.state.clone();
        let save_name = ev.save_name.clone();
        let compression_enabled = file_manager.compression_enabled;
        let compression_level = file_manager.compression_level;

        let save_dir = PathBuf::from(&save_file_manager.save_directory);
        let existing_names: HashSet<String> = save_file_manager
            .save_files
            .iter()
            .map(|save| save.name.clone())
            .collect();
        let (resolved_name, file_path) =
            resolve_unique_save_target(&save_dir, &save_name, &existing_names);

        let metadata = SaveFileMetadata {
            name: resolved_name.clone(),
            score: state.score,
            distance: state.distance_traveled,
            play_time: state.play_time,
            save_timestamp: state.save_timestamp,
            file_path: file_path.to_string_lossy().to_string(),
        };

        let task = ComputeTaskPool::get().spawn(async move {
            save_game_state_async(
                file_path,
                state,
                metadata,
                compression_enabled,
                compression_level,
            )
            .await
        });

        commands.spawn(SaveTask(task));
        save_task_spawned = true;

        save_load_ui_state.is_busy = true;
        save_load_ui_state.pending_load_index = None;
        save_load_ui_state.error_message.clear();
        save_load_ui_state.status_message = format!("Saving '{}'...", resolved_name);
        operation_progress.start_operation(format!("Saving '{}'", resolved_name));

        crate::debug_log!("馃捑 Spawned async save task for '{}'", save_name);
    }
}

/// System to handle load game requests by spawning them on the async compute pool.
pub fn handle_load_requests(
    mut commands: Commands,
    mut ev_load: MessageReader<StartLoadGame>,
    file_manager: Res<AsyncFileManager>,
    active_load_tasks: Query<(), With<LoadTask>>,
    mut operation_progress: ResMut<OperationProgress>,
    mut save_load_ui_state: ResMut<SaveLoadUiState>,
) {
    let mut load_task_spawned = false;

    for ev in ev_load.read() {
        if !active_load_tasks.is_empty() || load_task_spawned {
            save_load_ui_state.is_busy = true;
            save_load_ui_state.error_message.clear();
            save_load_ui_state.status_message =
                "A load operation is already running...".to_string();
            continue;
        }

        let file_path = PathBuf::from(&ev.file_path);
        let compression_enabled = file_manager.compression_enabled;

        let task = ComputeTaskPool::get()
            .spawn(async move { load_game_state_async(file_path, compression_enabled).await });

        commands.spawn(LoadTask(task));
        load_task_spawned = true;

        save_load_ui_state.is_busy = true;
        save_load_ui_state.error_message.clear();
        save_load_ui_state.status_message = "Loading save data...".to_string();
        operation_progress.start_operation("Loading save".to_string());

        crate::debug_log!("馃搨 Spawned async load task for '{}'", ev.file_path);
    }
}

/// System to poll the async save/load tasks and handle their completion.
pub fn poll_async_tasks(
    mut commands: Commands,
    mut save_tasks: Query<(Entity, &mut SaveTask)>,
    mut load_tasks: Query<(Entity, &mut LoadTask)>,
    mut state: PollAsyncTaskState,
) {
    // Poll save tasks
    for (entity, mut task) in &mut save_tasks {
        if let Some(result) = future::block_on(future::poll_once(&mut task.0)) {
            match result {
                Ok(_) => {
                    state.save_load_ui_state.is_busy = false;
                    state.save_load_ui_state.error_message.clear();
                    state.save_load_ui_state.status_message =
                        "Save completed successfully".to_string();
                    state.operation_progress.complete_operation();
                    crate::debug_log!("鉁?Async save task completed successfully.");
                }
                Err(e) => {
                    state.save_load_ui_state.is_busy = false;
                    state.save_load_ui_state.status_message.clear();
                    state.save_load_ui_state.error_message = e.to_user_message().to_string();
                    state.operation_progress.complete_operation();
                    crate::debug_log!("鉂?Async save task failed: {:?}", e);
                }
            }
            commands.entity(entity).despawn();
        }
    }

    // Poll load tasks
    for (entity, mut task) in &mut load_tasks {
        if let Some(result) = future::block_on(future::poll_once(&mut task.0)) {
            match result {
                Ok((game_state, metadata)) => {
                    crate::debug_log!(
                        "鉁?Async load task for '{}' completed successfully.",
                        metadata.name
                    );

                    // Prevent paused snapshot from overriding the loaded save
                    state.pause_manager.clear_pause_state();

                    state.loaded_game_state.state = Some(game_state);
                    state.loaded_game_state.should_restore = true;
                    state.loaded_game_state.previous_state = None;

                    state.save_load_ui_state.is_busy = false;
                    state.save_load_ui_state.error_message.clear();
                    state.save_load_ui_state.pending_load_index = None;
                    state.save_load_ui_state.status_message =
                        "Load completed, restoring scene...".to_string();
                    state.operation_progress.complete_operation();

                    state.next_state.set(GameState::Playing);
                }
                Err(e) => {
                    state.save_load_ui_state.is_busy = false;
                    state.save_load_ui_state.status_message.clear();
                    state.save_load_ui_state.error_message = e.to_user_message().to_string();
                    state.operation_progress.complete_operation();
                    crate::debug_log!("鉂?Async load task failed: {:?}", e);
                    // Here we could fire another event to show an error UI
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

    #[test]
    fn resolve_unique_save_target_reuses_requested_name_when_available() {
        let temp_dir = std::env::temp_dir().join(format!(
            "emiyashiro-save-target-test-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&temp_dir).expect("create temp save dir");

        let existing_names = HashSet::new();
        let (resolved_name, resolved_path) =
            resolve_unique_save_target(&temp_dir, "slot", &existing_names);

        assert_eq!(resolved_name, "slot");
        assert_eq!(
            resolved_path.file_name().and_then(|name| name.to_str()),
            Some("slot.json")
        );

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn resolve_unique_save_target_appends_suffix_for_conflicts() {
        let temp_dir = std::env::temp_dir().join(format!(
            "emiyashiro-save-target-test-{}",
            uuid::Uuid::new_v4()
        ));
        fs::create_dir_all(&temp_dir).expect("create temp save dir");
        fs::write(temp_dir.join("slot.json"), b"{}").expect("write existing slot");

        let mut existing_names = HashSet::new();
        existing_names.insert("slot".to_string());

        let (resolved_name, resolved_path) =
            resolve_unique_save_target(&temp_dir, "slot", &existing_names);

        assert_eq!(resolved_name, "slot_2");
        assert_eq!(
            resolved_path.file_name().and_then(|name| name.to_str()),
            Some("slot_2.json")
        );

        let _ = fs::remove_dir_all(temp_dir);
    }
}
