use std::{
    collections::{BTreeMap, HashSet},
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::GridCoords;
use serde_json::Value;

use crate::{
    components::{
        AttackAnimationState, Enemy, EnemyState, EnemyType, FacingDirection, Ground,
        LedgeTraversal, LedgeTraversalPhase, Player, PlayerState, SkyCheckpoint, SkyClimbAnchor,
        SkyCombatGate, SkyEncounterEnemy, SkyEncounterState, SkyEnemyKind, SkyEnemySpawn,
        SkyGateVisual, SkyLevelRuntime, SkyPlayerStart, Velocity,
    },
    systems::{collision::CollisionBox, sky_level},
};

fn asset_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join(relative)
}

fn sky_project() -> Value {
    let source = fs::read_to_string(asset_path("levels/sky_city_of_winds.ldtk"))
        .expect("production LDtk project must be readable");
    serde_json::from_str::<bevy_ecs_ldtk::ldtk::LdtkJson>(&source)
        .expect("bevy_ecs_ldtk must deserialize the production project");
    serde_json::from_str(&source).expect("production LDtk project must contain valid JSON")
}

fn field_value<'a>(entity: &'a Value, name: &str) -> &'a Value {
    entity["fieldInstances"]
        .as_array()
        .expect("fieldInstances array")
        .iter()
        .find(|field| field["__identifier"] == name)
        .unwrap_or_else(|| panic!("missing LDtk field {name}"))
        .get("__value")
        .expect("field value")
}

#[test]
fn sky_city_ldtk_contract_is_complete_and_batched() {
    let project = sky_project();
    assert_eq!(project["jsonVersion"], "1.5.3");

    let level = &project["levels"][0];
    assert_eq!(level["identifier"], "Windheart_Sky_City");
    assert_eq!(level["pxWid"], 12_288);
    assert_eq!(level["pxHei"], 1_536);

    let layers = level["layerInstances"]
        .as_array()
        .expect("layerInstances array");
    let by_name = layers
        .iter()
        .map(|layer| (layer["__identifier"].as_str().expect("layer name"), layer))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(
        by_name.keys().copied().collect::<Vec<_>>(),
        vec!["Collision", "Decor", "Gameplay", "TerrainTiles"]
    );

    let collision = by_name["Collision"]["intGridCsv"]
        .as_array()
        .expect("IntGrid CSV");
    assert_eq!(collision.len(), 384 * 48);
    let visible_cell_count = collision
        .iter()
        .filter(|cell| cell.as_i64() != Some(0))
        .count();
    assert!(
        visible_cell_count > 1_800,
        "level should have dense authored terrain"
    );

    let terrain_tiles = by_name["TerrainTiles"]["gridTiles"]
        .as_array()
        .expect("terrain tile instances");
    assert_eq!(terrain_tiles.len(), visible_cell_count);
    assert!(terrain_tiles.iter().all(|tile| {
        tile["t"].as_i64().is_some_and(|id| (0..8).contains(&id)) && tile["a"].as_f64() == Some(1.0)
    }));

    let solid_columns = (0..384)
        .map(|x| {
            (0..48).any(|top_y| {
                collision[top_y * 384 + x]
                    .as_i64()
                    .is_some_and(|value| value == 1 || value == 2)
            })
        })
        .collect::<Vec<_>>();
    let max_gap = solid_columns
        .split(|solid| *solid)
        .map(|gap| gap.len())
        .max()
        .unwrap_or_default();
    assert!(
        max_gap <= 3,
        "mandatory horizontal gaps must remain at most 96 px"
    );

    let atlas = &project["defs"]["tilesets"][0];
    assert_eq!(atlas["identifier"], "SkyCityTiles");
    assert_eq!(atlas["tileGridSize"], 32);
    assert_eq!(atlas["pxWid"], 256);
    assert_eq!(atlas["pxHei"], 32);
    assert!(asset_path("images/levels/sky_city_tiles.png").is_file());
}

