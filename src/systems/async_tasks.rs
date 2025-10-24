use crate::{
    events::{StartLoadGame, StartSaveGame},
    resources::{CompleteGameState, SaveFileManager, SaveFileMetadata},
    states::GameState,
    systems::{
        async_file_ops::{AsyncFileManager, load_game_state_async, save_game_state_async},
        error_handling::SaveSystemError,
    },
    ui::LoadedGameState,
};
use bevy::prelude::*;
use bevy::tasks::{ComputeTaskPool, Task};
use futures_lite::future;
use std::path::PathBuf;

#[derive(Component)]
pub struct SaveTask(Task<Result<(), SaveSystemError>>);

#[derive(Component)]
pub struct LoadTask(Task<Result<(CompleteGameState, SaveFileMetadata), SaveSystemError>>);

/// System to handle save game requests by spawning them on the async compute pool.
pub fn handle_save_requests(
    mut commands: Commands,
    mut ev_save: MessageReader<StartSaveGame>,
    file_manager: Res<AsyncFileManager>,
    save_file_manager: Res<SaveFileManager>,
) {
    for ev in ev_save.read() {
        let state = ev.state.clone();
        let save_name = ev.save_name.clone();
        let compression_enabled = file_manager.compression_enabled;
        let compression_level = file_manager.compression_level;

        let save_dir = PathBuf::from(&save_file_manager.save_directory);
        let file_path = save_dir.join(format!("{}.json", save_name));

        let metadata = SaveFileMetadata {
            name: save_name.clone(),
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
        println!("💾 Spawned async save task for '{}'", save_name);
    }
}

/// System to handle load game requests by spawning them on the async compute pool.
pub fn handle_load_requests(
    mut commands: Commands,
    mut ev_load: MessageReader<StartLoadGame>,
    file_manager: Res<AsyncFileManager>,
) {
    for ev in ev_load.read() {
        let file_path = PathBuf::from(&ev.file_path);
        let compression_enabled = file_manager.compression_enabled;

        let task = ComputeTaskPool::get()
            .spawn(async move { load_game_state_async(file_path, compression_enabled).await });

        commands.spawn(LoadTask(task));
        println!("📂 Spawned async load task for '{}'", ev.file_path);
    }
}

/// System to poll the async save/load tasks and handle their completion.
pub fn poll_async_tasks(
    mut commands: Commands,
    mut save_tasks: Query<(Entity, &mut SaveTask)>,
    mut load_tasks: Query<(Entity, &mut LoadTask)>,
    mut next_state: ResMut<NextState<GameState>>,
    mut loaded_game_state: ResMut<LoadedGameState>,
) {
    // Poll save tasks
    for (entity, mut task) in &mut save_tasks {
        if let Some(result) = future::block_on(future::poll_once(&mut task.0)) {
            match result {
                Ok(_) => println!("✅ Async save task completed successfully."),
                Err(e) => println!("❌ Async save task failed: {:?}", e),
            }
            commands.entity(entity).despawn();
        }
    }

    // Poll load tasks
    for (entity, mut task) in &mut load_tasks {
        if let Some(result) = future::block_on(future::poll_once(&mut task.0)) {
            match result {
                Ok((game_state, metadata)) => {
                    println!(
                        "✅ Async load task for '{}' completed successfully.",
                        metadata.name
                    );
                    loaded_game_state.state = Some(game_state);
                    loaded_game_state.should_restore = true;
                    next_state.set(GameState::Playing);
                }
                Err(e) => {
                    println!("❌ Async load task failed: {:?}", e);
                    // Here we could fire another event to show an error UI
                }
            }
            commands.entity(entity).despawn();
        }
    }
}
