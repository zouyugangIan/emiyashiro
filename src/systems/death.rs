//! 死亡系統 - 掉入谷底、重生

use bevy::prelude::*;
use crate::components::*;
use crate::resources::GameConfig;
use crate::states::GameState;

const DEATH_ZONE_Y: f32 = -400.0; // 死亡線

/// 檢測玩家是否掉入谷底
pub fn check_player_fall_death(
    player_query: Query<&Transform, With<Player>>,
    _next_state: ResMut<NextState<GameState>>,
) {
    if let Some(transform) = player_query.iter().next() {
        if transform.translation.y < DEATH_ZONE_Y {
            println!("💀 玩家掉入谷底！");
            // TODO: 顯示死亡畫面或重生
            // next_state.set(GameState::GameOver);
        }
    }
}

/// 玩家重生系統
pub fn respawn_player(
    mut player_query: Query<&mut Transform, With<Player>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    // R 鍵重生
    if keyboard.just_pressed(KeyCode::KeyR) {
        if let Some(mut transform) = player_query.iter_mut().next() {
            if transform.translation.y < DEATH_ZONE_Y {
                transform.translation = GameConfig::PLAYER_START_POS;
                println!("🔄 玩家重生！");
            }
        }
    }
}
