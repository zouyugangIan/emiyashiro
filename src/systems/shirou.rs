use bevy::prelude::*;

use crate::{
    asset_paths,
    components::{AttackAnimationState, AttackAnimationStyle, Health, Player, shirou::ShroudState},
    events::{DamageEvent, DamageSource},
};

fn shift_pressed(keyboard: &ButtonInput<KeyCode>) -> bool {
    keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight)
}

/// 处理绯红圣骸布开启输入。
pub fn handle_shroud_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<
        (
            Entity,
            &mut ShroudState,
            &Health,
            Option<&mut AttackAnimationState>,
        ),
        With<Player>,
    >,
    mut damage_writer: MessageWriter<DamageEvent>,
) {
    if shift_pressed(&keyboard) && keyboard.just_pressed(KeyCode::KeyV) {
        for (player_entity, mut shroud, health, attack_animation) in query.iter_mut() {
            if health.is_dead() {
                continue;
            }

            if !shroud.try_enable_release() {
                info!("Shroud activation ignored: Overedge mode is already active");
                continue;
            }

            damage_writer.write(DamageEvent {
                target: player_entity,
                amount: shroud.activation_health_cost,
                source: DamageSource::ShroudDrain,
            });

            if let Some(mut attack_animation) = attack_animation {
                let release_duration = (asset_paths::HF_SHIROU_OVEREDGE_RELEASE_FRAME_COUNT as f32
                    + 1.0)
                    * asset_paths::HF_SHIROU_OVEREDGE_ATTACK_FRAME_DURATION_SECS;
                attack_animation
                    .trigger_with_style(release_duration, AttackAnimationStyle::OveredgeRelease);
            }

            info!(
                "Shroud released: Crimson Overedge mode activated for {:.1}s",
                ShroudState::OVEREDGE_DURATION_SECS
            );
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
