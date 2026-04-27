//! 战斗系统 - 攻击、投射物、命中与伤害结算

use crate::{
    asset_paths,
    components::*,
    events::{CameraImpulseEvent, DamageEvent, DamageSource},
    resources::GameplayTuning,
    states::GameState,
};
use bevy::prelude::*;

const PLAYER_CONTACT_DAMAGE_COOLDOWN: f32 = 1.0;
const PROJECTILE_MUZZLE_X_OFFSET: f32 = 54.0;
const PROJECTILE_MUZZLE_Y_OFFSET: f32 = 18.0;

const ENEMY_PROJECTILE_RENDER_SIZE: Vec2 = Vec2::new(16.0, 16.0);
const ENEMY_PROJECTILE_COLLISION_SIZE: Vec2 = Vec2::new(16.0, 16.0);

#[derive(Clone, Copy)]
struct ProjectileConfig {
    projectile_type: ProjectileType,
    damage: i32,
    speed: f32,
    lifetime: f32,
    collision_size: Vec2,
    core_size: Vec2,
    core_color: Color,
    aura_size: Vec2,
    aura_color: Color,
    accent_size: Vec2,
    accent_color: Color,
    aura_offset: Vec3,
    accent_offset: Vec3,
    initial_rotation: f32,
    accent_rotation: f32,
    cooldown: f32,
    pulse_speed: f32,
    pulse_amount: f32,
    spin_speed: f32,
}

#[derive(Clone, Copy)]
struct KnifeAttackPreset {
    damage: f32,
    cooldown: f32,
    windup_secs: f32,
    animation_duration_secs: f32,
    lifetime: f32,
    hitbox_size: Vec2,
    x_offset: f32,
    y_offset: f32,
    crouch_y_offset: f32,
    slash_render_size: Vec2,
    slash_color: Color,
    knockback_x: f32,
    knockback_y: f32,
    hit_stop_secs: f32,
}

#[derive(Default)]
pub struct KnifeComboRuntime {
    cooldown: f32,
    combo_step: u8,
    combo_reset_timer: f32,
    queued_attack: Option<AttackAnimationStyle>,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct ProjectileVisualMotion {
    base_scale: Vec3,
    pulse_speed: f32,
    pulse_amount: f32,
    spin_speed: f32,
}

#[derive(Component, Debug)]
pub struct KnifeSlash {
    pub damage: f32,
    pub lifetime: Timer,
    pub combo_step: u8,
    pub knockback_x: f32,
    pub knockback_y: f32,
    pub hit_stop_secs: f32,
}

#[derive(Component, Debug)]
pub struct PendingKnifeAttack {
    owner: Entity,
    timer: Timer,
    combo_step: u8,
    facing: f32,
    is_crouching: bool,
    overedge_enabled: bool,
}

type PlayerKnifeAttackItem<'a> = (
    Entity,
    &'a Velocity,
    &'a PlayerState,
    Option<&'a FacingDirection>,
    &'a ShroudState,
    &'a mut AttackAnimationState,
);

struct KnifeAttackRequest<'a> {
    player_entity: Entity,
    player_velocity: &'a Velocity,
    player_state: &'a PlayerState,
    facing_sign: f32,
    knife_tuning: &'a crate::resources::KnifeCombatTuning,
    overedge_enabled: bool,
    requested_style: AttackAnimationStyle,
}

#[derive(Component, Debug, Clone)]
pub struct EnemyProjectile {
    pub damage: f32,
    pub lifetime: f32,
    pub elapsed: f32,
}

