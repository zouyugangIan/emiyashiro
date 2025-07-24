#[cfg(test)]
mod tests {
    use crate::components::*;
    use bevy::prelude::*;

    #[test]
    fn test_velocity_creation() {
        let velocity = Velocity::new(10.0, 20.0);
        assert_eq!(velocity.x, 10.0);
        assert_eq!(velocity.y, 20.0);
    }

    #[test]
    fn test_velocity_zero() {
        let velocity = Velocity::zero();
        assert_eq!(velocity.x, 0.0);
        assert_eq!(velocity.y, 0.0);
    }

    #[test]
    fn test_velocity_length() {
        let velocity = Velocity::new(3.0, 4.0);
        assert_eq!(velocity.length(), 5.0); // 3-4-5 triangle
    }

    #[test]
    fn test_player_state_default() {
        let state = PlayerState::default();
        assert!(state.is_grounded);
        assert!(!state.is_crouching);
    }

    #[test]
    fn test_player_state_can_jump() {
        let mut state = PlayerState::default();
        
        // Can jump when grounded and not crouching
        assert!(state.can_jump());
        
        // Cannot jump when not grounded
        state.is_grounded = false;
        assert!(!state.can_jump());
        
        // Cannot jump when crouching
        state.is_grounded = true;
        state.is_crouching = true;
        assert!(!state.can_jump());
    }

    #[test]
    fn test_player_state_new() {
        let state = PlayerState::new(false, true);
        assert!(!state.is_grounded);
        assert!(state.is_crouching);
    }

    #[test]
    fn test_cover_fade_state_default() {
        let fade_state = CoverFadeState::default();
        assert_eq!(fade_state.alpha, 1.0);
        assert_eq!(fade_state.fade_direction, -1.0);
    }

    #[test]
    fn test_cover_fade_state_new() {
        let fade_state = CoverFadeState::new(0.5, 1.0);
        assert_eq!(fade_state.alpha, 0.5);
        assert_eq!(fade_state.fade_direction, 1.0);
    }

    #[test]
    fn test_sound_type_variants() {
        let jump_sound = SoundType::Jump;
        let land_sound = SoundType::Land;
        let footstep_sound = SoundType::Footstep;
        
        // Test that variants can be created and compared
        assert_eq!(jump_sound, SoundType::Jump);
        assert_eq!(land_sound, SoundType::Land);
        assert_eq!(footstep_sound, SoundType::Footstep);
        
        // Test that different variants are not equal
        assert_ne!(jump_sound, land_sound);
        assert_ne!(land_sound, footstep_sound);
    }

    #[test]
    fn test_audio_trigger_creation() {
        let trigger = AudioTrigger {
            sound_type: SoundType::Jump,
            should_play: true,
        };
        
        assert_eq!(trigger.sound_type, SoundType::Jump);
        assert!(trigger.should_play);
    }
}