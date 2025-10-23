//! 增强暂停存档系统
//!
//! 实现完整的游戏状态保存、恢复和管理功能

use crate::{components::*, resources::*, states::*};
use bevy::prelude::*;
use std::fs;
use std::path::Path;

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
    state.player_count = PlayerCount::Single; // 目前只支持单人游戏

    // 捕获音频状态
    state.music_playing = audio_state_manager.music_playing;
    state.audio_volume = audio_state_manager.music_volume;
    state.music_position = 0.0; // TODO: 实现音频位置跟踪

    // 捕获实体快照（目前为空，未来可扩展）
    state.entities_snapshot = Vec::new();

    // 设置时间戳
    state.save_timestamp = chrono::Utc::now();

    println!("🎮 游戏状态已捕获:");
    println!(
        "   玩家位置: ({:.1}, {:.1})",
        state.player_position.x, state.player_position.y
    );
    println!("   动画状态: {}", state.player_animation_state);
    println!("   分数: {}", state.score);
    println!("   距离: {:.1}m", state.distance_traveled);
    println!("   时间: {:.1}s", state.play_time);
    println!("   音乐播放: {}", state.music_playing);

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

        println!(
            "🔄 恢复玩家状态: 位置({:.1}, {:.1}), 动画: {}",
            state.player_position.x, state.player_position.y, state.player_animation_state
        );
    }

    // 恢复摄像机状态
    if let Ok(mut camera_transform) = camera_query.single_mut() {
        camera_transform.translation = state.camera_position;
        println!(
            "🔄 恢复摄像机位置: ({:.1}, {:.1})",
            state.camera_position.x, state.camera_position.y
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

    println!("🔄 游戏状态已完全恢复:");
    println!(
        "   位置: ({:.1}, {:.1})",
        state.player_position.x, state.player_position.y
    );
    println!("   动画状态: {}", state.player_animation_state);
    println!("   分数: {}", state.score);
    println!("   距离: {:.1}m", state.distance_traveled);
    println!("   时间: {:.1}s", state.play_time);
    println!("   音乐播放: {}", state.music_playing);
}

/// 处理暂停/恢复输入
pub fn handle_pause_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
    mut pause_manager: ResMut<PauseManager>,
    player_query: Query<(&Transform, &Velocity, &PlayerState), With<Player>>,
    camera_query: Query<&Transform, (With<Camera>, Without<Player>)>,
    game_stats: Res<GameStats>,
    character_selection: Res<CharacterSelection>,
    audio_state_manager: Res<AudioStateManager>,
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
                    player_query,
                    camera_query,
                    game_stats,
                    character_selection,
                    audio_state_manager,
                );
                pause_manager.pause_game(state);
                next_state.set(GameState::Paused);
                println!("⏸️ Game Paused with enhanced state capture");
            }
        }
        GameState::Paused => {
            if esc_just_pressed {
                // ESC键恢复游戏
                next_state.set(GameState::Playing);
                println!("▶️ Game Resumed");
            } else if q_just_pressed {
                // Q键返回主菜单
                pause_manager.resume_game(); // 清理暂停状态
                next_state.set(GameState::Menu);
                println!("🏠 Back to Main Menu");
            }
        }
        _ => {}
    }
}

/// 恢复暂停的游戏状态
pub fn restore_paused_state(
    commands: Commands,
    mut pause_manager: ResMut<PauseManager>,
    player_query: Query<(&mut Transform, &mut Velocity, &mut PlayerState), With<Player>>,
    camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    game_stats: ResMut<GameStats>,
    character_selection: ResMut<CharacterSelection>,
    audio_state_manager: ResMut<AudioStateManager>,
) {
    if let Some(state) = pause_manager.resume_game() {
        restore_game_state(
            commands,
            state,
            player_query,
            camera_query,
            game_stats,
            character_selection,
            audio_state_manager,
        );
    }
}

