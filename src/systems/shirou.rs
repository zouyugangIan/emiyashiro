use bevy::prelude::*;

use crate::{
    components::{Health, Player, shirou::ShroudState},
    events::{DamageEvent, DamageSource},
};

/// 处理圣骸布开关输入。
pub fn handle_shroud_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(Entity, &mut ShroudState, &Health), With<Player>>,
    mut damage_writer: MessageWriter<DamageEvent>,
) {
    if keyboard.just_pressed(KeyCode::KeyK) {
        for (player_entity, mut shroud, health) in query.iter_mut() {
            if health.is_dead() {
                continue;
            }

            let released = shroud.toggle();
            damage_writer.write(DamageEvent {
                target: player_entity,
                amount: shroud.toggle_health_cost,
                source: DamageSource::ShroudDrain,
            });

            if released {
                info!(
                    "Shroud released: Overedge mode activated for {:.1}s",
                    ShroudState::OVEREDGE_DURATION_SECS
                );
            } else {
                info!("Shroud sealed: normal mode restored");
            }
        }
    }
}

/// 圣骸布状态更新：维护临时解放计时，超时自动恢复。
pub fn shroud_health_drain(time: Res<Time>, mut query: Query<&mut ShroudState, With<Player>>) {
    for mut shroud in query.iter_mut() {
        if !shroud.is_released {
            continue;
        }

        if shroud.tick(time.delta()) {
            info!("Shroud timeout reached: reverted to normal projectiles");
        }
    }
}
