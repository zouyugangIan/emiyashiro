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
                (
                    systems::input::update_game_input,
                    systems::player::update_player_facing_from_input,
                    systems::shirou::handle_shroud_input,
                    systems::combat::player_shoot_projectile,
                    systems::combat::player_knife_attack,
                    systems::attack_modules::handle_reference_attack_module_input,
                    systems::combat::resolve_pending_knife_attacks,
                )
                    .chain()
                    .in_set(GameSystemSet::Input)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                (
                    systems::player::update_game_stats,
                    systems::enemy::spawn_mushroom_enemies,
                    systems::enemy::cleanup_dead_enemies,
                    systems::enemy::cleanup_offscreen_enemies,
                    systems::combat::cleanup_expired_projectiles,
                    systems::combat::cleanup_expired_enemy_projectiles,
                    systems::combat::cleanup_expired_knife_slashes,
                )
                    .in_set(GameSystemSet::GameLogic)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                systems::combat::maintain_hit_stop_timescale.in_set(GameSystemSet::GameLogic),
            )
            .add_systems(
                Update,
                (
                    systems::camera::camera_follow,
                    systems::camera::consume_camera_impulse_events,
                    systems::camera::apply_camera_shake,
                )
                    .chain()
                    .in_set(GameSystemSet::Camera)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                FixedUpdate,
                (
                    (
                        systems::player::player_movement,
                        systems::player::player_jump,
                        systems::player::player_crouch,
                        systems::player::update_player_damage_invulnerability,
                        systems::player::physics_update_system,
                        systems::collision::collision_detection_system,
                    )
                        .chain(),
                    (
                        systems::enemy::enemy_patrol_ai,
                        systems::enemy::enemy_ranged_attack,
                        systems::combat::update_projectiles,
                        systems::combat::update_enemy_projectiles,
                        systems::combat::projectile_enemy_collision,
                        systems::combat::knife_enemy_collision,
                        systems::combat::enemy_projectile_player_collision,
                        systems::combat::player_enemy_collision,
                        systems::death::check_player_fall_death,
                        systems::shirou::shroud_health_drain,
                        systems::combat::apply_damage_events,
                    )
                        .chain(),
                )
                    .chain()
                    .in_set(GameSystemSet::GameLogic)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}
