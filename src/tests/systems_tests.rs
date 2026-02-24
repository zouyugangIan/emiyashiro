#[cfg(test)]
mod tests {
    use crate::systems::ui::{GameHUD, setup_game_hud};
    use crate::{components::*, resources::*, states::*, systems::*};
    use bevy::prelude::*;
    use std::fs;

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
        assert!(std::hint::black_box(GameConfig::GRAVITY) > 0.0);
        assert!(std::hint::black_box(GameConfig::JUMP_VELOCITY) > 0.0);
        assert!(std::hint::black_box(GameConfig::MOVE_SPEED) > 0.0);
        assert!(std::hint::black_box(GameConfig::CAMERA_FOLLOW_SPEED) > 0.0);

        // Test ground level is negative (below origin)
        assert!(std::hint::black_box(GameConfig::GROUND_LEVEL) < 0.0);
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

        assert_eq!(
            shirou1.get_texture_path(),
            "images/characters/shirou_idle1.jpg"
        );
        assert_eq!(
            shirou2.get_texture_path(),
            "images/characters/shirou_idle2.jpg"
        );
    }

    #[test]
    fn test_save_data_default() {
        let save_data = crate::resources::SaveData::default();

        assert_eq!(save_data.player_name, "DefaultSave");
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

    #[test]
    fn test_save_game_writes_v2_schema() {
        let temp_path =
            std::env::temp_dir().join(format!("emiyashiro-save-v2-{}.json", uuid::Uuid::new_v4()));

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<GameStats>()
            .init_resource::<CharacterSelection>()
            .init_resource::<SaveManager>()
            .add_systems(Update, save::save_game);

        app.world_mut().resource_mut::<SaveManager>().save_file_path =
            temp_path.to_string_lossy().to_string();
        app.world_mut()
            .resource_mut::<GameStats>()
            .distance_traveled = 321.0;
        app.world_mut().resource_mut::<GameStats>().jump_count = 7;
        app.world_mut().resource_mut::<GameStats>().play_time = 42.0;

        app.update();

        let json = fs::read_to_string(&temp_path).expect("save file should exist");
        let v2_save: SaveFileData = serde_json::from_str(&json).expect("should be SaveFileData v2");

        assert_eq!(v2_save.version, "2.0");
        assert_eq!(v2_save.game_state.distance_traveled, 321.0);
        assert_eq!(v2_save.game_state.jump_count, 7);
        assert!(v2_save.verify_checksum());

        let _ = fs::remove_file(temp_path);
    }

    #[test]
    fn test_load_game_migrates_legacy_save_data_to_v2() {
        let temp_path = std::env::temp_dir().join(format!(
            "emiyashiro-legacy-save-{}.json",
            uuid::Uuid::new_v4()
        ));

        let legacy = SaveData {
            player_name: "LegacyPlayer".to_string(),
            selected_character: CharacterType::Shirou2,
            best_distance: 888.0,
            total_jumps: 99,
            total_play_time: 123.0,
            save_time: chrono::Utc::now(),
        };
        let legacy_json = serde_json::to_string_pretty(&legacy).expect("serialize legacy save");
        fs::write(&temp_path, legacy_json.as_bytes()).expect("write legacy save");

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<CharacterSelection>()
            .init_resource::<SaveManager>()
            .add_systems(Update, save::load_game);

        app.world_mut().resource_mut::<SaveManager>().save_file_path =
            temp_path.to_string_lossy().to_string();

        app.update();

        let migrated_json = fs::read_to_string(&temp_path).expect("migrated save should exist");
        let v2_save: SaveFileData =
            serde_json::from_str(&migrated_json).expect("legacy save should be migrated to v2");

        assert_eq!(v2_save.version, "2.0");
        assert_eq!(v2_save.metadata.name, "LegacyPlayer");
        assert_eq!(
            v2_save.game_state.selected_character,
            CharacterType::Shirou2
        );
        assert_eq!(v2_save.game_state.distance_traveled, 888.0);

        let loaded_selection = app.world().resource::<CharacterSelection>();
        assert_eq!(loaded_selection.selected_character, CharacterType::Shirou2);

        let _ = fs::remove_file(temp_path);
    }

    #[test]
    fn test_damage_event_transitions_to_game_over() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(bevy::state::app::StatesPlugin)
            .init_state::<GameState>()
            .add_message::<crate::events::DamageEvent>()
            .add_systems(Update, crate::systems::combat::apply_damage_events);

        let player = app.world_mut().spawn((Player, Health::new(10.0))).id();

        app.world_mut()
            .resource_mut::<Messages<crate::events::DamageEvent>>()
            .write(crate::events::DamageEvent {
                target: player,
                amount: 99.0,
                source: crate::events::DamageSource::EnemyContact,
            });

        app.update();
        app.update();

        let state = app.world().resource::<State<GameState>>();
        assert_eq!(*state.get(), GameState::GameOver);
    }

    #[test]
    fn test_revive_flow_restores_player_state() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(bevy::state::app::StatesPlugin)
            .init_state::<GameState>()
            .init_resource::<GameStats>()
            .add_systems(
                OnEnter(GameState::Reviving),
                crate::systems::death::revive_player,
            );

        let player = app
            .world_mut()
            .spawn((
                Player,
                Transform::from_xyz(10.0, -999.0, 0.0),
                Velocity::new(13.0, -25.0),
                PlayerState {
                    is_grounded: false,
                    is_crouching: true,
                },
                Health {
                    current: 0.0,
                    max: 100.0,
                },
                ShroudState {
                    is_released: true,
                    ..Default::default()
                },
            ))
            .id();

        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Reviving);
        app.update(); // enter Reviving and run revive_player
        app.update(); // transition to Playing

        let state = app.world().resource::<State<GameState>>();
        assert_eq!(*state.get(), GameState::Playing);

        let entity = app.world().entity(player);
        let transform = entity.get::<Transform>().expect("player transform");
        let velocity = entity.get::<Velocity>().expect("player velocity");
        let player_state = entity.get::<PlayerState>().expect("player state");
        let health = entity.get::<Health>().expect("player health");
        let shroud = entity.get::<ShroudState>().expect("player shroud");

        assert_eq!(transform.translation, GameConfig::PLAYER_START_POS);
        assert_eq!(velocity.x, 0.0);
        assert_eq!(velocity.y, 0.0);
        assert!(player_state.is_grounded);
        assert!(!player_state.is_crouching);
        assert_eq!(health.current, health.max);
        assert!(!shroud.is_released);
    }

    // Test system behavior with mock data
    #[test]
    fn test_player_state_updates() {
        let mut app = create_test_app();

        // Spawn a test player entity
        let player_entity = app
            .world_mut()
            .spawn((
                Player,
                Transform::from_translation(Vec3::new(0.0, GameConfig::GROUND_LEVEL, 0.0)),
                PlayerState::default(),
            ))
            .id();

        // Add the update system
        app.add_systems(Update, crate::systems::player::update_player_state);

        // Run one update
        app.update();

        // Check that player state was updated correctly
        let player_state = app
            .world()
            .entity(player_entity)
            .get::<PlayerState>()
            .unwrap();
        assert!(player_state.is_grounded);
    }

    #[test]
    fn test_collision_system_keeps_grounded_when_near_ground_level() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_systems(
            Update,
            crate::systems::collision::collision_detection_system,
        );

        let player_entity = app
            .world_mut()
            .spawn((
                Player,
                Transform::from_translation(Vec3::new(0.0, GameConfig::GROUND_LEVEL, 0.0)),
                Velocity::default(),
                PlayerState {
                    is_grounded: false,
                    is_crouching: false,
                },
                crate::systems::collision::CollisionBox::new(GameConfig::PLAYER_SIZE),
            ))
            .id();

        app.world_mut().spawn((
            Ground,
            Transform::from_translation(GameConfig::GROUND_POS),
            crate::systems::collision::CollisionBox::new(GameConfig::GROUND_SIZE),
        ));

        app.update();

        let player_state = app
            .world()
            .entity(player_entity)
            .get::<PlayerState>()
            .expect("player state should exist");
        assert!(
            player_state.is_grounded,
            "near-ground fallback should prevent false airborne state"
        );
    }

    #[test]
    fn test_setup_game_hud_is_idempotent() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Update, setup_game_hud);

        app.update();
        app.update();

        let hud_count = {
            let world = app.world_mut();
            let mut hud_query = world.query_filtered::<Entity, With<GameHUD>>();
            hud_query.iter(world).count()
        };
        assert_eq!(hud_count, 1);
    }

    #[test]
    fn test_scene_decoration_setup_is_idempotent_when_already_initialized() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(bevy::asset::AssetPlugin::default())
            .add_systems(Update, scene_decoration::setup_parallax_background);

        app.world_mut().spawn(scene_decoration::SceneDecoration {
            layer: scene_decoration::DecorationLayer::FarBackground,
            speed_multiplier: 0.2,
        });

        app.update();

        let decoration_count = {
            let world = app.world_mut();
            let mut query = world.query::<&scene_decoration::SceneDecoration>();
            query.iter(world).count()
        };
        assert_eq!(decoration_count, 1);
    }

    #[test]
    fn test_cleanup_scene_decorations_removes_all_entities() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Update, scene_decoration::cleanup_scene_decorations);

        app.world_mut().spawn(scene_decoration::SceneDecoration {
            layer: scene_decoration::DecorationLayer::FarBackground,
            speed_multiplier: 0.2,
        });
        app.world_mut().spawn(scene_decoration::SceneDecoration {
            layer: scene_decoration::DecorationLayer::MidBackground,
            speed_multiplier: 0.5,
        });

        app.update();

        let decoration_count = {
            let world = app.world_mut();
            let mut query = world.query::<&scene_decoration::SceneDecoration>();
            query.iter(world).count()
        };
        assert_eq!(decoration_count, 0);
    }

    #[test]
    fn test_arrow_up_maps_to_jump_input() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ButtonInput::<KeyCode>::default())
            .init_resource::<input::GameInput>()
            .init_resource::<crate::systems::network::NetworkResource>()
            .add_systems(Update, input::update_game_input);

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowUp);

        app.update();

        let game_input = app.world().resource::<input::GameInput>();
        assert!(game_input.jump);
    }

    #[test]
    fn test_horizontal_conflict_keeps_jump_update() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ButtonInput::<KeyCode>::default())
            .init_resource::<input::GameInput>()
            .init_resource::<crate::systems::network::NetworkResource>()
            .add_systems(Update, input::update_game_input);

        {
            let mut keyboard = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keyboard.press(KeyCode::KeyA);
            keyboard.press(KeyCode::ArrowRight);
            keyboard.press(KeyCode::ArrowUp);
        }

        app.update();

        let game_input = app.world().resource::<input::GameInput>();
        assert!(
            game_input.jump,
            "jump input should not be dropped on left/right conflict"
        );
        assert!(!game_input.move_left);
        assert!(!game_input.move_right);
    }

    #[test]
    fn test_jump_allows_near_ground_recovery_when_ground_state_stale() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(Time::<Fixed>::from_hz(60.0))
            .insert_resource(ButtonInput::<KeyCode>::default())
            .init_resource::<input::GameInput>()
            .init_resource::<GameStats>()
            .add_systems(Update, crate::systems::player::player_jump);

        let player_entity = app
            .world_mut()
            .spawn((
                Player,
                Transform::from_translation(Vec3::new(0.0, GameConfig::GROUND_LEVEL + 1.0, 0.0)),
                Velocity::default(),
                PlayerState {
                    is_grounded: false,
                    is_crouching: false,
                },
            ))
            .id();

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowUp);

        app.update();

        let entity = app.world().entity(player_entity);
        let velocity = entity.get::<Velocity>().expect("velocity should exist");
        assert!(
            velocity.y > 0.0,
            "player should jump when near ground even if grounded flag is stale"
        );
    }

    #[test]
    fn test_character_select_and_start_same_frame_prefers_latest_selection() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(bevy::state::app::StatesPlugin)
            .init_state::<GameState>()
            .init_resource::<CharacterSelection>()
            .init_resource::<GameStats>()
            .init_resource::<PauseManager>()
            .init_resource::<crate::systems::ui::LoadedGameState>()
            .init_resource::<crate::systems::ui::SaveLoadUiState>()
            .add_systems(
                Update,
                (menu::handle_character_select, menu::handle_start_button)
                    .chain()
                    .run_if(in_state(GameState::Menu)),
            );

        // Bootstrap default state to Menu first.
        app.update();

        app.world_mut().spawn((
            CharacterSelectButton {
                character_type: CharacterType::Shirou2,
            },
            Interaction::Pressed,
            BackgroundColor(Color::NONE),
        ));

        app.world_mut().spawn((
            StartButton,
            Interaction::Pressed,
            BackgroundColor(Color::NONE),
        ));

        app.update();

        let selection = app.world().resource::<CharacterSelection>();
        assert_eq!(selection.selected_character, CharacterType::Shirou2);

        app.update();
        let state = app.world().resource::<State<GameState>>();
        assert_eq!(*state.get(), GameState::Playing);
    }

    #[test]
    fn test_escape_pause_toggle_is_edge_triggered() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(bevy::state::app::StatesPlugin)
            .init_state::<GameState>()
            .insert_resource(ButtonInput::<KeyCode>::default())
            .init_resource::<GameStats>()
            .init_resource::<CharacterSelection>()
            .init_resource::<AudioStateManager>()
            .init_resource::<PauseManager>()
            .add_systems(
                Update,
                pause_save::handle_pause_input
                    .run_if(in_state(GameState::Playing).or(in_state(GameState::Paused))),
            );

        app.world_mut().spawn((
            Player,
            Transform::from_translation(GameConfig::PLAYER_START_POS),
            Velocity::default(),
            PlayerState::default(),
        ));
        app.world_mut().spawn((Camera2d,));

        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update();
        app.update();

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);

        app.update();
        app.update();
        assert_eq!(
            *app.world().resource::<State<GameState>>().get(),
            GameState::Paused
        );

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(KeyCode::Escape);

        app.update();
        app.update();
        assert_eq!(
            *app.world().resource::<State<GameState>>().get(),
            GameState::Paused
        );

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();
        app.update();
        assert_eq!(
            *app.world().resource::<State<GameState>>().get(),
            GameState::Playing
        );
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

        // Test activation/deactivation
        text_input.activate();
        assert!(text_input.is_active);
        text_input.deactivate();
        assert!(!text_input.is_active);
    }

    #[test]
    fn test_save_name_validation() {
        let validator = text_input::InputValidator::new();

        // Test valid names
        assert!(validator.validate_save_name("MyGame").is_ok());
        assert!(validator.validate_save_name("Game_123").is_ok());
        assert!(validator.validate_save_name("Test-Save").is_ok());

        // Test empty name (should return default, not error)
        let result = validator.validate_save_name("");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "DefaultSave");
        assert!(
            validator
                .validate_save_name("A".repeat(30).as_str())
                .is_err()
        ); // Too long
        assert!(validator.validate_save_name("Game@#$").is_err()); // Invalid chars

        // Test empty name handling (returns default)
        let result = validator.validate_save_name("");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "DefaultSave");
    }

    #[test]
    fn test_game_state_serialization() {
        let game_state = CompleteGameState {
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
            player_count: PlayerCount::Single,
            save_timestamp: chrono::Utc::now(),
            entities_snapshot: vec![],
            player_grounded: true,
            player_crouching: false,
            player_animation_state: "idle".to_string(),
            camera_position: Vec3::ZERO,
            camera_target: Vec3::ZERO,
        };

        // Test serialization
        let json = serde_json::to_string(&game_state).unwrap();
        assert!(json.contains("100"));
        assert!(json.contains("1500"));
        assert!(json.contains("Shirou1"));

        // Test deserialization
        let deserialized: CompleteGameState = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.score, 1500);
        assert_eq!(deserialized.distance_traveled, 2500.0);
        assert_eq!(deserialized.selected_character, CharacterType::Shirou1);
    }

    #[test]
    fn test_file_operations() {
        let save_manager = SaveFileManager::default();

        // Test save directory path
        assert_eq!(save_manager.save_directory, "saves");

        // Test save file metadata
        let metadata = SaveFileMetadata {
            name: "TestSave".to_string(),
            score: 1000,
            distance: 500.0,
            play_time: 60.0,
            save_timestamp: chrono::Utc::now(),
            file_path: "test.json".to_string(),
            selected_character: CharacterType::Shirou1,
        };

        assert_eq!(metadata.name, "TestSave");
        assert_eq!(metadata.score, 1000);
    }

    #[test]
    fn test_english_text_constants() {
        use crate::systems::text_constants::SaveLoadText;

        // Test save dialog texts
        assert_eq!(SaveLoadText::SAVE_DIALOG_TITLE, "Save Game");
        assert_eq!(SaveLoadText::ENTER_SAVE_NAME, "Enter save name:");
        assert_eq!(SaveLoadText::SAVE_BUTTON, "Save");
        assert_eq!(SaveLoadText::CANCEL_BUTTON, "Cancel");

        // Test load table texts
        assert_eq!(SaveLoadText::LOAD_DIALOG_TITLE, "Load & Manage Saves");
        assert_eq!(SaveLoadText::COL_NAME, "Name");
        assert_eq!(SaveLoadText::COL_SCORE, "Score");

        // Test error messages
        assert_eq!(SaveLoadText::SAVE_ERROR, "Failed to save game");
        assert_eq!(SaveLoadText::LOAD_ERROR, "Failed to load game");
        assert_eq!(SaveLoadText::FILE_NOT_FOUND_ERROR, "Save file not found");
    }

    #[test]
    fn test_error_handling_system() {
        let mut error_manager = error_handling::ErrorRecoveryManager::new();

        // Test error message conversion
        let error = error_handling::SaveSystemError::FileNotFound("test.json".to_string());
        let message = error.to_user_message();
        assert_eq!(message, text_constants::SaveLoadText::FILE_NOT_FOUND_ERROR);

        // Test retry logic
        let action = error_manager.handle_save_error(error, "test_operation");
        assert!(
            matches!(
                action,
                error_handling::RecoveryAction::Retry
                    | error_handling::RecoveryAction::ShowError(_)
            ),
            "Unexpected recovery action: {:?}",
            action
        );
    }

    #[test]
    fn test_audio_state_management() {
        let mut audio_manager = AudioStateManager::default();

        // Test initial state
        assert!(!audio_manager.music_playing);
        assert_eq!(audio_manager.music_position, 0.0);
        assert_eq!(audio_manager.music_volume, 0.5);

        // Test state updates
        audio_manager.set_music_playing(true);
        audio_manager.set_music_position(30.5);
        audio_manager.music_volume = 0.8;

        assert!(audio_manager.music_playing);
        assert_eq!(audio_manager.music_position, 30.5);
        assert_eq!(audio_manager.music_volume, 0.8);
    }

    #[test]
    fn test_pause_manager_state_capture() {
        let mut pause_manager = PauseManager::default();

        // Test state capture
        let test_state = CompleteGameState {
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
            player_count: PlayerCount::Double,
            save_timestamp: chrono::Utc::now(),
            entities_snapshot: vec![],
            player_grounded: true,
            player_crouching: false,
            player_animation_state: "idle".to_string(),
            camera_position: Vec3::ZERO,
            camera_target: Vec3::ZERO,
        };

        pause_manager.preserved_state = Some(test_state.clone());

        // Test state retrieval
        let retrieved_state = pause_manager.preserved_state.as_ref().unwrap();
        assert_eq!(retrieved_state.score, 750);
        assert_eq!(retrieved_state.distance_traveled, 1250.0);
        assert_eq!(retrieved_state.selected_character, CharacterType::Shirou2);
    }
}
