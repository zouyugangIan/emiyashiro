//! LDtk-authored sky-city level components and runtime state.

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

pub const SKY_LEVEL_PATH: &str = "levels/sky_city_of_winds.ldtk";
pub const SKY_LEVEL_GRID: i32 = 32;
pub const SKY_LEVEL_WIDTH: f32 = 12_288.0;
pub const SKY_LEVEL_HEIGHT: f32 = 1_536.0;
pub const SKY_LEVEL_START: Vec3 = Vec3::new(144.0, 320.0, 1.0);
pub const SKY_LEVEL_KILL_Y: f32 = -160.0;

/// Runtime facts shared by physics, camera, death and checkpoint systems.
#[derive(Resource, Debug, Clone)]
pub struct SkyLevelRuntime {
    pub active: bool,
    pub level_ready: bool,
    pub player_initialized: bool,
    pub bounds: Rect,
    pub start_position: Vec3,
    pub checkpoint_position: Vec3,
    pub checkpoint_id: i32,
    pub checkpoint_needs_reconciliation: bool,
}

impl Default for SkyLevelRuntime {
    fn default() -> Self {
        Self {
            active: true,
            level_ready: false,
            player_initialized: false,
            bounds: Rect::new(0.0, 0.0, SKY_LEVEL_WIDTH, SKY_LEVEL_HEIGHT),
            start_position: SKY_LEVEL_START,
            checkpoint_position: SKY_LEVEL_START,
            checkpoint_id: 0,
            checkpoint_needs_reconciliation: false,
        }
    }
}

#[derive(Resource, Debug, Default)]
pub struct SkyEncounterState {
    pub active_arena: Option<i32>,
    pub completed_arenas: std::collections::HashSet<i32>,
}

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct SkyWallCell;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct SkyStoneCell;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct SkyCloudCell;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct SkyHazardCell;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct SkyWindCell;

#[derive(Component, Debug, Default)]
pub struct SkyCellDecorated;

#[derive(Component, Debug, Default)]
pub struct SkyMergedCollider;

#[derive(Default, Bundle, LdtkIntCell)]
pub struct SkyStoneCellBundle {
    pub wall: SkyWallCell,
    pub stone: SkyStoneCell,
}

#[derive(Default, Bundle, LdtkIntCell)]
pub struct SkyCloudCellBundle {
    pub wall: SkyWallCell,
    pub cloud: SkyCloudCell,
}

#[derive(Default, Bundle, LdtkIntCell)]
pub struct SkyHazardCellBundle {
    pub hazard: SkyHazardCell,
}

#[derive(Default, Bundle, LdtkIntCell)]
pub struct SkyWindCellBundle {
    pub wind: SkyWindCell,
}

#[derive(Component, Debug, Default)]
pub struct SkyPlayerStart;

#[derive(Default, Bundle, LdtkEntity)]
pub struct SkyPlayerStartBundle {
    pub marker: SkyPlayerStart,
    #[grid_coords]
    pub grid_coords: GridCoords,
}

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct SkyCheckpoint {
    pub id: i32,
}

impl SkyCheckpoint {
    fn from_entity(entity: &EntityInstance) -> Self {
        Self {
            id: entity.get_int_field("id").copied().unwrap_or_default(),
        }
    }
}

