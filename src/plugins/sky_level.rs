use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::{components::*, states::GameState, systems};

/// Loads the authored LDtk sky-city level and binds its editor data to gameplay.
pub struct SkyLevelPlugin;

impl Plugin for SkyLevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LdtkPlugin)
            .insert_resource(LevelSelection::index(0))
            .insert_resource(LdtkSettings {
                level_spawn_behavior: LevelSpawnBehavior::UseWorldTranslation {
                    load_level_neighbors: false,
                },
                set_clear_color: SetClearColor::FromLevelBackground,
                ..default()
            })
            .init_resource::<SkyLevelRuntime>()
            .init_resource::<SkyEncounterState>()
            .register_ldtk_int_cell::<SkyStoneCellBundle>(1)
            .register_ldtk_int_cell::<SkyCloudCellBundle>(2)
            .register_ldtk_int_cell::<SkyHazardCellBundle>(3)
            .register_ldtk_int_cell::<SkyWindCellBundle>(4)
            .register_ldtk_entity::<SkyPlayerStartBundle>("PlayerStart")
            .register_ldtk_entity::<SkyClimbAnchorBundle>("ClimbAnchor")
            .register_ldtk_entity::<SkyCheckpointBundle>("Checkpoint")
            .register_ldtk_entity::<SkyEnemySpawnBundle>("EnemySpawn")
            .register_ldtk_entity::<SkyCombatGateBundle>("CombatGate")
            .register_ldtk_entity::<SkyGoalBundle>("Goal")
            .register_ldtk_entity::<SkyBackdropBundle>("Backdrop")
            .add_systems(
                OnEnter(GameState::Playing),
                systems::sky_level::spawn_sky_level_world,
            )
            .add_systems(
                Update,
                (
                    systems::sky_level::build_merged_sky_colliders,
                    systems::sky_level::decorate_sky_cells,
                    systems::sky_level::decorate_sky_entities,
                    systems::sky_level::initialize_player_from_ldtk,
                    systems::sky_level::activate_map_enemies,
                    systems::sky_level::update_combat_gates,
                    systems::sky_level::activate_checkpoints,
                    systems::sky_level::apply_wind_lifts,
                    systems::sky_level::damage_sky_hazards,
                    systems::sky_level::detect_sky_goal,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                OnEnter(GameState::Victory),
                systems::sky_level::setup_victory_ui,
            )
            .add_systems(
                Update,
                systems::sky_level::handle_victory_input.run_if(in_state(GameState::Victory)),
            )
            .add_systems(
                OnExit(GameState::Victory),
                systems::sky_level::cleanup_victory_ui,
            );
    }
}
