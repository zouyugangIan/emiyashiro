use bevy::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};

use crate::{resources::*, states::*};

const AUTOSAVE_NAME: &str = "autosave";

type SaveButtonInteractionQuery<'w, 's> = Query<
    'w,
    's,
    (&'static Interaction, &'static mut BackgroundColor),
    (Changed<Interaction>, With<crate::components::SaveButton>),
>;

/// 保存游戏数据（统一写入 SaveFileData v2）。
pub fn save_game(
    game_stats: Res<GameStats>,
    character_selection: Res<CharacterSelection>,
    mut save_manager: ResMut<SaveManager>,
) {
    let previous_state = save_manager
        .current_save
        .as_ref()
        .map(|save| &save.game_state);
    let save_timestamp = chrono::Utc::now();
    let best_distance = game_stats.distance_traveled.max(
        previous_state
            .map(|state| state.distance_traveled)
            .unwrap_or(0.0),
    );
    let total_jumps =
        game_stats.jump_count + previous_state.map(|state| state.jump_count).unwrap_or(0);
    let total_play_time =
        game_stats.play_time + previous_state.map(|state| state.play_time).unwrap_or(0.0);

    let mut metadata = save_manager
        .current_save
        .as_ref()
        .map(|save| save.metadata.clone())
        .unwrap_or_else(|| SaveFileMetadata {
            name: AUTOSAVE_NAME.to_string(),
            score: 0,
            distance: 0.0,
            play_time: 0.0,
            save_timestamp,
            file_path: String::new(),
            selected_character: character_selection.selected_character.clone(),
        });

    metadata.file_path = save_manager.save_file_path.clone();
    metadata.selected_character = character_selection.selected_character.clone();
    metadata.distance = best_distance;
    metadata.play_time = total_play_time;
    metadata.save_timestamp = save_timestamp;

    let mut state = CompleteGameState::default();
    state.selected_character = metadata.selected_character.clone();
    state.distance_traveled = best_distance;
    state.jump_count = total_jumps;
    state.play_time = total_play_time;
    state.score = (state.distance_traveled * 10.0) as u32 + state.jump_count * 50;
    state.save_timestamp = save_timestamp;

    let save_path = PathBuf::from(&save_manager.save_file_path);
    metadata.score = state.score;
    metadata.distance = state.distance_traveled;
    metadata.play_time = state.play_time;
    metadata.file_path = save_path.to_string_lossy().to_string();

    let save_data = SaveFileData::new(metadata, state);
    match write_v2_save(&save_path, &save_data) {
        Ok(()) => {
            save_manager.current_save = Some(save_data);
            crate::debug_log!("Game saved (SaveFileData v2)");
        }
        Err(error) => {
            crate::debug_log!("Save failed: {}", error);
        }
    }
}

/// 加载游戏数据（仅支持 SaveFileData v2）。
pub fn load_game(
    mut save_manager: ResMut<SaveManager>,
    mut character_selection: ResMut<CharacterSelection>,
) {
    let save_path = PathBuf::from(&save_manager.save_file_path);
    let file_data = match fs::read(&save_path) {
        Ok(data) => data,
        Err(_) => {
            crate::debug_log!("No save file found, a new save will be created");
            return;
        }
    };

    let json_data = match crate::systems::shared_utils::decode_file_payload(&file_data) {
        Ok(data) => data,
        Err(error) => {
            crate::debug_log!("Failed to decode save file: {}", error);
            return;
        }
    };

    let mut v2_save = match serde_json::from_str::<SaveFileData>(&json_data) {
        Ok(save) => save,
        Err(error) => {
            crate::debug_log!(
                "Unsupported save format at {}: {}",
                save_path.display(),
                error
            );
            return;
        }
    };

    if !v2_save.verify_checksum() {
        crate::debug_log!("Checksum verification failed for {}", save_path.display());
        return;
    }

    v2_save.metadata.file_path = save_path.to_string_lossy().to_string();
    character_selection.selected_character = v2_save.game_state.selected_character.clone();
    save_manager.current_save = Some(v2_save);
    crate::debug_log!("Loaded v2 save: {}", save_path.display());
}

fn write_v2_save(save_path: &Path, save_data: &SaveFileData) -> Result<(), String> {
    if let Some(parent) = save_path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let json_data = serde_json::to_string_pretty(save_data).map_err(|error| error.to_string())?;
    crate::systems::shared_utils::atomic_write_file(save_path, json_data.as_bytes())
        .map_err(|error| error.to_string())
}

/// 处理存档按钮点击
pub fn handle_save_button_click(
    mut interaction_query: SaveButtonInteractionQuery,
    game_stats: Res<GameStats>,
    character_selection: Res<CharacterSelection>,
    save_manager: ResMut<SaveManager>,
) {
    let mut should_save = false;

    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(Color::srgba(0.05, 0.1, 0.05, 0.8));
                should_save = true;
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgba(0.15, 0.3, 0.15, 0.8));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgba(0.1, 0.2, 0.1, 0.8));
            }
        }
    }

    if should_save {
        save_game(game_stats, character_selection, save_manager);
    }
}

/// 自动保存系统
pub fn auto_save_system(
    mut timer: Local<Timer>,
    time: Res<Time>,
    game_stats: Res<GameStats>,
    character_selection: Res<CharacterSelection>,
    save_manager: ResMut<SaveManager>,
    current_state: Res<State<GameState>>,
) {
    if timer.duration().is_zero() {
        timer.set_duration(std::time::Duration::from_secs(30));
        timer.set_mode(bevy::time::TimerMode::Repeating);
    }
    timer.tick(time.delta());

    if timer.just_finished() && *current_state.get() == GameState::Playing {
        save_game(game_stats, character_selection, save_manager);
    }
}
