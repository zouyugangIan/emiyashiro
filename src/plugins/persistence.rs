use bevy::prelude::*;

use crate::{
    states::GameState,
    systems::{self, interfaces::GameSystemSet},
};

/// Persistence systems: save/load IO, async tasks, pause snapshot and DB hooks.
pub struct PersistencePlugin;

impl Plugin for PersistencePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                systems::save::auto_save_system,
                systems::database_service::database_stats_system,
                systems::database_service::cleanup_old_sessions,
            )
                .in_set(GameSystemSet::Persistence)
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            (
                systems::async_file_ops::update_operation_progress,
                systems::async_file_ops::display_progress_indicator,
                systems::ui::update_save_load_status_text,
            )
                .in_set(GameSystemSet::Persistence)
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
                systems::async_tasks::handle_save_requests,
                systems::async_tasks::handle_load_requests,
                systems::async_tasks::poll_async_tasks,
            )
                .in_set(GameSystemSet::Persistence),
        )
        .add_systems(
            Update,
            systems::pause_save::handle_pause_input
                .in_set(GameSystemSet::Input)
                .run_if(in_state(GameState::Playing).or(in_state(GameState::Paused))),
        )
        .add_systems(
            Update,
            systems::pause_save::restore_paused_state
                .in_set(GameSystemSet::Persistence)
                .run_if(in_state(GameState::Playing)),
        );
    }
}
