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
            .init_resource::<SaveFileManager>()
            .init_resource::<AudioStateManager>()
            .insert_resource(crate::systems::text_input::TextInputState::new(25));
        app
    }

    #[test]
    fn test_game_state_transitions() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).init_state::<GameState>();

        // Test initial state
        let state = app.world().resource::<State<GameState>>();
        assert_eq!(*state.get(), GameState::Menu);

        // Test state transition
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update();

        let state = app.world().resource::<State<GameState>>();
        assert_eq!(*state.get(), GameState::Playing);
    }

    #[test]
    fn test_player_spawn_and_movement() {
        let mut app = create_full_test_app();

        // Spawn player
        let player_entity = app
            .world_mut()
            .spawn((
                Player,
                Transform::from_translation(GameConfig::PLAYER_START_POS),
                Velocity::zero(),
                PlayerState::default(),
            ))
            .id();

        // Simulate input (this would normally come from keyboard)
        // For testing, we'll manually modify velocity
        {
            let mut world = app.world_mut();
            let mut entity = world.entity_mut(player_entity);
            entity.get_mut::<Velocity>().unwrap().x = GameConfig::MOVE_SPEED;
        }

        // Run several updates
        for _ in 0..10 {
            app.update();
        }

        // Check that player moved
        let entity = app.world().get_entity(player_entity).unwrap();
        let transform = entity.get::<Transform>().unwrap();
        assert!(transform.translation.x > GameConfig::PLAYER_START_POS.x);
    }

    #[test]
    fn test_game_stats_tracking() {
        let mut app = create_full_test_app();

        // Spawn player
        let player_entity = app
            .world_mut()
            .spawn((
                Player,
                Transform::from_translation(GameConfig::PLAYER_START_POS),
                Velocity::zero(),
                PlayerState::default(),
            ))
            .id();

        // Move player forward
        {
            let mut world = app.world_mut();
            let mut entity = world.entity_mut(player_entity);
            entity.get_mut::<Transform>().unwrap().translation.x += 100.0;
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
        let player_entity = app
            .world_mut()
            .spawn((
                Player,
                Transform::from_translation(Vec3::new(0.0, 100.0, 0.0)),
                Velocity::zero(),
                PlayerState::new(false, false), // Not grounded initially
            ))
            .id();

        // Run update
        app.update();

        // Check that player state reflects being in air
        let entity = app.world().get_entity(player_entity).unwrap();
        let player_state = entity.get::<PlayerState>().unwrap();
        assert!(!player_state.is_grounded);

        // Move player to ground level
        app.world_mut()
            .entity_mut(player_entity)
            .get_mut::<Transform>()
            .unwrap()
            .translation
            .y = GameConfig::GROUND_LEVEL;

        // Run update
        app.update();

        // Check that player is now grounded
        let entity = app.world().get_entity(player_entity).unwrap();
        let player_state = entity.get::<PlayerState>().unwrap();
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
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update();

        // 创建玩家实体
        let player_entity = app
            .world_mut()
            .spawn((
                Player,
                Transform::from_translation(Vec3::new(100.0, 0.0, 0.0)),
                Velocity::new(50.0, 0.0),
                PlayerState::default(),
            ))
            .id();

        // 运行几帧游戏
        for _ in 0..5 {
            app.update();
        }

        // 模拟暂停输入
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Paused);
        app.update();

        // 验证游戏已暂停
        let state = app.world().resource::<State<GameState>>();
        assert_eq!(*state.get(), GameState::Paused);

        // 模拟恢复
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update();

        // 验证游戏已恢复
        let state = app.world().resource::<State<GameState>>();
        assert_eq!(*state.get(), GameState::Playing);

        // 验证玩家实体仍然存在
        assert!(app.world().get_entity(player_entity).is_ok());
    }

    #[test]
    fn test_main_menu_load_game_workflow() {
        let mut app = create_full_test_app();

        // 开始在主菜单
        let state = app.world().resource::<State<GameState>>();
        assert_eq!(*state.get(), GameState::Menu);

        // 模拟点击加载按钮
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::LoadTable);
        app.update();

        // 验证进入加载表格状态
        let state = app.world().resource::<State<GameState>>();
        assert_eq!(*state.get(), GameState::LoadTable);

        // 模拟选择存档并加载
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update();

        // 验证进入游戏状态
        let state = app.world().resource::<State<GameState>>();
        assert_eq!(*state.get(), GameState::Playing);
    }

    #[test]
    fn test_save_dialog_workflow() {
        let mut app = create_full_test_app();

        // 进入保存对话框状态
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::SaveDialog);
        app.update();

        // 验证状态转换
        let state = app.world().resource::<State<GameState>>();
        assert_eq!(*state.get(), GameState::SaveDialog);
    }
}
