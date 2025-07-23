use bevy::prelude::*;
use crate::{
    components::*,
    resources::*,
};

/// 摄像机跟随系统
pub fn camera_follow(
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    player_query: Query<&Transform, (With<Player>, Without<Camera>)>,
    time: Res<Time>,
) {
    for mut camera_transform in camera_query.iter_mut() {
        if let Ok(player_transform) = player_query.single() {
            // 摄像机平滑跟随玩家
            let target_x = player_transform.translation.x + GameConfig::CAMERA_OFFSET;
            camera_transform.translation.x += 
                (target_x - camera_transform.translation.x) * GameConfig::CAMERA_FOLLOW_SPEED * time.delta_secs();
            
            // 调试信息
            if time.elapsed_secs() as u32 % 2 == 0 {
                println!("摄像机位置: {:.1}, 玩家位置: {:.1}", 
                    camera_transform.translation.x, 
                    player_transform.translation.x);
            }
        } else {
            // 没有玩家时摄像机缓慢移动
            camera_transform.translation.x += GameConfig::CAMERA_IDLE_SPEED * time.delta_secs();
        }
    }
}