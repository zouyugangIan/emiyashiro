//! 增强暂停存档系统
//! 
//! 实现完整的游戏状态保存、恢复和管理功能

use bevy::prelude::*;
use std::fs;
use std::path::Path;
use crate::{
    components::*,
    resources::*,
    states::*,
    systems::ui::*,
};

/// 捕获完整游戏状态
pub fn capture_game_state(
    player_query: Query<(&Transform, &Velocity, &PlayerState), With<Player>>,
    camera_query: Query<&Transform, (With<Camera>, Without<Player>)>,
    game_stats: Res<GameStats>,
    character_selection: Res<CharacterSelection>,
) -> CompleteGameState {
    let mut state = CompleteGameState::default();
    
    // 捕获玩家状态
    if let Ok((player_transform, player_velocity, player_state)) = player_query.single() {
        state.player_position = player_transform.translation;
        state.player_velocity = player_velocity.clone();
        state.player_grounded = player_state.is_grounded;
        state.player_crouching = player_state.is_crouching;
    }
    
    // 捕获摄像机状态
    if let Ok(camera_transform) = camera_query.single() {
        state.camera_position = camera_transform.translation;
    }
    
    // 捕获游戏统计
    state.score = (game_stats.distance_traveled * 10.0) as u32 + game_stats.jump_count * 50;
    state.distance_traveled = game_stats.distance_traveled;
    state.jump_count = game_stats.jump_count;
    state.play_time = game_stats.play_time;
    
    // 捕获角色选择
    state.selected_character = character_selection.selected_character.clone();
    
    // 设置时间戳
    state.save_timestamp = chrono::Utc::now();
    
    state
}

/// 恢复完整游戏状态
pub fn restore_game_state(
    mut commands: Commands,
    state: CompleteGameState,
    mut player_query: Query<(&mut Transform, &mut Velocity, &mut PlayerState), With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    mut game_stats: ResMut<GameStats>,
    mut character_selection: ResMut<CharacterSelection>,
) {
    // 恢复玩家状态
    if let Ok((mut player_transform, mut player_velocity, mut player_state)) = player_query.single_mut() {
        player_transform.translation = state.player_position;
        *player_velocity = state.player_velocity;
        player_state.is_grounded = state.player_grounded;
        player_state.is_crouching = state.player_crouching;
    }
    
    // 恢复摄像机状态
    if let Ok(mut camera_transform) = camera_query.single_mut() {
        camera_transform.translation = state.camera_position;
    }
    
    // 恢复游戏统计
    game_stats.distance_traveled = state.distance_traveled;
    game_stats.jump_count = state.jump_count;
    game_stats.play_time = state.play_time;
    
    // 恢复角色选择
    character_selection.selected_character = state.selected_character;
    
    println!("🔄 游戏状态已恢复:");
    println!("   位置: ({:.1}, {:.1})", state.player_position.x, state.player_position.y);
    println!("   分数: {}", state.score);
    println!("   距离: {:.1}m", state.distance_traveled);
    println!("   时间: {:.1}s", state.play_time);
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
                );
                pause_manager.pause_game(state);
                next_state.set(GameState::Paused);
                println!("⏸️ Game Paused");
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
    mut commands: Commands,
    mut pause_manager: ResMut<PauseManager>,
    player_query: Query<(&mut Transform, &mut Velocity, &mut PlayerState), With<Player>>,
    camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    game_stats: ResMut<GameStats>,
    character_selection: ResMut<CharacterSelection>,
) {
    if let Some(state) = pause_manager.resume_game() {
        restore_game_state(
            commands,
            state,
            player_query,
            camera_query,
            game_stats,
            character_selection,
        );
    }
}

/// 保存游戏到文件
pub fn save_game_to_file(
    save_name: String,
    state: CompleteGameState,
    mut save_file_manager: ResMut<SaveFileManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    // 确保保存目录存在
    let save_dir = Path::new(&save_file_manager.save_directory);
    if !save_dir.exists() {
        println!("创建保存目录: {}", save_dir.display());
        fs::create_dir_all(save_dir)?;
    }
    
    // 创建保存文件路径
    let file_name = format!("{}.json", save_name);
    let file_path = save_dir.join(&file_name);
    
    // 序列化游戏状态
    let json_data = serde_json::to_string_pretty(&state)?;
    
    // 写入文件
    fs::write(&file_path, json_data)?;
    
    // 更新元数据
    let metadata = SaveFileMetadata {
        name: save_name.clone(),
        score: state.score,
        distance: state.distance_traveled,
        play_time: state.play_time,
        save_timestamp: state.save_timestamp,
        file_path: file_path.to_string_lossy().to_string(),
    };
    
    // 更新或添加到保存文件列表
    if let Some(existing) = save_file_manager.save_files.iter_mut().find(|s| s.name == save_name) {
        *existing = metadata;
    } else {
        save_file_manager.save_files.push(metadata);
    }
    
    save_file_manager.current_save_name = Some(save_name.clone());
    
    println!("💾 游戏已保存: {}", save_name);
    Ok(())
}

/// 从文件加载游戏
pub fn load_game_from_file(
    file_path: &str,
) -> Result<CompleteGameState, Box<dyn std::error::Error>> {
    let json_data = fs::read_to_string(file_path)?;
    let state: CompleteGameState = serde_json::from_str(&json_data)?;
    
    println!("📂 游戏已加载: {}", file_path);
    Ok(state)
}

/// 扫描保存文件目录
pub fn scan_save_files(
    mut save_file_manager: ResMut<SaveFileManager>,
) {
    save_file_manager.save_files.clear();
    
    let save_dir = Path::new(&save_file_manager.save_directory);
    if !save_dir.exists() {
        return;
    }
    
    if let Ok(entries) = fs::read_dir(save_dir) {
        for entry in entries.flatten() {
            if let Some(extension) = entry.path().extension() {
                if extension == "json" {
                    if let Ok(json_data) = fs::read_to_string(entry.path()) {
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
                        }
                    }
                }
            }
        }
    }
    
    // 按时间排序，最新的在前
    save_file_manager.save_files.sort_by(|a, b| b.save_timestamp.cmp(&a.save_timestamp));
    
    println!("📁 发现 {} 个存档文件", save_file_manager.save_files.len());
}

/// 删除存档文件
pub fn delete_save_file(
    save_name: &str,
    mut save_file_manager: ResMut<SaveFileManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(index) = save_file_manager.save_files.iter().position(|s| s.name == save_name) {
        let metadata = &save_file_manager.save_files[index];
        fs::remove_file(&metadata.file_path)?;
        save_file_manager.save_files.remove(index);
        println!("🗑️ 已删除存档: {}", save_name);
    }
    Ok(())
}