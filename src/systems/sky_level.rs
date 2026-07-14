//! Runtime bridge between the authored LDtk level and the existing action game.

use std::collections::{BTreeMap, HashMap, HashSet};

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::{
    components::*,
    events::{DamageEvent, DamageSource},
    states::GameState,
    systems::collision::CollisionBox,
};

const MAP_ENEMY_ACTIVATION_DISTANCE: f32 = 900.0;

type NewSkyWallQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static GridCoords,
        Option<&'static SkyStoneCell>,
        Option<&'static SkyCloudCell>,
    ),
    (With<SkyWallCell>, Without<SkyCellDecorated>),
>;
type UndecoratedBackdropQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static SkyBackdrop, &'static GridCoords),
    (Without<SkyEntityDecorated>, With<SkyBackdrop>),
>;
type UndecoratedCheckpointQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static SkyCheckpoint, &'static GridCoords),
    (Without<SkyEntityDecorated>, With<SkyCheckpoint>),
>;
type UndecoratedGoalQuery<'w, 's> =
    Query<'w, 's, (Entity, &'static GridCoords), (Without<SkyEntityDecorated>, With<SkyGoal>)>;
type UndecoratedGateQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static SkyCombatGate, &'static GridCoords),
    (Without<SkyEntityDecorated>, With<SkyCombatGate>),
>;
type UndecoratedClimbAnchorQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static SkyClimbAnchor),
    (Without<SkyEntityDecorated>, With<SkyClimbAnchor>),
>;
type RuntimeGateQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static SkyCombatGate,
        &'static GridCoords,
        Option<&'static Ground>,
        &'static mut Sprite,
    ),
    With<SkyGateVisual>,
>;

fn grid_translation(coords: GridCoords, z: f32) -> Vec3 {
    bevy_ecs_ldtk::utils::grid_coords_to_translation(coords, IVec2::splat(SKY_LEVEL_GRID)).extend(z)
}

fn player_is_inside_arena(player_x: f32, bounds: (f32, f32)) -> bool {
    const ENTRY_MARGIN: f32 = 48.0;
    player_x > bounds.0 + ENTRY_MARGIN && player_x < bounds.1 - ENTRY_MARGIN
}