#[test]
fn every_critical_path_pit_has_a_lower_recovery_route() {
    let project = sky_project();
    let collision = project["levels"][0]["layerInstances"]
        .as_array()
        .expect("layerInstances array")
        .iter()
        .find(|layer| layer["__identifier"] == "Collision")
        .expect("Collision layer")["intGridCsv"]
        .as_array()
        .expect("IntGrid CSV");
    let cell = |x: usize, y: usize| {
        collision[(47 - y) * 384 + x]
            .as_i64()
            .expect("integer cell")
    };

    let islands = [
        (0, 37, 9),
        (41, 64, 10),
        (68, 94, 11),
        (98, 126, 9),
        (130, 158, 12),
        (162, 190, 11),
        (194, 222, 13),
        (226, 254, 10),
        (258, 286, 12),
        (290, 318, 11),
        (322, 350, 13),
        (354, 383, 10),
    ];

    for (index, pair) in islands.windows(2).enumerate() {
        let (_, left_right, left_top) = pair[0];
        let (right_left, _, right_top) = pair[1];
        let gap_left = left_right + 1;
        let gap_right = right_left - 1;
        let route_top = left_top.min(right_top);
        let rescue_top = route_top - 5;
        let shelf_top = route_top - 2;

        assert!(
            (gap_left..=gap_right).all(|x| cell(x, route_top - 1) == 0),
            "pit {index} must remain visibly open at critical-path height"
        );
        assert!(
            (gap_left..=gap_right).all(|x| cell(x, rescue_top - 1) == 2),
            "pit {index} needs a full-width cloud rescue floor"
        );
        assert_eq!(
            (gap_left..=gap_right)
                .filter(|&x| cell(x, shelf_top - 1) == 2)
                .count(),
            2,
            "pit {index} needs a two-cell mantle shelf"
        );
    }
}

#[test]
fn sky_city_gameplay_entities_obey_encounter_contract() {
    let project = sky_project();
    let gameplay = project["levels"][0]["layerInstances"]
        .as_array()
        .expect("layerInstances array")
        .iter()
        .find(|layer| layer["__identifier"] == "Gameplay")
        .expect("Gameplay layer");
    let entities = gameplay["entityInstances"]
        .as_array()
        .expect("gameplay entities");

    let count = |identifier: &str| {
        entities
            .iter()
            .filter(|entity| entity["__identifier"] == identifier)
            .count()
    };
    assert_eq!(count("PlayerStart"), 1);
    assert_eq!(count("Checkpoint"), 5);
    assert_eq!(count("EnemySpawn"), 32);
    assert_eq!(count("CombatGate"), 6);
    assert_eq!(count("Goal"), 1);
    assert_eq!(count("ClimbAnchor"), 33);

    for anchor in entities
        .iter()
        .filter(|entity| entity["__identifier"] == "ClimbAnchor")
    {
        assert!(matches!(
            field_value(anchor, "direction").as_i64(),
            Some(-1 | 1)
        ));
    }

    let mut gates_per_arena = BTreeMap::<i64, usize>::new();
    for gate in entities
        .iter()
        .filter(|entity| entity["__identifier"] == "CombatGate")
    {
        *gates_per_arena
            .entry(field_value(gate, "arena").as_i64().expect("arena integer"))
            .or_default() += 1;
        assert_eq!(gate["height"], 288);
    }
    assert_eq!(gates_per_arena, BTreeMap::from([(1, 2), (2, 2), (3, 2)]));

    let mut enemies_per_arena = BTreeMap::<i64, usize>::new();
    for enemy in entities
        .iter()
        .filter(|entity| entity["__identifier"] == "EnemySpawn")
    {
        let arena = field_value(enemy, "arena").as_i64().expect("arena integer");
        *enemies_per_arena.entry(arena).or_default() += 1;
        assert!(
            field_value(enemy, "healthMultiplier")
                .as_f64()
                .is_some_and(|health| health >= 0.8)
        );
        assert!(
            field_value(enemy, "patrolRange")
                .as_f64()
                .is_some_and(|range| range >= 64.0)
        );
    }
    assert_eq!(enemies_per_arena.get(&1), Some(&4));
    assert_eq!(enemies_per_arena.get(&2), Some(&4));
    assert_eq!(enemies_per_arena.get(&3), Some(&5));

    let mut iids = HashSet::new();
    assert!(iids.insert(project["iid"].as_str().expect("project IID")));
    assert!(iids.insert(project["levels"][0]["iid"].as_str().expect("level IID")));
    for layer in project["levels"][0]["layerInstances"]
        .as_array()
        .expect("layer instances")
    {
        assert!(iids.insert(layer["iid"].as_str().expect("layer IID")));
        for entity in layer["entityInstances"]
            .as_array()
            .expect("entity instances")
        {
            assert!(iids.insert(entity["iid"].as_str().expect("entity IID")));
        }
    }
}

