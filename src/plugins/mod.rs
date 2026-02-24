use bevy::prelude::*;

pub mod core;
pub mod gameplay;
pub mod netcode;
pub mod persistence;
pub mod presentation;
pub mod server;
pub mod ui;

/// Aggregated client-side plugin entrypoint.
/// Keeps `src/bin/client.rs` lightweight and focused on app bootstrap only.
pub struct EmiyaShiroClientPlugin;

impl Plugin for EmiyaShiroClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            core::CorePlugin,
            netcode::NetcodePlugin,
            gameplay::GameplayPlugin,
            persistence::PersistencePlugin,
            presentation::PresentationPlugin,
            ui::UiPlugin,
        ));
    }
}
