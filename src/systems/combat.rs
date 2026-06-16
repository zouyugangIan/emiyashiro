//! 战斗系统 - 攻击、投射物、命中与伤害结算

use crate::{
    asset_paths,
    components::*,
    events::{CameraImpulseEvent, DamageEvent, DamageSource},
    resources::{GameConfig, GameplayTuning},
    states::GameState,
};
use bevy::prelude::*;

const PLAYER_CONTACT_DAMAGE_COOLDOWN: f32 = 1.0;
const PROJECTILE_MUZZLE_X_OFFSET: f32 = 54.0;
const PROJECTILE_MUZZLE_Y_OFFSET: f32 = 18.0;
const REFERENCE_ATTACK_FRAME_SECS: f32 = asset_paths::HF_SHIROU_OVEREDGE_ATTACK_FRAME_DURATION_SECS;
const GROUND_LIGHT_RUNTIME_ROWS: &[u8] = &[1, 2, 3, 4, 5];
const AIR_COMBO_RUNTIME_ROWS: &[u8] = &[1, 2, 3, 4, 5];
const HEAVY_RUNTIME_ROWS: &[u8] = &[1, 2, 3, 4, 5];
const ULTIMATE_RUNTIME_ROWS: &[u8] = &[1, 2, 3];
const MOBILITY_RUNTIME_ROWS: &[u8] = &[1, 2];
const NINJUTSU_RUNTIME_ROWS: &[u8] = &[1, 2, 3];
const WEAPON_PROJ_RUNTIME_ROWS: &[u8] = &[1, 2, 3, 4];
const OVEREDGE_GROUND_LIGHT_ROWS: &[u8] = &[3, 4, 5];
const OVEREDGE_AIR_COMBO_ROWS: &[u8] = &[2, 4, 5];
const OVEREDGE_HEAVY_ROWS: &[u8] = &[3, 5, 4];
const OVEREDGE_ULTIMATE_ROWS: &[u8] = &[2, 3];
const OVEREDGE_MOBILITY_ROWS: &[u8] = &[3, 4, 2];
const OVEREDGE_NINJUTSU_ROWS: &[u8] = &[2, 3, 4];
const OVEREDGE_WEAPON_PROJ_ROWS: &[u8] = &[2, 3, 4];
const REFERENCE_ACTION_VFX_FRAME_SECS: f32 = 0.045;

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
    combo_family: Option<AttackComboFamily>,
    combo_reset_timer: f32,
    queued_attack: Option<AttackAnimationStyle>,
    ground_light_visual_step: u8,
    heavy_visual_step: u8,
    air_combo_visual_step: u8,
    mobility_visual_step: u8,
    ninjutsu_visual_step: u8,
    ultimate_visual_step: u8,
    weapon_proj_visual_step: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AttackComboFamily {
    GroundLight,
    AirCombo,
    Heavy,
    Ultimate,
    Mobility,
    Ninjutsu,
    WeaponProjection,
    OveredgeLight,
    OveredgeHeavy,
    Other,
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

#[derive(Component, Debug, Clone, Copy)]
pub struct KnifeSlashFeedback {
    pub camera_intensity: f32,
    pub camera_duration: f32,
    pub hit_stop_freeze_speed: f32,
    pub base_alpha: f32,
    pub visual_expand: f32,
    pub visual_spin: f32,
}

#[derive(Component, Debug)]
pub struct AttackReferenceActionVfx {
    timer: Timer,
    frame_start: usize,
    frame_count: usize,
    base_alpha: f32,
    drift: Vec2,
}

#[derive(Component, Debug)]
pub struct PendingKnifeAttack {
    owner: Entity,
    timer: Timer,
    combo_step: u8,
    facing: f32,
    is_crouching: bool,
    overedge_enabled: bool,
    attack_style: AttackAnimationStyle,
}

type PlayerKnifeAttackItem<'a> = (
    Entity,
    Option<&'a Transform>,
    Option<&'a Sprite>,
    Option<&'a SpriteAnimationSheets>,
    &'a mut Velocity,
    &'a PlayerState,
    Option<&'a FacingDirection>,
    &'a ShroudState,
    &'a mut AttackAnimationState,
);

struct KnifeAttackRequest<'a> {
    player_entity: Entity,
    player_transform: Option<&'a Transform>,
    player_sprite: Option<&'a Sprite>,
    sprite_sheets: Option<&'a SpriteAnimationSheets>,
    player_velocity: &'a mut Velocity,
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

fn projectile_config_for_attack_style(style: AttackAnimationStyle) -> ProjectileConfig {
    let mut config = projectile_config(false);

    match style {
        AttackAnimationStyle::NinjutsuRefRow(1) => {
            config.projectile_type = ProjectileType::Fireball;
            config.damage = 7;
            config.speed = 380.0;
            config.lifetime = 1.35;
            config.collision_size = Vec2::new(58.0, 42.0);
            config.core_size = Vec2::new(46.0, 28.0);
            config.core_color = Color::srgba(1.0, 0.32, 0.08, 0.96);
            config.aura_size = Vec2::new(74.0, 44.0);
            config.aura_color = Color::srgba(1.0, 0.10, 0.04, 0.36);
            config.accent_size = Vec2::new(54.0, 5.0);
            config.accent_color = Color::srgba(1.0, 0.84, 0.38, 0.82);
            config.pulse_amount = 0.10;
            config.spin_speed = 2.6;
        }
        AttackAnimationStyle::NinjutsuRefRow(2) => {
            config.damage = 5;
            config.speed = 500.0;
            config.lifetime = 1.05;
            config.collision_size = Vec2::new(76.0, 48.0);
            config.core_size = Vec2::new(72.0, 26.0);
            config.core_color = Color::srgba(0.62, 0.88, 1.0, 0.92);
            config.aura_size = Vec2::new(116.0, 54.0);
            config.aura_color = Color::srgba(0.24, 0.58, 1.0, 0.30);
            config.accent_size = Vec2::new(70.0, 4.0);
            config.accent_color = Color::srgba(0.92, 0.98, 1.0, 0.86);
            config.pulse_speed = 20.0;
        }
        AttackAnimationStyle::NinjutsuRefRow(3) => {
            config.damage = 8;
            config.speed = 260.0;
            config.lifetime = 0.8;
            config.collision_size = Vec2::new(42.0, 108.0);
            config.core_size = Vec2::new(18.0, 96.0);
            config.core_color = Color::srgba(0.78, 0.42, 1.0, 0.92);
            config.aura_size = Vec2::new(44.0, 128.0);
            config.aura_color = Color::srgba(0.56, 0.16, 1.0, 0.28);
            config.accent_size = Vec2::new(7.0, 96.0);
            config.accent_color = Color::srgba(0.96, 0.88, 1.0, 0.82);
            config.initial_rotation = 0.0;
            config.accent_rotation = 0.0;
            config.pulse_amount = 0.14;
        }
        AttackAnimationStyle::NinjutsuRefRow(4) => {
            config.projectile_type = ProjectileType::Overedge;
            config.damage = 9;
            config.speed = 440.0;
            config.lifetime = 1.0;
            config.collision_size = Vec2::new(96.0, 52.0);
            config.core_size = Vec2::new(90.0, 22.0);
            config.core_color = Color::srgba(0.98, 0.16, 0.18, 0.96);
            config.aura_size = Vec2::new(128.0, 44.0);
            config.aura_color = Color::srgba(0.72, 0.0, 0.0, 0.34);
            config.accent_size = Vec2::new(86.0, 5.0);
            config.accent_color = Color::srgba(1.0, 0.88, 0.82, 0.86);
            config.pulse_amount = 0.12;
        }
        _ => {}
    }

    config
}

fn reset_reference_visual_steps(runtime: &mut KnifeComboRuntime) {
    runtime.ground_light_visual_step = 0;
    runtime.heavy_visual_step = 0;
    runtime.air_combo_visual_step = 0;
    runtime.mobility_visual_step = 0;
    runtime.ninjutsu_visual_step = 0;
    runtime.ultimate_visual_step = 0;
    runtime.weapon_proj_visual_step = 0;
}

fn next_reference_visual_row(current: &mut u8, sequence: &[u8]) -> u8 {
    let Some((&first, rest)) = sequence.split_first() else {
        *current = 1;
        return *current;
    };

    let next_index = sequence
        .iter()
        .position(|row| *row == *current)
        .map(|index| (index + 1) % sequence.len())
        .unwrap_or(0);
    *current = if rest.is_empty() {
        first
    } else {
        sequence[next_index]
    };
    *current
}

fn direct_mobility_row(input_row: u8) -> u8 {
    input_row
        .max(1)
        .min(asset_paths::REFERENCE_BOARD_MOBILITY_ROWS as u8)
}