#[test]
fn sky_city_tileset_png_matches_ldtk_definition() {
    let bytes = fs::read(asset_path("images/levels/sky_city_tiles.png"))
        .expect("tileset PNG must be readable");
    assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
    assert_eq!(u32::from_be_bytes(bytes[16..20].try_into().unwrap()), 256);
    assert_eq!(u32::from_be_bytes(bytes[20..24].try_into().unwrap()), 32);
}

#[test]
fn authored_climb_anchor_catches_fall_and_mantles_to_safety() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .init_resource::<crate::systems::input::GameInput>()
        .add_systems(Update, crate::systems::player::player_ledge_traversal);

    let coords = GridCoords::new(10, 5);
    let anchor_position = bevy_ecs_ldtk::utils::grid_coords_to_translation(
        coords,
        IVec2::splat(crate::components::SKY_LEVEL_GRID),
    );
    app.world_mut()
        .spawn((SkyClimbAnchor { direction: -1.0 }, coords));
    let mut attack = AttackAnimationState::default();
    attack.trigger(0.4);
    let player = app
        .world_mut()
        .spawn((
            Player,
            Transform::from_xyz(anchor_position.x + 8.0, anchor_position.y + 8.0, 1.0),
            Velocity { x: 80.0, y: -140.0 },
            PlayerState::new(false, false),
            LedgeTraversal::default(),
            attack,
            FacingDirection::Right,
        ))
        .id();

    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .advance_by(Duration::from_secs_f32(1.0 / 60.0));
    app.update();

    let entity = app.world().entity(player);
    assert!(matches!(
        entity.get::<LedgeTraversal>().expect("traversal").phase,
        LedgeTraversalPhase::Hanging { .. }
    ));
    assert_eq!(
        entity.get::<FacingDirection>(),
        Some(&FacingDirection::Left)
    );
    assert!(
        !entity
            .get::<AttackAnimationState>()
            .expect("attack state")
            .is_active()
    );

    app.world_mut()
        .resource_mut::<crate::systems::input::GameInput>()
        .jump = true;
    for _ in 0..22 {
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .advance_by(Duration::from_secs_f32(1.0 / 60.0));
        app.update();
    }

    let entity = app.world().entity(player);
    assert!(matches!(
        entity.get::<LedgeTraversal>().expect("traversal").phase,
        LedgeTraversalPhase::Inactive
    ));
    assert!(
        entity
            .get::<PlayerState>()
            .expect("player state")
            .is_grounded
    );
    let position = entity.get::<Transform>().expect("transform").translation;
    assert!((position.x - (anchor_position.x - 42.0)).abs() < 0.01);
    assert!((position.y - (anchor_position.y + 46.0)).abs() < 0.01);
}

fn gate_test_app(player_x: f32, active_arena: Option<i32>) -> App {
    let mut app = App::new();
    app.insert_resource(SkyEncounterState {
        active_arena,
        ..default()
    })
    .add_systems(Update, sky_level::update_combat_gates);
    app.world_mut()
        .spawn((Player, Transform::from_xyz(player_x, 100.0, 0.0)));
    app.world_mut().spawn(SkyEnemySpawn {
        kind: SkyEnemyKind::Slime,
        arena: 1,
        health_multiplier: 1.0,
        patrol_range: 96.0,
    });
    for (x, height) in [(10, 224.0), (20, 224.0)] {
        app.world_mut().spawn((
            SkyCombatGate { arena: 1, height },
            GridCoords::new(x, 0),
            Sprite::default(),
            SkyGateVisual,
        ));
    }
    app
}

#[test]
fn arena_gates_close_from_authored_bounds_and_height() {
    let mut app = gate_test_app(500.0, None);
    app.update();

    assert_eq!(
        app.world().resource::<SkyEncounterState>().active_arena,
        Some(1)
    );
    let mut gates = app
        .world_mut()
        .query_filtered::<(&CollisionBox, &SkyCombatGate), With<Ground>>();
    let closed = gates.iter(app.world()).collect::<Vec<_>>();
    assert_eq!(closed.len(), 2);
    assert!(closed.iter().all(|(collision, gate)| {
        collision.size == Vec2::new(24.0, gate.height) && gate.height == 224.0
    }));
}

