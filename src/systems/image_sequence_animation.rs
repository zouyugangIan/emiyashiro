//! 樱的逐张图片动画正式实现。
//!
//! HF 士郎使用 `sprite_animation` 中的 TextureAtlas 主链。樱的原画不是规则网格，
//! 因此正式采用逐张图片动画；两种渲染方式共享同一套玩家状态语义。

use crate::{asset_paths, components::*, states::*};
use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct ImageSequenceAnimation {
    frames: Vec<Handle<Image>>,
    current_frame: usize,
    timer: Timer,
    looping: bool,
}

impl ImageSequenceAnimation {
    fn new(frames: Vec<Handle<Image>>, frame_duration: f32, looping: bool) -> Self {
        Self {
            frames,
            current_frame: 0,
            timer: Timer::from_seconds(frame_duration, TimerMode::Repeating),
            looping,
        }
    }

    fn replace_clip(&mut self, frames: Vec<Handle<Image>>, frame_duration: f32, looping: bool) {
        self.frames = frames;
        self.current_frame = 0;
        self.looping = looping;
        self.timer
            .set_duration(std::time::Duration::from_secs_f32(frame_duration));
        self.timer.reset();
        self.timer.unpause();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImageSequenceAnimationType {
    Idle,
    Running,
    Jumping,
    Crouching,
}

#[derive(Component, Debug)]
pub struct ImageSequenceAnimationState {
    current: ImageSequenceAnimationType,
    idle_frames: Vec<Handle<Image>>,
    running_frames: Vec<Handle<Image>>,
    jumping_frames: Vec<Handle<Image>>,
    crouching_frames: Vec<Handle<Image>>,
}

impl ImageSequenceAnimationState {
    fn clip(&self, animation_type: ImageSequenceAnimationType) -> (&[Handle<Image>], f32, bool) {
        match animation_type {
            ImageSequenceAnimationType::Idle => (&self.idle_frames, 0.15, true),
            ImageSequenceAnimationType::Running => (&self.running_frames, 0.10, true),
            ImageSequenceAnimationType::Jumping => (&self.jumping_frames, 0.12, false),
            ImageSequenceAnimationType::Crouching => (&self.crouching_frames, 0.15, true),
        }
    }
}

fn load_frames(asset_server: &AssetServer, paths: &[&'static str]) -> Vec<Handle<Image>> {
    paths.iter().map(|path| asset_server.load(*path)).collect()
}

/// 在樱刚生成时添加逐张图片动画组件。
pub fn setup_sakura_image_sequence_animation(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_query: Query<
        Entity,
        (
            Added<Player>,
            Without<crate::systems::sprite_animation::SpriteAnimation>,
        ),
    >,
    character_selection: Res<CharacterSelection>,
) {
    if character_selection.selected_character != CharacterType::Sakura {
        return;
    }

    for entity in player_query.iter() {
        let idle_frames = load_frames(&asset_server, asset_paths::SAKURA_IDLE_FRAMES);
        let running_frames = load_frames(&asset_server, asset_paths::SAKURA_RUNNING_FRAMES);
        let jumping_frames = load_frames(&asset_server, asset_paths::SAKURA_JUMPING_FRAMES);
        let crouching_frames = load_frames(&asset_server, asset_paths::SAKURA_CROUCHING_FRAMES);

        commands.entity(entity).insert((
            ImageSequenceAnimation::new(idle_frames.clone(), 0.15, true),
            ImageSequenceAnimationState {
                current: ImageSequenceAnimationType::Idle,
                idle_frames,
                running_frames,
                jumping_frames,
                crouching_frames,
            },
        ));
    }
}

/// 将玩家状态映射到樱的逐帧图片动画，切换时立即显示第一帧。
pub fn update_image_sequence_animation_state(
    mut query: Query<
        (
            &PlayerState,
            &Velocity,
            &mut ImageSequenceAnimationState,
            &mut ImageSequenceAnimation,
            &mut Sprite,
        ),
        With<Player>,
    >,
) {
    for (player_state, velocity, mut state, mut animation, mut sprite) in query.iter_mut() {
        let target = if !player_state.is_grounded {
            ImageSequenceAnimationType::Jumping
        } else if player_state.is_crouching {
            ImageSequenceAnimationType::Crouching
        } else if velocity.x.abs() > 10.0 {
            ImageSequenceAnimationType::Running
        } else {
            ImageSequenceAnimationType::Idle
        };

        if state.current == target {
            continue;
        }

        let (frames, duration, looping) = state.clip(target);
        let frames = frames.to_vec();
        if let Some(first_frame) = frames.first() {
            sprite.image = first_frame.clone();
        }
        animation.replace_clip(frames, duration, looping);
        state.current = target;
    }
}

/// 按时间推进逐张图片动画；卡顿后会补上本帧应跨过的帧数。
pub fn advance_image_sequence_animations(
    time: Res<Time>,
    mut query: Query<(&mut ImageSequenceAnimation, &mut Sprite)>,
) {
    for (mut animation, mut sprite) in query.iter_mut() {
        if animation.frames.len() <= 1 || animation.timer.is_paused() {
            continue;
        }

        animation.timer.tick(time.delta());
        let completed_intervals = animation.timer.times_finished_this_tick();
        for _ in 0..completed_intervals {
            let is_last_frame = animation.current_frame + 1 >= animation.frames.len();
            if is_last_frame && !animation.looping {
                animation.current_frame = animation.frames.len() - 1;
                animation.timer.pause();
                break;
            }
            animation.current_frame = (animation.current_frame + 1) % animation.frames.len();
        }

        if completed_intervals > 0 {
            sprite.image = animation.frames[animation.current_frame].clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::marker::PhantomData;

    fn image_handle(id: u128) -> Handle<Image> {
        Handle::Uuid(uuid::Uuid::from_u128(id), PhantomData)
    }

    #[test]
    fn state_change_applies_first_frame_immediately() {
        let idle = image_handle(1);
        let running = image_handle(2);
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Update, update_image_sequence_animation_state);
        app.world_mut().spawn((
            Player,
            PlayerState::default(),
            Velocity { x: 20.0, y: 0.0 },
            Sprite {
                image: idle.clone(),
                ..default()
            },
            ImageSequenceAnimation::new(vec![idle.clone()], 0.15, true),
            ImageSequenceAnimationState {
                current: ImageSequenceAnimationType::Idle,
                idle_frames: vec![idle],
                running_frames: vec![running.clone()],
                jumping_frames: vec![image_handle(3)],
                crouching_frames: vec![image_handle(4)],
            },
        ));

        app.update();

        let mut query = app
            .world_mut()
            .query::<(&Sprite, &ImageSequenceAnimationState)>();
        let (sprite, state) = query.single(app.world()).expect("image sequence player");
        assert_eq!(sprite.image, running);
        assert_eq!(state.current, ImageSequenceAnimationType::Running);
    }
}