fn resolve_reference_visual_style(
    runtime: &mut KnifeComboRuntime,
    style: AttackAnimationStyle,
) -> AttackAnimationStyle {
    match style {
        AttackAnimationStyle::GroundLight => {
            AttackAnimationStyle::GroundLightRow(next_reference_visual_row(
                &mut runtime.ground_light_visual_step,
                GROUND_LIGHT_RUNTIME_ROWS,
            ))
        }
        AttackAnimationStyle::AirCombo => AttackAnimationStyle::AirComboRow(
            next_reference_visual_row(&mut runtime.air_combo_visual_step, AIR_COMBO_RUNTIME_ROWS),
        ),
        AttackAnimationStyle::HeavyRef => AttackAnimationStyle::HeavyRefRow(
            next_reference_visual_row(&mut runtime.heavy_visual_step, HEAVY_RUNTIME_ROWS),
        ),
        AttackAnimationStyle::UltimateRef => AttackAnimationStyle::UltimateRefRow(
            next_reference_visual_row(&mut runtime.ultimate_visual_step, ULTIMATE_RUNTIME_ROWS),
        ),
        AttackAnimationStyle::MobilityRef => AttackAnimationStyle::MobilityRefRow(
            next_reference_visual_row(&mut runtime.mobility_visual_step, MOBILITY_RUNTIME_ROWS),
        ),
        AttackAnimationStyle::NinjutsuRef => AttackAnimationStyle::NinjutsuRefRow(
            next_reference_visual_row(&mut runtime.ninjutsu_visual_step, NINJUTSU_RUNTIME_ROWS),
        ),
        AttackAnimationStyle::WeaponProjRef => {
            AttackAnimationStyle::WeaponProjRefRow(next_reference_visual_row(
                &mut runtime.weapon_proj_visual_step,
                WEAPON_PROJ_RUNTIME_ROWS,
            ))
        }
        _ => style,
    }
}

fn resolve_overedge_reference_visual_style(
    runtime: &mut KnifeComboRuntime,
    style: AttackAnimationStyle,
) -> AttackAnimationStyle {
    match style {
        AttackAnimationStyle::GroundLight => {
            AttackAnimationStyle::GroundLightRow(next_reference_visual_row(
                &mut runtime.ground_light_visual_step,
                OVEREDGE_GROUND_LIGHT_ROWS,
            ))
        }
        AttackAnimationStyle::AirCombo => AttackAnimationStyle::AirComboRow(
            next_reference_visual_row(&mut runtime.air_combo_visual_step, OVEREDGE_AIR_COMBO_ROWS),
        ),
        AttackAnimationStyle::HeavyRef => AttackAnimationStyle::HeavyRefRow(
            next_reference_visual_row(&mut runtime.heavy_visual_step, OVEREDGE_HEAVY_ROWS),
        ),
        AttackAnimationStyle::UltimateRef => AttackAnimationStyle::UltimateRefRow(
            next_reference_visual_row(&mut runtime.ultimate_visual_step, OVEREDGE_ULTIMATE_ROWS),
        ),
        AttackAnimationStyle::MobilityRef => AttackAnimationStyle::MobilityRefRow(
            next_reference_visual_row(&mut runtime.mobility_visual_step, OVEREDGE_MOBILITY_ROWS),
        ),
        AttackAnimationStyle::NinjutsuRef => AttackAnimationStyle::NinjutsuRefRow(
            next_reference_visual_row(&mut runtime.ninjutsu_visual_step, OVEREDGE_NINJUTSU_ROWS),
        ),
        AttackAnimationStyle::WeaponProjRef => {
            AttackAnimationStyle::WeaponProjRefRow(next_reference_visual_row(
                &mut runtime.weapon_proj_visual_step,
                OVEREDGE_WEAPON_PROJ_ROWS,
            ))
        }
        _ => style,
    }
}

fn normalize_attack_style_for_overedge(
    style: AttackAnimationStyle,
    overedge_enabled: bool,
) -> AttackAnimationStyle {
    if !overedge_enabled {
        return style;
    }

    match style {
        AttackAnimationStyle::OveredgeRelease => style,
        AttackAnimationStyle::OveredgeLight1 => AttackAnimationStyle::GroundLightRow(1),
        AttackAnimationStyle::OveredgeLight2 => AttackAnimationStyle::GroundLightRow(3),
        AttackAnimationStyle::OveredgeLight3 => AttackAnimationStyle::GroundLightRow(5),
        AttackAnimationStyle::OveredgeHeavy => AttackAnimationStyle::HeavyRefRow(5),
        AttackAnimationStyle::Normal => AttackAnimationStyle::GroundLight,
        _ => style,
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
            slash_color: Color::srgba(0.96, 0.96, 1.0, 0.20),
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
            slash_color: Color::srgba(0.98, 0.88, 0.84, 0.23),
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
            slash_color: Color::srgba(1.0, 0.78, 0.65, 0.27),
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
            slash_color: Color::srgba(1.0, 0.58, 0.48, 0.32),
            knockback_x: base.knockback_x * 1.25,
            knockback_y: base.knockback_y * 1.2,
            hit_stop_secs: base.hit_stop_secs + 0.008,
        }
    } else {
        base
    }
}

fn knife_attack_preset_for_style(
    step: u8,
    overedge: bool,
    style: AttackAnimationStyle,
) -> KnifeAttackPreset {
    let base = knife_attack_preset(step, overedge);

    let family_preset = match style {
        AttackAnimationStyle::AirCombo | AttackAnimationStyle::AirComboRow(_) => {
            KnifeAttackPreset {
                cooldown: base.cooldown.max(0.24),
                windup_secs: 0.07,
                animation_duration_secs: base.animation_duration_secs.max(0.48),
                lifetime: 0.11,
                hitbox_size: base.hitbox_size * Vec2::new(1.08, 1.18),
                y_offset: 28.0,
                crouch_y_offset: 12.0,
                knockback_y: base.knockback_y + 26.0,
                slash_color: Color::srgba(0.84, 0.96, 1.0, 0.26),
                ..base
            }
        }
        AttackAnimationStyle::HeavyRef | AttackAnimationStyle::HeavyRefRow(_) => {
            KnifeAttackPreset {
                damage: base.damage * 1.45,
                cooldown: base.cooldown.max(0.42),
                windup_secs: 0.16,
                animation_duration_secs: base.animation_duration_secs.max(0.58),
                lifetime: 0.14,
                hitbox_size: base.hitbox_size * Vec2::new(1.35, 1.25),
                x_offset: base.x_offset + 18.0,
                slash_render_size: base.slash_render_size * Vec2::new(1.4, 1.3),
                slash_color: Color::srgba(1.0, 0.30, 0.24, 0.34),
                knockback_x: base.knockback_x * 1.45,
                knockback_y: base.knockback_y + 18.0,
                hit_stop_secs: base.hit_stop_secs + 0.025,
                ..base
            }
        }
        AttackAnimationStyle::UltimateRef | AttackAnimationStyle::UltimateRefRow(_) => {
            KnifeAttackPreset {
                damage: base.damage * 2.6,
                cooldown: base.cooldown.max(1.05),
                windup_secs: 0.22,
                animation_duration_secs: base.animation_duration_secs.max(0.72),
                lifetime: 0.18,
                hitbox_size: Vec2::new(190.0, 116.0),
                x_offset: 96.0,
                y_offset: 24.0,
                crouch_y_offset: 10.0,
                slash_render_size: Vec2::new(180.0, 42.0),
                slash_color: Color::srgba(1.0, 0.18, 0.14, 0.40),
                knockback_x: base.knockback_x * 2.2,
                knockback_y: 90.0,
                hit_stop_secs: base.hit_stop_secs + 0.06,
                ..base
            }
        }
        AttackAnimationStyle::MobilityRef | AttackAnimationStyle::MobilityRefRow(_) => {
            KnifeAttackPreset {
                damage: base.damage * 0.95,
                cooldown: base.cooldown.min(0.24),
                windup_secs: 0.045,
                animation_duration_secs: base.animation_duration_secs.max(0.36),
                lifetime: 0.10,
                hitbox_size: base.hitbox_size * Vec2::new(1.18, 0.95),
                x_offset: base.x_offset + 24.0,
                y_offset: 4.0,
                crouch_y_offset: -6.0,
                slash_render_size: base.slash_render_size * Vec2::new(1.35, 0.9),
                slash_color: Color::srgba(1.0, 0.22, 0.16, 0.28),
                knockback_x: base.knockback_x * 1.25,
                knockback_y: base.knockback_y * 0.7,
                hit_stop_secs: base.hit_stop_secs + 0.006,
                ..base
            }
        }
        AttackAnimationStyle::NinjutsuRef | AttackAnimationStyle::NinjutsuRefRow(_) => {
            KnifeAttackPreset {
                damage: base.damage,
                cooldown: base.cooldown.max(0.54),
                windup_secs: 0.18,
                animation_duration_secs: base.animation_duration_secs.max(0.56),
                lifetime: 0.0,
                hitbox_size: Vec2::ZERO,
                slash_render_size: Vec2::ZERO,
                slash_color: Color::srgba(0.0, 0.0, 0.0, 0.0),
                hit_stop_secs: 0.0,
                ..base
            }
        }
        AttackAnimationStyle::WeaponProjRef | AttackAnimationStyle::WeaponProjRefRow(_) => {
            KnifeAttackPreset {
                damage: base.damage * 1.25,
                cooldown: base.cooldown.max(0.46),
                windup_secs: 0.13,
                animation_duration_secs: base.animation_duration_secs.max(0.42),
                lifetime: 0.13,
                hitbox_size: base.hitbox_size * Vec2::new(1.25, 1.08),
                x_offset: base.x_offset + 12.0,
                slash_render_size: base.slash_render_size * Vec2::new(1.28, 1.05),
                slash_color: Color::srgba(1.0, 0.24, 0.20, 0.32),
                knockback_x: base.knockback_x * 1.28,
                knockback_y: base.knockback_y + 8.0,
                hit_stop_secs: base.hit_stop_secs + 0.014,
                ..base
            }
        }
        _ => base,
    };

    tune_reference_row_attack_preset(family_preset, style)
}