impl EnemyProjectile {
    pub fn new(damage: f32, lifetime: f32) -> Self {
        Self {
            damage,
            lifetime,
            elapsed: 0.0,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.elapsed >= self.lifetime
    }
}

/// 近战命中时的短暂停顿（HitStop）。
#[derive(Resource, Debug, Clone)]
pub struct HitStopState {
    pub remaining: f32,
    pub freeze_speed: f32,
}

impl Default for HitStopState {
    fn default() -> Self {
        Self {
            remaining: 0.0,
            freeze_speed: 1.0,
        }
    }
}

impl HitStopState {
    pub fn trigger(&mut self, duration: f32, freeze_speed: f32) {
        if duration <= 0.0 {
            return;
        }

        if duration > self.remaining {
            self.remaining = duration;
        }

        let clamped_speed = freeze_speed.clamp(0.05, 1.0);
        if self.freeze_speed >= 1.0 || clamped_speed < self.freeze_speed {
            self.freeze_speed = clamped_speed;
        }
    }
}

fn projectile_config(is_overedge: bool) -> ProjectileConfig {
    if is_overedge {
        ProjectileConfig {
            projectile_type: ProjectileType::Overedge,
            damage: 9,
            speed: 420.0,
            lifetime: 0.9,
            collision_size: Vec2::new(98.0, 52.0),
            core_size: Vec2::new(96.0, 22.0),
            core_color: Color::srgba(0.92, 0.20, 0.25, 0.96),
            aura_size: Vec2::new(128.0, 38.0),
            aura_color: Color::srgba(1.0, 0.30, 0.25, 0.32),
            accent_size: Vec2::new(102.0, 5.0),
            accent_color: Color::srgba(1.0, 0.9, 0.85, 0.75),
            aura_offset: Vec3::new(8.0, 0.0, -0.05),
            accent_offset: Vec3::new(0.0, 4.0, 0.05),
            initial_rotation: 0.04,
            accent_rotation: 0.03,
            cooldown: 0.55,
            pulse_speed: 12.0,
            pulse_amount: 0.05,
            spin_speed: 0.4,
        }
    } else {
        ProjectileConfig {
            projectile_type: ProjectileType::MagicWave,
            damage: 2,
            speed: 330.0,
            lifetime: 2.8,
            collision_size: Vec2::new(30.0, 24.0),
            core_size: Vec2::new(18.0, 18.0),
            core_color: Color::srgba(0.70, 0.92, 1.0, 0.98),
            aura_size: Vec2::new(34.0, 24.0),
            aura_color: Color::srgba(0.22, 0.56, 1.0, 0.34),
            accent_size: Vec2::new(20.0, 3.0),
            accent_color: Color::srgba(0.84, 0.95, 1.0, 0.8),
            aura_offset: Vec3::new(-2.0, 0.0, -0.05),
            accent_offset: Vec3::new(-12.0, 0.0, 0.05),
            initial_rotation: std::f32::consts::FRAC_PI_4,
            accent_rotation: 0.0,
            cooldown: 0.25,
            pulse_speed: 18.0,
            pulse_amount: 0.08,
            spin_speed: 3.0,
        }
    }
}

fn knife_attack_preset(step: u8, overedge: bool) -> KnifeAttackPreset {
    let base = match step {
        1 => KnifeAttackPreset {
            damage: 6.0,
            cooldown: 0.28,
            windup_secs: 0.11,
            animation_duration_secs: 0.30,
            lifetime: 0.09,
            hitbox_size: Vec2::new(72.0, 34.0),
            x_offset: 58.0,
            y_offset: 12.0,
            crouch_y_offset: -8.0,
            slash_render_size: Vec2::new(66.0, 16.0),
            slash_color: Color::srgba(0.96, 0.96, 1.0, 0.08),
            knockback_x: 90.0,
            knockback_y: 10.0,
            hit_stop_secs: 0.018,
        },
        2 => KnifeAttackPreset {
            damage: 8.0,
            cooldown: 0.26,
            windup_secs: 0.10,
            animation_duration_secs: 0.29,
            lifetime: 0.10,
            hitbox_size: Vec2::new(82.0, 38.0),
            x_offset: 66.0,
            y_offset: 14.0,
            crouch_y_offset: -6.0,
            slash_render_size: Vec2::new(74.0, 18.0),
            slash_color: Color::srgba(0.98, 0.88, 0.84, 0.10),
            knockback_x: 130.0,
            knockback_y: 20.0,
            hit_stop_secs: 0.024,
        },
        _ => KnifeAttackPreset {
            damage: 12.0,
            cooldown: 0.30,
            windup_secs: 0.08,
            animation_duration_secs: 0.28,
            lifetime: 0.12,
            hitbox_size: Vec2::new(96.0, 44.0),
            x_offset: 74.0,
            y_offset: 16.0,
            crouch_y_offset: -4.0,
            slash_render_size: Vec2::new(88.0, 22.0),
            slash_color: Color::srgba(1.0, 0.78, 0.65, 0.12),
            knockback_x: 220.0,
            knockback_y: 35.0,
            hit_stop_secs: 0.035,
        },
    };

    if overedge {
        KnifeAttackPreset {
            damage: base.damage * 1.25,
            cooldown: (base.cooldown - 0.03).max(0.14),
            windup_secs: (base.windup_secs - 0.01).max(0.04),
            animation_duration_secs: (base.animation_duration_secs - 0.02).max(0.18),
            lifetime: base.lifetime,
            hitbox_size: base.hitbox_size * Vec2::new(1.12, 1.06),
            x_offset: base.x_offset + 6.0,
            y_offset: base.y_offset + 2.0,
            crouch_y_offset: base.crouch_y_offset,
            slash_render_size: base.slash_render_size * Vec2::new(1.1, 1.1),
            slash_color: Color::srgba(1.0, 0.58, 0.48, 0.16),
            knockback_x: base.knockback_x * 1.25,
            knockback_y: base.knockback_y * 1.2,
            hit_stop_secs: base.hit_stop_secs + 0.008,
        }
    } else {
        base
    }
}

fn overedge_animation_duration(style: AttackAnimationStyle) -> Option<f32> {
    let frame_count = match style {
        AttackAnimationStyle::OveredgeRelease => {
            asset_paths::HF_SHIROU_OVEREDGE_RELEASE_FRAME_COUNT
        }
        AttackAnimationStyle::OveredgeLight1
        | AttackAnimationStyle::OveredgeLight2
        | AttackAnimationStyle::OveredgeLight3 => {
            asset_paths::HF_SHIROU_OVEREDGE_LIGHT_ATTACK_SEGMENT_FRAME_COUNT
        }
        AttackAnimationStyle::OveredgeHeavy => {
            asset_paths::HF_SHIROU_OVEREDGE_HEAVY_ATTACK_FRAME_COUNT
        }
        AttackAnimationStyle::Normal => return None,
    };

    Some((frame_count as f32 + 1.0) * asset_paths::HF_SHIROU_OVEREDGE_ATTACK_FRAME_DURATION_SECS)
}

fn attack_cooldown_floor(style: AttackAnimationStyle) -> f32 {
    overedge_animation_duration(style).unwrap_or(0.0)
}

fn spawn_projectile_with_style(
    commands: &mut Commands,
    spawn_position: Vec3,
    config: ProjectileConfig,
    facing_sign: f32,
) {
    let direction = if facing_sign < 0.0 { -1.0 } else { 1.0 };
    let projectile_rotation = if direction < 0.0 {
        std::f32::consts::PI - config.initial_rotation
    } else {
        config.initial_rotation
    };
    let aura_offset = Vec3::new(
        config.aura_offset.x * direction,
        config.aura_offset.y,
        config.aura_offset.z,
    );
    let accent_offset = Vec3::new(
        config.accent_offset.x * direction,
        config.accent_offset.y,
        config.accent_offset.z,
    );
    let accent_rotation = if direction < 0.0 {
        -config.accent_rotation
    } else {
        config.accent_rotation
    };

    commands
        .spawn((
            Sprite {
                color: config.core_color,
                custom_size: Some(config.core_size),
                ..default()
            },
            Transform::from_xyz(spawn_position.x, spawn_position.y, 2.0)
                .with_rotation(Quat::from_rotation_z(projectile_rotation)),
            Projectile,
            config.projectile_type,
            ProjectileData::new(config.damage, config.speed, config.lifetime),
            Velocity {
                x: config.speed * direction,
                y: 0.0,
            },
            crate::systems::collision::CollisionBox::new(config.collision_size),
            ProjectileVisualMotion {
                base_scale: Vec3::ONE,
                pulse_speed: config.pulse_speed,
                pulse_amount: config.pulse_amount,
                spin_speed: config.spin_speed,
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Sprite {
                    color: config.aura_color,
                    custom_size: Some(config.aura_size),
                    ..default()
                },
                Transform::from_translation(aura_offset),
            ));

            parent.spawn((
                Sprite {
                    color: config.accent_color,
                    custom_size: Some(config.accent_size),
                    ..default()
                },
                Transform::from_translation(accent_offset)
                    .with_rotation(Quat::from_rotation_z(accent_rotation)),
            ));
        });
}

