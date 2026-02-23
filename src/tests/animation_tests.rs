//! 角色动画系统测试（数据驱动版本）

#[cfg(test)]
mod tests {
    use crate::components::{
        animation::AnimationType,
        animation_data::{CharacterAnimationData, PlaybackMode},
    };
    use crate::systems::sprite_animation::frame_count_guideline;
    use std::fs;

    fn load_profile(file_name: &str) -> CharacterAnimationData {
        let path = format!("assets/animations/{}", file_name);
        let content = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("Failed to read '{}': {}", path, error));
        ron::from_str(&content)
            .unwrap_or_else(|error| panic!("Failed to parse '{}': {}", path, error))
    }

    #[test]
    fn test_legacy_frame_arrays_still_available() {
        assert!(!crate::asset_paths::SHIROU_ANIMATION_FRAMES.is_empty());
        assert!(!crate::asset_paths::SAKURA_ANIMATION_FRAMES.is_empty());
    }

    #[test]
    fn test_animation_profiles_can_parse() {
        let _ = load_profile("hf_shirou.ron");
        let _ = load_profile("shirou.ron");
        let _ = load_profile("sakura.ron");
    }

    #[test]
    fn test_animation_profiles_have_core_clips() {
        for file in ["hf_shirou.ron", "shirou.ron", "sakura.ron"] {
            let profile = load_profile(file);
            assert!(profile.animations.contains_key(&AnimationType::Idle));
            assert!(profile.animations.contains_key(&AnimationType::Running));
            assert!(profile.animations.contains_key(&AnimationType::Jumping));
        }
    }

    #[test]
    fn test_clip_frame_counts_match_guideline() {
        let profile = load_profile("hf_shirou.ron");

        for (animation_type, clip) in &profile.animations {
            let (minimum, _) = frame_count_guideline(animation_type);
            assert!(
                clip.frames.len() >= minimum,
                "{:?} should have at least {} frames, got {}",
                animation_type,
                minimum,
                clip.frames.len()
            );
        }
    }

    #[test]
    fn test_running_clip_uses_modern_playback_mode() {
        let profile = load_profile("hf_shirou.ron");
        let running_clip = profile
            .animations
            .get(&AnimationType::Running)
            .expect("Running clip should exist");

        assert_eq!(running_clip.playback_mode(), PlaybackMode::Loop);
        assert!(running_clip.speed_scale_by_velocity);
        assert!(running_clip.speed_reference > 0.0);
        assert!(running_clip.min_frame_duration > 0.0);
    }

    #[test]
    fn test_hf_shirou_has_attacking_clip() {
        let profile = load_profile("hf_shirou.ron");
        let attack_clip = profile
            .animations
            .get(&AnimationType::Attacking)
            .expect("Attacking clip should exist");

        assert_eq!(attack_clip.playback_mode(), PlaybackMode::Once);
        assert!(!attack_clip.frames.is_empty());
    }
}
