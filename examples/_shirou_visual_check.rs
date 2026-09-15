use bevy::{
    app::AppExit,
    asset::AssetPlugin,
    prelude::*,
    render::view::screenshot::{Screenshot, save_to_disk},
};
use emiyashiro::{
    plugins::{core::CorePlugin, gameplay::GameplayPlugin, netcode::NetcodePlugin,
        persistence::PersistencePlugin, presentation::PresentationPlugin, ui::UiPlugin},
    states::GameState,
    systems::interfaces::GameSystemSet,
};

#[derive(Default, Resource)]
struct Review {
    elapsed: f32,
    started: bool,
    captured: usize,
}

fn drive(
    time: Res<Time<Real>>,
    state: Res<State<GameState>>,
    mut next: ResMut<NextState<GameState>>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut review: ResMut<Review>,
    mut sky: ResMut<emiyashiro::components::SkyLevelRuntime>,
    mut commands: Commands,
    mut exit: MessageWriter<AppExit>,
) {
    if !review.started {
        sky.active = false;
        review.started = true;
        next.set(GameState::Playing);
    }
    if *state.get() != GameState::Playing {
        if *state.get() == GameState::GameOver {
            eprintln!("visual review reached GameOver");
            exit.write(AppExit::error());
        }
        return;
    }
    let previous = review.elapsed;
    review.elapsed += time.delta_secs().min(0.05);
    let t = review.elapsed;
    keys.reset_all();
    if (2.0..2.8).contains(&t) { keys.press(KeyCode::KeyD); }
    if (2.8..3.6).contains(&t) { keys.press(KeyCode::KeyA); }
    if (3.6..3.7).contains(&t) { keys.press(KeyCode::KeyD); }
    if (3.7..3.85).contains(&t) { keys.press(KeyCode::Space); }
    if (4.6..5.0).contains(&t) || (7.0..7.4).contains(&t) { keys.press(KeyCode::KeyS); }
    for (at, key) in [
        (5.2, KeyCode::KeyJ), (5.3, KeyCode::KeyJ),
        (6.0, KeyCode::KeyI), (6.4, KeyCode::KeyO),
        (7.05, KeyCode::KeyJ), (7.8, KeyCode::Space),
        (7.95, KeyCode::KeyJ), (8.6, KeyCode::KeyK),
        (9.5, KeyCode::KeyV), (10.0, KeyCode::KeyJ),
    ] {
        if previous < at && t >= at {
            keys.press(key);
            if key == KeyCode::KeyV { keys.press(KeyCode::ShiftLeft); }
        }
    }
    let shots = [
        (1.5, "idle"), (2.4, "run_right"), (3.2, "run_left"),
        (3.95, "jump"), (4.8, "crouch"), (5.38, "jab"),
        (6.13, "thrust"), (6.55, "sweep"), (7.18, "dash"),
        (8.1, "air_attack"), (8.8, "heavy"), (10.1, "overedge"),
    ];
    if let Some((at, name)) = shots.get(review.captured) {
        if t >= *at {
            println!("capturing {name} at {t:.2}s");
            commands.spawn(Screenshot::primary_window())
                .observe(save_to_disk(format!("tmp/shirou_repaired_review/game_{name}.png")));
            review.captured += 1;
        }
    }
    if t > 11.0 { exit.write(AppExit::Success); }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins
            .set(AssetPlugin {
                file_path: format!("{}/assets", env!("CARGO_MANIFEST_DIR")),
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Shirou animation check".into(),
                    resolution: (1024, 768).into(),
                    ..default()
                }),
                ..default()
            }))
        // Exercise the real player systems on the built-in flat floor, avoiding
        // level-specific wind/traversal during a frame-by-frame sprite review.
        .add_plugins((CorePlugin, NetcodePlugin, GameplayPlugin, PersistencePlugin,
            PresentationPlugin, UiPlugin))
        .init_resource::<emiyashiro::components::SkyLevelRuntime>()
        .init_resource::<emiyashiro::components::SkyEncounterState>()
        .init_resource::<Review>()
        .add_systems(Update, drive.before(GameSystemSet::Input))
        .run();
}