fn tune_reference_row_attack_preset(
    preset: KnifeAttackPreset,
    style: AttackAnimationStyle,
) -> KnifeAttackPreset {
    match style {
        AttackAnimationStyle::GroundLightRow(1) => KnifeAttackPreset {
            damage: preset.damage * 0.92,
            cooldown: preset.cooldown.min(0.22),
            windup_secs: 0.055,
            animation_duration_secs: preset.animation_duration_secs.min(0.26),
            lifetime: 0.075,
            hitbox_size: Vec2::new(66.0, 32.0),
            x_offset: 54.0,
            y_offset: 12.0,
            slash_render_size: Vec2::new(58.0, 14.0),
            knockback_x: 82.0,
            hit_stop_secs: preset.hit_stop_secs.min(0.018),
            ..preset
        },
        AttackAnimationStyle::GroundLightRow(2) => KnifeAttackPreset {
            damage: preset.damage * 1.04,
            hitbox_size: Vec2::new(78.0, 56.0),
            y_offset: 24.0,
            slash_render_size: Vec2::new(68.0, 30.0),
            knockback_x: preset.knockback_x * 1.05,
            knockback_y: 46.0,
            hit_stop_secs: preset.hit_stop_secs + 0.006,
            ..preset
        },
        AttackAnimationStyle::GroundLightRow(3) => KnifeAttackPreset {
            damage: preset.damage * 1.18,
            cooldown: preset.cooldown.max(0.30),
            windup_secs: 0.075,
            lifetime: 0.11,
            hitbox_size: Vec2::new(124.0, 34.0),
            x_offset: 82.0,
            slash_render_size: Vec2::new(112.0, 15.0),
            knockback_x: preset.knockback_x * 1.42,
            knockback_y: preset.knockback_y + 8.0,
            hit_stop_secs: preset.hit_stop_secs + 0.012,
            ..preset
        },
        AttackAnimationStyle::GroundLightRow(4) => KnifeAttackPreset {
            damage: preset.damage * 1.08,
            cooldown: preset.cooldown.max(0.31),
            lifetime: 0.13,
            hitbox_size: Vec2::new(138.0, 30.0),
            x_offset: 42.0,
            y_offset: 5.0,
            crouch_y_offset: -5.0,
            slash_render_size: Vec2::new(132.0, 18.0),
            knockback_x: preset.knockback_x * 1.18,
            knockback_y: 8.0,
            hit_stop_secs: preset.hit_stop_secs + 0.01,
            ..preset
        },
        AttackAnimationStyle::GroundLightRow(5) => KnifeAttackPreset {
            damage: preset.damage * 1.30,
            cooldown: preset.cooldown.max(0.36),
            windup_secs: 0.09,
            lifetime: 0.13,
            hitbox_size: Vec2::new(98.0, 86.0),
            x_offset: 68.0,
            y_offset: 38.0,
            slash_render_size: Vec2::new(96.0, 38.0),
            knockback_x: preset.knockback_x * 1.20,
            knockback_y: 94.0,
            hit_stop_secs: preset.hit_stop_secs + 0.02,
            ..preset
        },
        AttackAnimationStyle::HeavyRefRow(1) => KnifeAttackPreset {
            damage: preset.damage * 0.95,
            hitbox_size: Vec2::new(116.0, 58.0),
            x_offset: 78.0,
            slash_render_size: Vec2::new(112.0, 28.0),
            ..preset
        },
        AttackAnimationStyle::HeavyRefRow(2) => KnifeAttackPreset {
            damage: preset.damage * 1.05,
            hitbox_size: Vec2::new(104.0, 128.0),
            x_offset: 68.0,
            y_offset: 38.0,
            slash_render_size: Vec2::new(96.0, 72.0),
            knockback_y: 92.0,
            hit_stop_secs: preset.hit_stop_secs + 0.012,
            ..preset
        },
        AttackAnimationStyle::HeavyRefRow(3) => KnifeAttackPreset {
            damage: preset.damage * 1.18,
            cooldown: preset.cooldown.max(0.48),
            hitbox_size: Vec2::new(174.0, 44.0),
            x_offset: 112.0,
            y_offset: 18.0,
            slash_render_size: Vec2::new(164.0, 18.0),
            knockback_x: preset.knockback_x * 1.34,
            knockback_y: preset.knockback_y + 8.0,
            hit_stop_secs: preset.hit_stop_secs + 0.018,
            ..preset
        },
        AttackAnimationStyle::HeavyRefRow(4) => KnifeAttackPreset {
            damage: preset.damage * 1.12,
            cooldown: preset.cooldown.max(0.50),
            lifetime: 0.16,
            hitbox_size: Vec2::new(172.0, 48.0),
            x_offset: 42.0,
            y_offset: 8.0,
            slash_render_size: Vec2::new(168.0, 24.0),
            knockback_x: preset.knockback_x * 1.18,
            knockback_y: 18.0,
            hit_stop_secs: preset.hit_stop_secs + 0.016,
            ..preset
        },
        AttackAnimationStyle::HeavyRefRow(5) => KnifeAttackPreset {
            damage: preset.damage * 1.42,
            cooldown: preset.cooldown.max(0.64),
            windup_secs: 0.19,
            lifetime: 0.18,
            hitbox_size: Vec2::new(206.0, 126.0),
            x_offset: 86.0,
            y_offset: 34.0,
            slash_render_size: Vec2::new(198.0, 54.0),
            knockback_x: preset.knockback_x * 1.50,
            knockback_y: 76.0,
            hit_stop_secs: preset.hit_stop_secs + 0.04,
            ..preset
        },
        AttackAnimationStyle::AirComboRow(1) => KnifeAttackPreset {
            hitbox_size: Vec2::new(82.0, 58.0),
            y_offset: 32.0,
            knockback_y: 58.0,
            ..preset
        },
        AttackAnimationStyle::AirComboRow(2) => KnifeAttackPreset {
            damage: preset.damage * 1.08,
            hitbox_size: Vec2::new(126.0, 36.0),
            x_offset: 86.0,
            y_offset: 24.0,
            slash_render_size: Vec2::new(118.0, 15.0),
            knockback_x: preset.knockback_x * 1.28,
            ..preset
        },
        AttackAnimationStyle::AirComboRow(3) => KnifeAttackPreset {
            damage: preset.damage * 1.16,
            cooldown: preset.cooldown.max(0.30),
            hitbox_size: Vec2::new(78.0, 118.0),
            x_offset: 48.0,
            y_offset: -18.0,
            slash_render_size: Vec2::new(72.0, 68.0),
            knockback_y: -16.0,
            hit_stop_secs: preset.hit_stop_secs + 0.014,
            ..preset
        },
        AttackAnimationStyle::AirComboRow(4) => KnifeAttackPreset {
            damage: preset.damage * 1.20,
            cooldown: preset.cooldown.max(0.34),
            lifetime: 0.14,
            hitbox_size: Vec2::new(154.0, 74.0),
            x_offset: 34.0,
            y_offset: 8.0,
            slash_render_size: Vec2::new(150.0, 34.0),
            knockback_x: preset.knockback_x * 1.15,
            knockback_y: 44.0,
            ..preset
        },
        AttackAnimationStyle::AirComboRow(5) => KnifeAttackPreset {
            damage: preset.damage * 1.36,
            cooldown: preset.cooldown.max(0.40),
            windup_secs: 0.10,
            lifetime: 0.16,
            hitbox_size: Vec2::new(118.0, 136.0),
            x_offset: 58.0,
            y_offset: -26.0,
            slash_render_size: Vec2::new(106.0, 76.0),
            knockback_x: preset.knockback_x * 1.22,
            knockback_y: -34.0,
            hit_stop_secs: preset.hit_stop_secs + 0.026,
            ..preset
        },
        AttackAnimationStyle::MobilityRefRow(2) => KnifeAttackPreset {
            damage: preset.damage * 0.96,
            hitbox_size: Vec2::new(128.0, 28.0),
            x_offset: 88.0,
            y_offset: 0.0,
            crouch_y_offset: -5.0,
            slash_render_size: Vec2::new(128.0, 15.0),
            knockback_y: 6.0,
            ..preset
        },
        AttackAnimationStyle::MobilityRefRow(3) => KnifeAttackPreset {
            damage: preset.damage * 0.82,
            cooldown: preset.cooldown.max(0.30),
            hitbox_size: Vec2::new(88.0, 46.0),
            x_offset: 34.0,
            y_offset: 14.0,
            slash_render_size: Vec2::new(80.0, 16.0),
            knockback_x: -preset.knockback_x.abs() * 0.60,
            knockback_y: 30.0,
            ..preset
        },
        AttackAnimationStyle::MobilityRefRow(4) => KnifeAttackPreset {
            damage: preset.damage * 1.08,
            cooldown: preset.cooldown.max(0.34),
            hitbox_size: Vec2::new(96.0, 88.0),
            x_offset: 52.0,
            y_offset: 42.0,
            slash_render_size: Vec2::new(88.0, 46.0),
            knockback_x: preset.knockback_x * 0.92,
            knockback_y: 112.0,
            hit_stop_secs: preset.hit_stop_secs + 0.012,
            ..preset
        },
        AttackAnimationStyle::UltimateRefRow(1) => KnifeAttackPreset {
            damage: preset.damage * 0.95,
            hitbox_size: Vec2::new(202.0, 136.0),
            x_offset: 52.0,
            y_offset: 34.0,
            knockback_y: 76.0,
            ..preset
        },
        AttackAnimationStyle::UltimateRefRow(2) => KnifeAttackPreset {
            damage: preset.damage * 1.18,
            hitbox_size: Vec2::new(260.0, 82.0),
            x_offset: 142.0,
            y_offset: 24.0,
            slash_render_size: Vec2::new(248.0, 36.0),
            knockback_x: preset.knockback_x * 1.35,
            knockback_y: 54.0,
            hit_stop_secs: preset.hit_stop_secs + 0.026,
            ..preset
        },
        AttackAnimationStyle::UltimateRefRow(3) => KnifeAttackPreset {
            damage: preset.damage * 1.30,
            cooldown: preset.cooldown.max(1.20),
            windup_secs: 0.26,
            hitbox_size: Vec2::new(248.0, 168.0),
            x_offset: 76.0,
            y_offset: 54.0,
            slash_render_size: Vec2::new(232.0, 62.0),
            knockback_x: preset.knockback_x * 1.20,
            knockback_y: 124.0,
            hit_stop_secs: preset.hit_stop_secs + 0.04,
            ..preset
        },
        AttackAnimationStyle::WeaponProjRefRow(1) => KnifeAttackPreset {
            damage: preset.damage * 0.94,
            hitbox_size: Vec2::new(92.0, 46.0),
            x_offset: 68.0,
            ..preset
        },
        AttackAnimationStyle::WeaponProjRefRow(2) => KnifeAttackPreset {
            damage: preset.damage * 1.08,
            hitbox_size: Vec2::new(116.0, 58.0),
            x_offset: 64.0,
            slash_render_size: Vec2::new(112.0, 24.0),
            knockback_y: preset.knockback_y + 18.0,
            ..preset
        },
        AttackAnimationStyle::WeaponProjRefRow(3) => KnifeAttackPreset {
            damage: preset.damage * 1.22,
            cooldown: preset.cooldown.max(0.50),
            hitbox_size: Vec2::new(158.0, 50.0),
            x_offset: 106.0,
            slash_render_size: Vec2::new(150.0, 22.0),
            knockback_x: preset.knockback_x * 1.34,
            ..preset
        },
        AttackAnimationStyle::WeaponProjRefRow(4) => KnifeAttackPreset {
            damage: preset.damage * 1.16,
            cooldown: preset.cooldown.max(0.54),
            windup_secs: 0.15,
            hitbox_size: Vec2::new(136.0, 78.0),
            x_offset: 86.0,
            y_offset: 30.0,
            slash_render_size: Vec2::new(132.0, 32.0),
            knockback_x: preset.knockback_x * 1.16,
            knockback_y: preset.knockback_y + 44.0,
            hit_stop_secs: preset.hit_stop_secs + 0.018,
            ..preset
        },
        _ => preset,
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
        // Reference Board 模组帧数
        AttackAnimationStyle::GroundLight => {
            (asset_paths::REFERENCE_BOARD_GROUND_LIGHT_COLS
                * asset_paths::REFERENCE_BOARD_GROUND_LIGHT_ROWS) as usize
        }
        AttackAnimationStyle::GroundLightRow(_) => {
            asset_paths::REFERENCE_BOARD_GROUND_LIGHT_COLS as usize
        }
        AttackAnimationStyle::AirComboRow(_) => {
            asset_paths::REFERENCE_BOARD_AIR_COMBO_COLS as usize
        }
        AttackAnimationStyle::AirCombo => {
            (asset_paths::REFERENCE_BOARD_AIR_COMBO_COLS
                * asset_paths::REFERENCE_BOARD_AIR_COMBO_ROWS) as usize
        }
        AttackAnimationStyle::HeavyRef => {
            (asset_paths::REFERENCE_BOARD_HEAVY_COLS * asset_paths::REFERENCE_BOARD_HEAVY_ROWS)
                as usize
        }
        AttackAnimationStyle::HeavyRefRow(_) => asset_paths::REFERENCE_BOARD_HEAVY_COLS as usize,
        AttackAnimationStyle::UltimateRefRow(_) => {
            asset_paths::REFERENCE_BOARD_ULTIMATE_COLS as usize
        }
        AttackAnimationStyle::UltimateRef => {
            (asset_paths::REFERENCE_BOARD_ULTIMATE_COLS
                * asset_paths::REFERENCE_BOARD_ULTIMATE_ROWS) as usize
        }
        AttackAnimationStyle::MobilityRefRow(_) => {
            asset_paths::REFERENCE_BOARD_MOBILITY_COLS as usize
        }
        AttackAnimationStyle::MobilityRef => {
            (asset_paths::REFERENCE_BOARD_MOBILITY_COLS
                * asset_paths::REFERENCE_BOARD_MOBILITY_ROWS) as usize
        }
        AttackAnimationStyle::NinjutsuRefRow(_) => {
            asset_paths::REFERENCE_BOARD_NINJUTSU_COLS as usize
        }
        AttackAnimationStyle::NinjutsuRef => {
            (asset_paths::REFERENCE_BOARD_NINJUTSU_COLS
                * asset_paths::REFERENCE_BOARD_NINJUTSU_ROWS) as usize
        }
        AttackAnimationStyle::WeaponProjRefRow(_) => {
            asset_paths::REFERENCE_BOARD_WEAPON_PROJ_COLS as usize
        }
        AttackAnimationStyle::WeaponProjRef => {
            (asset_paths::REFERENCE_BOARD_WEAPON_PROJ_COLS
                * asset_paths::REFERENCE_BOARD_WEAPON_PROJ_ROWS) as usize
        }
        AttackAnimationStyle::AdvanceRef => {
            (asset_paths::REFERENCE_BOARD_ADVANCED_OVERVIEW_COLS
                * asset_paths::REFERENCE_BOARD_ADVANCED_OVERVIEW_ROWS) as usize
        }
        AttackAnimationStyle::Normal => return None,
    };

    Some((frame_count as f32 + 1.0) * REFERENCE_ATTACK_FRAME_SECS)
}

