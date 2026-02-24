use bevy::prelude::*;
use s_emiyashiro::plugins::EmiyaShiroClientPlugin;

fn main() {
    info!("Client startup: initializing Bevy app");
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "G-Engine Client (WebGPU)".into(),
                resolution: (1024, 768).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EmiyaShiroClientPlugin)
        .run();
}
