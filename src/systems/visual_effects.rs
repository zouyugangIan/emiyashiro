//! 视觉效果系统
//!
//! 提供简单的视觉反馈效果，如跳跃时的缩放、着陆时的震动等

use crate::components::*;
use bevy::prelude::*;

const BASE_PLAYER_SCALE: f32 = 1.0;

type VelocityChangedPlayerQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Transform,
        &'static Velocity,
        Option<&'static VisualEffect>,
    ),
    (With<Player>, Changed<Velocity>),
>;

type PlayerStateChangedQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Transform,
        &'static PlayerState,
        Option<&'static VisualEffect>,
    ),
    (With<Player>, Changed<PlayerState>),
>;

type ButtonHoverTransformQuery<'w, 's> = Query<
    'w,
    's,
    (&'static Interaction, &'static mut Transform),
    (Changed<Interaction>, With<Button>),
>;

/// 视觉效果组件
#[derive(Component)]
pub struct VisualEffect {
    pub effect_type: EffectType,
    pub duration: f32,
    pub elapsed: f32,
    pub intensity: f32,
    pub base_scale: Option<Vec3>,
    pub translation_offset: Vec3,
}

/// 效果类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectType {
    JumpScale,    // 跳跃时的缩放效果
    LandShake,    // 着陆时的震动效果
    RunBob,       // 跑步时的上下摆动
    CrouchSquash, // 蹲下时的压扁效果
}

impl VisualEffect {
    pub fn new(effect_type: EffectType, duration: f32, intensity: f32) -> Self {
        Self {
            effect_type,
            duration,
            elapsed: 0.0,
            intensity,
            base_scale: None,
            translation_offset: Vec3::ZERO,
        }
    }

    pub fn is_finished(&self) -> bool {
        self.elapsed >= self.duration
    }

    pub fn progress(&self) -> f32 {
        (self.elapsed / self.duration).clamp(0.0, 1.0)
    }
}

fn should_trigger_effect(active_effect: Option<&VisualEffect>, next_effect: EffectType) -> bool {
    match active_effect {
        None => true,
        Some(effect) => effect.is_finished() || effect.effect_type != next_effect,
    }
}

/// 跳跃视觉效果触发器
pub fn trigger_jump_effect(mut commands: Commands, mut player_query: VelocityChangedPlayerQuery) {
    for (entity, _transform, velocity, active_effect) in player_query.iter_mut() {
        // 检测跳跃（向上的速度突然增加）
        if velocity.y > 300.0 && should_trigger_effect(active_effect, EffectType::JumpScale) {
            commands.entity(entity).insert(VisualEffect::new(
                EffectType::JumpScale,
                0.3, // 0.3秒的效果
                1.2, // 120%的缩放
            ));
        }
    }
}

/// 着陆视觉效果触发器
pub fn trigger_land_effect(mut commands: Commands, mut player_query: PlayerStateChangedQuery) {
    for (entity, _transform, player_state, active_effect) in player_query.iter_mut() {
        // 检测着陆（刚刚接触地面）
        if player_state.is_grounded && should_trigger_effect(active_effect, EffectType::LandShake) {
            commands.entity(entity).insert(VisualEffect::new(
                EffectType::LandShake,
                0.2,  // 0.2秒的震动
                0.05, // 轻微的震动强度
            ));
        }
    }
}

/// 跑步视觉效果
pub fn trigger_run_effect(
    mut commands: Commands,
    mut player_query: Query<
        (
            Entity,
            &Transform,
            &Velocity,
            &PlayerState,
            Option<&VisualEffect>,
        ),
        With<Player>,
    >,
) {
    for (entity, _transform, velocity, player_state, active_effect) in player_query.iter_mut() {
        // 检测跑步（在地面上且有水平速度）
        if player_state.is_grounded
            && velocity.x.abs() > 50.0
            && should_trigger_effect(active_effect, EffectType::RunBob)
        {
            // 检查是否已经有跑步效果
            commands.entity(entity).insert(VisualEffect::new(
                EffectType::RunBob,
                0.5,  // 0.5秒的摆动周期
                0.02, // 轻微的上下摆动
            ));
        }
    }
}

