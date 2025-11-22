//! 敵人相關組件

use bevy::prelude::*;

/// 敵人標記組件
#[derive(Component, Debug)]
pub struct Enemy;

/// 敵人類型
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub enum EnemyType {
    Mushroom,  // 蘑菇敵人
}

/// 敵人狀態
#[derive(Component, Debug, Clone)]
pub struct EnemyState {
    pub health: i32,
    pub max_health: i32,
    pub is_alive: bool,
    pub patrol_left: f32,   // 巡邏左邊界
    pub patrol_right: f32,  // 巡邏右邊界
    pub move_direction: f32, // 移動方向 (-1.0 或 1.0)
}

impl Default for EnemyState {
    fn default() -> Self {
        Self {
            health: 3,
            max_health: 3,
            is_alive: true,
            patrol_left: 0.0,
            patrol_right: 200.0,
            move_direction: 1.0,
        }
    }
}

impl EnemyState {
    pub fn new(health: i32, patrol_range: f32) -> Self {
        Self {
            health,
            max_health: health,
            is_alive: true,
            patrol_left: 0.0,
            patrol_right: patrol_range,
            move_direction: 1.0,
        }
    }
    
    pub fn take_damage(&mut self, damage: i32) {
        self.health -= damage;
        if self.health <= 0 {
            self.health = 0;
            self.is_alive = false;
        }
    }
}
