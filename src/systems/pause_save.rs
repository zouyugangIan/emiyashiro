//! 增强暂停存档系统
//!
//! 实现完整的游戏状态保存、恢复和管理功能

use crate::{components::*, resources::*, states::*};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::fs;
use std::path::Path;

#[derive(SystemParam)]
pub struct PauseSnapshotParams<'w, 's> {
    player_query:
        Query<'w, 's, (&'static Transform, &'static Velocity, &'static PlayerState), With<Player>>,
    camera_query: Query<'w, 's, &'static Transform, (With<Camera>, Without<Player>)>,
    game_stats: Res<'w, GameStats>,
    character_selection: Res<'w, CharacterSelection>,
    audio_state_manager: Res<'w, AudioStateManager>,
}

#[derive(SystemParam)]
pub struct RestorePausedParams<'w, 's> {
    player_query: Query<
        'w,
        's,
        (
            &'static mut Transform,
            &'static mut Velocity,
            &'static mut PlayerState,
        ),
        With<Player>,
    >,
    camera_query: Query<'w, 's, &'static mut Transform, (With<Camera>, Without<Player>)>,
    game_stats: ResMut<'w, GameStats>,
    character_selection: ResMut<'w, CharacterSelection>,
    audio_state_manager: ResMut<'w, AudioStateManager>,
}

/// 捕获完整游戏状态
pub fn capture_game_state(
    player_query: Query<(&Transform, &Velocity, &PlayerState), With<Player>>,
    camera_query: Query<&Transform, (With<Camera>, Without<Player>)>,
    game_stats: Res<GameStats>,
    character_selection: Res<CharacterSelection>,
    audio_state_manager: Res<AudioStateManager>,
) -> CompleteGameState {
    let mut state = CompleteGameState::default();

    // 捕获玩家状态
    if let Ok((player_transform, player_velocity, player_state)) = player_query.single() {
        state.player_position = player_transform.translation;
        state.player_velocity = player_velocity.clone();
        state.player_grounded = player_state.is_grounded;
        state.player_crouching = player_state.is_crouching;

        // 根据玩家状态确定动画状态
        state.player_animation_state = if player_state.is_crouching {
            "crouch".to_string()
        } else if !player_state.is_grounded {
            "jump".to_string()
        } else if player_velocity.x.abs() > 0.1 {
            "run".to_string()
        } else {
            "idle".to_string()
        };
    }

    // 捕获摄像机状态
    if let Ok(camera_transform) = camera_query.single() {
        state.camera_position = camera_transform.translation;
        // 摄像机目标通常是玩家位置加偏移
        state.camera_target = state.player_position
            + Vec3::new(crate::resources::GameConfig::CAMERA_OFFSET, 0.0, 0.0);
    }

    // 捕获游戏统计
    state.score = (game_stats.distance_traveled * 10.0) as u32 + game_stats.jump_count * 50;
    state.distance_traveled = game_stats.distance_traveled;
    state.jump_count = game_stats.jump_count;
    state.play_time = game_stats.play_time;

    // 捕获角色选择和玩家数量
    state.selected_character = character_selection.selected_character.clone();
    state.player_count = match character_selection.selected_character {
        CharacterType::Shirou1 => PlayerCount::Single,
        CharacterType::Shirou2 => PlayerCount::Double,
    };

    // 捕获音频状态
    state.music_playing = audio_state_manager.music_playing;
    state.audio_volume = audio_state_manager.music_volume;
    state.music_position = audio_state_manager.music_position;

    // 捕获实体快照（目前为空，未来可扩展）
    state.entities_snapshot = Vec::new();

    // 设置时间戳
    state.save_timestamp = chrono::Utc::now();

    crate::debug_log!("Captured game state snapshot:");
    crate::debug_log!(
        "   Player position: ({:.1}, {:.1})",
        state.player_position.x,
        state.player_position.y
    );
    crate::debug_log!("   Animation state: {}", state.player_animation_state);
    crate::debug_log!("   Score: {}", state.score);
    crate::debug_log!("   Distance: {:.1}m", state.distance_traveled);
    crate::debug_log!("   Time: {:.1}s", state.play_time);
    crate::debug_log!("   Music playing: {}", state.music_playing);

    state
}

/// 恢复完整游戏状态
pub fn restore_game_state(
    _commands: Commands,
    state: CompleteGameState,
    mut player_query: Query<(&mut Transform, &mut Velocity, &mut PlayerState), With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    mut game_stats: ResMut<GameStats>,
    mut character_selection: ResMut<CharacterSelection>,
    mut audio_state_manager: ResMut<AudioStateManager>,
) {
    // 恢复玩家状态
    if let Ok((mut player_transform, mut player_velocity, mut player_state)) =
        player_query.single_mut()
    {
        player_transform.translation = state.player_position;
        *player_velocity = state.player_velocity;
        player_state.is_grounded = state.player_grounded;
        player_state.is_crouching = state.player_crouching;

        crate::debug_log!(
            "Restored player state: position({:.1}, {:.1}), animation: {}",
            state.player_position.x,
            state.player_position.y,
            state.player_animation_state
        );
    }

    // 恢复摄像机状态
    if let Ok(mut camera_transform) = camera_query.single_mut() {
        camera_transform.translation = state.camera_position;
        crate::debug_log!(
            "Restored camera position: ({:.1}, {:.1})",
            state.camera_position.x,
            state.camera_position.y
        );
    }

    // 恢复游戏统计
    game_stats.distance_traveled = state.distance_traveled;
    game_stats.jump_count = state.jump_count;
    game_stats.play_time = state.play_time;

    // 恢复角色选择
    character_selection.selected_character = state.selected_character;

    // 恢复音频状态
    audio_state_manager.music_playing = state.music_playing;
    audio_state_manager.music_volume = state.audio_volume;

    crate::debug_log!("Game state fully restored:");
    crate::debug_log!(
        "   Position: ({:.1}, {:.1})",
        state.player_position.x,
        state.player_position.y
    );
    crate::debug_log!("   Animation state: {}", state.player_animation_state);
    crate::debug_log!("   Score: {}", state.score);
    crate::debug_log!("   Distance: {:.1}m", state.distance_traveled);
    crate::debug_log!("   Time: {:.1}s", state.play_time);
    crate::debug_log!("   Music playing: {}", state.music_playing);
}

/// 处理暂停/恢复输入
pub fn handle_pause_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
    mut pause_manager: ResMut<PauseManager>,
    snapshot: PauseSnapshotParams,
    mut last_esc_state: Local<bool>,
) {
    let esc_pressed = keyboard_input.pressed(KeyCode::Escape);
    let esc_just_pressed = esc_pressed && !*last_esc_state;
    let q_just_pressed = keyboard_input.just_pressed(KeyCode::KeyQ);

    *last_esc_state = esc_pressed;

    match current_state.get() {
        GameState::Playing => {
            if esc_just_pressed {
                // 捕获当前游戏状态并暂停
                let state = capture_game_state(
                    snapshot.player_query,
                    snapshot.camera_query,
                    snapshot.game_stats,
                    snapshot.character_selection,
                    snapshot.audio_state_manager,
                );
                pause_manager.pause_game(state);
                next_state.set(GameState::Paused);
                crate::debug_log!("Game paused with state snapshot");
            }
        }
        GameState::Paused => {
            if esc_just_pressed {
                // ESC键恢复游戏
                next_state.set(GameState::Playing);
                crate::debug_log!("Game resumed");
            } else if q_just_pressed {
                // Q键返回主菜单
                pause_manager.resume_game(); // 清理暂停状态
                next_state.set(GameState::Menu);
                crate::debug_log!("Back to main menu");
            }
        }
        _ => {}
    }
}