/// 蹲下视觉效果
pub fn trigger_crouch_effect(mut commands: Commands, mut player_query: PlayerStateChangedQuery) {
    for (entity, _transform, player_state, active_effect) in player_query.iter_mut() {
        // 检测蹲下状态变化
        if player_state.is_crouching
            && should_trigger_effect(active_effect, EffectType::CrouchSquash)
        {
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
        if effect.base_scale.is_none() {
            effect.base_scale = Some(transform.scale);
        }

        // 先还原上一帧应用的位移偏移，保证游戏逻辑系统仍然掌控主位移。
        transform.translation -= effect.translation_offset;
        effect.translation_offset = Vec3::ZERO;

        effect.elapsed += time.delta_secs();

        if effect.is_finished() {
            // 重置变换并移除效果
            reset_transform(&effect, &mut transform);
            commands.entity(entity).remove::<VisualEffect>();
            continue;
        }

        // 应用效果
        effect.translation_offset = apply_visual_effect(&effect, &mut transform);
        transform.translation += effect.translation_offset;
    }
}

/// 应用视觉效果到变换
fn apply_visual_effect(effect: &VisualEffect, transform: &mut Transform) -> Vec3 {
    let progress = effect.progress();
    let base_scale = effect.base_scale.unwrap_or(transform.scale);

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
            transform.scale = Vec3::new(
                base_scale.x * scale_factor * BASE_PLAYER_SCALE,
                base_scale.y * scale_factor * BASE_PLAYER_SCALE,
                base_scale.z,
            );
            Vec3::ZERO
        }

        EffectType::LandShake => {
            // 着陆时的震动效果
            let shake_intensity = effect.intensity * (1.0 - progress); // 逐渐减弱
            let shake_x = (effect.elapsed * 50.0).sin() * shake_intensity;
            let shake_y = (effect.elapsed * 60.0).cos() * shake_intensity;

            transform.scale = base_scale;
            Vec3::new(shake_x, shake_y, 0.0)
        }

        EffectType::RunBob => {
            // 跑步时的上下摆动
            let bob_offset = (effect.elapsed * 8.0).sin() * effect.intensity;
            transform.scale = base_scale;
            Vec3::new(0.0, bob_offset, 0.0)
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
            transform.scale = Vec3::new(
                base_scale.x,
                base_scale.y * squash_factor * BASE_PLAYER_SCALE,
                base_scale.z,
            );
            Vec3::ZERO
        }
    }
}

/// 重置变换到默认状态
fn reset_transform(effect: &VisualEffect, transform: &mut Transform) {
    if let Some(base_scale) = effect.base_scale {
        transform.scale = base_scale;
    }
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
pub fn button_hover_effect(mut button_query: ButtonHoverTransformQuery) {
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
        let alpha = blink.min_alpha
            + (blink.max_alpha - blink.min_alpha)
                * (0.5 + 0.5 * (time.elapsed_secs() * blink.blink_speed).sin());

        text_color.0.set_alpha(alpha);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_effect_blocks_same_effect_retrigger() {
        let active = VisualEffect::new(EffectType::JumpScale, 0.3, 1.2);
        assert!(!should_trigger_effect(Some(&active), EffectType::JumpScale));
    }

    #[test]
    fn active_effect_allows_different_effect_switch() {
        let active = VisualEffect::new(EffectType::JumpScale, 0.3, 1.2);
        assert!(should_trigger_effect(Some(&active), EffectType::RunBob));
    }

    #[test]
    fn jump_scale_stays_within_configured_bounds() {
        let mut effect = VisualEffect::new(EffectType::JumpScale, 0.3, 1.2);
        effect.base_scale = Some(Vec3::ONE);

        let mut transform = Transform::default();
        for elapsed in [0.01, 0.05, 0.09, 0.15, 0.21, 0.27] {
            effect.elapsed = elapsed;
            let _ = apply_visual_effect(&effect, &mut transform);
            assert!(
                transform.scale.x >= 1.0 - 1e-4 && transform.scale.x <= 1.2 + 1e-4,
                "jump scale out of range: {}",
                transform.scale.x
            );
        }
    }
}