fn reference_action_lock_floor(style: AttackAnimationStyle) -> Option<f32> {
    match style {
        AttackAnimationStyle::GroundLightRow(1) => Some(0.18),
        AttackAnimationStyle::GroundLightRow(2) => Some(0.22),
        AttackAnimationStyle::GroundLightRow(3) => Some(0.28),
        AttackAnimationStyle::GroundLightRow(4) => Some(0.30),
        AttackAnimationStyle::GroundLightRow(5) => Some(0.34),
        AttackAnimationStyle::GroundLightRow(_) => Some(0.24),
        AttackAnimationStyle::AirComboRow(1) => Some(0.22),
        AttackAnimationStyle::AirComboRow(2) => Some(0.24),
        AttackAnimationStyle::AirComboRow(3) => Some(0.30),
        AttackAnimationStyle::AirComboRow(4) => Some(0.32),
        AttackAnimationStyle::AirComboRow(5) => Some(0.38),
        AttackAnimationStyle::AirComboRow(_) => Some(0.26),
        AttackAnimationStyle::HeavyRefRow(1) => Some(0.36),
        AttackAnimationStyle::HeavyRefRow(2) => Some(0.42),
        AttackAnimationStyle::HeavyRefRow(3) => Some(0.46),
        AttackAnimationStyle::HeavyRefRow(4) => Some(0.48),
        AttackAnimationStyle::HeavyRefRow(5) => Some(0.58),
        AttackAnimationStyle::HeavyRefRow(_) => Some(0.40),
        AttackAnimationStyle::UltimateRefRow(1) => Some(0.78),
        AttackAnimationStyle::UltimateRefRow(2) => Some(0.86),
        AttackAnimationStyle::UltimateRefRow(3) => Some(0.96),
        AttackAnimationStyle::UltimateRefRow(_) => Some(0.80),
        AttackAnimationStyle::MobilityRefRow(1) | AttackAnimationStyle::MobilityRefRow(2) => {
            Some(0.18)
        }
        AttackAnimationStyle::MobilityRefRow(3) => Some(0.26),
        AttackAnimationStyle::MobilityRefRow(4) => Some(0.32),
        AttackAnimationStyle::NinjutsuRefRow(4) => Some(0.50),
        AttackAnimationStyle::NinjutsuRefRow(_) => Some(0.44),
        AttackAnimationStyle::WeaponProjRefRow(1) => Some(0.32),
        AttackAnimationStyle::WeaponProjRefRow(2) => Some(0.36),
        AttackAnimationStyle::WeaponProjRefRow(3) => Some(0.42),
        AttackAnimationStyle::WeaponProjRefRow(4) => Some(0.46),
        AttackAnimationStyle::WeaponProjRefRow(_) => Some(0.34),
        _ => None,
    }
}