fn spawn_knife_slash(
    commands: &mut Commands,
    player_transform: &Transform,
    player_state: &PlayerState,
    combo_step: u8,
    facing: f32,
    overedge_enabled: bool,
) {
    let preset = knife_attack_preset(combo_step, overedge_enabled);
    let y_offset = if player_state.is_crouching {
        preset.crouch_y_offset
    } else {
        preset.y_offset
    };

    let slash_position = Vec3::new(
        player_transform.translation.x + preset.x_offset * facing,
        player_transform.translation.y + y_offset,
        2.4,
    );

    commands.spawn((
        Sprite {
            color: preset.slash_color,
            custom_size: Some(preset.slash_render_size),
            ..default()
        },
        Transform::from_translation(slash_position).with_rotation(Quat::from_rotation_z(
            if player_state.is_crouching {
                0.06
            } else {
                -0.12
            } * facing,
        )),
        KnifeSlash {
            damage: preset.damage,
            lifetime: Timer::from_seconds(preset.lifetime, TimerMode::Once),
            combo_step,
            knockback_x: preset.knockback_x * facing,
            knockback_y: preset.knockback_y,
            hit_stop_secs: preset.hit_stop_secs,
        },
        crate::systems::collision::CollisionBox::new(preset.hitbox_size),
    ));
}

