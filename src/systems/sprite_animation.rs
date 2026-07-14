use crate::{
    asset_paths,
    components::{
        animation_data::{AnimationDataMap, CharacterAnimationData, PlaybackMode},
        *,
    },
};
use bevy::prelude::*;
use bevy::sprite::Anchor;
use std::collections::HashMap;

const HF_SHIROU_PROFILE: &str = include_str!("../../assets/animations/hf_shirou.ron");

/// 动画运行时参数（可在后续做难度/手感调节）
#[derive(Resource, Debug, Clone)]
pub struct AnimationRuntimeConfig {
    pub run_speed_threshold: f32,
    pub airborne_vertical_threshold: f32,
}

impl Default for AnimationRuntimeConfig {
    fn default() -> Self {
        Self {
            run_speed_threshold: 10.0,
            airborne_vertical_threshold: 90.0,
        }
    }
}

/// Small presentation-only deformation that softens hard atlas changes while
/// keeping the collision body and feet baseline untouched.
#[derive(Component, Debug, Clone)]
pub struct SpriteAnimationVisual {
    pub base_size: Vec2,
    pub base_anchor: Vec2,
    pub current_scale: Vec2,
}

impl SpriteAnimationVisual {
    pub fn new(base_size: Vec2, base_anchor: Vec2) -> Self {
        Self {
            base_size,
            base_anchor,
            current_scale: Vec2::ONE,
        }
    }
}

/// 精灵动画组件
#[derive(Component, Debug)]
pub struct SpriteAnimation {
    pub current_animation: AnimationType,
    pub frame_timer: Timer,
    pub current_frame: usize,
    pub frame_direction: i8,
    pub last_attack_trigger_serial: u32,
    pub previous_grounded: bool,
    pub apply_immediate_frame: bool,
    /// Holds the animation data cloned from the central resource.
    pub animations: HashMap<AnimationType, AnimationClipData>,
}

impl Default for SpriteAnimation {
    fn default() -> Self {
        Self {
            current_animation: AnimationType::Idle,
            frame_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            current_frame: 0,
            frame_direction: 1,
            last_attack_trigger_serial: 0,
            previous_grounded: true,
            apply_immediate_frame: true,
            animations: HashMap::new(),
        }
    }
}

type SpriteAnimationUpdateItem<'a> = (
    &'a mut SpriteAnimation,
    &'a mut Sprite,
    Option<&'a Velocity>,
    Option<&'a AttackAnimationState>,
    Option<&'a SpriteAnimationSheets>,
    Option<&'a ShroudState>,
    Option<&'a PlayerState>,
    Option<&'a LedgeTraversal>,
    Option<&'a mut SpriteAnimationVisual>,
    Option<&'a mut Anchor>,
);

type PlayerAnimationStateItem<'a> = (
    &'a mut SpriteAnimation,
    &'a mut Sprite,
    &'a PlayerState,
    &'a Velocity,
    &'a AttackAnimationState,
    Option<&'a SpriteAnimationSheets>,
    Option<&'a ShroudState>,
    Option<&'a LedgeTraversal>,
);

/// 2026推荐：先给出最小可用品质，再给理想帧数。
pub fn frame_count_guideline(animation_type: &AnimationType) -> (usize, usize) {
    match animation_type {
        AnimationType::Idle => (4, 8),
        AnimationType::Running => (5, 10),
        AnimationType::Attacking => (4, 8),
        AnimationType::Jumping => (3, 5),
        AnimationType::Crouching => (2, 4),
        AnimationType::Landing => (2, 3),
    }
}

fn validate_clip_frame_counts(
    character_name: &str,
    animations: &HashMap<AnimationType, AnimationClipData>,
) {
    for (anim_type, clip) in animations {
        let frame_count = clip.frames.len();
        let (minimum, ideal) = frame_count_guideline(anim_type);

        if frame_count < minimum {
            warn!(
                "🎞️ [{}::{:?}] 帧数过少：{} 帧（建议至少 {} 帧，理想 {} 帧）",
                character_name, anim_type, frame_count, minimum, ideal
            );
        }
    }
}

fn validate_profile(character_name: &str, profile: &CharacterAnimationData) -> Result<(), String> {
    const REQUIRED_CLIPS: [AnimationType; 6] = [
        AnimationType::Idle,
        AnimationType::Running,
        AnimationType::Attacking,
        AnimationType::Jumping,
        AnimationType::Crouching,
        AnimationType::Landing,
    ];

    for animation_type in REQUIRED_CLIPS {
        let clip = profile
            .animations
            .get(&animation_type)
            .ok_or_else(|| format!("{character_name} 缺少必需动画 {animation_type:?}"))?;
        clip.validate()
            .map_err(|reason| format!("{character_name}::{animation_type:?}: {reason}"))?;
    }

    Ok(())
}

fn resolve_target_animation(
    animation: &mut SpriteAnimation,
    player_state: &PlayerState,
    velocity: &Velocity,
    has_active_attack: bool,
    has_move_input: bool,
    is_traversing: bool,
    runtime: &AnimationRuntimeConfig,
) -> AnimationType {
    let was_grounded = animation.previous_grounded;
    animation.previous_grounded = player_state.is_grounded;

    let just_landed = !was_grounded && player_state.is_grounded;

    if is_traversing {
        AnimationType::Jumping
    } else if has_active_attack && animation.animations.contains_key(&AnimationType::Attacking) {
        AnimationType::Attacking
    } else if !player_state.is_grounded {
        AnimationType::Jumping
    } else if just_landed && animation.animations.contains_key(&AnimationType::Landing) {
        AnimationType::Landing
    } else if player_state.is_crouching {
        AnimationType::Crouching
    } else if has_move_input || velocity.x.abs() > runtime.run_speed_threshold {
        AnimationType::Running
    } else {
        AnimationType::Idle
    }
}

fn resolved_attack_style(
    animation_type: &AnimationType,
    attack_style: AttackAnimationStyle,
    shroud: Option<&ShroudState>,
    player_state: Option<&PlayerState>,
) -> AttackAnimationStyle {
    let is_overedge = shroud.map(|state| state.is_released).unwrap_or(false);
    let is_airborne = player_state.map(|s| !s.is_grounded).unwrap_or(false);
    let is_crouching = player_state.map(|s| s.is_crouching).unwrap_or(false);

    if *animation_type == AnimationType::Attacking {
        if is_overedge {
            attack_style
        } else if attack_style != AttackAnimationStyle::Normal {
            // Reference Board 模组：根据玩家状态映射
            match attack_style {
                AttackAnimationStyle::GroundLightRow(_) | AttackAnimationStyle::HeavyRefRow(_) => {
                    attack_style
                }
                AttackAnimationStyle::AirComboRow(_)
                | AttackAnimationStyle::UltimateRefRow(_)
                | AttackAnimationStyle::MobilityRefRow(_)
                | AttackAnimationStyle::NinjutsuRefRow(_)
                | AttackAnimationStyle::WeaponProjRefRow(_) => attack_style,
                AttackAnimationStyle::GroundLight | AttackAnimationStyle::AirCombo => {
                    if is_airborne {
                        AttackAnimationStyle::AirCombo
                    } else if is_crouching {
                        AttackAnimationStyle::MobilityRef
                    } else {
                        AttackAnimationStyle::GroundLight
                    }
                }
                AttackAnimationStyle::HeavyRef | AttackAnimationStyle::UltimateRef => {
                    if is_crouching {
                        AttackAnimationStyle::UltimateRef
                    } else {
                        AttackAnimationStyle::HeavyRef
                    }
                }
                AttackAnimationStyle::NinjutsuRef | AttackAnimationStyle::WeaponProjRef => {
                    if is_crouching {
                        AttackAnimationStyle::WeaponProjRef
                    } else {
                        AttackAnimationStyle::NinjutsuRef
                    }
                }
                other => other,
            }
        } else {
            AttackAnimationStyle::Normal
        }
    } else {
        attack_style
    }
}