fn attack_cooldown_floor(style: AttackAnimationStyle) -> f32 {
    if let Some(lock_floor) = reference_action_lock_floor(style) {
        return lock_floor;
    }

    match style {
        AttackAnimationStyle::OveredgeRelease
        | AttackAnimationStyle::OveredgeLight1
        | AttackAnimationStyle::OveredgeLight2
        | AttackAnimationStyle::OveredgeLight3
        | AttackAnimationStyle::OveredgeHeavy => overedge_animation_duration(style).unwrap_or(0.0),
        _ => 0.0,
    }
}

fn attack_combo_family(style: AttackAnimationStyle) -> AttackComboFamily {
    match style {
        AttackAnimationStyle::GroundLight | AttackAnimationStyle::GroundLightRow(_) => {
            AttackComboFamily::GroundLight
        }
        AttackAnimationStyle::AirCombo | AttackAnimationStyle::AirComboRow(_) => {
            AttackComboFamily::AirCombo
        }
        AttackAnimationStyle::HeavyRef | AttackAnimationStyle::HeavyRefRow(_) => {
            AttackComboFamily::Heavy
        }
        AttackAnimationStyle::UltimateRef | AttackAnimationStyle::UltimateRefRow(_) => {
            AttackComboFamily::Ultimate
        }
        AttackAnimationStyle::MobilityRef | AttackAnimationStyle::MobilityRefRow(_) => {
            AttackComboFamily::Mobility
        }
        AttackAnimationStyle::NinjutsuRef | AttackAnimationStyle::NinjutsuRefRow(_) => {
            AttackComboFamily::Ninjutsu
        }
        AttackAnimationStyle::WeaponProjRef | AttackAnimationStyle::WeaponProjRefRow(_) => {
            AttackComboFamily::WeaponProjection
        }
        AttackAnimationStyle::OveredgeLight1
        | AttackAnimationStyle::OveredgeLight2
        | AttackAnimationStyle::OveredgeLight3
        | AttackAnimationStyle::OveredgeRelease => AttackComboFamily::OveredgeLight,
        AttackAnimationStyle::OveredgeHeavy => AttackComboFamily::OveredgeHeavy,
        AttackAnimationStyle::AdvanceRef | AttackAnimationStyle::Normal => AttackComboFamily::Other,
    }
}

fn reference_attack_row_key(keyboard: &ButtonInput<KeyCode>) -> Option<u8> {
    [
        KeyCode::KeyY,
        KeyCode::KeyU,
        KeyCode::KeyI,
        KeyCode::KeyO,
        KeyCode::KeyP,
    ]
    .into_iter()
    .position(|key| keyboard.just_pressed(key))
    .map(|index| index as u8 + 1)
}

fn shift_pressed(keyboard: &ButtonInput<KeyCode>) -> bool {
    keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight)
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

fn slash_feedback_for_style(
    combo_step: u8,
    style: AttackAnimationStyle,
    base_alpha: f32,
    facing: f32,
) -> KnifeSlashFeedback {
    let combo_weight = combo_step as f32;
    let (
        mut camera_intensity,
        mut camera_duration,
        mut hit_stop_freeze_speed,
        mut visual_expand,
        mut visual_spin,
    ): (f32, f32, f32, f32, f32) = match style {
        AttackAnimationStyle::UltimateRef | AttackAnimationStyle::UltimateRefRow(_) => {
            (9.0 + combo_weight * 0.7, 0.13, 0.06, 0.38, 0.46)
        }
        AttackAnimationStyle::HeavyRef
        | AttackAnimationStyle::HeavyRefRow(_)
        | AttackAnimationStyle::OveredgeHeavy => (5.4 + combo_weight * 0.9, 0.09, 0.09, 0.30, 0.34),
        AttackAnimationStyle::WeaponProjRef | AttackAnimationStyle::WeaponProjRefRow(_) => {
            (4.6 + combo_weight * 0.55, 0.075, 0.11, 0.26, 0.30)
        }
        AttackAnimationStyle::MobilityRef | AttackAnimationStyle::MobilityRefRow(_) => {
            (3.4 + combo_weight * 0.45, 0.055, 0.14, 0.24, 0.24)
        }
        AttackAnimationStyle::AirCombo | AttackAnimationStyle::AirComboRow(_) => {
            (3.2 + combo_weight * 0.55, 0.06, 0.13, 0.22, 0.28)
        }
        _ => (2.2 + combo_weight * 0.8, 0.06, 0.14, 0.18, 0.18),
    };

    match style {
        AttackAnimationStyle::GroundLightRow(1) => {
            camera_intensity *= 0.75;
            camera_duration *= 0.80;
            visual_expand *= 0.85;
        }
        AttackAnimationStyle::GroundLightRow(3) | AttackAnimationStyle::AirComboRow(2) => {
            camera_intensity *= 1.12;
            visual_expand *= 1.15;
            visual_spin *= 0.60;
        }
        AttackAnimationStyle::GroundLightRow(5)
        | AttackAnimationStyle::AirComboRow(5)
        | AttackAnimationStyle::WeaponProjRefRow(4) => {
            camera_intensity *= 1.22;
            camera_duration *= 1.12;
            hit_stop_freeze_speed *= 0.86;
            visual_expand *= 1.24;
        }
        AttackAnimationStyle::HeavyRefRow(1) => {
            camera_intensity *= 0.86;
            camera_duration *= 0.88;
            visual_expand *= 0.92;
        }
        AttackAnimationStyle::HeavyRefRow(3) => {
            camera_intensity *= 1.18;
            visual_spin *= 0.55;
        }
        AttackAnimationStyle::HeavyRefRow(5) => {
            camera_intensity *= 1.58;
            camera_duration *= 1.24;
            hit_stop_freeze_speed *= 0.74;
            visual_expand *= 1.42;
        }
        AttackAnimationStyle::UltimateRefRow(3) => {
            camera_intensity *= 1.32;
            camera_duration *= 1.22;
            hit_stop_freeze_speed *= 0.78;
            visual_expand *= 1.34;
        }
        AttackAnimationStyle::MobilityRefRow(3) => {
            camera_intensity *= 0.78;
            hit_stop_freeze_speed *= 1.20;
        }
        AttackAnimationStyle::MobilityRefRow(4) => {
            camera_intensity *= 1.10;
            visual_expand *= 1.18;
        }
        _ => {}
    }

    KnifeSlashFeedback {
        camera_intensity,
        camera_duration,
        hit_stop_freeze_speed: hit_stop_freeze_speed.clamp(0.04, 1.0),
        base_alpha,
        visual_expand,
        visual_spin: visual_spin * if facing < 0.0 { -1.0 } else { 1.0 },
    }
}

fn spawn_knife_slash(
    commands: &mut Commands,
    player_transform: &Transform,
    player_state: &PlayerState,
    combo_step: u8,
    facing: f32,
    overedge_enabled: bool,
    attack_style: AttackAnimationStyle,
) {
    let preset = knife_attack_preset_for_style(combo_step, overedge_enabled, attack_style);
    let base_alpha = match attack_style {
        AttackAnimationStyle::UltimateRef | AttackAnimationStyle::UltimateRefRow(_) => 0.40,
        AttackAnimationStyle::HeavyRef
        | AttackAnimationStyle::HeavyRefRow(_)
        | AttackAnimationStyle::OveredgeHeavy => 0.34,
        AttackAnimationStyle::WeaponProjRef | AttackAnimationStyle::WeaponProjRefRow(_) => 0.32,
        AttackAnimationStyle::MobilityRef | AttackAnimationStyle::MobilityRefRow(_) => 0.28,
        AttackAnimationStyle::AirCombo | AttackAnimationStyle::AirComboRow(_) => 0.26,
        _ => 0.22 + combo_step as f32 * 0.03,
    };
    let y_offset = if player_state.is_crouching {
        preset.crouch_y_offset
    } else {
        preset.y_offset
    }
    .max(-8.0);

    let slash_y = (player_transform.translation.y + y_offset).max(GameConfig::GROUND_LEVEL + 3.0);
    let slash_position = Vec3::new(
        player_transform.translation.x + preset.x_offset * facing,
        slash_y,
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
        slash_feedback_for_style(combo_step, attack_style, base_alpha, facing),
    ));
}

fn should_spawn_reference_action_vfx(style: AttackAnimationStyle) -> bool {
    matches!(
        style,
        AttackAnimationStyle::MobilityRefRow(3)
            | AttackAnimationStyle::MobilityRefRow(4)
            | AttackAnimationStyle::NinjutsuRefRow(4)
    )
}

fn reference_action_vfx_frame_range(style: AttackAnimationStyle) -> Option<(usize, usize)> {
    match style {
        AttackAnimationStyle::MobilityRefRow(row) if (3..=4).contains(&row) => Some((
            (row as usize - 1) * asset_paths::REFERENCE_BOARD_MOBILITY_COLS as usize,
            asset_paths::REFERENCE_BOARD_MOBILITY_COLS as usize,
        )),
        AttackAnimationStyle::NinjutsuRefRow(4) => Some((
            3 * asset_paths::REFERENCE_BOARD_NINJUTSU_COLS as usize,
            asset_paths::REFERENCE_BOARD_NINJUTSU_COLS as usize,
        )),
        _ => None,
    }
}

