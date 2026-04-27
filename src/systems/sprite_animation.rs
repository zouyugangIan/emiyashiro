use crate::{
    asset_paths,
    components::{
        animation_data::{AnimationDataMap, CharacterAnimationData, PlaybackMode},
        *,
    },
};
use bevy::prelude::*;
use std::{collections::HashMap, fs};

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
            airborne_vertical_threshold: 0.5,
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
);

type PlayerAnimationStateItem<'a> = (
    &'a mut SpriteAnimation,
    &'a mut Sprite,
    &'a PlayerState,
    &'a Velocity,
    &'a AttackAnimationState,
    Option<&'a SpriteAnimationSheets>,
    Option<&'a ShroudState>,
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

fn resolve_target_animation(
    animation: &mut SpriteAnimation,
    player_state: &PlayerState,
    velocity: &Velocity,
    has_active_attack: bool,
    has_move_input: bool,
    runtime: &AnimationRuntimeConfig,
) -> AnimationType {
    let was_grounded = animation.previous_grounded;
    animation.previous_grounded = player_state.is_grounded;

    let just_landed = !was_grounded && player_state.is_grounded;

    if has_active_attack
        && player_state.is_grounded
        && animation.animations.contains_key(&AnimationType::Attacking)
    {
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
) -> AttackAnimationStyle {
    if *animation_type == AnimationType::Attacking
        && shroud.map(|state| state.is_released).unwrap_or(false)
    {
        attack_style
    } else {
        AttackAnimationStyle::Normal
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
        AttackAnimationStyle::Normal => None,
    }
}

fn resolved_animation_clip(
    animation: &SpriteAnimation,
    animation_type: &AnimationType,
    sprite_sheets: Option<&SpriteAnimationSheets>,
    shroud: Option<&ShroudState>,
    attack_style: AttackAnimationStyle,
) -> Option<AnimationClipData> {
    let mut clip = animation.animations.get(animation_type)?.clone();
    let attack_style = resolved_attack_style(animation_type, attack_style, shroud);

    if let Some(frame_count) =
        sprite_sheets.and_then(|sheets| sheets.overedge_attacking_frame_count(attack_style))
        && let Some(frames) = overedge_attack_frames(attack_style, frame_count)
    {
        clip.frames = frames;
    }

    Some(clip)
}

fn current_clip_is_blocking(animation: &SpriteAnimation, clip: &AnimationClipData) -> bool {
    let is_once = clip.playback_mode() == PlaybackMode::Once;
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

    animation.current_animation = new_animation;
    animation.current_frame = 0;
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

    let (target_texture, target_layout) =
        sprite_sheets.select_sheet_for_attack_style(animation_type, attack_style);
    sprite.image = target_texture.clone();

    if let Some(ref mut atlas) = sprite.texture_atlas {
        atlas.layout = target_layout.clone();
    }
}

/// 在启动时加载所有动画配置文件
pub fn load_animation_data() -> AnimationDataMap {
    let mut animation_map = AnimationDataMap::default();

    // Check if directory exists before reading
    if let Ok(paths) = fs::read_dir("assets/animations") {
        for path in paths {
            let path = match path {
                Ok(value) => value.path(),
                Err(error) => {
                    warn!("Failed to read animation path entry: {}", error);
                    continue;
                }
            };

            if path.extension().and_then(|s| s.to_str()) == Some("ron") {
                let character_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap()
                    .to_string();

                let ron_string = match fs::read_to_string(&path) {
                    Ok(content) => content,
                    Err(error) => {
                        warn!(
                            "Failed to read animation file '{}': {}",
                            path.display(),
                            error
                        );
                        continue;
                    }
                };

                let anim_data: CharacterAnimationData = match ron::from_str(&ron_string) {
                    Ok(data) => data,
                    Err(error) => {
                        warn!(
                            "Failed to parse RON for '{}': {} (file: {})",
                            character_name,
                            error,
                            path.display()
                        );
                        continue;
                    }
                };

                validate_clip_frame_counts(&character_name, &anim_data.animations);

                animation_map.0.insert(character_name, anim_data);
            }
        }

        crate::debug_log!(
            "📂 Loaded {} character animation profiles.",
            animation_map.0.len()
        );
    } else {
        crate::debug_log!(
            "⚠️ Warning: assets/animations directory not found, using empty animation map"
        );
    }

    animation_map
}

/// 创建角色动画组件
pub fn create_character_animation(
    anim_data_map: &Res<AnimationDataMap>,
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
pub fn update_sprite_animations(time: Res<Time>, mut query: Query<SpriteAnimationUpdateItem>) {
    for (mut animation, mut sprite, velocity, attack_state, sprite_sheets, shroud) in
        query.iter_mut()
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
        ) else {
            continue;
        };

        if current_clip.frames.is_empty() {
            continue;
        }

        if animation.current_frame >= current_clip.frames.len() {
            animation.current_frame = current_clip.frames.len().saturating_sub(1);
        }

        if current_key == AnimationType::Attacking {
            let attack_style = resolved_attack_style(&current_key, attack_style, shroud);
            apply_animation_sheet(&mut sprite, sprite_sheets, &current_key, attack_style);
        }

        let horizontal_speed_abs = velocity.map(|v| v.x.abs()).unwrap_or(0.0);
        let target_duration = current_clip.frame_duration_for_speed(horizontal_speed_abs);
        let current_duration = animation.frame_timer.duration().as_secs_f32();

        if (current_duration - target_duration).abs() > 0.002 {
            animation
                .frame_timer
                .set_duration(std::time::Duration::from_secs_f32(target_duration));
        }

        if animation.apply_immediate_frame {
            if let Some(first_atlas_idx) = current_clip.frames.get(animation.current_frame).copied()
            {
                apply_atlas_frame(&mut sprite, first_atlas_idx);
            }
            animation.apply_immediate_frame = false;
        }

        animation.frame_timer.tick(time.delta());

        if animation.frame_timer.just_finished() {
            let frame_count = current_clip.frames.len();
            animation.current_frame = next_frame_index(
                animation.current_frame,
                frame_count,
                current_clip.playback_mode(),
                &mut animation.frame_direction,
            );

            if let Some(atlas_idx) = current_clip.frames.get(animation.current_frame).copied() {
                apply_atlas_frame(&mut sprite, atlas_idx);
            }
        }
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

    for (mut animation, mut sprite, player_state, velocity, attack_state, sprite_sheets, shroud) in
        query.iter_mut()
    {
        let attack_retriggered = attack_state.is_active()
            && attack_state.trigger_serial != animation.last_attack_trigger_serial;
        let new_animation = resolve_target_animation(
            &mut animation,
            player_state,
            velocity,
            attack_state.is_active(),
            has_move_input,
            runtime,
        );

        let clip_blocks_switch = resolved_animation_clip(
            &animation,
            &animation.current_animation,
            sprite_sheets,
            shroud,
            attack_state.style,
        )
        .as_ref()
        .map(|clip| current_clip_is_blocking(&animation, clip))
        .unwrap_or(false);
        if clip_blocks_switch && new_animation != animation.current_animation && !attack_retriggered
        {
            continue;
        }

        if animation.current_animation != new_animation || attack_retriggered {
            apply_animation_change(&mut animation, new_animation.clone(), velocity.x.abs());
            let attack_style = resolved_attack_style(&new_animation, attack_state.style, shroud);
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
            ) {
                if let Some(first_atlas_idx) = new_clip.frames.first().copied() {
                    apply_atlas_frame(&mut sprite, first_atlas_idx);
                }
                info!(
                    "🎭 切换动画: {:?} ({}帧, 模式: {:?})",
                    new_animation,
                    new_clip.frames.len(),
                    new_clip.playback_mode()
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