fn bounded_frame_range(
    start: usize,
    count: usize,
    available_frame_count: usize,
) -> Option<Vec<usize>> {
    let end = start.checked_add(count)?;
    (count > 0 && end <= available_frame_count).then(|| (start..end).collect())
}

fn reference_board_row_frames(
    row: u8,
    columns: u32,
    rows: u32,
    available_frame_count: usize,
) -> Option<Vec<usize>> {
    if row == 0 || row > rows as u8 {
        return None;
    }

    let columns = columns as usize;
    if available_frame_count == columns {
        return bounded_frame_range(0, columns, available_frame_count);
    }

    let start = (row as usize - 1).checked_mul(columns)?;
    bounded_frame_range(start, columns, available_frame_count)
}

fn overedge_attack_frames(
    attack_style: AttackAnimationStyle,
    available_frame_count: usize,
) -> Option<Vec<usize>> {
    match attack_style {
        AttackAnimationStyle::OveredgeRelease => bounded_frame_range(
            0,
            asset_paths::HF_SHIROU_OVEREDGE_RELEASE_FRAME_COUNT,
            available_frame_count,
        ),
        AttackAnimationStyle::OveredgeLight1 => bounded_frame_range(
            2,
            asset_paths::HF_SHIROU_OVEREDGE_LIGHT_ATTACK_SEGMENT_FRAME_COUNT,
            available_frame_count,
        ),
        AttackAnimationStyle::OveredgeLight2 => bounded_frame_range(
            5,
            asset_paths::HF_SHIROU_OVEREDGE_LIGHT_ATTACK_SEGMENT_FRAME_COUNT,
            available_frame_count,
        ),
        AttackAnimationStyle::OveredgeLight3 => bounded_frame_range(
            8,
            asset_paths::HF_SHIROU_OVEREDGE_LIGHT_ATTACK_SEGMENT_FRAME_COUNT,
            available_frame_count,
        ),
        AttackAnimationStyle::OveredgeHeavy => {
            (available_frame_count > 0).then(|| (0..available_frame_count).collect())
        }
        AttackAnimationStyle::GroundLightRow(row) => reference_board_row_frames(
            row,
            asset_paths::REFERENCE_BOARD_GROUND_LIGHT_COLS,
            asset_paths::REFERENCE_BOARD_GROUND_LIGHT_ROWS,
            available_frame_count,
        ),
        AttackAnimationStyle::AirComboRow(row) => reference_board_row_frames(
            row,
            asset_paths::REFERENCE_BOARD_AIR_COMBO_COLS,
            asset_paths::REFERENCE_BOARD_AIR_COMBO_ROWS,
            available_frame_count,
        ),
        AttackAnimationStyle::HeavyRefRow(row) => reference_board_row_frames(
            row,
            asset_paths::REFERENCE_BOARD_HEAVY_COLS,
            asset_paths::REFERENCE_BOARD_HEAVY_ROWS,
            available_frame_count,
        ),
        AttackAnimationStyle::UltimateRefRow(row) => reference_board_row_frames(
            row,
            asset_paths::REFERENCE_BOARD_ULTIMATE_COLS,
            asset_paths::REFERENCE_BOARD_ULTIMATE_ROWS,
            available_frame_count,
        ),
        AttackAnimationStyle::MobilityRefRow(row) => reference_board_row_frames(
            row,
            asset_paths::REFERENCE_BOARD_MOBILITY_COLS,
            asset_paths::REFERENCE_BOARD_MOBILITY_ROWS,
            available_frame_count,
        ),
        AttackAnimationStyle::NinjutsuRefRow(row) => reference_board_row_frames(
            row,
            asset_paths::REFERENCE_BOARD_NINJUTSU_COLS,
            asset_paths::REFERENCE_BOARD_NINJUTSU_ROWS,
            available_frame_count,
        ),
        AttackAnimationStyle::WeaponProjRefRow(row) => reference_board_row_frames(
            row,
            asset_paths::REFERENCE_BOARD_WEAPON_PROJ_COLS,
            asset_paths::REFERENCE_BOARD_WEAPON_PROJ_ROWS,
            available_frame_count,
        ),
        // Reference Board 模组播放全部帧
        AttackAnimationStyle::GroundLight
        | AttackAnimationStyle::AirCombo
        | AttackAnimationStyle::HeavyRef
        | AttackAnimationStyle::UltimateRef
        | AttackAnimationStyle::MobilityRef
        | AttackAnimationStyle::NinjutsuRef
        | AttackAnimationStyle::WeaponProjRef
        | AttackAnimationStyle::AdvanceRef => {
            (available_frame_count > 0).then(|| (0..available_frame_count).collect())
        }
        AttackAnimationStyle::Normal => None,
    }
}

fn base_sheet_frame_count(
    sprite_sheets: Option<&SpriteAnimationSheets>,
    animation_type: &AnimationType,
) -> Option<usize> {
    let sprite_sheets = sprite_sheets?;

    match animation_type.sprite_sheet_kind() {
        SpriteSheetKind::Core => Some(asset_paths::HF_SHIROU_CORE_FRAME_COUNT),
        SpriteSheetKind::Running => {
            if sprite_sheets.running_texture == sprite_sheets.core_texture
                && sprite_sheets.running_layout == sprite_sheets.core_layout
            {
                Some(asset_paths::HF_SHIROU_CORE_FRAME_COUNT)
            } else {
                Some(asset_paths::HF_SHIROU_RUN_FRAME_COUNT)
            }
        }
        SpriteSheetKind::Attacking => {
            if sprite_sheets.attacking_texture == sprite_sheets.core_texture
                && sprite_sheets.attacking_layout == sprite_sheets.core_layout
            {
                Some(asset_paths::HF_SHIROU_CORE_FRAME_COUNT)
            } else {
                Some(asset_paths::HF_SHIROU_ATTACK_FRAME_COUNT)
            }
        }
    }
}