fn perform_knife_attack(
    commands: &mut Commands,
    runtime: &mut KnifeComboRuntime,
    attack_animation: &mut AttackAnimationState,
    request: KnifeAttackRequest,
) {
    let max_combo_steps = request.knife_tuning.max_combo_steps.max(1);
    let requested_style = if request.overedge_enabled {
        request.requested_style
    } else {
        AttackAnimationStyle::Normal
    };
    let combo_step = if requested_style == AttackAnimationStyle::OveredgeHeavy {
        max_combo_steps
    } else if runtime.combo_reset_timer <= 0.0 || runtime.combo_step >= max_combo_steps {
        1
    } else {
        runtime.combo_step + 1
    };
    let attack_style = if request.overedge_enabled && requested_style.is_overedge_light() {
        match combo_step {
            1 => AttackAnimationStyle::OveredgeLight1,
            2 => AttackAnimationStyle::OveredgeLight2,
            _ => AttackAnimationStyle::OveredgeLight3,
        }
    } else {
        requested_style
    };
    runtime.combo_step = combo_step;
    runtime.combo_reset_timer = request.knife_tuning.combo_reset_window_secs.max(0.1);
    runtime.queued_attack = None;

    let preset = knife_attack_preset(combo_step, request.overedge_enabled);
    runtime.cooldown = preset.cooldown.max(attack_cooldown_floor(attack_style));
    let animation_duration =
        overedge_animation_duration(attack_style).unwrap_or(preset.animation_duration_secs);
    attack_animation.trigger_with_style(animation_duration, attack_style);

    let facing = if request.facing_sign < 0.0 {
        -1.0
    } else if request.facing_sign > 0.0 {
        1.0
    } else if request.player_velocity.x < -5.0 {
        -1.0
    } else {
        1.0
    };

    commands.spawn((PendingKnifeAttack {
        owner: request.player_entity,
        timer: Timer::from_seconds(preset.windup_secs, TimerMode::Once),
        combo_step,
        facing,
        is_crouching: request.player_state.is_crouching,
        overedge_enabled: request.overedge_enabled,
    },));
}

/// 玩家发射投射物。
pub fn player_shoot_projectile(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    game_input: Option<Res<crate::systems::input::GameInput>>,
    player_query: Query<(&Transform, Option<&FacingDirection>), With<Player>>,
    mut cooldown: Local<f32>,
    time: Res<Time>,
    shroud_query: Query<&ShroudState, With<Player>>,
) {
    *cooldown -= time.delta_secs();

    let projectile_pressed = game_input
        .as_deref()
        .map(|input| input.action2_pressed_this_frame)
        .unwrap_or(false)
        || keyboard.just_pressed(KeyCode::KeyX);
    let overedge_active = shroud_query
        .iter()
        .next()
        .map(|state| state.is_released)
        .unwrap_or(false);

    if projectile_pressed && overedge_active {
        return;
    }

    if projectile_pressed
        && *cooldown <= 0.0
        && let Some((player_transform, facing)) = player_query.iter().next()
    {
        let facing_sign = facing.copied().unwrap_or_default().sign();

        let config = projectile_config(false);
        *cooldown = config.cooldown;

        let spawn_position = Vec3::new(
            player_transform.translation.x + PROJECTILE_MUZZLE_X_OFFSET * facing_sign,
            player_transform.translation.y + PROJECTILE_MUZZLE_Y_OFFSET,
            2.0,
        );

        spawn_projectile_with_style(&mut commands, spawn_position, config, facing_sign);
    }
}

