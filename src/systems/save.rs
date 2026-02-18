use bevy::prelude::*;
use std::fs;
use std::path::PathBuf;

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
    let summary = SaveData {
        player_name: AUTOSAVE_NAME.to_string(),
        selected_character: character_selection.selected_character.clone(),
        best_distance: game_stats.distance_traveled.max(
            save_manager
                .current_save
                .as_ref()
                .map(|save| save.best_distance)
                .unwrap_or(0.0),
        ),
        total_jumps: game_stats.jump_count
            + save_manager
                .current_save
                .as_ref()
                .map(|save| save.total_jumps)
                .unwrap_or(0),
        total_play_time: game_stats.play_time
            + save_manager
                .current_save
                .as_ref()
                .map(|save| save.total_play_time)
                .unwrap_or(0.0),
        save_time: chrono::Utc::now(),
    };

    let mut state = CompleteGameState::default();
    state.selected_character = summary.selected_character.clone();
    state.distance_traveled = summary.best_distance;
    state.jump_count = summary.total_jumps;
    state.play_time = summary.total_play_time;
    state.score = (state.distance_traveled * 10.0) as u32 + state.jump_count * 50;
    state.save_timestamp = summary.save_time;

    let save_path = PathBuf::from(&save_manager.save_file_path);
    let metadata = SaveFileMetadata {
        name: summary.player_name.clone(),
        score: state.score,
        distance: state.distance_traveled,
        play_time: state.play_time,
        save_timestamp: state.save_timestamp,
        file_path: save_path.to_string_lossy().to_string(),
        selected_character: state.selected_character.clone(),
    };

    let save_data = SaveFileData::new(metadata, state);
    match write_v2_save(&save_path, &save_data) {
        Ok(()) => {
            save_manager.current_save = Some(summary);
            crate::debug_log!("Game saved (SaveFileData v2)");
        }
        Err(error) => {
            crate::debug_log!("Save failed: {}", error);
        }
    }
}

/// 加载游戏数据（兼容 legacy，只读导入后自动迁移到 v2）。
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

    if let Ok(v2_save) = serde_json::from_str::<SaveFileData>(&json_data) {
        character_selection.selected_character = v2_save.game_state.selected_character.clone();
        save_manager.current_save = Some(summary_from_v2(&v2_save));
        crate::debug_log!("Loaded v2 save: {}", save_path.display());
        return;
    }

    if let Ok(legacy_save) = serde_json::from_str::<SaveData>(&json_data) {
        character_selection.selected_character = legacy_save.selected_character.clone();
        save_manager.current_save = Some(legacy_save.clone());
        migrate_legacy_save_data(&save_path, legacy_save);
        return;
    }

    if let Ok(legacy_state) = serde_json::from_str::<CompleteGameState>(&json_data) {
        character_selection.selected_character = legacy_state.selected_character.clone();
        save_manager.current_save =
            Some(summary_from_state(AUTOSAVE_NAME.to_string(), &legacy_state));
        migrate_legacy_state(&save_path, legacy_state);
        return;
    }

    crate::debug_log!("Unknown save format: {}", save_path.display());
}

fn summary_from_v2(v2_save: &SaveFileData) -> SaveData {
    summary_from_state(v2_save.metadata.name.clone(), &v2_save.game_state)
}

fn summary_from_state(name: String, state: &CompleteGameState) -> SaveData {
    SaveData {
        player_name: name,
        selected_character: state.selected_character.clone(),
        best_distance: state.distance_traveled,
        total_jumps: state.jump_count,
        total_play_time: state.play_time,
        save_time: state.save_timestamp,
    }
}

fn migrate_legacy_save_data(save_path: &PathBuf, legacy_save: SaveData) {
    let mut state = CompleteGameState::default();
    state.selected_character = legacy_save.selected_character.clone();
    state.distance_traveled = legacy_save.best_distance;
    state.jump_count = legacy_save.total_jumps;
    state.play_time = legacy_save.total_play_time;
    state.score = (state.distance_traveled * 10.0) as u32 + state.jump_count * 50;
    state.save_timestamp = legacy_save.save_time;

    let metadata = SaveFileMetadata {
        name: legacy_save.player_name.clone(),
        score: state.score,
        distance: state.distance_traveled,
        play_time: state.play_time,
        save_timestamp: state.save_timestamp,
        file_path: save_path.to_string_lossy().to_string(),
        selected_character: state.selected_character.clone(),
    };

    let v2_save = SaveFileData::new(metadata, state);
    match write_v2_save(save_path, &v2_save) {
        Ok(()) => crate::debug_log!("Migrated legacy SaveData to v2: {}", save_path.display()),

        Err(error) => crate::debug_log!("Legacy SaveData migration failed: {}", error),
    }
}

fn migrate_legacy_state(save_path: &PathBuf, legacy_state: CompleteGameState) {
    let metadata = SaveFileMetadata {
        name: save_path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or(AUTOSAVE_NAME)
            .to_string(),
        score: legacy_state.score,
        distance: legacy_state.distance_traveled,
        play_time: legacy_state.play_time,
        save_timestamp: legacy_state.save_timestamp,
        file_path: save_path.to_string_lossy().to_string(),
        selected_character: legacy_state.selected_character.clone(),
    };

    let v2_save = SaveFileData::new(metadata, legacy_state);
    match write_v2_save(save_path, &v2_save) {
        Ok(()) => crate::debug_log!(
            "Migrated legacy CompleteGameState to v2: {}",
            save_path.display()
        ),
        Err(error) => crate::debug_log!("Legacy CompleteGameState migration failed: {}", error),
    }
}

fn write_v2_save(save_path: &PathBuf, save_data: &SaveFileData) -> Result<(), String> {
    if let Some(parent) = save_path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let json_data = serde_json::to_string_pretty(save_data).map_err(|error| error.to_string())?;
    fs::write(save_path, json_data).map_err(|error| error.to_string())
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
