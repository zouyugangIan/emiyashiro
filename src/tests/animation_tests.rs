//! HF 士郎数据驱动动画的资源级测试。

#[cfg(test)]
mod tests {
    use crate::components::{
        animation::AnimationType,
        animation_data::{CharacterAnimationData, PlaybackMode},
    };
    use crate::systems::sprite_animation::frame_count_guideline;

    fn load_profile() -> CharacterAnimationData {
        ron::from_str(include_str!("../../assets/animations/hf_shirou.ron"))
            .unwrap_or_else(|error| panic!("HF Shirou profile should parse: {error}"))
    }

    #[test]
    fn animation_profile_can_parse() {
        let _ = load_profile();
    }

    #[test]
    fn animation_profile_has_every_runtime_clip() {
        let profile = load_profile();
        for animation_type in [
            AnimationType::Idle,
            AnimationType::Running,
            AnimationType::Attacking,
            AnimationType::Jumping,
            AnimationType::Crouching,
            AnimationType::Landing,
        ] {
            let clip = profile
                .animations
                .get(&animation_type)
                .unwrap_or_else(|| panic!("missing {animation_type:?}"));
            clip.validate()
                .unwrap_or_else(|reason| panic!("invalid {animation_type:?}: {reason}"));
        }
    }

    #[test]
    fn clip_frame_counts_match_guideline() {
        let profile = load_profile();
        for (animation_type, clip) in &profile.animations {
            let (minimum, _) = frame_count_guideline(animation_type);
            assert!(
                clip.frames.len() >= minimum,
                "{animation_type:?} should have at least {minimum} frames, got {}",
                clip.frames.len()
            );
        }
    }

    #[test]
    fn running_clip_scales_with_velocity() {
        let profile = load_profile();
        let running = profile
            .animations
            .get(&AnimationType::Running)
            .expect("Running clip should exist");

        assert_eq!(running.playback_mode, PlaybackMode::Loop);
        assert!(running.speed_scale_by_velocity);
        assert!(running.frame_duration_for_speed(300.0) < running.frame_duration_for_speed(30.0));
    }

    #[test]
    fn crouch_uses_settle_pose_frames() {
        let profile = load_profile();
        let crouch = profile
            .animations
            .get(&AnimationType::Crouching)
            .expect("Crouching clip should exist");

        assert_eq!(crouch.frames, vec![6, 7]);
        assert_eq!(crouch.playback_mode, PlaybackMode::Once);
    }
}