fn reference_action_vfx_profile(
    style: AttackAnimationStyle,
    facing: f32,
) -> (Vec3, Vec2, Vec2, Color, f32, f32) {
    let direction = if facing < 0.0 { -1.0 } else { 1.0 };

    match style {
        AttackAnimationStyle::MobilityRefRow(3) => (
            Vec3::new(-direction * 46.0, 18.0, 2.18),
            Vec2::new(138.0, 26.0),
            Vec2::new(-direction * 120.0, 28.0),
            Color::srgba(1.0, 0.16, 0.08, 0.24),
            0.24,
            -0.12 * direction,
        ),
        AttackAnimationStyle::MobilityRefRow(4) => (
            Vec3::new(direction * 24.0, 52.0, 2.18),
            Vec2::new(82.0, 34.0),
            Vec2::new(direction * 92.0, 118.0),
            Color::srgba(0.54, 0.86, 1.0, 0.22),
            0.22,
            0.54 * direction,
        ),
        AttackAnimationStyle::NinjutsuRefRow(4) => (
            Vec3::new(-direction * 24.0, 18.0, 2.17),
            Vec2::new(170.0, 22.0),
            Vec2::new(-direction * 76.0, 8.0),
            Color::srgba(1.0, 0.05, 0.05, 0.22),
            0.22,
            -0.04 * direction,
        ),
        _ => (
            Vec3::new(0.0, 0.0, 2.16),
            Vec2::new(120.0, 22.0),
            Vec2::ZERO,
            Color::srgba(1.0, 0.20, 0.12, 0.20),
            0.20,
            0.0,
        ),
    }
}

fn reference_action_uses_split_row_sheet(
    sprite_sheets: Option<&SpriteAnimationSheets>,
    style: AttackAnimationStyle,
) -> bool {
    let Some(sprite_sheets) = sprite_sheets else {
        return false;
    };

    match style {
        AttackAnimationStyle::GroundLight | AttackAnimationStyle::GroundLightRow(_) => {
            !sprite_sheets.reference_ground_light_row_textures.is_empty()
        }
        AttackAnimationStyle::AirCombo | AttackAnimationStyle::AirComboRow(_) => {
            !sprite_sheets.reference_air_combo_row_textures.is_empty()
        }
        AttackAnimationStyle::HeavyRef | AttackAnimationStyle::HeavyRefRow(_) => {
            !sprite_sheets.reference_heavy_row_textures.is_empty()
        }
        AttackAnimationStyle::UltimateRef | AttackAnimationStyle::UltimateRefRow(_) => {
            !sprite_sheets.reference_ultimate_row_textures.is_empty()
        }
        AttackAnimationStyle::MobilityRef | AttackAnimationStyle::MobilityRefRow(_) => {
            !sprite_sheets.reference_mobility_row_textures.is_empty()
        }
        AttackAnimationStyle::NinjutsuRef | AttackAnimationStyle::NinjutsuRefRow(_) => {
            !sprite_sheets.reference_ninjutsu_row_textures.is_empty()
        }
        AttackAnimationStyle::WeaponProjRef | AttackAnimationStyle::WeaponProjRefRow(_) => {
            !sprite_sheets.reference_weapon_proj_row_textures.is_empty()
        }
        _ => false,
    }
}

fn reference_action_vfx_sheet<'a>(
    sprite_sheets: Option<&'a SpriteAnimationSheets>,
    style: AttackAnimationStyle,
) -> Option<(
    &'a Handle<Image>,
    &'a Handle<TextureAtlasLayout>,
    usize,
    usize,
)> {
    let sprite_sheets = sprite_sheets?;
    let (texture, layout) =
        sprite_sheets.select_sheet_for_attack_style(&AnimationType::Attacking, style)?;
    let (frame_start, frame_count) = reference_action_vfx_frame_range(style)?;
    let frame_start = if reference_action_uses_split_row_sheet(Some(sprite_sheets), style) {
        0
    } else {
        frame_start
    };

    Some((texture, layout, frame_start, frame_count))
}

fn spawn_reference_action_vfx(
    commands: &mut Commands,
    player_transform: &Transform,
    sprite_sheets: Option<&SpriteAnimationSheets>,
    facing: f32,
    attack_style: AttackAnimationStyle,
) {
    let Some((texture, layout, frame_start, frame_count)) =
        reference_action_vfx_sheet(sprite_sheets, attack_style)
    else {
        return;
    };

    let (offset, mut size, drift, _color, alpha, rotation) =
        reference_action_vfx_profile(attack_style, facing);
    size = size.max(Vec2::new(156.0, 144.0));
    let duration = frame_count as f32 * REFERENCE_ACTION_VFX_FRAME_SECS + 0.08;
    let transform = Transform::from_translation(player_transform.translation + offset)
        .with_rotation(Quat::from_rotation_z(rotation));

    commands.spawn((
        Sprite {
            image: texture.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: layout.clone(),
                index: frame_start,
            }),
            color: Color::srgba(1.0, 1.0, 1.0, alpha),
            custom_size: Some(size),
            flip_x: facing < 0.0,
            ..default()
        },
        transform,
        AttackReferenceActionVfx {
            timer: Timer::from_seconds(duration, TimerMode::Once),
            frame_start,
            frame_count,
            base_alpha: alpha,
            drift,
        },
    ));
}

fn stable_player_visual_style(style: AttackAnimationStyle) -> AttackAnimationStyle {
    match style {
        AttackAnimationStyle::MobilityRefRow(3) => AttackAnimationStyle::MobilityRefRow(1),
        AttackAnimationStyle::MobilityRefRow(4) => AttackAnimationStyle::MobilityRefRow(2),
        AttackAnimationStyle::NinjutsuRefRow(4) => AttackAnimationStyle::NinjutsuRefRow(1),
        _ => style,
    }
}

fn spawn_attack_reference_action_vfx(
    commands: &mut Commands,
    player_transform: Option<&Transform>,
    _player_sprite: Option<&Sprite>,
    sprite_sheets: Option<&SpriteAnimationSheets>,
    facing: f32,
    attack_style: AttackAnimationStyle,
) {
    if !should_spawn_reference_action_vfx(attack_style) {
        return;
    }

    let Some(player_transform) = player_transform else {
        return;
    };
    spawn_reference_action_vfx(
        commands,
        player_transform,
        sprite_sheets,
        facing,
        attack_style,
    );
}