/// Spawns the LDtk world and the full-map sky treatment once per play session.
pub fn spawn_sky_level_world(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    existing_world: Query<Entity, With<SkyLevelWorld>>,
    loaded_game_state: Option<Res<crate::systems::ui::LoadedGameState>>,
    mut runtime: ResMut<SkyLevelRuntime>,
) {
    if !existing_world.is_empty() {
        return;
    }

    runtime.active = true;
    runtime.level_ready = false;
    let loaded_position = loaded_game_state
        .as_deref()
        .filter(|state| state.should_restore)
        .and_then(|state| state.state.as_ref())
        .map(|state| state.player_position);
    runtime.player_initialized = loaded_position.is_some();
    runtime.bounds = Rect::new(0.0, 0.0, SKY_LEVEL_WIDTH, SKY_LEVEL_HEIGHT);
    runtime.start_position = SKY_LEVEL_START;
    runtime.checkpoint_position = loaded_position.unwrap_or(SKY_LEVEL_START);
    runtime.checkpoint_id = 0;
    runtime.checkpoint_needs_reconciliation = loaded_position.is_some();

    commands.spawn((
        Sprite {
            color: Color::srgb(0.46, 0.74, 0.92),
            custom_size: Some(Vec2::new(SKY_LEVEL_WIDTH, SKY_LEVEL_HEIGHT + 900.0)),
            ..default()
        },
        Transform::from_xyz(SKY_LEVEL_WIDTH * 0.5, SKY_LEVEL_HEIGHT * 0.5, -50.0),
        SkyLevelOwned,
    ));

    // Soft vertical color bands preserve a bright storybook sky behind the panorama.
    for (index, (color, y, height)) in [
        (Color::srgba(0.35, 0.68, 0.91, 0.72), 1_310.0, 460.0),
        (Color::srgba(0.63, 0.83, 0.94, 0.62), 910.0, 420.0),
        (Color::srgba(0.87, 0.93, 0.94, 0.54), 510.0, 420.0),
        (Color::srgba(0.98, 0.88, 0.66, 0.20), 250.0, 260.0),
    ]
    .into_iter()
    .enumerate()
    {
        commands.spawn((
            Sprite {
                color,
                custom_size: Some(Vec2::new(SKY_LEVEL_WIDTH, height)),
                ..default()
            },
            Transform::from_xyz(SKY_LEVEL_WIDTH * 0.5, y, -49.0 + index as f32 * 0.02),
            SkyLevelOwned,
        ));
    }

    let panorama = asset_server.load("images/levels/sky_city_panorama.png");
    for segment in 0..12 {
        commands.spawn((
            Sprite {
                image: panorama.clone(),
                custom_size: Some(Vec2::new(1_040.0, 1_024.0)),
                color: Color::srgba(1.0, 1.0, 1.0, 0.72),
                flip_x: segment % 2 == 1,
                ..default()
            },
            Transform::from_xyz(512.0 + segment as f32 * 1_024.0, 820.0, -45.0),
            SkyLevelOwned,
        ));
    }

    commands.spawn((
        LdtkWorldBundle {
            ldtk_handle: asset_server.load(SKY_LEVEL_PATH).into(),
            ..default()
        },
        SkyLevelWorld,
        SkyLevelOwned,
    ));
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct Plate {
    left: i32,
    right: i32,
}

#[derive(Clone, Copy, Debug)]
struct CellRect {
    left: i32,
    right: i32,
    top: i32,
    bottom: i32,
}

/// Coalesces IntGrid cells into large rectangles before adding them to the
/// existing AABB collision system.
pub fn build_merged_sky_colliders(
    mut commands: Commands,
    wall_query: Query<(&GridCoords, &ChildOf), Added<SkyWallCell>>,
    parent_query: Query<&ChildOf, Without<SkyWallCell>>,
    mut runtime: ResMut<SkyLevelRuntime>,
) {
    if wall_query.is_empty() {
        return;
    }

    let mut per_level: HashMap<Entity, HashSet<GridCoords>> = HashMap::new();
    for (&coords, child_of) in wall_query.iter() {
        if let Ok(layer_parent) = parent_query.get(child_of.parent()) {
            per_level
                .entry(layer_parent.parent())
                .or_default()
                .insert(coords);
        }
    }

    for (level_entity, cells) in per_level {
        let mut rows: HashMap<i32, Vec<Plate>> = HashMap::new();
        let max_y = cells.iter().map(|cell| cell.y).max().unwrap_or_default();
        let max_x = cells.iter().map(|cell| cell.x).max().unwrap_or_default();

        for y in 0..=max_y {
            let mut plates = Vec::new();
            let mut start = None;
            for x in 0..=max_x + 1 {
                match (start, cells.contains(&GridCoords { x, y })) {
                    (None, true) => start = Some(x),
                    (Some(left), false) => {
                        plates.push(Plate { left, right: x - 1 });
                        start = None;
                    }
                    _ => {}
                }
            }
            rows.insert(y, plates);
        }

        let mut builders: HashMap<Plate, CellRect> = HashMap::new();
        let mut previous = Vec::<Plate>::new();
        let mut rectangles = Vec::<CellRect>::new();

        for y in 0..=max_y + 1 {
            let current = rows.remove(&y).unwrap_or_default();
            for plate in &previous {
                if !current.contains(plate)
                    && let Some(rectangle) = builders.remove(plate)
                {
                    rectangles.push(rectangle);
                }
            }
            for plate in &current {
                builders
                    .entry(plate.clone())
                    .and_modify(|rectangle| rectangle.top = y)
                    .or_insert(CellRect {
                        left: plate.left,
                        right: plate.right,
                        bottom: y,
                        top: y,
                    });
            }
            previous = current;
        }

        commands.entity(level_entity).with_children(|level| {
            for rectangle in rectangles {
                let width = (rectangle.right - rectangle.left + 1) as f32 * SKY_LEVEL_GRID as f32;
                let height = (rectangle.top - rectangle.bottom + 1) as f32 * SKY_LEVEL_GRID as f32;
                let x = (rectangle.left + rectangle.right + 1) as f32 * SKY_LEVEL_GRID as f32 * 0.5;
                let y = (rectangle.bottom + rectangle.top + 1) as f32 * SKY_LEVEL_GRID as f32 * 0.5;

                level.spawn((
                    Transform::from_xyz(x, y, 0.0),
                    Ground,
                    CollisionBox::new(Vec2::new(width, height)),
                    SkyMergedCollider,
                ));
            }
        });
    }

    runtime.level_ready = true;
}

/// Turns invisible IntGrid authoring cells into a varied, readable floating-city surface.
pub fn decorate_sky_cells(
    mut commands: Commands,
    all_walls: Query<&GridCoords, With<SkyWallCell>>,
    new_walls: NewSkyWallQuery,
    hazards: Query<Entity, (With<SkyHazardCell>, Without<SkyCellDecorated>)>,
    winds: Query<Entity, (With<SkyWindCell>, Without<SkyCellDecorated>)>,
) {
    let occupied = all_walls.iter().copied().collect::<HashSet<_>>();

    for (entity, coords, _stone, cloud) in new_walls.iter() {
        let is_surface = !occupied.contains(&GridCoords::new(coords.x, coords.y + 1));
        commands.entity(entity).insert(SkyCellDecorated);

        if is_surface {
            commands.entity(entity).with_children(|cell| {
                cell.spawn((
                    Sprite {
                        color: if cloud.is_some() {
                            Color::srgba(1.0, 1.0, 1.0, 0.86)
                        } else {
                            Color::srgb(0.38, 0.56, 0.31)
                        },
                        custom_size: Some(Vec2::new(33.0, 7.0)),
                        ..default()
                    },
                    Transform::from_xyz(0.0, 13.5, 0.2),
                ));

                if cloud.is_none() && (coords.x + coords.y).rem_euclid(5) == 0 {
                    for offset in [-9.0_f32, 0.0, 8.0] {
                        cell.spawn((
                            Sprite {
                                color: Color::srgb(0.45, 0.68, 0.37),
                                custom_size: Some(Vec2::new(3.0, 10.0 + offset.abs() * 0.25)),
                                ..default()
                            },
                            Transform::from_xyz(offset, 20.0, 0.22)
                                .with_rotation(Quat::from_rotation_z(offset * 0.025)),
                        ));
                    }
                }
            });
        }
    }

    for entity in hazards.iter() {
        commands.entity(entity).insert(SkyCellDecorated);
    }

    for entity in winds.iter() {
        commands.entity(entity).insert(SkyCellDecorated);
    }
}

/// Builds landmark silhouettes and readable gameplay props from LDtk entities.
pub fn decorate_sky_entities(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    backdrops: UndecoratedBackdropQuery,
    checkpoints: UndecoratedCheckpointQuery,
    goals: UndecoratedGoalQuery,
    gates: UndecoratedGateQuery,
    climb_anchors: UndecoratedClimbAnchorQuery,
) {
    let cloud_texture = asset_server.load("images/cloud/cloud_soft_01.png");

    for (entity, backdrop, coords) in backdrops.iter() {
        let z = -30.0 + backdrop.depth as f32 * 2.0;
        commands.entity(entity).insert((
            Transform::from_translation(grid_translation(*coords, z)),
            SkyEntityDecorated,
        ));

        commands
            .entity(entity)
            .with_children(|parent| match backdrop.kind {
                SkyBackdropKind::Cloud => {
                    parent.spawn((
                        Sprite {
                            image: cloud_texture.clone(),
                            custom_size: Some(Vec2::new(330.0, 150.0) * backdrop.scale),
                            color: Color::srgba(1.0, 1.0, 1.0, 0.50),
                            ..default()
                        },
                        Transform::default(),
                    ));
                }
                SkyBackdropKind::Island => spawn_island_landmark(parent, backdrop.scale),
                SkyBackdropKind::Tower => spawn_tower_landmark(parent, backdrop.scale),
                SkyBackdropKind::Windmill => spawn_windmill_landmark(parent, backdrop.scale),
                SkyBackdropKind::Aqueduct => spawn_aqueduct_landmark(parent, backdrop.scale),
                SkyBackdropKind::Tree => {
                    spawn_tree_landmark(parent, backdrop.scale, cloud_texture.clone())
                }
                SkyBackdropKind::Waterfall => spawn_waterfall_landmark(parent, backdrop.scale),
                SkyBackdropKind::Temple => spawn_temple_landmark(parent, backdrop.scale),
                SkyBackdropKind::Crystal => spawn_crystal_landmark(parent, backdrop.scale),
            });
    }

    for (entity, checkpoint, coords) in checkpoints.iter() {
        commands
            .entity(entity)
            .insert(SkyEntityDecorated)
            .with_children(|parent| {
                parent.spawn((
                    Sprite {
                        color: Color::srgb(0.25, 0.39, 0.50),
                        custom_size: Some(Vec2::new(18.0, 70.0)),
                        ..default()
                    },
                    Transform::from_xyz(0.0, 28.0, 3.0),
                ));
                parent.spawn((
                    Sprite {
                        color: if checkpoint.id == 0 {
                            Color::srgb(0.52, 0.88, 1.0)
                        } else {
                            Color::srgb(0.98, 0.78, 0.32)
                        },
                        custom_size: Some(Vec2::splat(30.0)),
                        ..default()
                    },
                    Transform::from_xyz(0.0, 68.0, 3.2)
                        .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
                ));
            });
        let _ = coords;
    }

    for (entity, _) in goals.iter() {
        commands
            .entity(entity)
            .insert(SkyEntityDecorated)
            .with_children(|parent| {
                for radius in [118.0_f32, 88.0, 58.0] {
                    parent.spawn((
                        Sprite {
                            color: Color::srgba(0.42, 0.91, 1.0, 0.18 + radius / 700.0),
                            custom_size: Some(Vec2::splat(radius)),
                            ..default()
                        },
                        Transform::from_xyz(0.0, 58.0, 5.0 + radius * 0.001)
                            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
                    ));
                }
            });
    }

    for (entity, gate, coords) in gates.iter() {
        let mut position = grid_translation(*coords, 4.0);
        position.y += (gate.height - SKY_LEVEL_GRID as f32) * 0.5;
        commands.entity(entity).insert((
            Transform::from_translation(position),
            Sprite {
                color: Color::srgba(0.26, 0.52, 0.62, 0.14),
                custom_size: Some(Vec2::new(24.0, gate.height)),
                ..default()
            },
            SkyGateVisual,
            SkyEntityDecorated,
        ));
    }

    // Catchable edges use a compact cyan/gold grip marker. It is bright
    // enough to read during a fall without competing with combat telegraphs.
    for (entity, anchor) in climb_anchors.iter() {
        commands
            .entity(entity)
            .insert(SkyEntityDecorated)
            .with_children(|parent| {
                parent.spawn((
                    Sprite {
                        color: Color::srgba(0.38, 0.93, 1.0, 0.92),
                        custom_size: Some(Vec2::new(22.0, 5.0)),
                        ..default()
                    },
                    Transform::from_xyz(anchor.direction * 14.0, 14.0, 4.5),
                ));
                parent.spawn((
                    Sprite {
                        color: Color::srgba(1.0, 0.78, 0.28, 0.84),
                        custom_size: Some(Vec2::new(5.0, 14.0)),
                        ..default()
                    },
                    Transform::from_xyz(anchor.direction * 14.0, 8.0, 4.4),
                ));
            });
    }
}

fn spawn_island_landmark(parent: &mut ChildSpawnerCommands, scale: f32) {
    parent.spawn((
        Sprite {
            color: Color::srgba(0.33, 0.38, 0.38, 0.70),
            custom_size: Some(Vec2::new(310.0, 100.0) * scale),
            ..default()
        },
        Transform::from_xyz(0.0, -32.0 * scale, 0.0),
    ));
    parent.spawn((
        Sprite {
            color: Color::srgba(0.35, 0.56, 0.35, 0.78),
            custom_size: Some(Vec2::new(330.0, 24.0) * scale),
            ..default()
        },
        Transform::from_xyz(0.0, 30.0 * scale, 0.2),
    ));
    for index in 0..5 {
        parent.spawn((
            Sprite {
                color: Color::srgba(0.28, 0.31, 0.32, 0.66),
                custom_size: Some(Vec2::new(42.0, (55.0 + index as f32 * 18.0) * scale)),
                ..default()
            },
            Transform::from_xyz((-105.0 + index as f32 * 52.0) * scale, -90.0 * scale, 0.1)
                .with_rotation(Quat::from_rotation_z((index as f32 - 2.0) * 0.08)),
        ));
    }
}

fn spawn_tower_landmark(parent: &mut ChildSpawnerCommands, scale: f32) {
    parent.spawn((
        Sprite {
            color: Color::srgba(0.56, 0.57, 0.49, 0.72),
            custom_size: Some(Vec2::new(120.0, 330.0) * scale),
            ..default()
        },
        Transform::from_xyz(0.0, 125.0 * scale, 0.0),
    ));
    for y in [20.0_f32, 105.0, 190.0, 275.0] {
        parent.spawn((
            Sprite {
                color: Color::srgba(0.32, 0.48, 0.51, 0.70),
                custom_size: Some(Vec2::new(38.0, 52.0) * scale),
                ..default()
            },
            Transform::from_xyz(0.0, y * scale, 0.2),
        ));
    }
    parent.spawn((
        Sprite {
            color: Color::srgba(0.64, 0.46, 0.25, 0.78),
            custom_size: Some(Vec2::new(152.0, 42.0) * scale),
            ..default()
        },
        Transform::from_xyz(0.0, 310.0 * scale, 0.3),
    ));
}

fn spawn_windmill_landmark(parent: &mut ChildSpawnerCommands, scale: f32) {
    parent.spawn((
        Sprite {
            color: Color::srgba(0.50, 0.48, 0.39, 0.70),
            custom_size: Some(Vec2::new(42.0, 250.0) * scale),
            ..default()
        },
        Transform::from_xyz(0.0, 70.0 * scale, 0.0),
    ));
    for angle in [
        0.0_f32,
        std::f32::consts::FRAC_PI_2,
        std::f32::consts::PI,
        std::f32::consts::PI + std::f32::consts::FRAC_PI_2,
    ] {
        parent.spawn((
            Sprite {
                color: Color::srgba(0.72, 0.63, 0.44, 0.76),
                custom_size: Some(Vec2::new(34.0, 180.0) * scale),
                ..default()
            },
            Transform::from_xyz(0.0, 210.0 * scale, 0.3)
                .with_rotation(Quat::from_rotation_z(angle)),
        ));
    }
}

fn spawn_aqueduct_landmark(parent: &mut ChildSpawnerCommands, scale: f32) {
    parent.spawn((
        Sprite {
            color: Color::srgba(0.61, 0.61, 0.53, 0.66),
            custom_size: Some(Vec2::new(430.0, 42.0) * scale),
            ..default()
        },
        Transform::from_xyz(0.0, 140.0 * scale, 0.0),
    ));
    for x in [-165.0_f32, -55.0, 55.0, 165.0] {
        parent.spawn((
            Sprite {
                color: Color::srgba(0.58, 0.58, 0.50, 0.62),
                custom_size: Some(Vec2::new(34.0, 270.0) * scale),
                ..default()
            },
            Transform::from_xyz(x * scale, 10.0, 0.0),
        ));
    }
}

fn spawn_tree_landmark(parent: &mut ChildSpawnerCommands, scale: f32, cloud: Handle<Image>) {
    parent.spawn((
        Sprite {
            color: Color::srgba(0.35, 0.25, 0.16, 0.78),
            custom_size: Some(Vec2::new(90.0, 360.0) * scale),
            ..default()
        },
        Transform::from_xyz(0.0, 135.0 * scale, 0.0),
    ));
    for (x, y, size) in [
        (-105.0, 300.0, 230.0),
        (80.0, 325.0, 250.0),
        (0.0, 410.0, 280.0),
    ] {
        parent.spawn((
            Sprite {
                image: cloud.clone(),
                color: Color::srgba(0.25, 0.52, 0.30, 0.88),
                custom_size: Some(Vec2::new(size, size * 0.62) * scale),
                ..default()
            },
            Transform::from_xyz(x * scale, y * scale, 0.2),
        ));
    }
}

fn spawn_waterfall_landmark(parent: &mut ChildSpawnerCommands, scale: f32) {
    parent.spawn((
        Sprite {
            color: Color::srgba(0.48, 0.84, 0.96, 0.42),
            custom_size: Some(Vec2::new(54.0, 430.0) * scale),
            ..default()
        },
        Transform::from_xyz(0.0, -185.0 * scale, 0.0),
    ));
    parent.spawn((
        Sprite {
            color: Color::srgba(0.88, 0.98, 1.0, 0.44),
            custom_size: Some(Vec2::new(12.0, 430.0) * scale),
            ..default()
        },
        Transform::from_xyz(-12.0 * scale, -185.0 * scale, 0.2),
    ));
}

fn spawn_temple_landmark(parent: &mut ChildSpawnerCommands, scale: f32) {
    for x in [-150.0_f32, -75.0, 0.0, 75.0, 150.0] {
        parent.spawn((
            Sprite {
                color: Color::srgba(0.70, 0.68, 0.57, 0.72),
                custom_size: Some(Vec2::new(34.0, 250.0) * scale),
                ..default()
            },
            Transform::from_xyz(x * scale, 90.0 * scale, 0.0),
        ));
    }
    parent.spawn((
        Sprite {
            color: Color::srgba(0.59, 0.56, 0.45, 0.78),
            custom_size: Some(Vec2::new(410.0, 50.0) * scale),
            ..default()
        },
        Transform::from_xyz(0.0, 240.0 * scale, 0.2),
    ));
}

fn spawn_crystal_landmark(parent: &mut ChildSpawnerCommands, scale: f32) {
    for (index, x) in [-54.0_f32, 0.0, 54.0].into_iter().enumerate() {
        parent.spawn((
            Sprite {
                color: Color::srgba(0.42, 0.92, 1.0, 0.70),
                custom_size: Some(Vec2::new(44.0, 150.0 + index as f32 * 38.0) * scale),
                ..default()
            },
            Transform::from_xyz(x * scale, 70.0 * scale, 0.0)
                .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
        ));
    }
}

pub fn initialize_player_from_ldtk(
    starts: Query<&GridCoords, With<SkyPlayerStart>>,
    mut players: Query<(&mut Transform, &mut Velocity, &mut PlayerState), With<Player>>,
    mut cameras: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    mut runtime: ResMut<SkyLevelRuntime>,
) {
    if runtime.player_initialized || !runtime.level_ready {
        return;
    }
    let Some(coords) = starts.iter().next() else {
        return;
    };
    let mut spawn = grid_translation(*coords, 1.0);
    spawn.y += 14.0;

    if let Some((mut transform, mut velocity, mut state)) = players.iter_mut().next() {
        transform.translation = spawn;
        velocity.x = 0.0;
        velocity.y = 0.0;
        state.is_grounded = false;
    }
    for mut camera in cameras.iter_mut() {
        camera.translation = Vec3::new(spawn.x + 190.0, spawn.y + 120.0, camera.translation.z);
    }

    runtime.start_position = spawn;
    runtime.checkpoint_position = spawn;
    runtime.player_initialized = true;
}

pub fn activate_map_enemies(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    tuning: Option<Res<crate::resources::GameplayTuning>>,
    players: Query<&Transform, With<Player>>,
    spawns: Query<(Entity, &SkyEnemySpawn, &GridCoords)>,
) {
    let Some(player) = players.iter().next() else {
        return;
    };
    let default_tuning = crate::resources::GameplayTuning::default();
    let tuning = tuning.as_deref().unwrap_or(&default_tuning);

    for (entity, spawn, coords) in spawns.iter() {
        let mut position = grid_translation(*coords, 1.3);
        position.y += match spawn.kind {
            SkyEnemyKind::Slime => 3.0,
            SkyEnemyKind::Familiar => 96.0,
            SkyEnemyKind::HeroicSpirit => 34.0,
        };
        if (position.x - player.translation.x).abs() > MAP_ENEMY_ACTIVATION_DISTANCE {
            continue;
        }

        let enemy = crate::systems::enemy::spawn_authored_enemy(
            &mut commands,
            &asset_server,
            position,
            spawn.kind,
            spawn.health_multiplier,
            spawn.patrol_range,
            tuning,
        );
        commands.entity(enemy).insert(SkyEncounterEnemy {
            arena: spawn.arena,
            anchor_y: position.y,
        });
        commands.entity(entity).despawn();
    }
}

pub fn update_combat_gates(
    mut commands: Commands,
    players: Query<&Transform, With<Player>>,
    spawns: Query<&SkyEnemySpawn>,
    enemies: Query<(&SkyEncounterEnemy, &EnemyState), With<Enemy>>,
    mut gates: RuntimeGateQuery,
    mut encounters: ResMut<SkyEncounterState>,
) {
    let Some(player) = players.iter().next() else {
        return;
    };

    let mut arena_bounds = BTreeMap::<i32, (f32, f32)>::new();
    for (_, gate, coords, _, _) in gates.iter_mut() {
        let gate_x = grid_translation(*coords, 0.0).x;
        arena_bounds
            .entry(gate.arena)
            .and_modify(|bounds| {
                bounds.0 = bounds.0.min(gate_x);
                bounds.1 = bounds.1.max(gate_x);
            })
            .or_insert((gate_x, gate_x));
    }

    if let Some(arena) = encounters.active_arena
        && arena_bounds
            .get(&arena)
            .is_none_or(|&bounds| !player_is_inside_arena(player.translation.x, bounds))
    {
        // A revive or external reposition must never leave the player locked
        // outside a closed arena. Re-entering resumes the same encounter.
        encounters.active_arena = None;
    }

    if encounters.active_arena.is_none() {
        for (&arena, &bounds) in &arena_bounds {
            if player_is_inside_arena(player.translation.x, bounds)
                && !encounters.completed_arenas.contains(&arena)
            {
                encounters.active_arena = Some(arena);
                break;
            }
        }
    }

    if let Some(arena) = encounters.active_arena {
        let pending = spawns.iter().any(|spawn| spawn.arena == arena);
        let alive = enemies
            .iter()
            .any(|(member, state)| member.arena == arena && state.is_alive);
        if !pending && !alive {
            encounters.completed_arenas.insert(arena);
            encounters.active_arena = None;
        }
    }

    for (entity, gate, _, ground, mut sprite) in &mut gates {
        let should_close = encounters.active_arena == Some(gate.arena)
            && !encounters.completed_arenas.contains(&gate.arena)
            && arena_bounds
                .get(&gate.arena)
                .is_some_and(|&bounds| player_is_inside_arena(player.translation.x, bounds));
        if should_close {
            sprite.color = Color::srgba(0.35, 0.90, 1.0, 0.82);
            if ground.is_none() {
                commands
                    .entity(entity)
                    .insert((Ground, CollisionBox::new(Vec2::new(24.0, gate.height))));
            }
        } else {
            sprite.color = Color::srgba(0.35, 0.90, 1.0, 0.12);
            if ground.is_some() {
                commands.entity(entity).remove::<(Ground, CollisionBox)>();
            }
        }
    }
}

pub fn activate_checkpoints(
    players: Query<&Transform, With<Player>>,
    checkpoints: Query<(&SkyCheckpoint, &GridCoords)>,
    mut runtime: ResMut<SkyLevelRuntime>,
) {
    let Some(player) = players.iter().next() else {
        return;
    };

    if runtime.checkpoint_needs_reconciliation && !checkpoints.is_empty() {
        let resolved = checkpoints
            .iter()
            .map(|(checkpoint, coords)| {
                let mut position = grid_translation(*coords, 1.0);
                position.y += 14.0;
                (checkpoint.id, position)
            })
            .filter(|(_, position)| position.x <= player.translation.x + 16.0)
            .max_by_key(|(id, _)| *id);
        if let Some((id, position)) = resolved {
            runtime.checkpoint_id = id;
            runtime.checkpoint_position = position;
        }
        runtime.checkpoint_needs_reconciliation = false;
    }

    for (checkpoint, coords) in checkpoints.iter() {
        let mut position = grid_translation(*coords, 1.0);
        position.y += 14.0;
        if checkpoint.id > runtime.checkpoint_id
            && player.translation.truncate().distance(position.truncate()) < 76.0
        {
            runtime.checkpoint_id = checkpoint.id;
            runtime.checkpoint_position = position;
        }
    }
}

pub fn apply_wind_lifts(
    winds: Query<&GridCoords, With<SkyWindCell>>,
    mut players: Query<(&Transform, &mut Velocity), With<Player>>,
) {
    let Some((player, mut velocity)) = players.iter_mut().next() else {
        return;
    };
    for coords in winds.iter() {
        let position = grid_translation(*coords, 0.0);
        if (position.x - player.translation.x).abs() < 30.0
            && (position.y - player.translation.y).abs() < 44.0
        {
            velocity.y = velocity.y.max(315.0);
            break;
        }
    }
}

pub fn damage_sky_hazards(
    hazards: Query<&GridCoords, With<SkyHazardCell>>,
    players: Query<(Entity, &Transform, &CollisionBox), With<Player>>,
    mut damage_writer: MessageWriter<DamageEvent>,
) {
    let Some((player_entity, player, collision)) = players.iter().next() else {
        return;
    };
    for coords in hazards.iter() {
        let position = grid_translation(*coords, 0.0);
        let dx = (position.x - player.translation.x).abs();
        let dy = (position.y - player.translation.y).abs();
        if dx < (SKY_LEVEL_GRID as f32 + collision.size.x) * 0.5
            && dy < (SKY_LEVEL_GRID as f32 + collision.size.y) * 0.5
        {
            damage_writer.write(DamageEvent {
                target: player_entity,
                amount: 24.0,
                source: DamageSource::EnemyContact,
            });
            break;
        }
    }
}

pub fn detect_sky_goal(
    goals: Query<&GridCoords, With<SkyGoal>>,
    players: Query<&Transform, With<Player>>,
    encounters: Res<SkyEncounterState>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if !encounters.completed_arenas.contains(&3) {
        return;
    }
    let Some(player) = players.iter().next() else {
        return;
    };
    for coords in goals.iter() {
        let mut goal = grid_translation(*coords, 0.0);
        goal.y += 48.0;
        if player.translation.truncate().distance(goal.truncate()) < 96.0 {
            NextState::set_if_neq(&mut next_state, GameState::Victory);
            break;
        }
    }
}

pub fn setup_victory_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(18.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.03, 0.10, 0.17, 0.76)),
            SkyVictoryUi,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("SKY CITY LIBERATED"),
                TextFont {
                    font: font.clone().into(),
                    font_size: FontSize::Px(54.0),
                    ..default()
                },
                TextColor(Color::srgb(0.80, 0.96, 1.0)),
            ));
            parent.spawn((
                Text::new("Windheart Sanctuary · Complete"),
                TextFont {
                    font: font.clone().into(),
                    font_size: FontSize::Px(28.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.82, 0.42)),
            ));
            parent.spawn((
                Text::new("Press Enter or M to return to the menu"),
                TextFont {
                    font: font.into(),
                    font_size: FontSize::Px(22.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

pub fn handle_victory_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::KeyM) {
        NextState::set_if_neq(&mut next_state, GameState::Menu);
    }
}

pub fn cleanup_victory_ui(mut commands: Commands, query: Query<Entity, With<SkyVictoryUi>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn cleanup_sky_level(
    mut commands: Commands,
    query: Query<Entity, With<SkyLevelOwned>>,
    mut runtime: ResMut<SkyLevelRuntime>,
    mut encounters: ResMut<SkyEncounterState>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    *runtime = SkyLevelRuntime::default();
    *encounters = SkyEncounterState::default();
}