/// 恢复暂停的游戏状态
pub fn restore_paused_state(
    commands: Commands,
    mut pause_manager: ResMut<PauseManager>,
    current_state: Res<State<GameState>>,
    next_state: Res<NextState<GameState>>,
    restore: RestorePausedParams,
) {
    // If this frame already requested a pause transition, do not consume the snapshot yet.
    if matches!(next_state.as_ref(), NextState::Pending(GameState::Paused)) {
        return;
    }

    if *current_state.get() != GameState::Playing || !pause_manager.is_paused {
        return;
    }

    if let Some(state) = pause_manager.resume_game() {
        restore_game_state(
            commands,
            state,
            restore.player_query,
            restore.camera_query,
            restore.game_stats,
            restore.character_selection,
            restore.audio_state_manager,
        );
    }
}

/// 扫描保存文件目录
pub fn scan_save_files(mut save_file_manager: ResMut<SaveFileManager>) {
    crate::debug_log!("Scanning save files...");
    save_file_manager.save_files.clear();

    let save_dir = Path::new(&save_file_manager.save_directory);
    if !save_dir.exists() {
        crate::debug_log!("Save directory does not exist: {}", save_dir.display());
        return;
    }

    let mut valid_files = 0;
    let mut corrupted_files = 0;

    if let Ok(entries) = fs::read_dir(save_dir) {
        for entry in entries.flatten() {
            if let Some(extension) = entry.path().extension()
                && extension == "json"
            {
                match process_save_file(&entry, &mut save_file_manager) {
                    Ok(true) => valid_files += 1,
                    Ok(false) => corrupted_files += 1,
                    Err(e) => {
                        crate::debug_log!("Error processing {}: {}", entry.path().display(), e);
                        corrupted_files += 1;
                    }
                }
            }
        }
    }

    // 按时间排序，最新的在前
    save_file_manager
        .save_files
        .sort_by(|a, b| b.save_timestamp.cmp(&a.save_timestamp));

    crate::debug_log!("Save file scan complete:");
    crate::debug_log!("   Valid save files: {}", valid_files);
    if corrupted_files > 0 {
        crate::debug_log!("   Corrupted/unreadable files: {}", corrupted_files);
    }
    crate::debug_log!(
        "   Total usable saves: {}",
        save_file_manager.save_files.len()
    );
}

