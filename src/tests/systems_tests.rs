#[cfg(test)]
mod tests {
    use crate::{components::*, resources::*, states::*};
    use bevy::prelude::*;

    fn create_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<GameStats>()
            .init_resource::<CharacterSelection>()
            .init_resource::<AudioSettings>();
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
}