fn selected_sheet_frame_count(
    sprite_sheets: Option<&SpriteAnimationSheets>,
    animation_type: &AnimationType,
    attack_style: AttackAnimationStyle,
) -> Option<usize> {
    if *animation_type == AnimationType::Attacking
        && let Some(frame_count) =
            sprite_sheets.and_then(|sheets| sheets.attacking_frame_count(attack_style))
    {
        return Some(frame_count);
    }

    base_sheet_frame_count(sprite_sheets, animation_type)
}

fn sanitize_clip_frames(
    clip: &mut AnimationClipData,
    available_frame_count: Option<usize>,
    character_frame: usize,
) {
    let Some(available_frame_count) = available_frame_count else {
        return;
    };

    if available_frame_count == 0 {
        return;
    }

    clip.frames.retain(|frame| *frame < available_frame_count);
    if clip.frames.is_empty() {
        clip.frames
            .push(character_frame.min(available_frame_count - 1));
    }
}

fn resolved_animation_clip(
    animation: &SpriteAnimation,
    animation_type: &AnimationType,
    sprite_sheets: Option<&SpriteAnimationSheets>,
    shroud: Option<&ShroudState>,
    attack_style: AttackAnimationStyle,
    player_state: Option<&PlayerState>,
) -> Option<AnimationClipData> {
    let mut clip = animation.animations.get(animation_type)?.clone();
    let attack_style = resolved_attack_style(animation_type, attack_style, shroud, player_state);

    if let Some(frame_count) =
        sprite_sheets.and_then(|sheets| sheets.attacking_frame_count(attack_style))
        && let Some(frames) = overedge_attack_frames(attack_style, frame_count)
    {
        clip.frames = frames;
    }

    sanitize_clip_frames(
        &mut clip,
        selected_sheet_frame_count(sprite_sheets, animation_type, attack_style),
        animation.current_frame,
    );

    Some(clip)
}

fn current_clip_is_blocking(animation: &SpriteAnimation, clip: &AnimationClipData) -> bool {
    let is_once = clip.playback_mode == PlaybackMode::Once;
    let not_finished = animation.current_frame + 1 < clip.frames.len();
    is_once && not_finished
}

fn apply_animation_change(
    animation: &mut SpriteAnimation,
    new_animation: AnimationType,
    horizontal_speed_abs: f32,
) {
    let Some(new_clip) = animation.animations.get(&new_animation) else {
        return;
    };

    let duration = new_clip.frame_duration_for_speed(horizontal_speed_abs);

    let previous_animation = animation.current_animation.clone();
    let entry_frame = match (&previous_animation, &new_animation) {
        // A compressed passing pose avoids snapping straight from a tall idle
        // silhouette into the widest stride in the run sheet.
        (AnimationType::Idle | AnimationType::Crouching, AnimationType::Running)
            if new_clip.frames.len() > 1 =>
        {
            1
        }
        _ => 0,
    };

    animation.current_animation = new_animation;
    animation.current_frame = entry_frame;
    animation.frame_direction = 1;
    animation
        .frame_timer
        .set_duration(std::time::Duration::from_secs_f32(duration));
    animation.frame_timer.reset();
    animation.apply_immediate_frame = true;
}

fn next_frame_index(
    current_frame: usize,
    frame_count: usize,
    playback_mode: PlaybackMode,
    direction: &mut i8,
) -> usize {
    if frame_count <= 1 {
        return 0;
    }

    match playback_mode {
        PlaybackMode::Loop => (current_frame + 1) % frame_count,
        PlaybackMode::Once => (current_frame + 1).min(frame_count - 1),
        PlaybackMode::PingPong => {
            if *direction >= 0 {
                if current_frame + 1 >= frame_count {
                    *direction = -1;
                    frame_count.saturating_sub(2)
                } else {
                    current_frame + 1
                }
            } else if current_frame == 0 {
                *direction = 1;
                1
            } else {
                current_frame - 1
            }
        }
    }
}

fn advance_frame_index(
    mut current_frame: usize,
    frame_count: usize,
    playback_mode: PlaybackMode,
    direction: &mut i8,
    completed_intervals: u32,
) -> usize {
    for _ in 0..completed_intervals {
        let next_frame = next_frame_index(current_frame, frame_count, playback_mode, direction);
        if next_frame == current_frame && playback_mode == PlaybackMode::Once {
            break;
        }
        current_frame = next_frame;
    }
    current_frame
}

fn apply_atlas_frame(sprite: &mut Sprite, atlas_index: usize) {
    if let Some(ref mut atlas) = sprite.texture_atlas {
        atlas.index = atlas_index;
    }
}

fn apply_animation_sheet(
    sprite: &mut Sprite,
    sprite_sheets: Option<&SpriteAnimationSheets>,
    animation_type: &AnimationType,
    attack_style: AttackAnimationStyle,
) {
    let Some(sprite_sheets) = sprite_sheets else {
        return;
    };

    let Some((target_texture, target_layout)) =
        sprite_sheets.select_sheet_for_attack_style(animation_type, attack_style)
    else {
        return;
    };
    let texture_changed = sprite.image != target_texture.clone();
    if texture_changed {
        sprite.image = target_texture.clone();
    }

    if let Some(ref mut atlas) = sprite.texture_atlas {
        let layout_changed = atlas.layout != target_layout.clone();
        if layout_changed {
            atlas.layout = target_layout.clone();
        }
        if texture_changed || layout_changed {
            atlas.index = 0;
        }
    } else {
        sprite.texture_atlas = Some(TextureAtlas {
            layout: target_layout.clone(),
            index: 0,
        });
    }
}

fn paced_frame_duration(
    clip: &AnimationClipData,
    horizontal_speed_abs: f32,
    animation_type: &AnimationType,
    attack_state: Option<&AttackAnimationState>,
) -> f32 {
    let fallback = clip.frame_duration_for_speed(horizontal_speed_abs);
    if *animation_type != AnimationType::Attacking {
        return fallback;
    }

    let Some(attack) = attack_state.filter(|state| state.duration > 0.0) else {
        return fallback;
    };

    // Fast action sheets need to finish inside the gameplay lock window. The
    // bounds preserve readability while still allowing eight-frame attacks to
    // complete during a short Ninja-Gaiden-style cancel window.
    (attack.duration / clip.frames.len().max(1) as f32).clamp(0.032, 0.12)
}

fn airborne_frame_position(
    frame_count: usize,
    vertical_speed: f32,
    vertical_threshold: f32,
    traversal: Option<&LedgeTraversal>,
) -> usize {
    if frame_count <= 1 {
        return 0;
    }

    if let Some(traversal) = traversal.filter(|state| state.is_active()) {
        if traversal.is_hanging() {
            return 1.min(frame_count - 1);
        }
        return if traversal.climb_progress().unwrap_or_default() < 0.45 {
            1.min(frame_count - 1)
        } else {
            0
        };
    }

    let threshold = vertical_threshold.max(1.0);
    if vertical_speed > threshold {
        0
    } else if vertical_speed < -threshold {
        frame_count - 1
    } else {
        1.min(frame_count - 1)
    }
}