/// 玩家近战刀攻击（L/U）：
/// 支持轻量三段连击与输入缓冲。
pub fn player_knife_attack(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    game_input: Option<Res<crate::systems::input::GameInput>>,
    mut player_query: Query<PlayerKnifeAttackItem, With<Player>>,
    tuning: Option<Res<GameplayTuning>>,
    mut runtime: Local<KnifeComboRuntime>,
    time: Res<Time>,
) {
    let default_tuning = GameplayTuning::default();
    let knife_tuning = &tuning.as_deref().unwrap_or(&default_tuning).knife;

    runtime.cooldown = (runtime.cooldown - time.delta_secs()).max(0.0);
    runtime.combo_reset_timer = (runtime.combo_reset_timer - time.delta_secs()).max(0.0);

    if runtime.combo_reset_timer <= 0.0 {
        runtime.combo_step = 0;
        runtime.queued_attack = None;
    }

    let overedge_enabled = player_query
        .iter_mut()
        .next()
        .map(|(_, _, _, _, shroud, _)| shroud.is_released)
        .unwrap_or(false);
    let light_attack_pressed = game_input
        .as_deref()
        .map(|input| input.action1_pressed_this_frame)
        .unwrap_or(false)
        || keyboard.just_pressed(KeyCode::KeyL)
        || keyboard.just_pressed(KeyCode::KeyU);
    let heavy_attack_pressed = overedge_enabled && keyboard.just_pressed(KeyCode::KeyK);
    let requested_attack = if heavy_attack_pressed {
        Some(AttackAnimationStyle::OveredgeHeavy)
    } else if light_attack_pressed {
        Some(if overedge_enabled {
            AttackAnimationStyle::OveredgeLight1
        } else {
            AttackAnimationStyle::Normal
        })
    } else {
        None
    };

    if let Some(attack_style) = requested_attack {
        if runtime.cooldown <= 0.0 {
            if let Some((
                player_entity,
                player_velocity,
                player_state,
                facing,
                shroud,
                mut attack_animation,
            )) = player_query.iter_mut().next()
            {
                let facing_sign = facing.copied().unwrap_or_default().sign();
                perform_knife_attack(
                    &mut commands,
                    &mut runtime,
                    &mut attack_animation,
                    KnifeAttackRequest {
                        player_entity,
                        player_velocity,
                        player_state,
                        facing_sign,
                        knife_tuning,
                        overedge_enabled: shroud.is_released,
                        requested_style: attack_style,
                    },
                );
            }
            return;
        }

        if runtime.cooldown <= knife_tuning.combo_buffer_window_secs {
            runtime.queued_attack = Some(attack_style);
        }
    }

    if let Some(attack_style) = runtime.queued_attack
        && runtime.cooldown <= 0.0
        && let Some((
            player_entity,
            player_velocity,
            player_state,
            facing,
            shroud,
            mut attack_animation,
        )) = player_query.iter_mut().next()
    {
        if attack_style == AttackAnimationStyle::OveredgeHeavy && !shroud.is_released {
            runtime.queued_attack = None;
            return;
        }

        let facing_sign = facing.copied().unwrap_or_default().sign();
        perform_knife_attack(
            &mut commands,
            &mut runtime,
            &mut attack_animation,
            KnifeAttackRequest {
                player_entity,
                player_velocity,
                player_state,
                facing_sign,
                knife_tuning,
                overedge_enabled: shroud.is_released,
                requested_style: attack_style,
            },
        );
    }
}

pub fn resolve_pending_knife_attacks(
    mut commands: Commands,
    time: Res<Time>,
    mut pending_query: Query<(Entity, &mut PendingKnifeAttack)>,
    player_query: Query<&Transform, With<Player>>,
) {
    for (pending_entity, mut pending) in pending_query.iter_mut() {
        pending.timer.tick(time.delta());
        if !pending.timer.just_finished() && !pending.timer.is_finished() {
            continue;
        }

        let Ok(player_transform) = player_query.get(pending.owner) else {
            commands.entity(pending_entity).despawn();
            continue;
        };

        let player_state = PlayerState {
            is_grounded: true,
            is_crouching: pending.is_crouching,
        };
        spawn_knife_slash(
            &mut commands,
            player_transform,
            &player_state,
            pending.combo_step,
            pending.facing,
            pending.overedge_enabled,
        );
        commands.entity(pending_entity).despawn();
    }
}

