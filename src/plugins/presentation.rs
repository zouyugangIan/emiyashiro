use bevy::prelude::*;

use crate::{
    states::GameState,
    systems::{self, interfaces::GameSystemSet},
};

/// Rendering-adjacent systems: animation, audio transitions, VFX and scene decoration.
pub struct PresentationPlugin;

impl Plugin for PresentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            systems::audio::play_menu_music
                .in_set(GameSystemSet::Audio)
                .run_if(in_state(GameState::Menu)),
        )
        .add_systems(
            OnEnter(GameState::Playing),
            (
                systems::audio::play_game_music_and_stop_menu,
                systems::scene_decoration::setup_parallax_background,
            ),
        )
        .add_systems(
            Update,
            (
                systems::animation::trigger_audio_effects,
                systems::frame_animation::update_frame_animations,
                systems::frame_animation::update_character_animations,
                systems::frame_animation::setup_player_animation,
                systems::frame_animation::debug_animations,
                systems::sprite_animation::update_sprite_animations,
                systems::sprite_animation::update_character_animation_state,
                systems::audio::handle_music_transitions,
            )
                .in_set(GameSystemSet::Animation)
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            (
                systems::scene_decoration::spawn_enhanced_clouds,
                systems::scene_decoration::spawn_ground_decorations,
                systems::scene_decoration::move_scene_decorations,
                systems::scene_decoration::cleanup_offscreen_decorations,
                systems::scene_decoration::loop_far_background,
                systems::scene_decoration::dynamic_lighting,
            )
                .in_set(GameSystemSet::Animation)
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            (
                systems::visual_effects::trigger_jump_effect,
                systems::visual_effects::trigger_land_effect,
                systems::visual_effects::trigger_run_effect,
                systems::visual_effects::trigger_crouch_effect,
                systems::visual_effects::update_visual_effects,
                systems::visual_effects::update_blinking_text,
            )
                .in_set(GameSystemSet::Animation)
                .run_if(in_state(GameState::Playing)),
        );
    }
}