fn animation_visual_target(
    animation_type: &AnimationType,
    frame_position: usize,
    velocity: Option<&Velocity>,
    traversal: Option<&LedgeTraversal>,
) -> Vec2 {
    if let Some(traversal) = traversal.filter(|state| state.is_active()) {
        return if traversal.is_hanging() {
            Vec2::new(1.015, 0.985)
        } else {
            Vec2::new(0.975, 1.03)
        };
    }

    match animation_type {
        AnimationType::Idle => {
            if frame_position == 1 || frame_position == 2 {
                Vec2::new(0.997, 1.006)
            } else {
                Vec2::ONE
            }
        }
        AnimationType::Running => match frame_position % 5 {
            0 | 3 => Vec2::new(1.018, 0.982),
            1 | 4 => Vec2::new(0.988, 1.014),
            _ => Vec2::ONE,
        },
        AnimationType::Jumping => {
            let vertical_speed = velocity.map(|value| value.y).unwrap_or_default();
            if vertical_speed > 90.0 {
                Vec2::new(0.97, 1.035)
            } else if vertical_speed < -90.0 {
                Vec2::new(0.985, 1.02)
            } else {
                Vec2::new(1.025, 0.975)
            }
        }
        AnimationType::Crouching => Vec2::new(1.018, 0.985),
        AnimationType::Landing if frame_position == 0 => Vec2::new(1.045, 0.94),
        AnimationType::Landing | AnimationType::Attacking => Vec2::ONE,
    }
}

fn baseline_compensated_anchor_y(base_anchor_y: f32, scale_y: f32) -> f32 {
    (0.5 + base_anchor_y) / scale_y.max(0.01) - 0.5
}

fn apply_animation_visual(
    sprite: &mut Sprite,
    visual: Option<&mut SpriteAnimationVisual>,
    anchor: Option<&mut Anchor>,
    target_scale: Vec2,
    delta_secs: f32,
) {
    let Some(visual) = visual else {
        return;
    };

    let blend = 1.0 - (-22.0 * delta_secs.max(0.0)).exp();
    visual.current_scale = visual.current_scale.lerp(target_scale, blend);
    sprite.custom_size = Some(visual.base_size * visual.current_scale);

    if let Some(anchor) = anchor {
        // Counter-scale the normalized anchor so the world-space foot line
        // remains fixed while the sprite eases through squash/stretch.
        anchor.0.x = visual.base_anchor.x;
        anchor.0.y = baseline_compensated_anchor_y(visual.base_anchor.y, visual.current_scale.y);
    }
}

/// 加载编译时内嵌的角色动画配置。
///
/// 目前只有 HF 士郎使用图集主链。显式列出配置可以避免运行时工作目录不同导致
/// `assets/animations` 找不到，也不会把误放入目录的 RON 自动当成正式角色。
pub fn load_animation_data() -> AnimationDataMap {
    let mut animation_map = AnimationDataMap::default();

    let profile: CharacterAnimationData = ron::from_str(HF_SHIROU_PROFILE)
        .unwrap_or_else(|error| panic!("HF 士郎动画配置无法解析: {error}"));
    validate_profile("hf_shirou", &profile)
        .unwrap_or_else(|error| panic!("HF 士郎动画配置无效: {error}"));
    validate_clip_frame_counts("hf_shirou", &profile.animations);
    animation_map.0.insert("hf_shirou".to_string(), profile);

    animation_map
}

/// 创建角色动画组件
pub fn create_character_animation(
    anim_data_map: &AnimationDataMap,
    character_name: &str,
) -> SpriteAnimation {
    let Some(character_data) = anim_data_map.0.get(character_name) else {
        warn!(
            "Animation data for '{}' not found, using default controller",
            character_name
        );
        return SpriteAnimation::default();
    };

    let starting_animation = if character_data.animations.contains_key(&AnimationType::Idle) {
        AnimationType::Idle
    } else {
        character_data
            .animations
            .keys()
            .next()
            .cloned()
            .unwrap_or(AnimationType::Idle)
    };

    let initial_clip = character_data
        .animations
        .get(&starting_animation)
        .or_else(|| character_data.animations.values().next());

    let initial_duration = initial_clip
        .map(|clip| clip.frame_duration_for_speed(0.0))
        .unwrap_or(0.1);

    SpriteAnimation {
        current_animation: starting_animation,
        frame_timer: Timer::from_seconds(initial_duration, TimerMode::Repeating),
        current_frame: 0,
        frame_direction: 1,
        last_attack_trigger_serial: 0,
        previous_grounded: true,
        apply_immediate_frame: true,
        animations: character_data.animations.clone(),
    }
}

/// 更新精灵动画系统
pub fn update_sprite_animations(
    time: Res<Time>,
    runtime_config: Option<Res<AnimationRuntimeConfig>>,
    mut query: Query<SpriteAnimationUpdateItem>,
) {
    let default_runtime = AnimationRuntimeConfig::default();
    let runtime = runtime_config.as_deref().unwrap_or(&default_runtime);

    for (
        mut animation,
        mut sprite,
        velocity,
        attack_state,
        sprite_sheets,
        shroud,
        player_state,
        traversal,
        mut visual,
        mut anchor,
    ) in query.iter_mut()
    {
        let current_key = animation.current_animation.clone();
        let attack_style = attack_state
            .map(|state| state.style)
            .unwrap_or(AttackAnimationStyle::Normal);

        let Some(current_clip) = resolved_animation_clip(
            &animation,
            &current_key,
            sprite_sheets,
            shroud,
            attack_style,
            player_state,
        ) else {
            continue;
        };

        if current_clip.frames.is_empty() {
            continue;
        }

        if animation.current_frame >= current_clip.frames.len() {
            animation.current_frame = current_clip.frames.len().saturating_sub(1);
        }

        let sheet_attack_style =
            resolved_attack_style(&current_key, attack_style, shroud, player_state);
        apply_animation_sheet(&mut sprite, sprite_sheets, &current_key, sheet_attack_style);

        let horizontal_speed_abs = velocity.map(|v| v.x.abs()).unwrap_or(0.0);
        let target_duration = paced_frame_duration(
            &current_clip,
            horizontal_speed_abs,
            &current_key,
            attack_state,
        );
        let current_duration = animation.frame_timer.duration().as_secs_f32();

        if (current_duration - target_duration).abs() > 0.002 {
            animation
                .frame_timer
                .set_duration(std::time::Duration::from_secs_f32(target_duration));
        }

        if current_key == AnimationType::Jumping {
            animation.current_frame = airborne_frame_position(
                current_clip.frames.len(),
                velocity.map(|value| value.y).unwrap_or_default(),
                runtime.airborne_vertical_threshold,
                traversal,
            );
            if let Some(atlas_idx) = current_clip.frames.get(animation.current_frame).copied() {
                apply_atlas_frame(&mut sprite, atlas_idx);
            }
            animation.apply_immediate_frame = false;
            animation.frame_timer.reset();
        } else if animation.apply_immediate_frame {
            if let Some(first_atlas_idx) = current_clip.frames.get(animation.current_frame).copied()
            {
                apply_atlas_frame(&mut sprite, first_atlas_idx);
            }
            animation.apply_immediate_frame = false;
        }

        if current_key != AnimationType::Jumping {
            animation.frame_timer.tick(time.delta());
            let completed_intervals = animation.frame_timer.times_finished_this_tick();
            if completed_intervals > 0 {
                let frame_count = current_clip.frames.len();
                animation.current_frame = advance_frame_index(
                    animation.current_frame,
                    frame_count,
                    current_clip.playback_mode,
                    &mut animation.frame_direction,
                    completed_intervals,
                );

                if let Some(atlas_idx) = current_clip.frames.get(animation.current_frame).copied() {
                    apply_atlas_frame(&mut sprite, atlas_idx);
                }
            }
        }

        let target_scale =
            animation_visual_target(&current_key, animation.current_frame, velocity, traversal);
        apply_animation_visual(
            &mut sprite,
            visual.as_deref_mut(),
            anchor.as_deref_mut(),
            target_scale,
            time.delta_secs(),
        );
    }
}

