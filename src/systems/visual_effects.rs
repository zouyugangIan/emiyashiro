//! 视觉效果系统
//! 
//! 提供简单的视觉反馈效果，如跳跃时的缩放、着陆时的震动等

use bevy::prelude::*;
use crate::{
    components::*,
    resources::*,
};

/// 视觉效果组件
#[derive(Component)]
pub struct VisualEffect {
    pub effect_type: EffectType,
    pub duration: f32,
    pub elapsed: f32,
    pub intensity: f32,
}

/// 效果类型
#[derive(Debug, Clone)]
pub enum EffectType {
    JumpScale,      // 跳跃时的缩放效果
    LandShake,      // 着陆时的震动效果
    RunBob,         // 跑步时的上下摆动
    CrouchSquash,   // 蹲下时的压扁效果
}

impl VisualEffect {
    pub fn new(effect_type: EffectType, duration: f32, intensity: f32) -> Self {
        Self {
            effect_type,
            duration,
            elapsed: 0.0,
            intensity,
        }
    }
    
    pub fn is_finished(&self) -> bool {
        self.elapsed >= self.duration
    }
    
    pub fn progress(&self) -> f32 {
        (self.elapsed / self.duration).clamp(0.0, 1.0)
    }
}

/// 跳跃视觉效果触发器
pub fn trigger_jump_effect(
    mut commands: Commands,
    mut player_query: Query<(Entity, &Transform, &Velocity), (With<Player>, Changed<Velocity>)>,
) {
    for (entity, _transform, velocity) in player_query.iter_mut() {
        // 检测跳跃（向上的速度突然增加）
        if velocity.y > 300.0 {
            commands.entity(entity).insert(VisualEffect::new(
                EffectType::JumpScale,
                0.3, // 0.3秒的效果
                1.2, // 120%的缩放
            ));
        }
    }
}

/// 着陆视觉效果触发器
pub fn trigger_land_effect(
    mut commands: Commands,
    mut player_query: Query<(Entity, &Transform, &PlayerState), (With<Player>, Changed<PlayerState>)>,
) {
    for (entity, _transform, player_state) in player_query.iter_mut() {
        // 检测着陆（刚刚接触地面）
        if player_state.is_grounded {
            commands.entity(entity).insert(VisualEffect::new(
                EffectType::LandShake,
                0.2, // 0.2秒的震动
                0.05, // 轻微的震动强度
            ));
        }
    }
}

/// 跑步视觉效果
pub fn trigger_run_effect(
    mut commands: Commands,
    mut player_query: Query<(Entity, &Transform, &Velocity, &PlayerState), With<Player>>,
) {
    for (entity, _transform, velocity, player_state) in player_query.iter_mut() {
        // 检测跑步（在地面上且有水平速度）
        if player_state.is_grounded && velocity.x.abs() > 50.0 {
            // 检查是否已经有跑步效果
            commands.entity(entity).insert(VisualEffect::new(
                EffectType::RunBob,
                0.5, // 0.5秒的摆动周期
                0.02, // 轻微的上下摆动
            ));
        }
    }
}

/// 蹲下视觉效果
pub fn trigger_crouch_effect(
    mut commands: Commands,
    mut player_query: Query<(Entity, &Transform, &PlayerState), (With<Player>, Changed<PlayerState>)>,
) {
    for (entity, _transform, player_state) in player_query.iter_mut() {
        // 检测蹲下状态变化
        if player_state.is_crouching {
            commands.entity(entity).insert(VisualEffect::new(
                EffectType::CrouchSquash,
                0.2, // 0.2秒的压扁效果
                0.8, // 80%的高度
            ));
        }
    }
}

/// 更新视觉效果
pub fn update_visual_effects(
    mut commands: Commands,
    mut effect_query: Query<(Entity, &mut VisualEffect, &mut Transform), With<Player>>,
    time: Res<Time>,
) {
    for (entity, mut effect, mut transform) in effect_query.iter_mut() {
        effect.elapsed += time.delta_secs();
        
        if effect.is_finished() {
            // 重置变换并移除效果
            reset_transform(&mut transform);
            commands.entity(entity).remove::<VisualEffect>();
            continue;
        }
        
        // 应用效果
        apply_visual_effect(&effect, &mut transform);
    }
}