#[derive(Default, Bundle, LdtkEntity)]
pub struct SkyCheckpointBundle {
    #[with(SkyCheckpoint::from_entity)]
    pub checkpoint: SkyCheckpoint,
    #[grid_coords]
    pub grid_coords: GridCoords,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SkyEnemyKind {
    #[default]
    Slime,
    Familiar,
    HeroicSpirit,
}

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct SkyEnemySpawn {
    pub kind: SkyEnemyKind,
    pub arena: i32,
    pub health_multiplier: f32,
    pub patrol_range: f32,
}

impl SkyEnemySpawn {
    fn from_entity(entity: &EntityInstance) -> Self {
        let kind = match entity
            .get_enum_field("kind")
            .map(String::as_str)
            .unwrap_or("Slime")
        {
            "Familiar" => SkyEnemyKind::Familiar,
            "HeroicSpirit" => SkyEnemyKind::HeroicSpirit,
            _ => SkyEnemyKind::Slime,
        };

        Self {
            kind,
            arena: entity.get_int_field("arena").copied().unwrap_or_default(),
            health_multiplier: entity
                .get_float_field("healthMultiplier")
                .copied()
                .unwrap_or(1.0)
                .max(0.25),
            patrol_range: entity
                .get_float_field("patrolRange")
                .copied()
                .unwrap_or(96.0)
                .max(0.0),
        }
    }
}

#[derive(Default, Bundle, LdtkEntity)]
pub struct SkyEnemySpawnBundle {
    #[with(SkyEnemySpawn::from_entity)]
    pub spawn: SkyEnemySpawn,
    #[grid_coords]
    pub grid_coords: GridCoords,
}

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct SkyEncounterEnemy {
    pub arena: i32,
    pub anchor_y: f32,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct SkyCombatGate {
    pub arena: i32,
    pub height: f32,
}

impl Default for SkyCombatGate {
    fn default() -> Self {
        Self {
            arena: 0,
            height: SKY_LEVEL_GRID as f32,
        }
    }
}

impl SkyCombatGate {
    fn from_entity(entity: &EntityInstance) -> Self {
        Self {
            arena: entity.get_int_field("arena").copied().unwrap_or_default(),
            height: (entity.height as f32).max(SKY_LEVEL_GRID as f32),
        }
    }
}

#[derive(Default, Bundle, LdtkEntity)]
pub struct SkyCombatGateBundle {
    #[with(SkyCombatGate::from_entity)]
    pub gate: SkyCombatGate,
    #[grid_coords]
    pub grid_coords: GridCoords,
}

#[derive(Component, Debug, Default)]
pub struct SkyGateVisual;

#[derive(Component, Debug, Default)]
pub struct SkyGoal;

#[derive(Default, Bundle, LdtkEntity)]
pub struct SkyGoalBundle {
    pub goal: SkyGoal,
    #[grid_coords]
    pub grid_coords: GridCoords,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SkyBackdropKind {
    #[default]
    Cloud,
    Island,
    Tower,
    Windmill,
    Aqueduct,
    Tree,
    Waterfall,
    Temple,
    Crystal,
}

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct SkyBackdrop {
    pub kind: SkyBackdropKind,
    pub scale: f32,
    pub depth: i32,
}

impl SkyBackdrop {
    fn from_entity(entity: &EntityInstance) -> Self {
        let kind = match entity
            .get_enum_field("kind")
            .map(String::as_str)
            .unwrap_or("Cloud")
        {
            "Island" => SkyBackdropKind::Island,
            "Tower" => SkyBackdropKind::Tower,
            "Windmill" => SkyBackdropKind::Windmill,
            "Aqueduct" => SkyBackdropKind::Aqueduct,
            "Tree" => SkyBackdropKind::Tree,
            "Waterfall" => SkyBackdropKind::Waterfall,
            "Temple" => SkyBackdropKind::Temple,
            "Crystal" => SkyBackdropKind::Crystal,
            _ => SkyBackdropKind::Cloud,
        };

        Self {
            kind,
            scale: entity
                .get_float_field("scale")
                .copied()
                .unwrap_or(1.0)
                .max(0.1),
            depth: entity.get_int_field("depth").copied().unwrap_or_default(),
        }
    }
}

#[derive(Default, Bundle, LdtkEntity)]
pub struct SkyBackdropBundle {
    #[with(SkyBackdrop::from_entity)]
    pub backdrop: SkyBackdrop,
    #[grid_coords]
    pub grid_coords: GridCoords,
}

#[derive(Component, Debug, Default)]
pub struct SkyEntityDecorated;

#[derive(Component, Debug, Default)]
pub struct SkyLevelWorld;

/// Top-level entities owned by the sky-level session and removed together.
#[derive(Component, Debug, Default)]
pub struct SkyLevelOwned;

#[derive(Component, Debug, Default)]
pub struct SkyVictoryUi;
