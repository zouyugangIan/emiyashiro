//! a/src/components/animation_data.rs
use crate::components::animation::AnimationType;
use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

/// Represents a single animation clip's data loaded from a config file.
#[derive(Debug, Deserialize, Clone)]
pub struct AnimationClipData {
    /// Vector of frame indices in the texture atlas.
    pub frames: Vec<usize>,
    /// Duration of each frame in seconds.
    pub frame_duration: f32,
    /// Whether the animation should loop.
    pub looping: bool,
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