/// 应用视觉效果到变换
fn apply_visual_effect(effect: &VisualEffect, transform: &mut Transform) {
    let progress = effect.progress();
    
    match effect.effect_type {
        EffectType::JumpScale => {
            // 跳跃时的缩放效果（快速放大然后恢复）
            let scale_factor = if progress < 0.3 {
                // 前30%时间快速放大
                1.0 + (effect.intensity - 1.0) * (progress / 0.3)
            } else {
                // 后70%时间恢复正常
                effect.intensity - (effect.intensity - 1.0) * ((progress - 0.3) / 0.7)
            };
            transform.scale = Vec3::new(scale_factor * 0.2, scale_factor * 0.2, 1.0);
        }
        
        EffectType::LandShake => {
            // 着陆时的震动效果
            let shake_intensity = effect.intensity * (1.0 - progress); // 逐渐减弱
            let shake_x = (effect.elapsed * 50.0).sin() * shake_intensity;
            let shake_y = (effect.elapsed * 60.0).cos() * shake_intensity;
            
            // 在原始位置基础上添加震动
            transform.translation.x += shake_x;
            transform.translation.y += shake_y;
        }
        
        EffectType::RunBob => {
            // 跑步时的上下摆动
            let bob_offset = (effect.elapsed * 8.0).sin() * effect.intensity;
            transform.translation.y += bob_offset;
        }
        
        EffectType::CrouchSquash => {
            // 蹲下时的压扁效果
            let squash_factor = if progress < 0.5 {
                // 前50%时间压扁
                1.0 - (1.0 - effect.intensity) * (progress / 0.5)
            } else {
                // 后50%时间恢复
                effect.intensity + (1.0 - effect.intensity) * ((progress - 0.5) / 0.5)
            };
            transform.scale.y = squash_factor * 0.2; // 保持与原始缩放一致
        }
    }
}

/// 重置变换到默认状态
fn reset_transform(transform: &mut Transform) {
    // 重置缩放到默认的角色缩放
    transform.scale = Vec3::new(0.2, 0.2, 1.0);
    // 注意：不重置位置，因为位置由其他系统管理
}

/// 清理过期的视觉效果
pub fn cleanup_visual_effects(
    mut commands: Commands,
    effect_query: Query<(Entity, &VisualEffect)>,
) {
    for (entity, effect) in effect_query.iter() {
        if effect.is_finished() {
            commands.entity(entity).remove::<VisualEffect>();
        }
    }
}

/// 简单的UI反馈效果
pub fn button_hover_effect(
    mut button_query: Query<(&Interaction, &mut Transform), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, mut transform) in button_query.iter_mut() {
        match *interaction {
            Interaction::Hovered => {
                transform.scale = Vec3::splat(1.05); // 轻微放大
            }
            Interaction::Pressed => {
                transform.scale = Vec3::splat(0.95); // 轻微缩小
            }
            Interaction::None => {
                transform.scale = Vec3::splat(1.0); // 恢复正常
            }
        }
    }
}

/// 文本闪烁效果
#[derive(Component)]
pub struct BlinkingText {
    pub blink_speed: f32,
    pub min_alpha: f32,
    pub max_alpha: f32,
}

impl Default for BlinkingText {
    fn default() -> Self {
        Self {
            blink_speed: 2.0,
            min_alpha: 0.3,
            max_alpha: 1.0,
        }
    }
}

/// 更新闪烁文本
pub fn update_blinking_text(
    mut text_query: Query<(&mut TextColor, &BlinkingText)>,
    time: Res<Time>,
) {
    for (mut text_color, blink) in text_query.iter_mut() {
        let alpha = blink.min_alpha + 
            (blink.max_alpha - blink.min_alpha) * 
            (0.5 + 0.5 * (time.elapsed_secs() * blink.blink_speed).sin());
        
        text_color.0.set_alpha(alpha);
    }
}