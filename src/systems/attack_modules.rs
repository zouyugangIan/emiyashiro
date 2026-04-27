use bevy::prelude::*;

use crate::{
    asset_paths,
    components::{FacingDirection, Player, PlayerState, ShroudState},
};

const PREVIEW_Z: f32 = 7.0;
const PREVIEW_Y_OFFSET: f32 = 178.0;
const PREVIEW_X_OFFSET: f32 = 250.0;
const PREVIEW_DURATION_SECS: f32 = 0.95;
const PREVIEW_FADE_SECS: f32 = 0.22;
const MODULE_FRAME_DURATION_SECS: f32 = 0.045;

type ReferenceAttackModulePlayerItem<'a> = (
    &'a Transform,
    &'a PlayerState,
    &'a ShroudState,
    Option<&'a FacingDirection>,
);

#[derive(Debug, Clone, Copy)]
struct ReferenceAttackModuleGrid {
    tile_size: UVec2,
    columns: u32,
    rows: u32,
    offset: UVec2,
    preview_size: Vec2,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceAttackModuleKind {
    Overview,
    AdvancedOverview,
    GroundLight,
    Heavy,
    AirCombo,
    Mobility,
    NinjutsuProjectiles,
    Ultimate,
    WeaponProjection,
}

impl ReferenceAttackModuleKind {
    fn image_path(self) -> &'static str {
        match self {
            Self::Overview => asset_paths::IMAGE_HF_SHIROU_ATTACK_MODULES_OVERVIEW,
            Self::AdvancedOverview => asset_paths::IMAGE_HF_SHIROU_ADVANCED_ATTACK_MODULES_OVERVIEW,
            Self::GroundLight => asset_paths::IMAGE_HF_SHIROU_ATTACK_GROUND_LIGHT_REFERENCE,
            Self::Heavy => asset_paths::IMAGE_HF_SHIROU_ATTACK_HEAVY_REFERENCE,
            Self::AirCombo => asset_paths::IMAGE_HF_SHIROU_ATTACK_AIR_COMBO_REFERENCE,
            Self::Mobility => asset_paths::IMAGE_HF_SHIROU_ATTACK_MOBILITY_REFERENCE,
            Self::NinjutsuProjectiles => {
                asset_paths::IMAGE_HF_SHIROU_ATTACK_NINJUTSU_PROJECTILES_REFERENCE
            }
            Self::Ultimate => asset_paths::IMAGE_HF_SHIROU_ATTACK_ULTIMATE_REFERENCE,
            Self::WeaponProjection => {
                asset_paths::IMAGE_HF_SHIROU_ATTACK_WEAPON_PROJECTION_REFERENCE
            }
        }
    }

    fn board_preview_size(self) -> Vec2 {
        match self {
            Self::Heavy => Vec2::new(590.0, 332.0),
            Self::AdvancedOverview => Vec2::new(500.0, 400.0),
            Self::Overview => Vec2::new(540.0, 360.0),
            Self::GroundLight
            | Self::AirCombo
            | Self::Mobility
            | Self::NinjutsuProjectiles
            | Self::Ultimate
            | Self::WeaponProjection => Vec2::new(540.0, 382.0),
        }
    }

