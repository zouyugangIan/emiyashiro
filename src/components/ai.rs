use bevy::prelude::*;

#[derive(Component)]
pub struct BotController {
    pub patrol_start: f32,
    pub patrol_end: f32,
    pub direction: f32,
}

impl Default for BotController {
    fn default() -> Self {
        Self {
            patrol_start: 0.0,
            patrol_end: 500.0,
            direction: 1.0,
        }
    }
}
