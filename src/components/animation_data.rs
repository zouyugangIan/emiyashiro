//! a/src/components/animation_data.rs
use crate::components::animation::AnimationType;
use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

fn default_frame_duration() -> f32 {
    0.1
}

fn default_speed_reference() -> f32 {
    150.0
}

fn default_min_frame_duration() -> f32 {
    0.05
}

fn positive_or(value: f32, fallback: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        fallback
    }
}

/// 播放模式（2026最佳实践：显式声明循环语义，避免靠布尔字段猜测）
#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackMode {
    Loop,
    Once,
    PingPong,
}

/// Represents a single animation clip's data loaded from a config file.
#[derive(Debug, Deserialize, Clone)]
pub struct AnimationClipData {
    /// Vector of frame indices in the texture atlas.
    pub frames: Vec<usize>,
    /// Duration of each frame in seconds.
    #[serde(default = "default_frame_duration")]
    pub frame_duration: f32,
    /// Explicit playback mode for this clip.
    pub playback_mode: PlaybackMode,
    /// Whether to scale playback speed based on character horizontal velocity.
    #[serde(default)]
    pub speed_scale_by_velocity: bool,
    /// Velocity where `frame_duration` is considered baseline.
    #[serde(default = "default_speed_reference")]
    pub speed_reference: f32,
    /// Hard lower bound to avoid over-fast flickering.
    #[serde(default = "default_min_frame_duration")]
    pub min_frame_duration: f32,
}

impl AnimationClipData {
    pub fn frame_duration_for_speed(&self, horizontal_speed_abs: f32) -> f32 {
        let frame_duration = positive_or(self.frame_duration, default_frame_duration());
        let minimum = positive_or(self.min_frame_duration, default_min_frame_duration());
        if !self.speed_scale_by_velocity {
            return frame_duration.max(minimum);
        }

        let speed = if horizontal_speed_abs.is_finite() {
            horizontal_speed_abs.abs()
        } else {
            0.0
        };
        let reference = positive_or(self.speed_reference, default_speed_reference());
        let normalized = (speed / reference).clamp(0.35, 2.5);
        (frame_duration / normalized).max(minimum)
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.frames.is_empty() {
            return Err("帧列表不能为空");
        }
        if !self.frame_duration.is_finite() || self.frame_duration <= 0.0 {
            return Err("frame_duration 必须是正的有限数");
        }
        if !self.min_frame_duration.is_finite() || self.min_frame_duration <= 0.0 {
            return Err("min_frame_duration 必须是正的有限数");
        }
        if self.speed_scale_by_velocity
            && (!self.speed_reference.is_finite() || self.speed_reference <= 0.0)
        {
            return Err("速度联动开启时 speed_reference 必须是正的有限数");
        }
        Ok(())
    }
}

/// Contains all animation clips for a single character, loaded from a RON file.
/// The key of the map is the animation type (e.g., "Idle", "Running").
#[derive(Debug, Deserialize, Clone, Resource)]
pub struct CharacterAnimationData {
    #[serde(flatten)]
    pub animations: HashMap<AnimationType, AnimationClipData>,
}

/// A resource to hold all loaded character animation data, keyed by a character identifier string.
#[derive(Debug, Default, Clone, Resource)]
pub struct AnimationDataMap(pub HashMap<String, CharacterAnimationData>);
