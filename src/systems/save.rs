use crate::{resources::*, states::*};
use bevy::prelude::*;
use std::fs;

/// 保存游戏数据
pub fn save_game(
    game_stats: Res<GameStats>,
    character_selection: Res<CharacterSelection>,
    mut save_manager: ResMut<SaveManager>,
) {
    let save_data = SaveData {
        player_name: "士郎".to_string(),
        selected_character: character_selection.selected_character.clone(),
        best_distance: game_stats.distance_traveled.max(
            save_manager
                .current_save
                .as_ref()
                .map(|s| s.best_distance)
                .unwrap_or(0.0),
        ),
        total_jumps: game_stats.jump_count
            + save_manager
                .current_save
                .as_ref()
                .map(|s| s.total_jumps)
                .unwrap_or(0),
        total_play_time: game_stats.play_time
            + save_manager
                .current_save
                .as_ref()
                .map(|s| s.total_play_time)
                .unwrap_or(0.0),
        save_time: chrono::Utc::now(),
    };

    match serde_json::to_string_pretty(&save_data) {
        Ok(json_string) => match fs::write(&save_manager.save_file_path, json_string) {
            Ok(_) => {
                save_manager.current_save = Some(save_data);
                println!("💾 游戏已保存！");
                println!(
                    "   最佳距离: {:.1}m",
                    save_manager.current_save.as_ref().unwrap().best_distance
                );
                println!(
                    "   总跳跃次数: {}",
                    save_manager.current_save.as_ref().unwrap().total_jumps
                );
                println!(
                    "   总游戏时间: {:.1}s",
                    save_manager.current_save.as_ref().unwrap().total_play_time
                );
            }
            Err(e) => {
                println!("❌ 保存失败: {}", e);
            }
        },
        Err(e) => {
            println!("❌ 序列化失败: {}", e);
        }
    }
}

/// 加载游戏数据
pub fn load_game(
    mut save_manager: ResMut<SaveManager>,
    mut character_selection: ResMut<CharacterSelection>,
) {
    match fs::read_to_string(&save_manager.save_file_path) {
        Ok(json_string) => match serde_json::from_str::<SaveData>(&json_string) {
            Ok(save_data) => {
                character_selection.selected_character = save_data.selected_character.clone();
                save_manager.current_save = Some(save_data.clone());
                println!("📂 存档已加载！");
                println!("   角色: {:?}", save_data.selected_character);
                println!("   最佳距离: {:.1}m", save_data.best_distance);
                println!("   总跳跃次数: {}", save_data.total_jumps);
                println!("   总游戏时间: {:.1}s", save_data.total_play_time);
                println!(
                    "   保存时间: {}",
                    save_data.save_time.format("%Y-%m-%d %H:%M:%S")
                );
            }
            Err(e) => {
                println!("❌ 存档文件损坏: {}", e);
            }
        },
        Err(_) => {
            println!("📂 没有找到存档文件，将创建新的存档");
        }
    }
}

/// 处理存档按钮点击
pub fn handle_save_button_click(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<crate::components::SaveButton>),
    >,
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
        println!("🎮 存档按钮被点击！");
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
    // 每30秒自动保存一次
    if timer.duration().is_zero() {
        timer.set_duration(std::time::Duration::from_secs(30));
        timer.set_mode(bevy::time::TimerMode::Repeating);
    }
    timer.tick(time.delta());

    if timer.just_finished() && *current_state.get() == GameState::Playing {
        save_game(game_stats, character_selection, save_manager);
    }
}