/// 保存游戏到文件
pub fn save_game_to_file(
    save_name: String,
    state: CompleteGameState,
    mut save_file_manager: ResMut<SaveFileManager>,
) -> Result<(), crate::systems::error_handling::SaveSystemError> {
    use crate::systems::error_handling::{SaveSystemError, convert_io_error};
    use crate::systems::shared_utils::calculate_checksum;

    println!("Creating save directory...");

    // 确保保存目录存在
    let save_dir = Path::new(&save_file_manager.save_directory);
    if !save_dir.exists() {
        println!("📁 Creating save directory: {}", save_dir.display());
        fs::create_dir_all(save_dir).map_err(|e| {
            convert_io_error(e, &format!("create directory {}", save_dir.display()))
        })?;
    }

    // 验证保存名称
    let validator = crate::systems::text_input::InputValidator::new();
    let validated_name = validator
        .validate_save_name(&save_name)
        .map_err(|e| SaveSystemError::InvalidFileName(e.to_string()))?;

    // 检查名称是否已存在（用于覆盖确认）
    let file_name = format!("{}.json", validated_name);
    let file_path = save_dir.join(&file_name);

    println!("Writing save file...");

    // 创建完整的保存文件数据结构
    let mut save_file_data = SaveFileData {
        version: "1.0".to_string(),
        metadata: SaveFileMetadata {
            name: validated_name.clone(),
            score: state.score,
            distance: state.distance_traveled,
            play_time: state.play_time,
            save_timestamp: state.save_timestamp,
            file_path: file_path.to_string_lossy().to_string(),
        },
        game_state: state.clone(),
        checksum: String::new(), // 初始为空
    };

    // 序列化一次以计算校验和
    let json_for_checksum = serde_json::to_string_pretty(&save_file_data)
        .map_err(|e| SaveSystemError::SerializationFailed(e.to_string()))?;

    // 计算并设置校验和
    save_file_data.checksum = calculate_checksum(json_for_checksum.as_bytes());

    // 再次序列化以获得最终的、包含正确校验和的JSON
    let json_data = serde_json::to_string_pretty(&save_file_data)
        .map_err(|e| SaveSystemError::SerializationFailed(e.to_string()))?;

    // 写入文件
    fs::write(&file_path, &json_data)
        .map_err(|e| convert_io_error(e, &format!("write file {}", file_path.display())))?;

    // 更新元数据
    let metadata = save_file_data.metadata;

    // 更新或添加到保存文件列表
    if let Some(existing) = save_file_manager
        .save_files
        .iter_mut()
        .find(|s| s.name == validated_name)
    {
        *existing = metadata;
    } else {
        save_file_manager.save_files.push(metadata);
    }

    save_file_manager.current_save_name = Some(validated_name.clone());

    println!("💾 Game saved successfully: {}", validated_name);
    println!("   File: {}", file_path.display());
    println!("   Size: {} bytes", json_data.len());

    Ok(())
}

/// 保存文件数据结构
#[derive(serde::Serialize, serde::Deserialize)]
pub struct SaveFileData {
    pub version: String,
    pub metadata: SaveFileMetadata,
    pub game_state: CompleteGameState,
    pub checksum: String,
}

/// 从文件加载游戏
pub fn load_game_from_file(
    file_path: &str,
) -> Result<CompleteGameState, crate::systems::error_handling::SaveSystemError> {
    use crate::systems::error_handling::{SaveSystemError, convert_io_error};
    use crate::systems::shared_utils::calculate_checksum;

    println!("Reading save file...");

    let json_data = fs::read_to_string(file_path)
        .map_err(|e| convert_io_error(e, &format!("read file {}", file_path)))?;

    // 尝试加载新格式的存档文件
    if let Ok(mut save_file_data) = serde_json::from_str::<SaveFileData>(&json_data) {
        // 验证校验和
        let received_checksum = save_file_data.checksum.clone();
        save_file_data.checksum = String::new(); // 重置以进行校验

        let json_for_check = serde_json::to_string_pretty(&save_file_data)
            .map_err(|e| SaveSystemError::SerializationFailed(e.to_string()))?;

        let calculated_checksum = calculate_checksum(json_for_check.as_bytes());

        if received_checksum != calculated_checksum {
            println!("⚠️ Warning: Checksum mismatch for save file, but continuing...");
            // 可以选择返回错误或继续
            // return Err(SaveSystemError::ChecksumMismatch);
        }

        println!("📂 Game loaded successfully: {}", file_path);
        println!("   Version: {}", save_file_data.version);
        println!("   Save name: {}", save_file_data.metadata.name);
        println!("   Score: {}", save_file_data.game_state.score);
        println!(
            "   Distance: {:.1}m",
            save_file_data.game_state.distance_traveled
        );

        Ok(save_file_data.game_state)
    }
    // 回退到旧格式的存档文件
    else if let Ok(state) = serde_json::from_str::<CompleteGameState>(&json_data) {
        println!("📂 Legacy save file loaded: {}", file_path);
        println!("⚠️ Consider re-saving to upgrade to new format");
        Ok(state)
    } else {
        Err(SaveSystemError::FileCorrupted(file_path.to_string()))
    }
}

