use bevy::prelude::*;

use crate::{
    components::{Health, Player, shirou::ShroudState},
    events::{DamageEvent, DamageSource},
};

/// 处理圣骸布开关输入。
pub fn handle_shroud_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut ShroudState, With<Player>>,
) {
    if keyboard.just_pressed(KeyCode::KeyK) {
        for mut shroud in query.iter_mut() {
            let released = shroud.toggle();
            if released {
                info!("Shroud released: Overedge mode activated");
            } else {
                info!("Shroud sealed: normal mode restored");
            }
        }
    }
}

/// 圣骸布持续扣血效果：仅发伤害事件，统一进入伤害结算系统。
pub fn shroud_health_drain(
    time: Res<Time>,
    mut query: Query<(Entity, &mut ShroudState, &Health), With<Player>>,
    mut damage_writer: MessageWriter<DamageEvent>,
) {
    for (player_entity, mut shroud, health) in query.iter_mut() {
        if !shroud.is_released || health.is_dead() {
            continue;
        }

        shroud.health_drain_timer.tick(time.delta());
        if shroud.health_drain_timer.just_finished() {
            damage_writer.write(DamageEvent {
                target: player_entity,
                amount: shroud.health_drain_amount,
                source: DamageSource::ShroudDrain,
            });
        }
    }
}
