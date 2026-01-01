use bevy::prelude::*;
use crate::components::*;
use crate::components::shirou::ShroudState;
// Note: We might need to import Player or PlayerState if we modify them, 
// but currently we just fetch Entity/Health.
// Assuming EnemyState has 'health', maybe Player needs a Health component too?
// Let's check player.rs first or assume we add one. 
// Wait, I didn't see a Health component on Player in previous exploration, only PlayerState.
// I should double check if Player has health, otherwise I need to add it.
// Going to assume I need to handle health myself or add it.
// For now, let's create a stand-alone system and I will verify Player health in the next step.

/// 处理圣骸布开关输入
pub fn handle_shroud_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut ShroudState, With<Player>>,
) {
    if keyboard.just_pressed(KeyCode::KeyK) {
        for mut shroud in query.iter_mut() {
            let released = shroud.toggle();
            if released {
                println!("🔴 圣骸布解放！Overedge Mode Activated! (HP Drain Start)");
            } else {
                println!("⚪ 圣骸布封印。Return to Normal.");
            }
        }
    }
}

/// 圣骸布持续扣血效果
pub fn shroud_health_drain(
    time: Res<Time>,
    mut query: Query<(&mut ShroudState, &mut Health), With<Player>>,
) {
    for (mut shroud, mut health) in query.iter_mut() {
        if shroud.is_released {
            shroud.health_drain_timer.tick(time.delta());
            
            if shroud.health_drain_timer.just_finished() {
                // 扣除生命值
                if !health.is_dead() {
                    health.take_damage(shroud.health_drain_amount);
                    println!("🩸 侵蚀伤害: -{} (当前 HP: {:.1}/{:.1})", 
                        shroud.health_drain_amount, health.current, health.max);
                    
                    if health.is_dead() {
                        // 可能会触发死亡逻辑，交由 death system 处理
                        println!("💀 身体崩坏...");
                    }
                }
            }
        }
    }
}
