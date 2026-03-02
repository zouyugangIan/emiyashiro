//! 敵人相關組件

use bevy::prelude::*;

/// 敵人標記組件
#[derive(Component, Debug)]
pub struct Enemy;

/// 敵人類型
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub enum EnemyType {
    Slime,             // 史莱姆敌人
    Familiar,          // 使魔
    EnemyHeroicSpirit, // 敌方英灵
}

/// 敵人狀態
#[derive(Component, Debug, Clone)]
pub struct EnemyState {
    pub health: i32,
    pub max_health: i32,
    pub is_alive: bool,
    pub patrol_left: f32,    // 相对生成点的巡邏左邊界
    pub patrol_right: f32,   // 相对生成点的巡邏右邊界
    pub move_direction: f32, // 移動方向 (-1.0 或 1.0)
    pub spawn_origin_x: f32, // 生成锚点，用于稳定巡逻
    pub base_speed: f32,     // 基础移动速度
    pub contact_damage: f32, // 接触伤害
    pub hover_phase: f32,    // 浮空类敌人的相位
    pub attack_cooldown: f32,
    pub ranged_cooldown: f32,
    pub ranged_windup_timer: f32,
    pub pending_ranged_shot: bool,
    pub ranged_shot_direction: Vec2,
    pub hit_stun_timer: f32,
    pub dash_charge_timer: f32,
    pub dash_active_timer: f32,
    pub dash_direction: f32,
}

impl Default for EnemyState {
    fn default() -> Self {
        Self {
            health: 3,
            max_health: 3,
            is_alive: true,
            patrol_left: -100.0,
            patrol_right: 100.0,
            move_direction: 1.0,
            spawn_origin_x: 0.0,
            base_speed: 55.0,
            contact_damage: 12.0,
            hover_phase: 0.0,
            attack_cooldown: 0.0,
            ranged_cooldown: 0.0,
            ranged_windup_timer: 0.0,
            pending_ranged_shot: false,
            ranged_shot_direction: Vec2::ZERO,
            hit_stun_timer: 0.0,
            dash_charge_timer: 0.0,
            dash_active_timer: 0.0,
            dash_direction: 1.0,
        }
    }
}

impl EnemyState {
    pub fn new(health: i32, patrol_range: f32) -> Self {
        let half_range = patrol_range.max(40.0) * 0.5;
        Self {
            health,
            max_health: health,
            is_alive: true,
            patrol_left: -half_range,
            patrol_right: half_range,
            move_direction: 1.0,
            spawn_origin_x: 0.0,
            base_speed: 55.0,
            contact_damage: 12.0,
            hover_phase: 0.0,
            attack_cooldown: 0.0,
            ranged_cooldown: 0.0,
            ranged_windup_timer: 0.0,
            pending_ranged_shot: false,
            ranged_shot_direction: Vec2::ZERO,
            hit_stun_timer: 0.0,
            dash_charge_timer: 0.0,
            dash_active_timer: 0.0,
            dash_direction: 1.0,
        }
    }

    pub fn with_spawn_origin(mut self, spawn_origin_x: f32) -> Self {
        self.spawn_origin_x = spawn_origin_x;
        self
    }

    pub fn with_movement(mut self, base_speed: f32, contact_damage: f32, hover_phase: f32) -> Self {
        self.base_speed = base_speed;
        self.contact_damage = contact_damage;
        self.hover_phase = hover_phase;
        self
    }

    pub fn patrol_world_bounds(&self) -> (f32, f32) {
        (
            self.spawn_origin_x + self.patrol_left,
            self.spawn_origin_x + self.patrol_right,
        )
    }

    pub fn take_damage(&mut self, damage: i32) {
        self.health -= damage;
        if self.health <= 0 {
            self.health = 0;
            self.is_alive = false;
        }
    }

    pub fn tick_timers(&mut self, delta_secs: f32) {
        self.attack_cooldown = (self.attack_cooldown - delta_secs).max(0.0);
        self.ranged_cooldown = (self.ranged_cooldown - delta_secs).max(0.0);
        self.ranged_windup_timer = (self.ranged_windup_timer - delta_secs).max(0.0);
        self.hit_stun_timer = (self.hit_stun_timer - delta_secs).max(0.0);
        self.dash_charge_timer = (self.dash_charge_timer - delta_secs).max(0.0);
        self.dash_active_timer = (self.dash_active_timer - delta_secs).max(0.0);
    }

    pub fn apply_hit_stun(&mut self, duration: f32) {
        self.hit_stun_timer = self.hit_stun_timer.max(duration);
        self.dash_charge_timer = 0.0;
        self.dash_active_timer = 0.0;
        self.ranged_windup_timer = 0.0;
        self.pending_ranged_shot = false;
        self.ranged_shot_direction = Vec2::ZERO;
    }
}
