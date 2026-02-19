use crate::{
    components::{
        animation_data::{AnimationDataMap, CharacterAnimationData, PlaybackMode},
        *,
    },
    resources::GameAssets,
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
            previous_grounded: true,
            apply_immediate_frame: true,
            animations: HashMap::new(),
        }
    }
}

/// 2026推荐：先给出最小可用品质，再给理想帧数。
pub fn frame_count_guideline(animation_type: &AnimationType) -> (usize, usize) {
    match animation_type {
        AnimationType::Idle => (4, 8),
        AnimationType::Running => (5, 10),
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
    has_move_input: bool,
    runtime: &AnimationRuntimeConfig,
) -> AnimationType {
    let was_grounded = animation.previous_grounded;
    animation.previous_grounded = player_state.is_grounded;

    let just_landed = !was_grounded && player_state.is_grounded;

    if !player_state.is_grounded {
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

fn current_clip_is_blocking(animation: &SpriteAnimation) -> bool {
    let Some(clip) = animation.animations.get(&animation.current_animation) else {
        return false;
    };

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
        previous_grounded: true,
        apply_immediate_frame: true,
        animations: character_data.animations.clone(),
    }
}

/// 更新精灵动画系统
pub fn update_sprite_animations(
    time: Res<Time>,
    mut query: Query<(&mut SpriteAnimation, &mut Sprite, Option<&Velocity>)>,
) {
    for (mut animation, mut sprite, velocity) in query.iter_mut() {
        let current_key = animation.current_animation.clone();

        let Some(current_clip) = animation.animations.get(&current_key).cloned() else {
            continue;
        };

        if current_clip.frames.is_empty() {
            continue;
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

/// 根据玩家状态更新动画
pub fn update_character_animation_state(
    mut query: Query<(&mut SpriteAnimation, &mut Sprite, &PlayerState, &Velocity), With<Player>>,
    game_assets: Option<Res<GameAssets>>,
    game_input: Option<Res<crate::systems::input::GameInput>>,
    runtime_config: Option<Res<AnimationRuntimeConfig>>,
) {
    let default_runtime = AnimationRuntimeConfig::default();
    let runtime = runtime_config.as_deref().unwrap_or(&default_runtime);
    let has_move_input = game_input
        .as_deref()
        .map(|input| input.move_left || input.move_right)
        .unwrap_or(false);

    for (mut animation, mut sprite, player_state, velocity) in query.iter_mut() {
        let new_animation = resolve_target_animation(
            &mut animation,
            player_state,
            velocity,
            has_move_input,
            runtime,
        );

        let clip_blocks_switch = current_clip_is_blocking(&animation);
        if clip_blocks_switch && new_animation != animation.current_animation {
            continue;
        }

        // 只有当动画类型改变时才切换
        if animation.current_animation != new_animation {
            apply_animation_change(&mut animation, new_animation.clone(), velocity.x.abs());

            // === Dynamic Layout Switching for Mixed-Grid Sprite Sheets ===
            if let Some(assets) = &game_assets
                && let Some(ref mut atlas) = sprite.texture_atlas
            {
                // Check if we need the 5-column layout (Run) or 4-column (Rest)
                if new_animation == AnimationType::Running {
                    if let Some(run_layout) = &assets.shirou_atlas_run {
                        atlas.layout = run_layout.clone();
                    }
                } else if let Some(std_layout) = &assets.shirou_atlas {
                    atlas.layout = std_layout.clone();
                }
            }

            if let Some(new_clip) = animation.animations.get(&new_animation) {
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

    #[test]
    fn test_frame_guidelines_are_monotonic() {
        let all_types = [
            AnimationType::Idle,
            AnimationType::Running,
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
}
