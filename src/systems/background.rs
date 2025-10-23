use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::components::Cloud;

const CLOUD_SPEED: f32 = 50.0;
const CLOUD_SPAWN_TIME: f32 = 5.0;

#[derive(Resource)]
pub struct CloudSpawnTimer(pub Timer);

pub fn setup_cloud_spawner(mut commands: Commands) {
    commands.insert_resource(CloudSpawnTimer(Timer::from_seconds(
        CLOUD_SPAWN_TIME,
        TimerMode::Repeating,
    )));
}

pub fn spawn_clouds_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
    mut cloud_spawn_timer: ResMut<CloudSpawnTimer>,
) {
    if cloud_spawn_timer.0.tick(time.delta()).just_finished() {
        let window = window_query.single().unwrap();

        // HACK: Using time for pseudo-randomness to avoid rand dependency issue.
        let pseudo_random = (time.elapsed_secs() * 100.0) as u32;
        let cloud_y =
            (pseudo_random % (window.height() * 0.4) as u32) as f32 + window.height() * 0.5;

        commands
            .spawn((
                Transform::from_xyz(window.width() + 100.0, cloud_y, 0.0),
                Cloud,
            ))
            .with_children(|parent| {
                // Simple cloud shape made of circles
                parent.spawn((
                    Mesh2d(meshes.add(Circle::new(30.0))),
                    MeshMaterial2d(materials.add(ColorMaterial::from(Color::WHITE))),
                    Transform::from_xyz(0.0, 0.0, 0.0),
                ));
                parent.spawn((
                    Mesh2d(meshes.add(Circle::new(25.0))),
                    MeshMaterial2d(materials.add(ColorMaterial::from(Color::WHITE))),
                    Transform::from_xyz(-20.0, -10.0, 0.0),
                ));
                parent.spawn((
                    Mesh2d(meshes.add(Circle::new(25.0))),
                    MeshMaterial2d(materials.add(ColorMaterial::from(Color::WHITE))),
                    Transform::from_xyz(20.0, -10.0, 0.0),
                ));
            });
    }
}

pub fn move_clouds_system(mut cloud_query: Query<&mut Transform, With<Cloud>>, time: Res<Time>) {
    for mut transform in cloud_query.iter_mut() {
        transform.translation.x -= CLOUD_SPEED * time.delta_secs();
    }
}

pub fn despawn_offscreen_clouds_system(
    mut commands: Commands,
    cloud_query: Query<(Entity, &Transform), With<Cloud>>,
) {
    for (entity, transform) in cloud_query.iter() {
        if transform.translation.x < -200.0 {
            // Despawn when off-screen
            commands.entity(entity).despawn();
        }
    }
}
