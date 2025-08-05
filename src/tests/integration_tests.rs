#[cfg(test)]
mod tests {
    use crate::{components::*, resources::*, states::*};
    use bevy::prelude::*;

    fn create_full_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_state::<GameState>()
            .init_resource::<CharacterSelection>()
            .init_resource::<GameStats>()
            .init_resource::<AudioSettings>()
            .init_resource::<crate::systems::audio::AudioManager>()
            .add_systems(Update, (
                crate::systems::player::player_movement,
                crate::systems::player::update_player_state,
                crate::systems::player::update_game_stats,
            ));
        app
    }

    #[test]
    fn test_game_state_transitions() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_state::<GameState>();
        
        // Test initial state
        let state = app.world().resource::<State<GameState>>();
        assert_eq!(*state.get(), GameState::Menu);
        
        // Test state transition
        app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Playing);
        app.update();
        
        let state = app.world().resource::<State<GameState>>();
        assert_eq!(*state.get(), GameState::Playing);
    }

    #[test]
    fn test_player_spawn_and_movement() {
        let mut app = create_full_test_app();
        
        // Spawn player
        let player_entity = app.world_mut().spawn((
            Player,
            Transform::from_translation(GameConfig::PLAYER_START_POS),
            Velocity::zero(),
            PlayerState::default(),
        )).id();
        
        // Simulate input (this would normally come from keyboard)
        // For testing, we'll manually modify velocity
        {
            let mut world = app.world_mut();
            let mut velocity = world.entity_mut(player_entity).get_mut::<Velocity>().unwrap();
            velocity.x = GameConfig::MOVE_SPEED;
        }
        
        // Run several updates
        for _ in 0..10 {
            app.update();
        }
        
        // Check that player moved
        let transform = app.world().entity(player_entity).get::<Transform>().unwrap();
        assert!(transform.translation.x > GameConfig::PLAYER_START_POS.x);
    }

    #[test]
    fn test_game_stats_tracking() {
        let mut app = create_full_test_app();
        
        // Spawn player
        let player_entity = app.world_mut().spawn((
            Player,
            Transform::from_translation(GameConfig::PLAYER_START_POS),
            Velocity::zero(),
            PlayerState::default(),
        )).id();
        
        // Move player forward
        {
            let mut world = app.world_mut();
            let mut transform = world.entity_mut(player_entity).get_mut::<Transform>().unwrap();
            transform.translation.x += 100.0;
        }
        
        // Run update to trigger stats system
        app.update();
        
        // Check that stats were updated
        let stats = app.world().resource::<GameStats>();
        assert!(stats.distance_traveled > 0.0);
        assert!(stats.play_time > 0.0);
    }

    #[test]
    fn test_player_grounding_system() {
        let mut app = create_full_test_app();
        
        // Spawn player above ground
        let player_entity = app.world_mut().spawn((
            Player,
            Transform::from_translation(Vec3::new(0.0, 100.0, 0.0)),
            Velocity::zero(),
            PlayerState::new(false, false), // Not grounded initially
        )).id();
        
        // Run update
        app.update();
        
        // Check that player state reflects being in air
        let player_state = app.world().entity(player_entity).get::<PlayerState>().unwrap();
        assert!(!player_state.is_grounded);
        
        // Move player to ground level
        {
            let mut world = app.world_mut();
            let mut transform = world.entity_mut(player_entity).get_mut::<Transform>().unwrap();
            transform.translation.y = GameConfig::GROUND_LEVEL;
        }
        
        // Run update
        app.update();
        
        // Check that player is now grounded
        let player_state = app.world().entity(player_entity).get::<PlayerState>().unwrap();
        assert!(player_state.is_grounded);
    }

    #[test]
    fn test_audio_manager_state() {
        let audio_manager = crate::systems::audio::AudioManager::default();
        
        assert!(!audio_manager.menu_music_playing);
        assert!(!audio_manager.game_music_playing);
    }

    #[test]
    fn test_character_selection_persistence() {
        let mut selection = CharacterSelection::default();
        assert_eq!(selection.selected_character, CharacterType::Shirou1);
        
        // Change selection
        selection.selected_character = CharacterType::Shirou2;
        assert_eq!(selection.selected_character, CharacterType::Shirou2);
    }

    #[test]
    fn test_save_data_serialization() {
        let save_data = crate::resources::SaveData {
            player_name: "Test Player".to_string(),
            selected_character: CharacterType::Shirou2,
            best_distance: 1000.0,
            total_jumps: 50,
            total_play_time: 300.0,
            save_time: chrono::Utc::now(),
        };
        
        // Test serialization
        let json = serde_json::to_string(&save_data).unwrap();
        assert!(json.contains("Test Player"));
        assert!(json.contains("Shirou2"));
        
        // Test deserialization
        let deserialized: crate::resources::SaveData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.player_name, "Test Player");
        assert_eq!(deserialized.selected_character, CharacterType::Shirou2);
        assert_eq!(deserialized.best_distance, 1000.0);
    }

    // 统一存档系统集成测试
    #[test]
    fn test_complete_pause_save_resume_workflow() {
        let mut app = create_full_test_app();
        
        // 初始化游戏状态
        app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Playing);
        app.update();
        
        // 创建玩家实体
        let player_entity = app.world_mut().spawn((
            Player,
            Transform::from_translation(Vec3::new(100.0, 0.0, 0.0)),
            Velocity::new(50.0, 0.0),
            PlayerState::default(),
        )).id();
        
        // 运行几帧游戏
        for _ in 0..5 {
            app.update();
        }
        
        // 模拟暂停输入
        app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Paused);
        app.update();
        
        // 验证游戏已暂停
        let state = app.world().resource::<State<GameState>>();
        assert_eq!(*state.get(), GameState::Paused);
        
        // 模拟恢复
        app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Playing);
        app.update();
        
        // 验证游戏已恢复
        let state = app.world().resource::<State<GameState>>();
        assert_eq!(*state.get(), GameState::Playing);
        
        // 验证玩家实体仍然存在
        assert!(app.world().get_entity(player_entity).is_some());
    }

    #[test]
    fn test_main_menu_load_game_workflow() {
        let mut app = create_full_test_app();
        
        // 开始在主菜单
        let state = app.world().resource::<State<GameState>>();
        assert_eq!(*state.get(), GameState::Menu);
        
        // 模拟点击加载按钮
        app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::LoadTable);
        app.update();
        
        // 验证进入加载表格状态
        let state = app.world().resource::<State<GameState>>();
        assert_eq!(*state.get(), GameState::LoadTable);
        
        // 模拟选择存档并加载
        app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Playing);
        app.update();
        
        // 验证进入游戏状态
        let state = app.world().resource::<State<GameState>>();
        assert_eq!(*state.get(), GameState::Playing);
    }

    #[test]
    fn test_save_dialog_workflow() {
        let mut app = create_full_test_app();
        
        // 进入保存对话框状态
        app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::SaveDialog);
        app.update();
        
        // 验证状态转换
        let state = app.world().resource::<State<GameState>>();
        assert_eq!(*state.get(), GameState::SaveDialog);
        
        // 模拟文本输入
        let mut text_input = app.world_mut().resource_mut::<systems::text_input::TextInputState>();
        text_input.current_text = "TestSave".to_string();
        
        // 模拟保存确认
        app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Paused);
        app.update();
        
        // 验证返回暂停状态
        let state = app.world().resource::<State<GameState>>();
        assert_eq!(*state.get(), GameState::Paused);
    }

    #[test]
    fn test_rename_dialog_workflow() {
        let mut app = create_full_test_app();
        
        // 进入重命名对话框状态
        app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::RenameDialog);
        app.update();
        
        // 验证状态转换
        let state = app.world().resource::<State<GameState>>();
        assert_eq!(*state.get(), GameState::RenameDialog);
        
        // 模拟重命名输入
        let mut text_input = app.world_mut().resource_mut::<systems::text_input::TextInputState>();
        text_input.current_text = "NewName".to_string();
        
        // 模拟重命名确认
        app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::LoadTable);
        app.update();
        
        // 验证返回加载表格状态
        let state = app.world().resource::<State<GameState>>();
        assert_eq!(*state.get(), GameState::LoadTable);
    }

    #[test]
    fn test_ui_state_transitions() {
        let mut app = create_full_test_app();
        
        // 测试所有状态转换
        let states = vec![
            GameState::Menu,
            GameState::Playing,
            GameState::Paused,
            GameState::SaveDialog,
            GameState::LoadTable,
            GameState::RenameDialog,
        ];
        
        for state in states {
            app.world_mut().resource_mut::<NextState<GameState>>().set(state.clone());
            app.update();
            
            let current_state = app.world().resource::<State<GameState>>();
            assert_eq!(*current_state.get(), state);
        }
    }

    #[test]
    fn test_error_handling_scenarios() {
        let mut app = create_full_test_app();
        
        // 测试文件不存在错误
        let error = systems::error_handling::SaveSystemError::FileNotFound("nonexistent.json".to_string());
        let message = error.to_user_message();
        assert_eq!(message, systems::text_constants::UnifiedSaveText::ERROR_FILE_NOT_FOUND);
        
        // 测试权限错误
        let error = systems::error_handling::SaveSystemError::PermissionDenied("access denied".to_string());
        let message = error.to_user_message();
        assert_eq!(message, systems::text_constants::UnifiedSaveText::ERROR_PERMISSION_DENIED);
        
        // 测试文件损坏错误
        let error = systems::error_handling::SaveSystemError::FileCorrupted("invalid format".to_string());
        let message = error.to_user_message();
        assert_eq!(message, systems::text_constants::UnifiedSaveText::ERROR_CORRUPTED_FILE);
    }

    #[test]
    fn test_audio_continuity_during_pause() {
        let mut app = create_full_test_app();
        
        // 初始化音频管理器
        let mut audio_manager = app.world_mut().resource_mut::<AudioStateManager>();
        audio_manager.set_music_playing(true);
        audio_manager.set_music_position(30.0);
        
        // 进入游戏状态
        app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Playing);
        app.update();
        
        // 暂停游戏
        app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Paused);
        app.update();
        
        // 验证音乐仍在播放
        let audio_manager = app.world().resource::<AudioStateManager>();
        assert!(audio_manager.music_playing);
        assert_eq!(audio_manager.music_position, 30.0);
        
        // 恢复游戏
        app.world_mut().resource_mut::<NextState<GameState>>().set(GameState::Playing);
        app.update();
        
        // 验证音乐状态保持
        let audio_manager = app.world().resource::<AudioStateManager>();
        assert!(audio_manager.music_playing);
    }

    #[test]
    fn test_save_file_management() {
        let mut app = create_full_test_app();
        
        // 测试存档文件管理器
        let mut save_manager = app.world_mut().resource_mut::<SaveFileManager>();
        
        // 创建测试存档元数据
        let metadata = systems::pause_save::SaveFileMetadata {
            name: "TestSave".to_string(),
            player_count: PlayerCount::OnePlayer,
            score: 1000,
            distance: 500.0,
            play_time: 60.0,
            save_timestamp: chrono::Utc::now(),
            file_path: std::path::PathBuf::from("test.json"),
            file_size: 1024,
        };
        
        save_manager.save_files.push(metadata.clone());
        
        // 验证存档文件已添加
        assert_eq!(save_manager.save_files.len(), 1);
        assert_eq!(save_manager.save_files[0].name, "TestSave");
        assert_eq!(save_manager.save_files[0].score, 1000);
        
        // 测试选择存档
        save_manager.selected_save_index = Some(0);
        assert_eq!(save_manager.selected_save_index, Some(0));
    }
}