/// 更新投射物移动。
pub fn update_projectiles(
    mut projectile_query: Query<(&mut Transform, &Velocity, &mut ProjectileData), With<Projectile>>,
    time: Res<Time>,
) {
    for (mut transform, velocity, mut data) in projectile_query.iter_mut() {
        transform.translation.x += velocity.x * time.delta_secs();
        transform.translation.y += velocity.y * time.delta_secs();
        data.elapsed += time.delta_secs();
    }
}

/// 更新敌方投射物移动。
pub fn update_enemy_projectiles(
    mut projectile_query: Query<(&mut Transform, &Velocity, &mut EnemyProjectile)>,
    time: Res<Time>,
) {
    for (mut transform, velocity, mut data) in projectile_query.iter_mut() {
        transform.translation.x += velocity.x * time.delta_secs();
        transform.translation.y += velocity.y * time.delta_secs();
        data.elapsed += time.delta_secs();
    }
}

/// 投射物视觉动画：轻微脉冲 + 旋转，让法术弹更有魔术感。
pub fn animate_projectile_visuals(
    time: Res<Time>,
    mut query: Query<(&ProjectileVisualMotion, &mut Transform), With<Projectile>>,
) {
    let elapsed = time.elapsed_secs();
    let delta = time.delta_secs();

    for (motion, mut transform) in query.iter_mut() {
        let pulse = 1.0 + motion.pulse_amount * (elapsed * motion.pulse_speed).sin();
        transform.scale = motion.base_scale * pulse;
        transform.rotate_z(motion.spin_speed * delta);
    }
}

/// 清理超时投射物。
pub fn cleanup_expired_projectiles(
    mut commands: Commands,
    projectile_query: Query<(Entity, &ProjectileData), With<Projectile>>,
) {
    for (entity, data) in projectile_query.iter() {
        if data.is_expired() {
            commands.entity(entity).despawn();
        }
    }
}

/// 清理超时敌方投射物。
pub fn cleanup_expired_enemy_projectiles(
    mut commands: Commands,
    projectile_query: Query<(Entity, &EnemyProjectile)>,
) {
    for (entity, data) in projectile_query.iter() {
        if data.is_expired() {
            commands.entity(entity).despawn();
        }
    }
}