pub fn tick_attack_animation_states(
    time: Res<Time>,
    mut query: Query<&mut AttackAnimationState, With<Player>>,
) {
    for mut attack_state in query.iter_mut() {
        attack_state.tick(time.delta_secs());
    }
}

/// 根据玩家状态更新动画
pub fn update_character_animation_state(
    mut query: Query<PlayerAnimationStateItem, With<Player>>,
    game_input: Option<Res<crate::systems::input::GameInput>>,
    runtime_config: Option<Res<AnimationRuntimeConfig>>,
) {
    let default_runtime = AnimationRuntimeConfig::default();
    let runtime = runtime_config.as_deref().unwrap_or(&default_runtime);
    let has_move_input = game_input
        .as_deref()
        .map(|input| input.move_left || input.move_right)
        .unwrap_or(false);

    for (
        mut animation,
        mut sprite,
        player_state,
        velocity,
        attack_state,
        sprite_sheets,
        shroud,
        traversal,
    ) in query.iter_mut()
    {
        let is_traversing = traversal.is_some_and(LedgeTraversal::is_active);
        let attack_retriggered = attack_state.is_active()
            && attack_state.trigger_serial != animation.last_attack_trigger_serial;
        let new_animation = resolve_target_animation(
            &mut animation,
            player_state,
            velocity,
            attack_state.is_active(),
            has_move_input,
            is_traversing,
            runtime,
        );

        let clip_blocks_switch = resolved_animation_clip(
            &animation,
            &animation.current_animation,
            sprite_sheets,
            shroud,
            attack_state.style,
            Some(player_state),
        )
        .as_ref()
        .map(|clip| current_clip_is_blocking(&animation, clip))
        .unwrap_or(false);
        let landing_cancelled_into_run = animation.current_animation == AnimationType::Landing
            && new_animation == AnimationType::Running;
        if clip_blocks_switch
            && new_animation != animation.current_animation
            && !attack_retriggered
            && !is_traversing
            && !landing_cancelled_into_run
        {
            continue;
        }

        if animation.current_animation != new_animation || attack_retriggered {
            apply_animation_change(&mut animation, new_animation.clone(), velocity.x.abs());
            let attack_style = resolved_attack_style(
                &new_animation,
                attack_state.style,
                shroud,
                Some(player_state),
            );
            apply_animation_sheet(&mut sprite, sprite_sheets, &new_animation, attack_style);
            if new_animation == AnimationType::Attacking {
                animation.last_attack_trigger_serial = attack_state.trigger_serial;
            }

            if let Some(new_clip) = resolved_animation_clip(
                &animation,
                &new_animation,
                sprite_sheets,
                shroud,
                attack_state.style,
                Some(player_state),
            ) {
                if let Some(entry_atlas_idx) = new_clip.frames.get(animation.current_frame).copied()
                {
                    apply_atlas_frame(&mut sprite, entry_atlas_idx);
                }
                info!(
                    "🎭 切换动画: {:?} ({}帧, 模式: {:?})",
                    new_animation,
                    new_clip.frames.len(),
                    new_clip.playback_mode
                );
            } else {
                warn!(
                    "Requested animation {:?} not found in current character profile",
                    new_animation
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::animation_data::AnimationClipData;
    use std::marker::PhantomData;

    fn test_image_handle(id: u128) -> Handle<Image> {
        Handle::Uuid(uuid::Uuid::from_u128(id), PhantomData)
    }

    fn test_layout_handle(id: u128) -> Handle<TextureAtlasLayout> {
        Handle::Uuid(uuid::Uuid::from_u128(id), PhantomData)
    }

    fn distinct_test_sheets() -> SpriteAnimationSheets {
        SpriteAnimationSheets {
            core_texture: test_image_handle(0xA001),
            core_layout: test_layout_handle(0xB001),
            running_texture: test_image_handle(0xA002),
            running_layout: test_layout_handle(0xB002),
            attacking_texture: test_image_handle(0xA003),
            attacking_layout: test_layout_handle(0xB003),
            overedge_light_attacking_texture: Some(test_image_handle(0xA004)),
            overedge_light_attacking_layout: Some(test_layout_handle(0xB004)),
            overedge_light_attacking_frame_count:
                asset_paths::HF_SHIROU_OVEREDGE_LIGHT_ATTACK_FRAME_COUNT,
            overedge_heavy_attacking_texture: Some(test_image_handle(0xA005)),
            overedge_heavy_attacking_layout: Some(test_layout_handle(0xB005)),
            overedge_heavy_attacking_frame_count:
                asset_paths::HF_SHIROU_OVEREDGE_HEAVY_ATTACK_FRAME_COUNT,
            reference_ground_light_texture: Some(test_image_handle(0xA006)),
            reference_ground_light_row_textures: vec![
                test_image_handle(0xA106),
                test_image_handle(0xA107),
                test_image_handle(0xA108),
                test_image_handle(0xA109),
                test_image_handle(0xA10A),
            ],
            reference_ground_light_layout: Some(test_layout_handle(0xB006)),
            reference_ground_light_frame_count: asset_paths::REFERENCE_BOARD_GROUND_LIGHT_COLS
                as usize,
            reference_air_combo_texture: Some(test_image_handle(0xA007)),
            reference_air_combo_row_textures: vec![
                test_image_handle(0xA10B),
                test_image_handle(0xA10C),
                test_image_handle(0xA10D),
                test_image_handle(0xA10E),
                test_image_handle(0xA10F),
            ],
            reference_air_combo_layout: Some(test_layout_handle(0xB007)),
            reference_air_combo_frame_count: asset_paths::REFERENCE_BOARD_AIR_COMBO_COLS as usize,
            reference_heavy_texture: Some(test_image_handle(0xA008)),
            reference_heavy_row_textures: vec![
                test_image_handle(0xA110),
                test_image_handle(0xA111),
                test_image_handle(0xA112),
                test_image_handle(0xA113),
                test_image_handle(0xA114),
            ],
            reference_heavy_layout: Some(test_layout_handle(0xB008)),
            reference_heavy_frame_count: asset_paths::REFERENCE_BOARD_HEAVY_COLS as usize,
            reference_ultimate_texture: Some(test_image_handle(0xA009)),
            reference_ultimate_row_textures: vec![
                test_image_handle(0xA115),
                test_image_handle(0xA116),
                test_image_handle(0xA117),
            ],
            reference_ultimate_layout: Some(test_layout_handle(0xB009)),
            reference_ultimate_frame_count: asset_paths::REFERENCE_BOARD_ULTIMATE_COLS as usize,
            reference_mobility_texture: Some(test_image_handle(0xA00A)),
            reference_mobility_row_textures: vec![
                test_image_handle(0xA118),
                test_image_handle(0xA119),
                test_image_handle(0xA11A),
                test_image_handle(0xA11B),
            ],
            reference_mobility_layout: Some(test_layout_handle(0xB00A)),
            reference_mobility_frame_count: asset_paths::REFERENCE_BOARD_MOBILITY_COLS as usize,
            reference_ninjutsu_texture: Some(test_image_handle(0xA00B)),
            reference_ninjutsu_row_textures: vec![
                test_image_handle(0xA11C),
                test_image_handle(0xA11D),
                test_image_handle(0xA11E),
                test_image_handle(0xA11F),
            ],
            reference_ninjutsu_layout: Some(test_layout_handle(0xB00B)),
            reference_ninjutsu_frame_count: asset_paths::REFERENCE_BOARD_NINJUTSU_COLS as usize,
            reference_weapon_proj_texture: Some(test_image_handle(0xA00C)),
            reference_weapon_proj_row_textures: vec![
                test_image_handle(0xA120),
                test_image_handle(0xA121),
                test_image_handle(0xA122),
                test_image_handle(0xA123),
            ],
            reference_weapon_proj_layout: Some(test_layout_handle(0xB00C)),
            reference_weapon_proj_frame_count: asset_paths::REFERENCE_BOARD_WEAPON_PROJ_COLS
                as usize,
            reference_advance_texture: None,
            reference_advance_layout: None,
            reference_advance_frame_count: 0,
        }
    }

    #[test]
    fn test_frame_guidelines_are_monotonic() {
        let all_types = [
            AnimationType::Idle,
            AnimationType::Running,
            AnimationType::Attacking,
            AnimationType::Jumping,
            AnimationType::Crouching,
            AnimationType::Landing,
        ];

        for animation_type in all_types {
            let (minimum, ideal) = frame_count_guideline(&animation_type);
            assert!(minimum > 0);
            assert!(ideal >= minimum);
        }
    }

    #[test]
    fn test_airborne_pose_tracks_physics_and_traversal() {
        assert_eq!(airborne_frame_position(3, 240.0, 90.0, None), 0);
        assert_eq!(airborne_frame_position(3, 20.0, 90.0, None), 1);
        assert_eq!(airborne_frame_position(3, -240.0, 90.0, None), 2);

        let hanging = LedgeTraversal {
            phase: LedgeTraversalPhase::Hanging {
                anchor: Vec2::ZERO,
                direction: 1.0,
                elapsed_secs: 0.1,
            },
            ..default()
        };
        assert_eq!(
            airborne_frame_position(3, -500.0, 90.0, Some(&hanging)),
            1,
            "ledge hang must use the authored overhead pose instead of the fall pose"
        );
    }

    #[test]
    fn test_attack_frames_fit_the_gameplay_action_window() {
        let clip = AnimationClipData {
            frames: (0..8).collect(),
            frame_duration: 0.07,
            playback_mode: PlaybackMode::Once,
            speed_scale_by_velocity: false,
            speed_reference: 150.0,
            min_frame_duration: 0.05,
        };
        let mut attack = AttackAnimationState::default();
        attack.trigger_with_style(0.32, AttackAnimationStyle::GroundLightRow(1));

        let duration = paced_frame_duration(&clip, 0.0, &AnimationType::Attacking, Some(&attack));
        assert!((duration - 0.04).abs() < f32::EPSILON);
        assert!((duration * clip.frames.len() as f32 - attack.duration).abs() < 0.001);
    }

    #[test]
    fn test_visual_squash_preserves_sprite_baseline() {
        let base_anchor = crate::systems::game::PLAYER_VISUAL_BASELINE_ANCHOR_Y;
        let base_height = crate::systems::game::PLAYER_RENDER_SIZE.y;
        let base_bottom = -(0.5 + base_anchor) * base_height;

        for scale_y in [0.94, 0.975, 1.02, 1.035] {
            let compensated = baseline_compensated_anchor_y(base_anchor, scale_y);
            let scaled_bottom = -(0.5 + compensated) * base_height * scale_y;
            assert!((scaled_bottom - base_bottom).abs() < 0.001);
        }
    }

    #[test]
    fn test_ping_pong_direction_switch() {
        let mut direction = 1;
        let mut frame = 0usize;

        frame = next_frame_index(frame, 4, PlaybackMode::PingPong, &mut direction);
        assert_eq!(frame, 1);
        frame = next_frame_index(frame, 4, PlaybackMode::PingPong, &mut direction);
        assert_eq!(frame, 2);
        frame = next_frame_index(frame, 4, PlaybackMode::PingPong, &mut direction);
        assert_eq!(frame, 3);
        frame = next_frame_index(frame, 4, PlaybackMode::PingPong, &mut direction);
        assert_eq!(frame, 2);
        frame = next_frame_index(frame, 4, PlaybackMode::PingPong, &mut direction);
        assert_eq!(frame, 1);
        frame = next_frame_index(frame, 4, PlaybackMode::PingPong, &mut direction);
        assert_eq!(frame, 0);
    }

    #[test]
    fn test_animation_catches_up_after_long_frame() {
        let mut direction = 1;
        assert_eq!(
            advance_frame_index(0, 5, PlaybackMode::Loop, &mut direction, 3),
            3
        );
        assert_eq!(
            advance_frame_index(0, 3, PlaybackMode::Once, &mut direction, 8),
            2,
            "Once 动画应停在末帧，不应因卡顿越界"
        );
    }

    #[test]
    fn test_embedded_profile_is_the_only_runtime_profile() {
        let profiles = load_animation_data();
        assert_eq!(profiles.0.len(), 1);
        assert!(profiles.0.contains_key("hf_shirou"));
    }

    #[test]
    fn test_apply_animation_sheet_recreates_missing_atlas() {
        let sheets = distinct_test_sheets();
        let mut sprite = Sprite {
            image: sheets.core_texture.clone(),
            texture_atlas: None,
            ..default()
        };

        apply_animation_sheet(
            &mut sprite,
            Some(&sheets),
            &AnimationType::Attacking,
            AttackAnimationStyle::GroundLightRow(2),
        );

        assert_eq!(
            sprite.image,
            sheets
                .reference_ground_light_row_textures
                .get(1)
                .cloned()
                .expect("reference row texture")
        );
        let atlas = sprite.texture_atlas.as_ref().expect("texture atlas");
        assert_eq!(
            atlas.layout,
            sheets
                .reference_ground_light_layout
                .clone()
                .expect("reference layout")
        );
        assert_eq!(atlas.index, 0);
    }

    #[test]
    fn test_missing_reference_sheet_falls_back_to_base_attack_sheet() {
        let sheets = distinct_test_sheets();
        let mut sprite = Sprite {
            image: sheets
                .reference_ground_light_texture
                .clone()
                .expect("old reference texture"),
            texture_atlas: Some(TextureAtlas {
                layout: sheets
                    .reference_ground_light_layout
                    .clone()
                    .expect("old reference layout"),
                index: 9,
            }),
            ..default()
        };

        apply_animation_sheet(
            &mut sprite,
            Some(&sheets),
            &AnimationType::Attacking,
            AttackAnimationStyle::AdvanceRef,
        );

        assert_eq!(sprite.image, sheets.attacking_texture);
        let atlas = sprite.texture_atlas.as_ref().expect("texture atlas");
        assert_eq!(atlas.layout, sheets.attacking_layout);
    }

    #[test]
    fn test_non_attack_sheet_restores_core_after_reference_attack() {
        let sheets = distinct_test_sheets();
        let mut sprite = Sprite {
            image: sheets
                .reference_ground_light_texture
                .clone()
                .expect("old reference texture"),
            texture_atlas: Some(TextureAtlas {
                layout: sheets
                    .reference_ground_light_layout
                    .clone()
                    .expect("old reference layout"),
                index: 17,
            }),
            ..default()
        };

        apply_animation_sheet(
            &mut sprite,
            Some(&sheets),
            &AnimationType::Idle,
            AttackAnimationStyle::GroundLightRow(3),
        );

        assert_eq!(sprite.image, sheets.core_texture);
        let atlas = sprite.texture_atlas.as_ref().expect("texture atlas");
        assert_eq!(atlas.layout, sheets.core_layout);
    }

    #[test]
    fn test_update_sprite_animations_restores_missing_core_atlas() {
        let sheets = distinct_test_sheets();
        let mut animations = std::collections::HashMap::new();
        animations.insert(
            AnimationType::Idle,
            AnimationClipData {
                frames: vec![0, 1],
                frame_duration: 0.1,
                playback_mode: PlaybackMode::Loop,
                speed_scale_by_velocity: false,
                speed_reference: 150.0,
                min_frame_duration: 0.05,
            },
        );

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Update, update_sprite_animations);
        app.world_mut().spawn((
            SpriteAnimation {
                current_animation: AnimationType::Idle,
                frame_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
                current_frame: 0,
                frame_direction: 1,
                last_attack_trigger_serial: 0,
                previous_grounded: true,
                apply_immediate_frame: true,
                animations,
            },
            Sprite {
                image: sheets.core_texture.clone(),
                texture_atlas: None,
                ..default()
            },
            sheets.clone(),
        ));

        app.update();

        let mut query = app.world_mut().query::<&Sprite>();
        let sprite = query.single(app.world()).expect("animated sprite");
        let atlas = sprite
            .texture_atlas
            .as_ref()
            .expect("sprite animation should restore missing core atlas");
        assert_eq!(sprite.image, sheets.core_texture);
        assert_eq!(atlas.layout, sheets.core_layout);
        assert_eq!(atlas.index, 0);
    }

    #[test]
    fn test_update_sprite_animations_clamps_invalid_core_frame_indices() {
        let sheets = distinct_test_sheets();
        let mut animations = std::collections::HashMap::new();
        animations.insert(
            AnimationType::Jumping,
            AnimationClipData {
                frames: vec![16, 17, 18],
                frame_duration: 0.1,
                playback_mode: PlaybackMode::Once,
                speed_scale_by_velocity: false,
                speed_reference: 150.0,
                min_frame_duration: 0.05,
            },
        );

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Update, update_sprite_animations);
        app.world_mut().spawn((
            SpriteAnimation {
                current_animation: AnimationType::Jumping,
                frame_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
                current_frame: 0,
                frame_direction: 1,
                last_attack_trigger_serial: 0,
                previous_grounded: false,
                apply_immediate_frame: true,
                animations,
            },
            Sprite {
                image: sheets.core_texture.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: sheets.core_layout.clone(),
                    index: 99,
                }),
                ..default()
            },
            sheets.clone(),
        ));

        app.update();

        let mut query = app.world_mut().query::<&Sprite>();
        let sprite = query.single(app.world()).expect("animated sprite");
        let atlas = sprite.texture_atlas.as_ref().expect("texture atlas");
        assert_eq!(sprite.image, sheets.core_texture);
        assert_eq!(atlas.layout, sheets.core_layout);
        assert_eq!(
            atlas.index, 0,
            "invalid frame indices must not reach Bevy's atlas renderer"
        );
    }

    #[test]
    fn test_sheet_switch_resets_stale_atlas_index() {
        let sheets = distinct_test_sheets();
        let mut sprite = Sprite {
            image: sheets
                .reference_ground_light_texture
                .clone()
                .expect("old reference texture"),
            texture_atlas: Some(TextureAtlas {
                layout: sheets
                    .reference_ground_light_layout
                    .clone()
                    .expect("old reference layout"),
                index: 33,
            }),
            ..default()
        };

        apply_animation_sheet(
            &mut sprite,
            Some(&sheets),
            &AnimationType::Running,
            AttackAnimationStyle::GroundLightRow(5),
        );

        let atlas = sprite.texture_atlas.as_ref().expect("texture atlas");
        assert_eq!(sprite.image, sheets.running_texture);
        assert_eq!(atlas.layout, sheets.running_layout);
        assert_eq!(
            atlas.index, 0,
            "switching sheets must clear stale multi-row attack indices"
        );
    }

    #[test]
    fn test_same_sheet_switch_preserves_current_atlas_index() {
        let sheets = distinct_test_sheets();
        let mut sprite = Sprite {
            image: sheets.running_texture.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: sheets.running_layout.clone(),
                index: 2,
            }),
            ..default()
        };

        apply_animation_sheet(
            &mut sprite,
            Some(&sheets),
            &AnimationType::Running,
            AttackAnimationStyle::Normal,
        );

        let atlas = sprite.texture_atlas.as_ref().expect("texture atlas");
        assert_eq!(sprite.image, sheets.running_texture);
        assert_eq!(atlas.layout, sheets.running_layout);
        assert_eq!(
            atlas.index, 2,
            "reapplying the active sheet must not rewind a running animation"
        );
    }

    #[test]
    fn test_overedge_attack_frame_segments_match_runtime_sheets() {
        assert_eq!(
            overedge_attack_frames(
                AttackAnimationStyle::OveredgeRelease,
                crate::asset_paths::HF_SHIROU_OVEREDGE_LIGHT_ATTACK_FRAME_COUNT
            ),
            Some(vec![0, 1, 2])
        );
        assert_eq!(
            overedge_attack_frames(
                AttackAnimationStyle::OveredgeLight1,
                crate::asset_paths::HF_SHIROU_OVEREDGE_LIGHT_ATTACK_FRAME_COUNT
            ),
            Some(vec![2, 3, 4])
        );
        assert_eq!(
            overedge_attack_frames(
                AttackAnimationStyle::OveredgeLight2,
                crate::asset_paths::HF_SHIROU_OVEREDGE_LIGHT_ATTACK_FRAME_COUNT
            ),
            Some(vec![5, 6, 7])
        );
        assert_eq!(
            overedge_attack_frames(
                AttackAnimationStyle::OveredgeLight3,
                crate::asset_paths::HF_SHIROU_OVEREDGE_LIGHT_ATTACK_FRAME_COUNT
            ),
            Some(vec![8, 9, 10])
        );

        let heavy_frames = overedge_attack_frames(
            AttackAnimationStyle::OveredgeHeavy,
            crate::asset_paths::HF_SHIROU_OVEREDGE_HEAVY_ATTACK_FRAME_COUNT,
        )
        .expect("heavy frames");
        assert_eq!(
            heavy_frames.len(),
            crate::asset_paths::HF_SHIROU_OVEREDGE_HEAVY_ATTACK_FRAME_COUNT
        );
        assert_eq!(heavy_frames.first().copied(), Some(0));
        assert_eq!(heavy_frames.last().copied(), Some(16));
    }

    #[test]
    fn test_overedge_attack_frame_segments_do_not_overrun_available_frames() {
        assert_eq!(
            overedge_attack_frames(AttackAnimationStyle::OveredgeRelease, 2),
            None
        );
        assert_eq!(
            overedge_attack_frames(AttackAnimationStyle::OveredgeLight3, 10),
            None
        );
        assert_eq!(
            overedge_attack_frames(AttackAnimationStyle::OveredgeHeavy, 0),
            None
        );
    }

    #[test]
    fn test_reference_attack_row_frames_match_yuiop_rows() {
        assert_eq!(
            overedge_attack_frames(
                AttackAnimationStyle::GroundLightRow(1),
                crate::asset_paths::REFERENCE_BOARD_GROUND_LIGHT_COLS as usize
            ),
            Some(vec![0, 1, 2, 3, 4, 5, 6, 7])
        );
        assert_eq!(
            overedge_attack_frames(
                AttackAnimationStyle::GroundLightRow(5),
                crate::asset_paths::REFERENCE_BOARD_GROUND_LIGHT_COLS as usize
            ),
            Some(vec![0, 1, 2, 3, 4, 5, 6, 7])
        );
        assert_eq!(
            overedge_attack_frames(
                AttackAnimationStyle::HeavyRefRow(3),
                crate::asset_paths::REFERENCE_BOARD_HEAVY_COLS as usize
            ),
            Some(vec![0, 1, 2, 3, 4, 5, 6, 7])
        );
        assert_eq!(
            overedge_attack_frames(
                AttackAnimationStyle::AirComboRow(5),
                crate::asset_paths::REFERENCE_BOARD_AIR_COMBO_COLS as usize
            ),
            Some(vec![0, 1, 2, 3, 4, 5, 6, 7])
        );
        assert_eq!(
            overedge_attack_frames(
                AttackAnimationStyle::MobilityRefRow(4),
                crate::asset_paths::REFERENCE_BOARD_MOBILITY_COLS as usize
            ),
            Some(vec![0, 1, 2, 3, 4, 5])
        );
        assert_eq!(
            overedge_attack_frames(
                AttackAnimationStyle::NinjutsuRefRow(4),
                crate::asset_paths::REFERENCE_BOARD_NINJUTSU_COLS as usize
            ),
            Some(vec![0, 1, 2, 3, 4, 5, 6, 7])
        );
        assert_eq!(
            overedge_attack_frames(
                AttackAnimationStyle::UltimateRefRow(3),
                crate::asset_paths::REFERENCE_BOARD_ULTIMATE_COLS as usize
            ),
            Some(vec![0, 1, 2, 3, 4, 5, 6, 7])
        );
        assert_eq!(
            overedge_attack_frames(
                AttackAnimationStyle::WeaponProjRefRow(4),
                crate::asset_paths::REFERENCE_BOARD_WEAPON_PROJ_COLS as usize
            ),
            Some(vec![0, 1, 2, 3, 4, 5])
        );
    }

    #[test]
    fn test_attack_retrigger_resets_sprite_animation_from_first_frame() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Update, update_character_animation_state);

        let mut attack_state = AttackAnimationState::default();
        attack_state.trigger(0.30);
        attack_state.trigger(0.30);

        let mut animations = HashMap::new();
        animations.insert(
            AnimationType::Idle,
            AnimationClipData {
                frames: vec![0],
                frame_duration: 0.1,
                playback_mode: PlaybackMode::Loop,
                speed_scale_by_velocity: false,
                speed_reference: 150.0,
                min_frame_duration: 0.05,
            },
        );
        animations.insert(
            AnimationType::Attacking,
            AnimationClipData {
                frames: vec![0, 1, 2, 3],
                frame_duration: 0.07,
                playback_mode: PlaybackMode::Once,
                speed_scale_by_velocity: false,
                speed_reference: 150.0,
                min_frame_duration: 0.05,
            },
        );

        app.world_mut().spawn((
            Player,
            Sprite::default(),
            PlayerState::default(),
            Velocity::default(),
            attack_state,
            SpriteAnimation {
                current_animation: AnimationType::Attacking,
                frame_timer: Timer::from_seconds(0.07, TimerMode::Repeating),
                current_frame: 2,
                frame_direction: 1,
                last_attack_trigger_serial: 1,
                previous_grounded: true,
                apply_immediate_frame: false,
                animations,
            },
        ));

        app.update();

        let mut query = app.world_mut().query::<&SpriteAnimation>();
        let animation = query.single(app.world()).expect("sprite animation");
        assert_eq!(animation.current_animation, AnimationType::Attacking);
        assert_eq!(animation.current_frame, 0);
        assert_eq!(animation.last_attack_trigger_serial, 2);
    }
}