/// 扫描保存文件目录
pub fn scan_save_files(mut save_file_manager: ResMut<SaveFileManager>) {
    println!("Scanning save files...");
    save_file_manager.save_files.clear();

    let save_dir = Path::new(&save_file_manager.save_directory);
    if !save_dir.exists() {
        println!("📁 Save directory does not exist: {}", save_dir.display());
        return;
    }

    let mut valid_files = 0;
    let mut corrupted_files = 0;

    if let Ok(entries) = fs::read_dir(save_dir) {
        for entry in entries.flatten() {
            if let Some(extension) = entry.path().extension() {
                if extension == "json" {
                    match process_save_file(&entry, &mut save_file_manager) {
                        Ok(true) => valid_files += 1,
                        Ok(false) => corrupted_files += 1,
                        Err(e) => {
                            println!("⚠️ Error processing {}: {}", entry.path().display(), e);
                            corrupted_files += 1;
                        }
                    }
                }
            }
        }
    }

    // 按时间排序，最新的在前
    save_file_manager
        .save_files
        .sort_by(|a, b| b.save_timestamp.cmp(&a.save_timestamp));

    println!("📁 Scan complete:");
    println!("   Valid save files: {}", valid_files);
    if corrupted_files > 0 {
        println!("   Corrupted/unreadable files: {}", corrupted_files);
    }
    println!(
        "   Total usable saves: {}",
        save_file_manager.save_files.len()
    );
}

/// 处理单个存档文件
fn process_save_file(
    entry: &std::fs::DirEntry,
    save_file_manager: &mut SaveFileManager,
) -> Result<bool, Box<dyn std::error::Error>> {
    use crate::systems::shared_utils::calculate_checksum;
    let json_data = fs::read_to_string(entry.path())?;

    // 尝试新格式
    if let Ok(mut save_file_data) = serde_json::from_str::<SaveFileData>(&json_data) {
        // 验证校验和
        let received_checksum = save_file_data.checksum.clone();
        save_file_data.checksum = String::new();

        if let Ok(json_for_check) = serde_json::to_string_pretty(&save_file_data) {
            let calculated_checksum = calculate_checksum(json_for_check.as_bytes());

            if received_checksum != calculated_checksum {
                println!(
                    "⚠️ Checksum mismatch for {}, but loading anyway",
                    entry.path().display()
                );
            }
        } else {
            println!(
                "⚠️ Could not re-serialize for checksum validation: {}",
                entry.path().display()
            );
        }

        save_file_manager.save_files.push(save_file_data.metadata);
        return Ok(true);
    }

    // 尝试旧格式
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
        };

        save_file_manager.save_files.push(metadata);
        println!("📂 Legacy format detected: {}", entry.path().display());
        return Ok(true);
    }

    // 文件无法解析
    println!("❌ Corrupted save file: {}", entry.path().display());
    Ok(false)
}

/// 删除存档文件
pub fn delete_save_file(
    save_name: &str,
    save_file_manager: &mut SaveFileManager,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Deleting save file...");
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

        println!("🗑️ Save file deleted successfully: {}", save_name);
        println!("   File: {}", file_path);

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
        fs::rename(&old_path, &new_path)
            .map_err(|e| format!("Failed to rename save file: {}", e))?;

        // 更新元数据
        let metadata = &mut save_file_manager.save_files[index];
        metadata.name = validated_new_name.clone();
        metadata.file_path = new_path.to_string_lossy().to_string();

        println!(
            "✏️ Save file renamed successfully: {} -> {}",
            old_name, validated_new_name
        );
        println!("   Old file: {}", old_path.display());
        println!("   New file: {}", new_path.display());

        Ok(())
    } else {
        Err(format!("Save file '{}' not found", old_name).into())
    }
}
