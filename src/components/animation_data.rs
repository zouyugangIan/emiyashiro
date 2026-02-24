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
    pub fn playback_mode(&self) -> PlaybackMode {
        self.playback_mode
    }

    pub fn frame_duration_for_speed(&self, horizontal_speed_abs: f32) -> f32 {
        if !self.speed_scale_by_velocity {
            return self.frame_duration.max(self.min_frame_duration);
        }

        let normalized = (horizontal_speed_abs / self.speed_reference.max(1.0)).clamp(0.35, 2.5);
        (self.frame_duration / normalized).max(self.min_frame_duration)
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