#[test]
fn arena_gates_reopen_if_revive_moves_player_outside() {
    let mut app = gate_test_app(100.0, Some(1));
    for entity in app
        .world_mut()
        .query_filtered::<Entity, With<SkyGateVisual>>()
        .iter(app.world())
        .collect::<Vec<_>>()
    {
        app.world_mut()
            .entity_mut(entity)
            .insert((Ground, CollisionBox::new(Vec2::new(24.0, 224.0))));
    }
    app.update();

    assert_eq!(
        app.world().resource::<SkyEncounterState>().active_arena,
        None
    );
    let open_gate_count = app
        .world_mut()
        .query_filtered::<Entity, (With<SkyGateVisual>, Without<Ground>)>()
        .iter(app.world())
        .count();
    assert_eq!(open_gate_count, 2);
}

#[test]
fn initialized_ldtk_player_does_not_overwrite_restored_save_position() {
    let saved_position = Vec3::new(3_400.0, 720.0, 1.0);
    let mut app = App::new();
    app.insert_resource(SkyLevelRuntime {
        level_ready: true,
        player_initialized: true,
        checkpoint_position: saved_position,
        ..default()
    })
    .add_systems(Update, sky_level::initialize_player_from_ldtk);
    app.world_mut().spawn((
        Player,
        Transform::from_translation(saved_position),
        Velocity { x: 25.0, y: -10.0 },
        PlayerState::default(),
    ));
    app.world_mut()
        .spawn((SkyPlayerStart, GridCoords::new(4, 10)));
    app.update();

    let transform = app
        .world_mut()
        .query_filtered::<&Transform, With<Player>>()
        .single(app.world())
        .expect("single player");
    assert_eq!(transform.translation, saved_position);
}

#[test]
fn loaded_position_reconciles_to_highest_passed_checkpoint() {
    let saved_position = Vec3::new(500.0, 720.0, 1.0);
    let mut app = App::new();
    app.insert_resource(SkyLevelRuntime {
        checkpoint_position: saved_position,
        checkpoint_needs_reconciliation: true,
        ..default()
    })
    .add_systems(Update, sky_level::activate_checkpoints);
    app.world_mut()
        .spawn((Player, Transform::from_translation(saved_position)));
    for (id, x) in [(0, 4), (1, 10), (2, 20)] {
        app.world_mut()
            .spawn((SkyCheckpoint { id }, GridCoords::new(x, 10)));
    }
    app.update();

    let runtime = app.world().resource::<SkyLevelRuntime>();
    assert_eq!(runtime.checkpoint_id, 1);
    assert!(!runtime.checkpoint_needs_reconciliation);
    assert!(runtime.checkpoint_position.x < saved_position.x);
}

#[test]
fn authored_enemy_keeps_floating_island_anchor_and_fights_bidirectionally() {
    let anchor_y = 640.0;
    let mut app = App::new();
    app.insert_resource(Time::<()>::default())
        .add_systems(Update, crate::systems::enemy::enemy_patrol_ai);
    app.world_mut()
        .resource_mut::<Time<()>>()
        .advance_by(Duration::from_millis(16));
    app.world_mut()
        .spawn((Player, Transform::from_xyz(500.0, anchor_y, 0.0)));
    app.world_mut().spawn((
        Enemy,
        EnemyType::Slime,
        Transform::from_xyz(400.0, anchor_y, 0.0),
        EnemyState::new(3, 96.0)
            .with_spawn_origin(400.0)
            .with_movement(60.0, 12.0, 0.0),
        Velocity { x: 0.0, y: 0.0 },
        SkyEncounterEnemy { arena: 1, anchor_y },
    ));
    app.update();

    let (transform, velocity) = app
        .world_mut()
        .query_filtered::<(&Transform, &Velocity), With<Enemy>>()
        .single(app.world())
        .expect("single authored enemy");
    assert_eq!(transform.translation.y, anchor_y);
    assert!(
        velocity.x > 0.0,
        "authored enemies may engage from either side"
    );
}
