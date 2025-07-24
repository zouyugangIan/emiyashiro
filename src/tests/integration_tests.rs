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
}