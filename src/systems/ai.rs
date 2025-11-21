use bevy::prelude::*;
use crate::components::ai::BotController;
use crate::components::player::PlayerInputState;

pub fn bot_control_system(
    mut query: Query<(&mut BotController, &mut PlayerInputState, &Transform)>,
) {
    for (mut controller, mut input, transform) in query.iter_mut() {
        // Simple patrol logic
        if transform.translation.x >= controller.patrol_end {
            controller.direction = -1.0;
        } else if transform.translation.x <= controller.patrol_start {
            controller.direction = 1.0;
        }

        input.move_x = controller.direction;
        
        // Random jump
        if rand::random::<f32>() < 0.01 {
            input.jump_pressed = true;
        }
    }
}
