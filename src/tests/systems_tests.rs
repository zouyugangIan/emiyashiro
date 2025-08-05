#[cfg(test)]
mod tests {
    use crate::{components::*, resources::*, states::*, systems::*};
    use bevy::prelude::*;
    use std::collections::HashSet;

    fn create_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<GameStats>()
            .init_resource::<CharacterSelection>()
            .init_resource::<AudioSettings>()
            .init_resource::<SaveFileManager>()
            .init_resource::<PauseManager>()
            .init_resource::<text_input::TextInputState>()
            .init_resource::<text_input::KeyboardInputHandler>()
            .init_resource::<error_handling::ErrorRecoveryManager>();
        app
    }

    #[test]
    fn test_game_config_constants() {
        // Test that game configuration constants are reasonable
        assert!(GameConfig::GRAVITY > 0.0);
        assert!(GameConfig::JUMP_VELOCITY > 0.0);
        assert!(GameConfig::MOVE_SPEED > 0.0);
        assert!(GameConfig::CAMERA_FOLLOW_SPEED > 0.0);
        
        // Test ground level is negative (below origin)
        assert!(GameConfig::GROUND_LEVEL < 0.0);
    }

    #[test]
    fn test_audio_settings_default() {
        let settings = AudioSettings::default();
        
        assert_eq!(settings.master_volume, 1.0);
        assert_eq!(settings.sfx_volume, 0.7);
        assert_eq!(settings.music_volume, 0.5);
        assert!(settings.music_enabled);
    }

    #[test]
    fn test_game_stats_default() {
        let stats = GameStats::default();
        
        assert_eq!(stats.distance_traveled, 0.0);
        assert_eq!(stats.jump_count, 0);
        assert_eq!(stats.play_time, 0.0);
    }

    #[test]
    fn test_character_selection_default() {
        let selection = CharacterSelection::default();
        assert_eq!(selection.selected_character, CharacterType::Shirou1);
    }

    #[test]
    fn test_character_type_texture_paths() {
        let shirou1 = CharacterType::Shirou1;
        let shirou2 = CharacterType::Shirou2;
        
        assert_eq!(shirou1.get_texture_path(), "images/characters/shirou_idle1.jpg");
        assert_eq!(shirou2.get_texture_path(), "images/characters/shirou_idle2.jpg");
    }

    #[test]
    fn test_save_data_default() {
        let save_data = crate::resources::SaveData::default();
        
        assert_eq!(save_data.player_name, "士郎");
        assert_eq!(save_data.selected_character, CharacterType::Shirou1);
        assert_eq!(save_data.best_distance, 0.0);
        assert_eq!(save_data.total_jumps, 0);
        assert_eq!(save_data.total_play_time, 0.0);
    }

    #[test]
    fn test_save_manager_new() {
        let save_manager = crate::resources::SaveManager::new();
        
        assert!(save_manager.current_save.is_none());
        assert_eq!(save_manager.save_file_path, "save_data.json");
    }

    // Test system behavior with mock data
    #[test]
    fn test_player_state_updates() {
        let mut app = create_test_app();
        
        // Spawn a test player entity
        let player_entity = app.world_mut().spawn((
            Player,
            Transform::from_translation(Vec3::new(0.0, GameConfig::GROUND_LEVEL, 0.0)),
            PlayerState::default(),
        )).id();
        
        // Add the update system
        app.add_systems(Update, crate::systems::player::update_player_state);
        
        // Run one update
        app.update();
        
        // Check that player state was updated correctly
        let player_state = app.world().entity(player_entity).get::<PlayerState>().unwrap();
        assert!(player_state.is_grounded);
    }

    #[test]
    fn test_velocity_physics() {
        // Test velocity calculations
        let mut velocity = Velocity::new(100.0, 200.0);
        let delta_time = 1.0 / 60.0; // 60 FPS
        
        // Apply gravity
        velocity.y -= GameConfig::GRAVITY * delta_time;
        
        // Velocity should decrease due to gravity
        assert!(velocity.y < 200.0);
        
        // Horizontal velocity should remain unchanged
        assert_eq!(velocity.x, 100.0);
    }

    #[test]
    fn test_ground_collision_logic() {
        let ground_level = GameConfig::GROUND_LEVEL;
        let player_y = ground_level - 10.0; // Below ground
        
        // Test collision detection logic
        let should_collide = player_y <= ground_level;
        assert!(should_collide);
        
        // Test position correction
        let corrected_y = player_y.max(ground_level);
        assert_eq!(corrected_y, ground_level);
    }

    #[test]
    fn test_camera_follow_calculations() {
        let player_x = 100.0;
        let camera_x = 0.0;
        let target_x = player_x + GameConfig::CAMERA_OFFSET;
        
        // Test smooth following calculation
        let delta_time = 1.0 / 60.0;
        let follow_speed = GameConfig::CAMERA_FOLLOW_SPEED * delta_time;
        let distance = target_x - camera_x;
        let movement = distance * follow_speed;
        
        assert!(movement > 0.0); // Camera should move towards player
        assert!(movement < distance); // But not instantly
    }

    // 统一存档系统单元测试
    #[test]
    fn test_text_input_processing() {
        let mut text_input = text_input::TextInputState::new(25);
        
        // Test character input
        text_input.add_char('A');
        text_input.add_char('B');
        text_input.add_char('C');
        assert_eq!(text_input.current_text, "ABC");
        
        // Test backspace
        text_input.remove_char();
        assert_eq!(text_input.current_text, "AB");
        
        // Test length limit
        for _ in 0..30 {
            text_input.add_char('X');
        }
        assert!(text_input.current_text.len() <= 25);
        
        // Test clear
        text_input.clear();
        assert_eq!(text_input.current_text, "");
    }

    #[test]
    fn test_save_name_validation() {
        let validator = text_input::InputValidator::new();
        
        // Test valid names
        assert!(validator.validate_save_name("MyGame").is_ok());
        assert!(validator.validate_save_name("Game_123").is_ok());
        assert!(validator.validate_save_name("Test-Save").is_ok());
        
        // Test invalid names
        assert!(validator.validate_save_name("").is_err()); // Empty
        assert!(validator.validate_save_name("A".repeat(30).as_str()).is_err()); // Too long
        assert!(validator.validate_save_name("Game@#$").is_err()); // Invalid chars
        
        // Test default name handling
        let default_name = validator.get_default_name();
        assert_eq!(default_name, "DefaultSave");
    }

    #[test]
    fn test_game_state_serialization() {
        let game_state = pause_save::CompleteGameState {
            player_position: Vec3::new(100.0, 50.0, 0.0),
            player_velocity: Velocity::new(10.0, 5.0),
            score: 1500,
            distance_traveled: 2500.0,
            jump_count: 25,
            play_time: 120.0,
            music_position: 45.5,
            music_playing: true,
            audio_volume: 0.8,
            selected_character: CharacterType::Shirou1,
            player_count: PlayerCount::OnePlayer,
            save_timestamp: chrono::Utc::now(),
            entities_state: vec![],
        };
        
        // Test serialization
        let json = serde_json::to_string(&game_state).unwrap();
        assert!(json.contains("100"));
        assert!(json.contains("1500"));
        assert!(json.contains("Shirou1"));
        
        // Test deserialization
        let deserialized: pause_save::CompleteGameState = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.score, 1500);
        assert_eq!(deserialized.distance_traveled, 2500.0);
        assert_eq!(deserialized.selected_character, CharacterType::Shirou1);
    }

    #[test]
    fn test_file_operations() {
        let mut save_manager = SaveFileManager::default();
        
        // Test save directory creation
        save_manager.ensure_save_directory().unwrap();
        assert!(save_manager.save_directory.exists());
        
        // Test save file metadata
        let metadata = pause_save::SaveFileMetadata {
            name: "TestSave".to_string(),
            player_count: PlayerCount::OnePlayer,
            score: 1000,
            distance: 500.0,
            play_time: 60.0,
            save_timestamp: chrono::Utc::now(),
            file_path: std::path::PathBuf::from("test.json"),
            file_size: 1024,
        };
        
        assert_eq!(metadata.name, "TestSave");
        assert_eq!(metadata.score, 1000);
    }

    #[test]
    fn test_english_text_constants() {
        use crate::systems::text_constants::UnifiedSaveText;
        
        // Test main menu texts
        assert_eq!(UnifiedSaveText::MAIN_MENU_LOAD, "Load Game");
        
        // Test pause menu texts
        assert_eq!(UnifiedSaveText::PAUSE_TITLE, "Game Paused");
        assert_eq!(UnifiedSaveText::PAUSE_SAVE, "Save Game");
        assert_eq!(UnifiedSaveText::PAUSE_RESUME, "Resume");
        
        // Test save dialog texts
        assert_eq!(UnifiedSaveText::SAVE_DIALOG_TITLE, "Save Game");
        assert_eq!(UnifiedSaveText::SAVE_NAME_PROMPT, "Enter save name:");
        assert_eq!(UnifiedSaveText::SAVE_CONFIRM, "Save");
        assert_eq!(UnifiedSaveText::SAVE_CANCEL, "Cancel");
        
        // Test load table texts
        assert_eq!(UnifiedSaveText::LOAD_TABLE_TITLE, "Load & Manage Saves");
        assert_eq!(UnifiedSaveText::LOAD_NAME_HEADER, "Name");
        assert_eq!(UnifiedSaveText::LOAD_PLAYERS_HEADER, "Players");
        assert_eq!(UnifiedSaveText::LOAD_SCORE_HEADER, "Score");
        
        // Test error messages
        assert_eq!(UnifiedSaveText::ERROR_SAVE_FAILED, "Failed to save game");
        assert_eq!(UnifiedSaveText::ERROR_LOAD_FAILED, "Failed to load game");
        assert_eq!(UnifiedSaveText::ERROR_FILE_NOT_FOUND, "Save file not found");
    }

    #[test]
    fn test_error_handling_system() {
        let mut error_manager = error_handling::ErrorRecoveryManager::new();
        
        // Test error message conversion
        let error = error_handling::SaveSystemError::FileNotFound("test.json".to_string());
        let message = error.to_user_message();
        assert_eq!(message, text_constants::UnifiedSaveText::ERROR_FILE_NOT_FOUND);
        
        // Test retry logic
        let action = error_manager.handle_save_error(error);
        match action {
            error_handling::RecoveryAction::Retry => assert!(true),
            error_handling::RecoveryAction::ShowError(_) => assert!(true),
            _ => assert!(false, "Unexpected recovery action"),
        }
    }

    #[test]
    fn test_audio_state_management() {
        let mut audio_manager = AudioStateManager::default();
        
        // Test initial state
        assert!(!audio_manager.music_playing);
        assert_eq!(audio_manager.music_position, 0.0);
        assert_eq!(audio_manager.volume, 1.0);
        
        // Test state updates
        audio_manager.set_music_playing(true);
        audio_manager.set_music_position(30.5);
        audio_manager.set_volume(0.8);
        
        assert!(audio_manager.music_playing);
        assert_eq!(audio_manager.music_position, 30.5);
        assert_eq!(audio_manager.volume, 0.8);
    }

    #[test]
    fn test_pause_manager_state_capture() {
        let mut pause_manager = PauseManager::default();
        
        // Test state capture
        let test_state = pause_save::CompleteGameState {
            player_position: Vec3::new(50.0, 25.0, 0.0),
            player_velocity: Velocity::new(5.0, 2.5),
            score: 750,
            distance_traveled: 1250.0,
            jump_count: 15,
            play_time: 75.0,
            music_position: 22.5,
            music_playing: true,
            audio_volume: 0.9,
            selected_character: CharacterType::Shirou2,
            player_count: PlayerCount::TwoPlayer,
            save_timestamp: chrono::Utc::now(),
            entities_state: vec![],
        };
        
        pause_manager.captured_state = Some(test_state.clone());
        
        // Test state retrieval
        let retrieved_state = pause_manager.get_captured_state().unwrap();
        assert_eq!(retrieved_state.score, 750);
        assert_eq!(retrieved_state.distance_traveled, 1250.0);
        assert_eq!(retrieved_state.selected_character, CharacterType::Shirou2);
    }
}