fn apply_attack_movement(
    velocity: &mut Velocity,
    player_state: &PlayerState,
    combo_step: u8,
    facing: f32,
    attack_style: AttackAnimationStyle,
) {
    let direction = if facing < 0.0 { -1.0 } else { 1.0 };

    match attack_style {
        AttackAnimationStyle::GroundLightRow(1) => {
            let target = direction * 92.0;
            if velocity.x.signum() != direction || velocity.x.abs() < target.abs() {
                velocity.x = target;
            }
        }
        AttackAnimationStyle::GroundLightRow(2) => {
            let target = direction * 118.0;
            if velocity.x.signum() != direction || velocity.x.abs() < target.abs() {
                velocity.x = target;
            }
            velocity.y = velocity.y.max(24.0);
        }
        AttackAnimationStyle::GroundLightRow(3) => {
            velocity.x = velocity.x * 0.28 + direction * 172.0;
            velocity.y = velocity.y.max(0.0);
        }
        AttackAnimationStyle::GroundLightRow(4) => {
            velocity.x = velocity.x * 0.34 + direction * 88.0;
            velocity.y = velocity.y.max(0.0);
        }
        AttackAnimationStyle::GroundLightRow(5) => {
            velocity.x = velocity.x * 0.42 + direction * 110.0;
            velocity.y = velocity.y.max(82.0);
        }
        AttackAnimationStyle::GroundLightRow(_) | AttackAnimationStyle::GroundLight => {
            let target = direction * (96.0 + combo_step as f32 * 18.0);
            if velocity.x.signum() != direction || velocity.x.abs() < target.abs() {
                velocity.x = target;
            }
        }
        AttackAnimationStyle::HeavyRefRow(2) => {
            velocity.x = velocity.x * 0.30 + direction * 42.0;
            velocity.y = velocity.y.max(48.0);
        }
        AttackAnimationStyle::HeavyRefRow(3) => {
            velocity.x = velocity.x * 0.28 + direction * 122.0;
            velocity.y = velocity.y.max(0.0);
        }
        AttackAnimationStyle::HeavyRefRow(4) => {
            velocity.x *= 0.18;
            velocity.y = velocity.y.max(0.0);
        }
        AttackAnimationStyle::HeavyRefRow(5) => {
            velocity.x = velocity.x * 0.16 + direction * 36.0;
            velocity.y = velocity.y.max(28.0);
        }
        AttackAnimationStyle::HeavyRefRow(_) | AttackAnimationStyle::HeavyRef => {
            velocity.x = velocity.x * 0.36 + direction * 54.0;
            velocity.y = velocity.y.max(0.0);
        }
        AttackAnimationStyle::UltimateRefRow(2) => {
            velocity.x = direction * 86.0;
            velocity.y = velocity.y.max(0.0);
        }
        AttackAnimationStyle::UltimateRefRow(3) => {
            velocity.x *= 0.10;
            velocity.y = velocity.y.max(44.0);
        }
        AttackAnimationStyle::UltimateRefRow(_) | AttackAnimationStyle::UltimateRef => {
            velocity.x *= 0.18;
            velocity.y = velocity.y.max(0.0);
        }
        AttackAnimationStyle::MobilityRefRow(1) | AttackAnimationStyle::MobilityRef => {
            velocity.x = direction * 420.0;
            velocity.y = velocity.y.max(0.0);
        }
        AttackAnimationStyle::MobilityRefRow(2) => {
            velocity.x = direction * 330.0;
            velocity.y = velocity.y.max(18.0);
        }
        AttackAnimationStyle::MobilityRefRow(3) => {
            velocity.x = -direction * 300.0;
            velocity.y = velocity.y.max(92.0);
        }
        AttackAnimationStyle::MobilityRefRow(4) => {
            velocity.x = direction * 380.0;
            velocity.y = velocity.y.max(220.0);
        }
        AttackAnimationStyle::AirComboRow(2) => {
            velocity.x += direction * 126.0;
            if !player_state.is_grounded {
                velocity.y = velocity.y.max(82.0);
            }
        }
        AttackAnimationStyle::AirComboRow(3) => {
            velocity.x += direction * 58.0;
            if !player_state.is_grounded {
                velocity.y = velocity.y.min(-80.0);
            }
        }
        AttackAnimationStyle::AirComboRow(4) => {
            velocity.x += direction * 42.0;
            if !player_state.is_grounded {
                velocity.y = velocity.y.max(58.0);
            }
        }
        AttackAnimationStyle::AirComboRow(5) => {
            velocity.x += direction * 76.0;
            if !player_state.is_grounded {
                velocity.y = velocity.y.min(-160.0);
            }
        }
        AttackAnimationStyle::AirComboRow(_) | AttackAnimationStyle::AirCombo => {
            velocity.x += direction * 72.0;
            if !player_state.is_grounded {
                velocity.y = velocity.y.max(115.0);
            }
        }
        AttackAnimationStyle::NinjutsuRefRow(4) => {
            velocity.x *= 0.25;
            velocity.y = velocity.y.max(0.0);
        }
        AttackAnimationStyle::WeaponProjRefRow(2) => {
            velocity.x = velocity.x * 0.45 + direction * 72.0;
            velocity.y = velocity.y.max(20.0);
        }
        AttackAnimationStyle::WeaponProjRefRow(3) => {
            velocity.x = velocity.x * 0.35 + direction * 128.0;
            velocity.y = velocity.y.max(0.0);
        }
        AttackAnimationStyle::WeaponProjRefRow(4) => {
            velocity.x = velocity.x * 0.42 + direction * 96.0;
            velocity.y = velocity.y.max(64.0);
        }
        AttackAnimationStyle::WeaponProjRefRow(_) | AttackAnimationStyle::WeaponProjRef => {
            velocity.x = velocity.x * 0.55 + direction * 88.0;
            velocity.y = velocity.y.max(0.0);
        }
        _ => {}
    }
}

fn attack_momentum_for_style(style: AttackAnimationStyle, facing: f32) -> Option<AttackMomentum> {
    let direction = if facing < 0.0 { -1.0 } else { 1.0 };

    match style {
        AttackAnimationStyle::GroundLightRow(3) => {
            Some(AttackMomentum::new(direction, 135.0, 0.08, 2.8, 0.0, 0.0))
        }
        AttackAnimationStyle::GroundLightRow(5) => {
            Some(AttackMomentum::new(direction, 90.0, 0.10, 2.6, 62.0, 0.07))
        }
        AttackAnimationStyle::HeavyRefRow(3) => {
            Some(AttackMomentum::new(direction, 104.0, 0.10, 2.4, 0.0, 0.0))
        }
        AttackAnimationStyle::HeavyRefRow(5) => {
            Some(AttackMomentum::new(direction, 36.0, 0.12, 3.2, 18.0, 0.05))
        }
        AttackAnimationStyle::UltimateRefRow(2) => {
            Some(AttackMomentum::new(direction, 72.0, 0.12, 2.6, 0.0, 0.0))
        }
        AttackAnimationStyle::UltimateRefRow(3) => {
            Some(AttackMomentum::new(direction, 0.0, 0.14, 4.0, 36.0, 0.06))
        }
        AttackAnimationStyle::MobilityRef | AttackAnimationStyle::MobilityRefRow(1) => {
            Some(AttackMomentum::new(direction, 340.0, 0.16, 1.8, 0.0, 0.0))
        }
        AttackAnimationStyle::MobilityRefRow(2) => {
            Some(AttackMomentum::new(direction, 260.0, 0.14, 2.0, 18.0, 0.08))
        }
        AttackAnimationStyle::MobilityRefRow(3) => Some(AttackMomentum::new(
            -direction, 240.0, 0.20, 2.4, 74.0, 0.10,
        )),
        AttackAnimationStyle::MobilityRefRow(4) => Some(AttackMomentum::new(
            direction, 330.0, 0.24, 1.6, 178.0, 0.14,
        )),
        AttackAnimationStyle::AirComboRow(2) => {
            Some(AttackMomentum::new(direction, 122.0, 0.09, 2.4, 58.0, 0.05))
        }
        AttackAnimationStyle::AirComboRow(3) => {
            Some(AttackMomentum::new(direction, 48.0, 0.10, 2.8, 0.0, 0.0))
        }
        AttackAnimationStyle::AirComboRow(5) => {
            Some(AttackMomentum::new(direction, 68.0, 0.12, 2.8, 0.0, 0.0))
        }
        AttackAnimationStyle::AirCombo | AttackAnimationStyle::AirComboRow(_) => {
            Some(AttackMomentum::new(direction, 90.0, 0.08, 2.5, 84.0, 0.07))
        }
        AttackAnimationStyle::WeaponProjRefRow(3) => {
            Some(AttackMomentum::new(direction, 118.0, 0.10, 2.6, 0.0, 0.0))
        }
        AttackAnimationStyle::WeaponProjRefRow(4) => {
            Some(AttackMomentum::new(direction, 84.0, 0.12, 2.8, 48.0, 0.06))
        }
        AttackAnimationStyle::WeaponProjRef | AttackAnimationStyle::WeaponProjRefRow(_) => {
            Some(AttackMomentum::new(direction, 78.0, 0.08, 3.0, 0.0, 0.0))
        }
        _ => None,
    }
}

fn perform_knife_attack(
    commands: &mut Commands,
    runtime: &mut KnifeComboRuntime,
    attack_animation: &mut AttackAnimationState,
    request: KnifeAttackRequest,
) {
    let max_combo_steps = request.knife_tuning.max_combo_steps.max(1);
    let base_style =
        normalize_attack_style_for_overedge(request.requested_style, request.overedge_enabled);
    let combo_family = attack_combo_family(base_style);
    let same_combo_family = runtime.combo_family == Some(combo_family);
    let combo_step = if base_style == AttackAnimationStyle::OveredgeHeavy {
        max_combo_steps
    } else if runtime.combo_reset_timer <= 0.0
        || runtime.combo_step >= max_combo_steps
        || !same_combo_family
    {
        1
    } else {
        runtime.combo_step + 1
    };
    let attack_style = if request.overedge_enabled && base_style.is_overedge_light() {
        match combo_step {
            1 => AttackAnimationStyle::OveredgeLight1,
            2 => AttackAnimationStyle::OveredgeLight2,
            _ => AttackAnimationStyle::OveredgeLight3,
        }
    } else {
        base_style
    };
    let attack_style = if request.overedge_enabled {
        resolve_overedge_reference_visual_style(runtime, attack_style)
    } else {
        resolve_reference_visual_style(runtime, attack_style)
    };
    runtime.combo_step = combo_step;
    runtime.combo_family = Some(combo_family);
    runtime.combo_reset_timer = request.knife_tuning.combo_reset_window_secs.max(0.1);
    runtime.queued_attack = None;

    let preset = knife_attack_preset_for_style(combo_step, request.overedge_enabled, attack_style);
    runtime.cooldown = preset.cooldown.max(attack_cooldown_floor(attack_style));
    let player_visual_style = stable_player_visual_style(attack_style);
    let animation_duration =
        overedge_animation_duration(player_visual_style).unwrap_or(preset.animation_duration_secs);
    attack_animation.trigger_with_style(animation_duration, player_visual_style);

    let facing = if request.facing_sign < 0.0 {
        -1.0
    } else if request.facing_sign > 0.0 {
        1.0
    } else if request.player_velocity.x < -5.0 {
        -1.0
    } else {
        1.0
    };

    apply_attack_movement(
        request.player_velocity,
        request.player_state,
        combo_step,
        facing,
        attack_style,
    );
    if let Some(momentum) = attack_momentum_for_style(attack_style, facing) {
        commands.entity(request.player_entity).insert(momentum);
    }
    spawn_attack_reference_action_vfx(
        commands,
        request.player_transform,
        request.player_sprite,
        request.sprite_sheets,
        facing,
        attack_style,
    );

    commands.spawn((PendingKnifeAttack {
        owner: request.player_entity,
        timer: Timer::from_seconds(preset.windup_secs, TimerMode::Once),
        combo_step,
        facing,
        is_crouching: request.player_state.is_crouching,
        overedge_enabled: request.overedge_enabled,
        attack_style,
    },));
}

