use bevy::prelude::*;

use crate::{
    states::GameState,
    systems::{self, interfaces::GameSystemSet},
};

/// Persistence systems: local save/load tasks and pause snapshots.
pub struct PersistencePlugin;

impl Plugin for PersistencePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            systems::save::auto_save_system
                .in_set(GameSystemSet::Persistence)
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            systems::ui::update_save_load_status_text
                .in_set(GameSystemSet::Persistence)
                .run_if(
                    in_state(GameState::Playing)
                        .or_else(in_state(GameState::Paused))
                        .or_else(in_state(GameState::SaveDialog))
                        .or_else(in_state(GameState::LoadTable)),
                ),
        )
        .add_systems(
            Update,
            (
                systems::save::handle_save_requests,
                systems::save::handle_load_requests,
                systems::save::poll_save_tasks,
            )
                .in_set(GameSystemSet::Persistence),
        )
        .add_systems(
            Update,
            systems::pause::handle_pause_input
                .in_set(GameSystemSet::Input)
                .run_if(in_state(GameState::Playing).or_else(in_state(GameState::Paused))),
        )
        .add_systems(
            Update,
            systems::pause::restore_paused_state
                .in_set(GameSystemSet::Persistence)
                .run_if(in_state(GameState::Playing)),
        );
    }
}