/// 清理超时刀攻击命中盒。
pub fn cleanup_expired_knife_slashes(
    mut commands: Commands,
    mut knife_query: Query<(Entity, &mut KnifeSlash)>,
    time: Res<Time>,
) {
    for (entity, mut knife_slash) in knife_query.iter_mut() {
        knife_slash.lifetime.tick(time.delta());
        if knife_slash.lifetime.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

/// 维护战斗 HitStop，全局时间缩放使用 `Time<Virtual>`。
pub fn maintain_hit_stop_timescale(
    time_real: Res<Time<Real>>,
    mut time_virtual: ResMut<Time<Virtual>>,
    mut hit_stop: ResMut<HitStopState>,
) {
    if hit_stop.remaining > 0.0 {
        hit_stop.remaining = (hit_stop.remaining - time_real.delta_secs()).max(0.0);
        time_virtual.set_relative_speed(hit_stop.freeze_speed);

        if hit_stop.remaining <= 0.0 {
            time_virtual.set_relative_speed(1.0);
            hit_stop.freeze_speed = 1.0;
        }
        return;
    }

    if (time_virtual.relative_speed() - 1.0).abs() > 0.001 {
        time_virtual.set_relative_speed(1.0);
    }
}

/// 刀攻击命中敌人后统一发伤害事件，并施加击退/硬直。
pub fn knife_enemy_collision(
    mut commands: Commands,
    mut damage_writer: MessageWriter<DamageEvent>,
    mut camera_impulse_writer: MessageWriter<CameraImpulseEvent>,
    knife_query: Query<(
        Entity,
        &Transform,
        &KnifeSlash,
        &crate::systems::collision::CollisionBox,
    )>,
    mut enemy_query: Query<
        (
            Entity,
            &Transform,
            &mut EnemyState,
            &mut Velocity,
            &crate::systems::collision::CollisionBox,
        ),
        With<Enemy>,
    >,
    mut hit_stop: Option<ResMut<HitStopState>>,
) {
    for (slash_entity, slash_transform, slash, slash_box) in knife_query.iter() {
        let mut hit_target = None;

        for (enemy_entity, enemy_transform, mut enemy_state, mut enemy_velocity, enemy_box) in
            enemy_query.iter_mut()
        {
            if !enemy_state.is_alive {
                continue;
            }

            let dx = (slash_transform.translation.x - enemy_transform.translation.x).abs();
            let dy = (slash_transform.translation.y - enemy_transform.translation.y).abs();
            let collision_x = dx < (slash_box.size.x + enemy_box.size.x) / 2.0;
            let collision_y = dy < (slash_box.size.y + enemy_box.size.y) / 2.0;

            if collision_x && collision_y {
                enemy_velocity.x = slash.knockback_x;
                enemy_velocity.y = slash.knockback_y;
                enemy_state.apply_hit_stun(0.12 + slash.hit_stop_secs * 3.0);
                hit_target = Some(enemy_entity);
                break;
            }
        }

        if let Some(enemy_entity) = hit_target {
            damage_writer.write(DamageEvent {
                target: enemy_entity,
                amount: slash.damage,
                source: DamageSource::Knife,
            });

            let shake_intensity = 2.2 + slash.combo_step as f32 * 0.8;
            camera_impulse_writer.write(CameraImpulseEvent {
                intensity: shake_intensity,
                duration: 0.06,
            });

            if let Some(hit_stop) = hit_stop.as_deref_mut() {
                hit_stop.trigger(slash.hit_stop_secs, 0.12);
            }

            commands.entity(slash_entity).despawn();
        }
    }
}

/// 投射物命中敌人后只发伤害事件，统一交给伤害结算系统处理。
pub fn projectile_enemy_collision(
    mut commands: Commands,
    mut damage_writer: MessageWriter<DamageEvent>,
    projectile_query: Query<
        (
            Entity,
            &Transform,
            &ProjectileData,
            &crate::systems::collision::CollisionBox,
        ),
        With<Projectile>,
    >,
    enemy_query: Query<
        (
            Entity,
            &Transform,
            &EnemyState,
            &crate::systems::collision::CollisionBox,
        ),
        With<Enemy>,
    >,
) {
    for (projectile_entity, projectile_transform, projectile_data, projectile_box) in
        projectile_query.iter()
    {
        let mut hit_target = None;

        for (enemy_entity, enemy_transform, enemy_state, enemy_box) in enemy_query.iter() {
            if !enemy_state.is_alive {
                continue;
            }

            let dx = (projectile_transform.translation.x - enemy_transform.translation.x).abs();
            let dy = (projectile_transform.translation.y - enemy_transform.translation.y).abs();

            let collision_x = dx < (projectile_box.size.x + enemy_box.size.x) / 2.0;
            let collision_y = dy < (projectile_box.size.y + enemy_box.size.y) / 2.0;

            if collision_x && collision_y {
                hit_target = Some(enemy_entity);
                break;
            }
        }

        if let Some(enemy_entity) = hit_target {
            damage_writer.write(DamageEvent {
                target: enemy_entity,
                amount: projectile_data.damage as f32,
                source: DamageSource::Projectile,
            });
            commands.entity(projectile_entity).despawn();
        }
    }
}

/// 敌方投射物命中玩家。
pub fn enemy_projectile_player_collision(
    mut commands: Commands,
    mut damage_writer: MessageWriter<DamageEvent>,
    projectile_query: Query<(
        Entity,
        &Transform,
        &EnemyProjectile,
        &crate::systems::collision::CollisionBox,
    )>,
    player_query: Query<
        (Entity, &Transform, &crate::systems::collision::CollisionBox),
        With<Player>,
    >,
) {
    let Some((player_entity, player_transform, player_box)) = player_query.iter().next() else {
        return;
    };

    for (projectile_entity, projectile_transform, projectile_data, projectile_box) in
        projectile_query.iter()
    {
        let dx = (projectile_transform.translation.x - player_transform.translation.x).abs();
        let dy = (projectile_transform.translation.y - player_transform.translation.y).abs();

        let collision_x = dx < (projectile_box.size.x + player_box.size.x) / 2.0;
        let collision_y = dy < (projectile_box.size.y + player_box.size.y) / 2.0;

        if collision_x && collision_y {
            damage_writer.write(DamageEvent {
                target: player_entity,
                amount: projectile_data.damage,
                source: DamageSource::EnemyProjectile,
            });
            commands.entity(projectile_entity).despawn();
        }
    }
}

/// 玩家与敌人接触伤害（带冷却），伤害通过事件统一结算。
pub fn player_enemy_collision(
    player_query: Query<
        (Entity, &Transform, &crate::systems::collision::CollisionBox),
        With<Player>,
    >,
    enemy_query: Query<
        (
            &Transform,
            &EnemyState,
            &crate::systems::collision::CollisionBox,
        ),
        With<Enemy>,
    >,
    mut last_damage_time: Local<f32>,
    time: Res<Time>,
    mut damage_writer: MessageWriter<DamageEvent>,
) {
    *last_damage_time += time.delta_secs();

    if let Some((player_entity, player_transform, player_box)) = player_query.iter().next() {
        for (enemy_transform, enemy_state, enemy_box) in enemy_query.iter() {
            if !enemy_state.is_alive {
                continue;
            }

            let dx = (player_transform.translation.x - enemy_transform.translation.x).abs();
            let dy = (player_transform.translation.y - enemy_transform.translation.y).abs();

            let collision_x = dx < (player_box.size.x + enemy_box.size.x) / 2.0;
            let collision_y = dy < (player_box.size.y + enemy_box.size.y) / 2.0;

            if collision_x && collision_y && *last_damage_time >= PLAYER_CONTACT_DAMAGE_COOLDOWN {
                damage_writer.write(DamageEvent {
                    target: player_entity,
                    amount: enemy_state.contact_damage,
                    source: DamageSource::EnemyContact,
                });
                *last_damage_time = 0.0;
                break;
            }
        }
    }
}

/// 统一伤害结算管线，处理玩家和敌人的受击逻辑。
pub fn apply_damage_events(
    mut damage_events: MessageReader<DamageEvent>,
    mut player_query: Query<&mut Health, With<Player>>,
    mut player_guard_query: Query<&mut DamageInvulnerability, With<Player>>,
    mut enemy_query: Query<&mut EnemyState, With<Enemy>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut camera_impulse_writer: MessageWriter<CameraImpulseEvent>,
) {
    for event in damage_events.read() {
        if let Ok(mut health) = player_query.get_mut(event.target) {
            if health.is_dead() {
                continue;
            }

            let is_hostile_damage = matches!(
                event.source,
                DamageSource::EnemyContact | DamageSource::EnemyProjectile
            );

            if is_hostile_damage && let Ok(mut guard) = player_guard_query.get_mut(event.target) {
                if guard.is_active() {
                    continue;
                }
                guard.trigger(0.45);
            }

            health.take_damage(event.amount);

            if is_hostile_damage {
                camera_impulse_writer.write(CameraImpulseEvent {
                    intensity: 3.0,
                    duration: 0.08,
                });
            }

            if health.is_dead() {
                next_state.set(GameState::GameOver);
                camera_impulse_writer.write(CameraImpulseEvent {
                    intensity: 8.0,
                    duration: 0.2,
                });
            }

            continue;
        }

        if let Ok(mut enemy_state) = enemy_query.get_mut(event.target) {
            if !enemy_state.is_alive {
                continue;
            }

            enemy_state.take_damage(event.amount.ceil() as i32);
        }
    }
}

/// 供敌人系统生成敌方飞弹使用。
pub fn spawn_enemy_projectile(
    commands: &mut Commands,
    position: Vec3,
    direction: Vec2,
    speed: f32,
    damage: f32,
    lifetime: f32,
) {
    let dir = direction.normalize_or_zero();
    if dir == Vec2::ZERO {
        return;
    }

    commands.spawn((
        Sprite {
            color: Color::srgba(0.96, 0.55, 0.86, 0.9),
            custom_size: Some(ENEMY_PROJECTILE_RENDER_SIZE),
            ..default()
        },
        Transform::from_xyz(position.x, position.y, 2.1),
        EnemyProjectile::new(damage, lifetime),
        Velocity {
            x: dir.x * speed,
            y: dir.y * speed,
        },
        crate::systems::collision::CollisionBox::new(ENEMY_PROJECTILE_COLLISION_SIZE),
    ));
}