/// 玩家发射投射物。
pub fn player_shoot_projectile(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    game_input: Option<Res<crate::systems::input::GameInput>>,
    player_query: Query<
        (
            &Transform,
            Option<&FacingDirection>,
            Option<&AttackAnimationState>,
        ),
        With<Player>,
    >,
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
        && let Some((player_transform, facing, attack_state)) = player_query.iter().next()
    {
        if attack_state.is_some() {
            return;
        }

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

/// 玩家近战刀攻击：
/// 支持轻量三段连击与输入缓冲。
/// 也支持 Reference Board 攻击模组（Shift+V 未激活时）。
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
        runtime.combo_family = None;
        runtime.queued_attack = None;
        reset_reference_visual_steps(&mut runtime);
    }

    let Some((
        player_entity,
        player_transform,
        player_sprite,
        sprite_sheets,
        mut player_velocity,
        player_state,
        facing,
        shroud,
        mut attack_animation,
    )) = player_query.iter_mut().next()
    else {
        return;
    };

    let overedge_enabled = shroud.is_released;
    let is_airborne = !player_state.is_grounded;
    let is_crouching = player_state.is_crouching;

    let shift_is_pressed = shift_pressed(&keyboard);
    let reference_row_attack = reference_attack_row_key(&keyboard).map(|row| {
        if shift_is_pressed {
            AttackAnimationStyle::HeavyRefRow(row)
        } else if is_airborne {
            AttackAnimationStyle::AirComboRow(row)
        } else if is_crouching {
            AttackAnimationStyle::MobilityRefRow(direct_mobility_row(row))
        } else {
            AttackAnimationStyle::GroundLightRow(row)
        }
    });
    let light_attack_pressed = game_input
        .as_deref()
        .map(|input| input.action1_pressed_this_frame)
        .unwrap_or(false)
        || keyboard.just_pressed(KeyCode::KeyL);
    let heavy_attack_pressed = keyboard.just_pressed(KeyCode::KeyK);
    let projectile_pressed = game_input
        .as_deref()
        .map(|input| input.action2_pressed_this_frame)
        .unwrap_or(false)
        || keyboard.just_pressed(KeyCode::KeyX);

    // 解析请求的攻击样式
    let requested_attack: Option<AttackAnimationStyle> =
        if let Some(attack_style) = reference_row_attack {
            Some(attack_style)
        } else if heavy_attack_pressed {
            if is_crouching {
                Some(AttackAnimationStyle::UltimateRef)
            } else {
                Some(AttackAnimationStyle::HeavyRef)
            }
        } else if projectile_pressed {
            if shift_is_pressed {
                Some(AttackAnimationStyle::NinjutsuRefRow(4))
            } else if is_crouching {
                Some(AttackAnimationStyle::WeaponProjRef)
            } else {
                Some(AttackAnimationStyle::NinjutsuRef)
            }
        } else if light_attack_pressed {
            if is_airborne {
                Some(AttackAnimationStyle::AirCombo)
            } else if is_crouching {
                Some(AttackAnimationStyle::MobilityRef)
            } else {
                Some(AttackAnimationStyle::GroundLight)
            }
        } else {
            None
        };

    if let Some(attack_style) = requested_attack {
        let attack_style = normalize_attack_style_for_overedge(attack_style, overedge_enabled);
        if runtime.cooldown <= 0.0 {
            let facing_sign = facing.copied().unwrap_or_default().sign();
            perform_knife_attack(
                &mut commands,
                &mut runtime,
                &mut attack_animation,
                KnifeAttackRequest {
                    player_entity,
                    player_transform,
                    player_sprite,
                    sprite_sheets,
                    player_velocity: &mut player_velocity,
                    player_state,
                    facing_sign,
                    knife_tuning,
                    overedge_enabled,
                    requested_style: attack_style,
                },
            );
            return;
        }

        let combo_buffer_window = if attack_style.uses_reference_sheet() {
            knife_tuning
                .combo_buffer_window_secs
                .max(REFERENCE_ATTACK_FRAME_SECS * 8.0)
        } else {
            knife_tuning.combo_buffer_window_secs
        };
        if runtime.cooldown <= combo_buffer_window {
            runtime.queued_attack = Some(attack_style);
        }
    }

    if let Some(attack_style) = runtime.queued_attack
        && runtime.cooldown <= 0.0
    {
        let facing_sign = facing.copied().unwrap_or_default().sign();
        perform_knife_attack(
            &mut commands,
            &mut runtime,
            &mut attack_animation,
            KnifeAttackRequest {
                player_entity,
                player_transform,
                player_sprite,
                sprite_sheets,
                player_velocity: &mut player_velocity,
                player_state,
                facing_sign,
                knife_tuning,
                overedge_enabled,
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
        if matches!(
            pending.attack_style,
            AttackAnimationStyle::NinjutsuRef | AttackAnimationStyle::NinjutsuRefRow(_)
        ) {
            let spawn_position = Vec3::new(
                player_transform.translation.x + PROJECTILE_MUZZLE_X_OFFSET * pending.facing,
                player_transform.translation.y + PROJECTILE_MUZZLE_Y_OFFSET,
                2.0,
            );
            spawn_projectile_with_style(
                &mut commands,
                spawn_position,
                projectile_config_for_attack_style(pending.attack_style),
                pending.facing,
            );
        } else {
            spawn_knife_slash(
                &mut commands,
                player_transform,
                &player_state,
                pending.combo_step,
                pending.facing,
                pending.overedge_enabled,
                pending.attack_style,
            );
        }
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
    mut knife_query: Query<(
        Entity,
        &mut KnifeSlash,
        Option<&KnifeSlashFeedback>,
        Option<&mut Sprite>,
        Option<&mut Transform>,
    )>,
    time: Res<Time>,
) {
    for (entity, mut knife_slash, feedback, sprite, transform) in knife_query.iter_mut() {
        knife_slash.lifetime.tick(time.delta());
        if knife_slash.lifetime.is_finished() {
            commands.entity(entity).despawn();
            continue;
        }

        if let Some(feedback) = feedback {
            let duration = knife_slash
                .lifetime
                .duration()
                .as_secs_f32()
                .max(f32::EPSILON);
            let progress = (knife_slash.lifetime.elapsed_secs() / duration).clamp(0.0, 1.0);
            let fade = 1.0 - progress;

            if let Some(mut sprite) = sprite {
                sprite.color.set_alpha(feedback.base_alpha * fade);
            }

            if let Some(mut transform) = transform {
                transform.scale = Vec3::new(
                    1.0 + feedback.visual_expand * progress,
                    1.0 + feedback.visual_expand * 0.35 * progress,
                    1.0,
                );
                transform.rotate_z(feedback.visual_spin * time.delta_secs() * fade);
            }
        }
    }
}

pub fn animate_attack_reference_action_vfx(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &mut AttackReferenceActionVfx,
        &mut Sprite,
        &mut Transform,
    )>,
) {
    let delta = time.delta_secs();

    for (entity, mut vfx, mut sprite, mut transform) in query.iter_mut() {
        vfx.timer.tick(time.delta());
        if vfx.timer.is_finished() {
            commands.entity(entity).despawn();
            continue;
        }

        let elapsed = vfx.timer.elapsed_secs();
        if let Some(atlas) = sprite.texture_atlas.as_mut() {
            let frame_offset = (elapsed / REFERENCE_ACTION_VFX_FRAME_SECS).floor() as usize;
            atlas.index = vfx.frame_start + frame_offset.min(vfx.frame_count.saturating_sub(1));
        }

        let duration = vfx.timer.duration().as_secs_f32().max(f32::EPSILON);
        let progress = (elapsed / duration).clamp(0.0, 1.0);
        let fade = 1.0 - progress;
        sprite.color.set_alpha(vfx.base_alpha * fade);
        transform.translation +=
            Vec3::new(vfx.drift.x * delta * fade, vfx.drift.y * delta * fade, 0.0);
        let grow = 1.0 + 0.08 * progress;
        transform.scale.y = grow;
        transform.scale.x = transform.scale.x.signum() * grow;
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
        Option<&KnifeSlashFeedback>,
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
    for (slash_entity, slash_transform, slash, feedback, slash_box) in knife_query.iter() {
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

            let shake_intensity = feedback
                .map(|feedback| feedback.camera_intensity)
                .unwrap_or(2.2 + slash.combo_step as f32 * 0.8);
            let shake_duration = feedback
                .map(|feedback| feedback.camera_duration)
                .unwrap_or(0.06);
            camera_impulse_writer.write(CameraImpulseEvent {
                intensity: shake_intensity,
                duration: shake_duration,
            });

            if let Some(hit_stop) = hit_stop.as_deref_mut() {
                let freeze_speed = feedback
                    .map(|feedback| feedback.hit_stop_freeze_speed)
                    .unwrap_or(0.12);
                hit_stop.trigger(slash.hit_stop_secs, freeze_speed);
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
