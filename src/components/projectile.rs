//! 投射物（法波、子弹等）组件

use bevy::prelude::*;

/// 投射物标记组件
#[derive(Component, Debug)]
pub struct Projectile;

/// 投射物类型
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub enum ProjectileType {
    MagicWave, // 法波
    Fireball,  // 火球
    Overedge,  // 鹤翼三连·Overedge
}

/// 投射物数据
#[derive(Component, Debug, Clone)]
pub struct ProjectileData {
    pub damage: i32,
    pub speed: f32,
    pub lifetime: f32, // 存活时间（秒）
    pub elapsed: f32,  // 已经过时间
}

impl ProjectileData {
    pub fn new(damage: i32, speed: f32, lifetime: f32) -> Self {
        Self {
            damage,
            speed,
            lifetime,
            elapsed: 0.0,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.elapsed >= self.lifetime
    }
}