/// 处理单个存档文件
fn process_save_file(
    entry: &std::fs::DirEntry,
    save_file_manager: &mut SaveFileManager,
) -> Result<bool, Box<dyn std::error::Error>> {
    let file_data = fs::read(entry.path())?;
    let json_data = crate::systems::shared_utils::decode_file_payload(&file_data)?;

    // 尝试新格式（SaveFileData with metadata and checksum）
    if let Ok(save_file_data) = serde_json::from_str::<SaveFileData>(&json_data) {
        // 验证校验和
        if !save_file_data.verify_checksum() {
            crate::debug_log!(
                "Checksum mismatch for {}, loading anyway",
                entry.path().display()
            );
        }

        let mut metadata = save_file_data.metadata;
        metadata.file_path = entry.path().to_string_lossy().to_string();

        save_file_manager.save_files.push(metadata);
        crate::debug_log!("Loaded save in v2 format: {}", entry.path().display());
        return Ok(true);
    }

    // 尝试旧格式（直接是 CompleteGameState）
    if let Ok(state) = serde_json::from_str::<CompleteGameState>(&json_data) {
        let file_name_owned = entry.file_name().to_string_lossy().to_string();
        let save_name = file_name_owned.trim_end_matches(".json").to_string();

        let metadata = SaveFileMetadata {
            name: save_name,
            score: state.score,
            distance: state.distance_traveled,
            play_time: state.play_time,
            save_timestamp: state.save_timestamp,
            file_path: entry.path().to_string_lossy().to_string(),
            selected_character: state.selected_character.clone(),
        };

        save_file_manager.save_files.push(metadata);
        crate::debug_log!("Detected legacy save format: {}", entry.path().display());
        return Ok(true);
    }

    // 文件无法解析
    crate::debug_log!("Corrupted save file: {}", entry.path().display());
    Ok(false)
}

/// 删除存档文件
pub fn delete_save_file(
    save_name: &str,
    save_file_manager: &mut SaveFileManager,
) -> Result<(), Box<dyn std::error::Error>> {
    crate::debug_log!("Deleting save file...");
    if let Some(index) = save_file_manager
        .save_files
        .iter()
        .position(|s| s.name == save_name)
    {
        // 先获取文件路径的副本
        let file_path = save_file_manager.save_files[index].file_path.clone();

        // 检查文件是否存在
        if !Path::new(&file_path).exists() {
            return Err(format!("Save file not found: {}", file_path).into());
        }

        // 删除文件
        fs::remove_file(&file_path)
            .map_err(|e| format!("Failed to delete save file '{}': {}", file_path, e))?;

        // 从列表中移除
        save_file_manager.save_files.remove(index);

        crate::debug_log!("Save file deleted successfully: {}", save_name);
        crate::debug_log!("   File: {}", file_path);

        Ok(())
    } else {
        Err(format!("Save file '{}' not found in manager", save_name).into())
    }
}

/// 重命名存档文件
pub fn rename_save_file(
    old_name: &str,
    new_name: &str,
    save_file_manager: &mut SaveFileManager,
) -> Result<(), Box<dyn std::error::Error>> {
    // 验证新名称
    let validator = crate::systems::text_input::InputValidator::new();
    let validated_new_name = validator.validate_save_name(new_name)?;

    // 检查新名称是否已存在
    if save_file_manager
        .save_files
        .iter()
        .any(|s| s.name == validated_new_name && s.name != old_name)
    {
        return Err("Save name already exists".into());
    }

    if let Some(index) = save_file_manager
        .save_files
        .iter()
        .position(|s| s.name == old_name)
    {
        // 先获取旧文件路径的副本
        let old_file_path = save_file_manager.save_files[index].file_path.clone();
        let old_path = Path::new(&old_file_path);

        // 构建新文件路径
        let save_dir = old_path.parent().unwrap();
        let new_file_name = format!("{}.json", validated_new_name);
        let new_path = save_dir.join(&new_file_name);

        // 重命名文件
        fs::rename(old_path, &new_path)
            .map_err(|e| format!("Failed to rename save file: {}", e))?;

        // 更新元数据
        let metadata = &mut save_file_manager.save_files[index];
        metadata.name = validated_new_name.clone();
        metadata.file_path = new_path.to_string_lossy().to_string();

        crate::debug_log!(
            "Save file renamed successfully: {} -> {}",
            old_name,
            validated_new_name
        );
        crate::debug_log!("   Old file: {}", old_path.display());
        crate::debug_log!("   New file: {}", new_path.display());

        Ok(())
    } else {
        Err(format!("Save file '{}' not found", old_name).into())
    }
}
