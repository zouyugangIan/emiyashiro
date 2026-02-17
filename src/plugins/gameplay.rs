use bevy::prelude::*;

use crate::{
    states::GameState,
    systems::{self, interfaces::GameSystemSet},
};

/// Core gameplay loop systems: state flow, movement, combat and enemy logic.
pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), systems::game::setup_game)
            .add_systems(
                OnEnter(GameState::GameOver),
                systems::death::setup_game_over_ui,
            )
            .add_systems(
                OnExit(GameState::GameOver),
                systems::death::cleanup_game_over_ui,
            )
            .add_systems(
                Update,
                systems::death::handle_game_over_input.run_if(in_state(GameState::GameOver)),
            )
            .add_systems(OnEnter(GameState::Reviving), systems::death::revive_player)
            .add_systems(
                Update,
                systems::game::restore_loaded_game_entities
                    .in_set(GameSystemSet::GameLogic)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                systems::game::handle_game_input
                    .in_set(GameSystemSet::Input)
                    .run_if(in_state(GameState::Playing).or(in_state(GameState::Paused))),
            )
            .add_systems(
                Update,
                (
                    systems::input::update_game_input,
                    systems::shirou::handle_shroud_input,
                    systems::combat::player_shoot_projectile,
                )
                    .in_set(GameSystemSet::Input)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                (
                    systems::player::update_game_stats,
                    systems::camera::camera_follow.in_set(GameSystemSet::Camera),
                    systems::enemy::spawn_mushroom_enemies,
                    systems::enemy::cleanup_dead_enemies,
                    systems::enemy::cleanup_offscreen_enemies,
                    systems::combat::cleanup_expired_projectiles,
                )
                    .in_set(GameSystemSet::GameLogic)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                FixedUpdate,
                (
                    (
                        systems::player::player_movement,
                        systems::player::player_jump,
                        systems::player::player_crouch,
                        systems::player::physics_update_system,
                        systems::collision::collision_detection_system,
                        systems::player::update_player_state,
                    ),
                    (
                        systems::enemy::enemy_patrol_ai,
                        systems::combat::update_projectiles,
                        systems::combat::projectile_enemy_collision,
                        systems::combat::player_enemy_collision,
                        systems::death::check_player_fall_death,
                        systems::shirou::shroud_health_drain,
                        systems::combat::apply_damage_events,
                    ),
                )
                    .chain()
                    .in_set(GameSystemSet::GameLogic)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}
