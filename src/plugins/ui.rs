use bevy::prelude::*;

use crate::{
    states::GameState,
    systems::{self, interfaces::GameSystemSet},
};

/// UI stateflow systems: menu, HUD, pause menu and save/load dialogs.
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Menu),
            (
                systems::game::cleanup_game,
                systems::ui::cleanup_game_hud,
                systems::scene_decoration::cleanup_scene_decorations,
                systems::audio::stop_game_music,
                systems::menu::setup_menu,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                (
                    systems::menu::handle_character_select,
                    systems::menu::handle_start_button,
                )
                    .chain(),
                systems::menu::handle_load_button,
                systems::save::handle_save_button_click,
                systems::menu::cover_fade_animation,
                systems::visual_effects::button_hover_effect,
            )
                .in_set(GameSystemSet::UI)
                .run_if(in_state(GameState::Menu)),
        )
        .add_systems(
            OnExit(GameState::Menu),
            (systems::menu::cleanup_menu, systems::audio::stop_menu_music),
        )
        .add_systems(OnEnter(GameState::Playing), systems::ui::setup_game_hud)
        .add_systems(
            Update,
            systems::ui::update_game_hud
                .in_set(GameSystemSet::UI)
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            OnEnter(GameState::Paused),
            (
                systems::pause_save::scan_save_files,
                systems::ui::setup_pause_menu,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                systems::ui::handle_pause_menu_interactions,
                systems::audio::maintain_audio_during_pause,
            )
                .in_set(GameSystemSet::UI)
                .run_if(in_state(GameState::Paused)),
        )
        .add_systems(OnExit(GameState::Paused), systems::ui::cleanup_pause_menu)
        .add_systems(
            OnEnter(GameState::SaveDialog),
            systems::ui::setup_save_dialog,
        )
        .add_systems(
            Update,
            (
                systems::text_input::handle_keyboard_input,
                systems::ui::handle_save_name_input,
                systems::ui::handle_save_dialog_interactions,
                systems::ui::update_text_cursor,
            )
                .in_set(GameSystemSet::UI)
                .run_if(in_state(GameState::SaveDialog)),
        )
        .add_systems(
            OnExit(GameState::SaveDialog),
            systems::ui::cleanup_save_dialog,
        )
        .add_systems(
            OnEnter(GameState::LoadTable),
            (
                systems::pause_save::scan_save_files,
                systems::ui::setup_load_table,
            )
                .chain(),
        )
        .add_systems(
            Update,
            systems::ui::handle_load_table_interactions
                .in_set(GameSystemSet::UI)
                .run_if(in_state(GameState::LoadTable)),
        )
        .add_systems(
            OnExit(GameState::LoadTable),
            systems::ui::cleanup_load_table,
        )
        .add_systems(
            OnEnter(GameState::RenameDialog),
            systems::ui::setup_rename_dialog,
        )
        .add_systems(
            Update,
            (
                systems::text_input::handle_keyboard_input,
                systems::ui::handle_rename_input,
                systems::ui::handle_rename_dialog_interactions,
            )
                .in_set(GameSystemSet::UI)
                .run_if(in_state(GameState::RenameDialog)),
        )
        .add_systems(
            OnExit(GameState::RenameDialog),
            systems::ui::cleanup_rename_dialog,
        );
    }
}