    fn grid(self) -> Option<ReferenceAttackModuleGrid> {
        match self {
            Self::Overview | Self::AdvancedOverview => None,
            Self::GroundLight => Some(ReferenceAttackModuleGrid {
                tile_size: UVec2::new(183, 178),
                columns: 8,
                rows: 5,
                offset: UVec2::new(14, 82),
                preview_size: Vec2::new(238.0, 231.0),
            }),
            Self::Heavy => Some(ReferenceAttackModuleGrid {
                tile_size: UVec2::new(206, 153),
                columns: 8,
                rows: 5,
                offset: UVec2::new(15, 84),
                preview_size: Vec2::new(270.0, 201.0),
            }),
            Self::AirCombo => Some(ReferenceAttackModuleGrid {
                tile_size: UVec2::new(183, 178),
                columns: 8,
                rows: 5,
                offset: UVec2::new(14, 82),
                preview_size: Vec2::new(238.0, 231.0),
            }),
            Self::Mobility => Some(ReferenceAttackModuleGrid {
                tile_size: UVec2::new(244, 207),
                columns: 6,
                rows: 4,
                offset: UVec2::new(15, 110),
                preview_size: Vec2::new(256.0, 217.0),
            }),
            Self::NinjutsuProjectiles => Some(ReferenceAttackModuleGrid {
                tile_size: UVec2::new(183, 210),
                columns: 8,
                rows: 4,
                offset: UVec2::new(14, 94),
                preview_size: Vec2::new(230.0, 264.0),
            }),
            Self::Ultimate => Some(ReferenceAttackModuleGrid {
                tile_size: UVec2::new(183, 267),
                columns: 8,
                rows: 3,
                offset: UVec2::new(14, 92),
                preview_size: Vec2::new(212.0, 309.0),
            }),
            Self::WeaponProjection => Some(ReferenceAttackModuleGrid {
                tile_size: UVec2::new(244, 197),
                columns: 6,
                rows: 4,
                offset: UVec2::new(15, 118),
                preview_size: Vec2::new(260.0, 210.0),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct ReferenceAttackModuleInput {
    module_active: bool,
    shift_v_pressed: bool,
    light_pressed: bool,
    heavy_pressed: bool,
    projectile_pressed: bool,
    crouching: bool,
    airborne: bool,
    shift_move_pressed: bool,
}

#[derive(Component, Debug)]
pub struct ReferenceAttackModulePreview {
    timer: Timer,
    frame_timer: Timer,
    frame_count: usize,
}

fn shift_pressed(keyboard: &ButtonInput<KeyCode>) -> bool {
    keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight)
}

fn resolve_reference_attack_module(
    input: ReferenceAttackModuleInput,
) -> Option<ReferenceAttackModuleKind> {
    if input.shift_v_pressed {
        return Some(if input.module_active {
            ReferenceAttackModuleKind::AdvancedOverview
        } else {
            ReferenceAttackModuleKind::Overview
        });
    }

    if !input.module_active {
        return None;
    }

    if input.heavy_pressed {
        return Some(if input.crouching {
            ReferenceAttackModuleKind::Ultimate
        } else {
            ReferenceAttackModuleKind::Heavy
        });
    }

    if input.projectile_pressed {
        return Some(if input.crouching {
            ReferenceAttackModuleKind::WeaponProjection
        } else {
            ReferenceAttackModuleKind::NinjutsuProjectiles
        });
    }

    if input.light_pressed {
        return Some(if input.airborne {
            ReferenceAttackModuleKind::AirCombo
        } else if input.crouching {
            ReferenceAttackModuleKind::Mobility
        } else {
            ReferenceAttackModuleKind::GroundLight
        });
    }

    input
        .shift_move_pressed
        .then_some(ReferenceAttackModuleKind::Mobility)
}

fn spawn_reference_attack_module_preview(
    commands: &mut Commands,
    asset_server: &AssetServer,
    texture_atlases: &mut Assets<TextureAtlasLayout>,
    player_transform: &Transform,
    facing: Option<&FacingDirection>,
    kind: ReferenceAttackModuleKind,
    preview_query: &Query<Entity, With<ReferenceAttackModulePreview>>,
) {
    for entity in preview_query.iter() {
        commands.entity(entity).despawn();
    }

    let facing_sign = facing.copied().unwrap_or_default().sign();
    let position = Vec3::new(
        player_transform.translation.x + PREVIEW_X_OFFSET * facing_sign,
        player_transform.translation.y + PREVIEW_Y_OFFSET,
        PREVIEW_Z,
    );
    let grid = kind.grid();
    let frame_count = grid
        .map(|grid| (grid.columns * grid.rows) as usize)
        .unwrap_or(0);
    let duration = if frame_count > 0 {
        frame_count as f32 * MODULE_FRAME_DURATION_SECS
    } else {
        PREVIEW_DURATION_SECS
    };
    let texture_atlas = grid.map(|grid| TextureAtlas {
        layout: texture_atlases.add(TextureAtlasLayout::from_grid(
            grid.tile_size,
            grid.columns,
            grid.rows,
            None,
            Some(grid.offset),
        )),
        index: 0,
    });
    let custom_size = grid
        .map(|grid| grid.preview_size)
        .unwrap_or_else(|| kind.board_preview_size());

    commands.spawn((
        Sprite {
            image: asset_server.load(kind.image_path()),
            color: Color::srgba(1.0, 1.0, 1.0, 0.92),
            custom_size: Some(custom_size),
            texture_atlas,
            ..default()
        },
        Transform::from_translation(position).with_scale(Vec3::splat(0.92)),
        kind,
        ReferenceAttackModulePreview {
            timer: Timer::from_seconds(duration, TimerMode::Once),
            frame_timer: Timer::from_seconds(MODULE_FRAME_DURATION_SECS, TimerMode::Repeating),
            frame_count,
        },
    ));
}

pub fn handle_reference_attack_module_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    game_input: Option<Res<crate::systems::input::GameInput>>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    player_query: Query<ReferenceAttackModulePlayerItem, With<Player>>,
    preview_query: Query<Entity, With<ReferenceAttackModulePreview>>,
) {
    let Some((player_transform, player_state, shroud, facing)) = player_query.iter().next() else {
        return;
    };

    let shift_is_pressed = shift_pressed(&keyboard);
    let light_pressed = game_input
        .as_deref()
        .map(|input| input.action1_pressed_this_frame)
        .unwrap_or(false)
        || keyboard.just_pressed(KeyCode::KeyL)
        || keyboard.just_pressed(KeyCode::KeyU);
    let projectile_pressed = game_input
        .as_deref()
        .map(|input| input.action2_pressed_this_frame)
        .unwrap_or(false)
        || keyboard.just_pressed(KeyCode::KeyX);
    let shift_move_pressed = shift_is_pressed
        && (keyboard.just_pressed(KeyCode::KeyA)
            || keyboard.just_pressed(KeyCode::KeyD)
            || keyboard.just_pressed(KeyCode::ArrowLeft)
            || keyboard.just_pressed(KeyCode::ArrowRight));

    let module_input = ReferenceAttackModuleInput {
        module_active: shroud.is_released,
        shift_v_pressed: shift_is_pressed && keyboard.just_pressed(KeyCode::KeyV),
        light_pressed,
        heavy_pressed: keyboard.just_pressed(KeyCode::KeyK),
        projectile_pressed,
        crouching: player_state.is_crouching,
        airborne: !player_state.is_grounded,
        shift_move_pressed,
    };

    if let Some(kind) = resolve_reference_attack_module(module_input) {
        spawn_reference_attack_module_preview(
            &mut commands,
            &asset_server,
            &mut texture_atlases,
            player_transform,
            facing,
            kind,
            &preview_query,
        );
    }
}

pub fn update_reference_attack_module_previews(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &mut ReferenceAttackModulePreview,
        &mut Sprite,
        &mut Transform,
    )>,
) {
    for (entity, mut preview, mut sprite, mut transform) in query.iter_mut() {
        preview.timer.tick(time.delta());

        if preview.timer.is_finished() {
            commands.entity(entity).despawn();
            continue;
        }

        let elapsed = preview.timer.elapsed_secs();
        let duration = preview.timer.duration().as_secs_f32().max(f32::EPSILON);
        let progress = (elapsed / duration).clamp(0.0, 1.0);
        let fade_alpha = (preview.timer.remaining_secs() / PREVIEW_FADE_SECS).clamp(0.0, 1.0);
        let alpha = 0.92 * fade_alpha.min(1.0);
        let scale = 0.92 + 0.08 * (1.0 - (progress * std::f32::consts::PI).cos()) * 0.5;

        sprite.color.set_alpha(alpha);
        transform.scale = Vec3::splat(scale);

        if preview.frame_count > 0 {
            preview.frame_timer.tick(time.delta());
            if preview.frame_timer.just_finished()
                && let Some(atlas) = sprite.texture_atlas.as_mut()
            {
                atlas.index = (atlas.index + 1).min(preview.frame_count.saturating_sub(1));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_attack_modules_are_all_reachable_from_existing_buttons() {
        assert_eq!(
            resolve_reference_attack_module(ReferenceAttackModuleInput {
                shift_v_pressed: true,
                ..default()
            }),
            Some(ReferenceAttackModuleKind::Overview)
        );
        assert_eq!(
            resolve_reference_attack_module(ReferenceAttackModuleInput {
                module_active: true,
                shift_v_pressed: true,
                ..default()
            }),
            Some(ReferenceAttackModuleKind::AdvancedOverview)
        );
        assert_eq!(
            resolve_reference_attack_module(ReferenceAttackModuleInput {
                module_active: true,
                light_pressed: true,
                ..default()
            }),
            Some(ReferenceAttackModuleKind::GroundLight)
        );
        assert_eq!(
            resolve_reference_attack_module(ReferenceAttackModuleInput {
                module_active: true,
                light_pressed: true,
                airborne: true,
                ..default()
            }),
            Some(ReferenceAttackModuleKind::AirCombo)
        );
        assert_eq!(
            resolve_reference_attack_module(ReferenceAttackModuleInput {
                module_active: true,
                light_pressed: true,
                crouching: true,
                ..default()
            }),
            Some(ReferenceAttackModuleKind::Mobility)
        );
        assert_eq!(
            resolve_reference_attack_module(ReferenceAttackModuleInput {
                module_active: true,
                heavy_pressed: true,
                ..default()
            }),
            Some(ReferenceAttackModuleKind::Heavy)
        );
        assert_eq!(
            resolve_reference_attack_module(ReferenceAttackModuleInput {
                module_active: true,
                heavy_pressed: true,
                crouching: true,
                ..default()
            }),
            Some(ReferenceAttackModuleKind::Ultimate)
        );
        assert_eq!(
            resolve_reference_attack_module(ReferenceAttackModuleInput {
                module_active: true,
                projectile_pressed: true,
                ..default()
            }),
            Some(ReferenceAttackModuleKind::NinjutsuProjectiles)
        );
        assert_eq!(
            resolve_reference_attack_module(ReferenceAttackModuleInput {
                module_active: true,
                projectile_pressed: true,
                crouching: true,
                ..default()
            }),
            Some(ReferenceAttackModuleKind::WeaponProjection)
        );
    }

    #[test]
    fn reference_attack_modules_do_not_trigger_when_mode_is_inactive() {
        assert_eq!(
            resolve_reference_attack_module(ReferenceAttackModuleInput {
                light_pressed: true,
                heavy_pressed: true,
                projectile_pressed: true,
                ..default()
            }),
            None
        );
    }

    #[test]
    fn actionable_reference_modules_have_runtime_grids() {
        assert!(ReferenceAttackModuleKind::Overview.grid().is_none());
        assert!(ReferenceAttackModuleKind::AdvancedOverview.grid().is_none());

        for kind in [
            ReferenceAttackModuleKind::GroundLight,
            ReferenceAttackModuleKind::Heavy,
            ReferenceAttackModuleKind::AirCombo,
            ReferenceAttackModuleKind::Mobility,
            ReferenceAttackModuleKind::NinjutsuProjectiles,
            ReferenceAttackModuleKind::Ultimate,
            ReferenceAttackModuleKind::WeaponProjection,
        ] {
            let grid = kind.grid().expect("action module grid");
            assert!(grid.columns > 0);
            assert!(grid.rows > 0);
            assert!(grid.tile_size.x > 0);
            assert!(grid.tile_size.y > 0);
        }
    }
}
