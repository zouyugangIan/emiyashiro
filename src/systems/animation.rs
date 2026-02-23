use crate::{asset_paths, components::*, resources::*};
use bevy::audio::Volume;
use bevy::prelude::*;

/// 角色动画系统
pub fn animate_character(
    mut character_query: Query<
        (
            &mut CharacterAnimation,
            &mut Sprite,
            &PlayerState,
            &Velocity,
            &AnimationFrames,
        ),
        With<Player>,
    >,
    time: Res<Time>,
) {
    for (mut animation, mut sprite, player_state, velocity, frames) in character_query.iter_mut() {
        // 根据玩家状态确定应该播放的动画
        let target_animation = determine_animation(player_state, velocity);

        // 如果动画类型改变，重置帧计数器
        if animation.current_animation != target_animation {
            animation.current_animation = target_animation;
            animation.current_frame = 0;
            animation.frame_timer.reset();
        }

        // 更新动画帧
        animation.frame_timer.tick(time.delta());
        if animation.frame_timer.just_finished() {
            let current_frames = get_animation_frames(&animation.current_animation, frames);

            if !current_frames.is_empty() {
                animation.current_frame = (animation.current_frame + 1) % current_frames.len();
                sprite.image = current_frames[animation.current_frame].clone();
            }
        }
    }
}

/// 根据玩家状态确定动画类型
fn determine_animation(player_state: &PlayerState, velocity: &Velocity) -> AnimationType {
    if player_state.is_crouching {
        AnimationType::Crouching
    } else if !player_state.is_grounded {
        if velocity.y > 0.0 {
            AnimationType::Jumping
        } else {
            AnimationType::Landing
        }
    } else if velocity.x.abs() > 0.1 {
        AnimationType::Running
    } else {
        AnimationType::Idle
    }
}

/// 获取指定动画类型的帧序列
fn get_animation_frames<'a>(
    animation_type: &AnimationType,
    frames: &'a AnimationFrames,
) -> &'a Vec<Handle<Image>> {
    match animation_type {
        AnimationType::Idle => &frames.idle_frames,
        AnimationType::Running => &frames.running_frames,
        AnimationType::Attacking => &frames.running_frames,
        AnimationType::Jumping => &frames.jumping_frames,
        AnimationType::Crouching => &frames.crouching_frames,
        AnimationType::Landing => &frames.jumping_frames, // 复用跳跃帧
    }
}

/// 设置角色动画资源
pub fn setup_character_animation(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    character_query: Query<Entity, (With<Player>, Without<CharacterAnimation>)>,
) {
    for entity in character_query.iter() {
        // 为现有角色添加动画组件 - 只使用一个角色图片，不要切换
        let animation_frames = AnimationFrames {
            idle_frames: vec![
                asset_server.load(asset_paths::IMAGE_CHAR_SHIROU_IDLE1), // 只使用一个图片
            ],
            running_frames: vec![
                asset_server.load(asset_paths::IMAGE_CHAR_SHIROU_IDLE1), // 保持一致
            ],
            jumping_frames: vec![
                asset_server.load(asset_paths::IMAGE_CHAR_SHIROU_IDLE1), // 保持一致
            ],
            crouching_frames: vec![
                asset_server.load(asset_paths::IMAGE_CHAR_SHIROU_IDLE1), // 保持一致
            ],
        };

        commands
            .entity(entity)
            .insert((CharacterAnimation::default(), animation_frames));
    }
}

/// 音效触发系统
pub fn trigger_audio_effects(
    mut commands: Commands,
    mut audio_query: Query<(Entity, &mut AudioTrigger)>,
    audio_settings: Res<AudioSettings>,
    game_assets: Res<GameAssets>,
) {
    for (entity, mut trigger) in audio_query.iter_mut() {
        if trigger.should_play {
            let audio_source = match trigger.sound_type {
                SoundType::Jump => game_assets.jump_sound.clone(),
                SoundType::Land => game_assets.land_sound.clone(),
                SoundType::Footstep => game_assets.footstep_sound.clone(),
            };

            // 播放音效
            commands.spawn((
                AudioPlayer::new(audio_source),
                PlaybackSettings::DESPAWN.with_volume(Volume::Linear(
                    audio_settings.sfx_volume * audio_settings.master_volume,
                )),
            ));

            trigger.should_play = false;

            // 移除已处理的音效触发器
            commands.entity(entity).despawn();
        }
    }
}

/// 背景音乐系统
pub fn manage_background_music(
    mut commands: Commands,
    audio_settings: Res<AudioSettings>,
    game_assets: Res<GameAssets>,
    music_query: Query<Entity, With<AudioPlayer>>,
    mut music_started: Local<bool>,
) {
    if audio_settings.music_enabled && !*music_started {
        commands.spawn((
            AudioPlayer::new(game_assets.background_music.clone()),
            PlaybackSettings::LOOP.with_volume(Volume::Linear(
                audio_settings.music_volume * audio_settings.master_volume,
            )),
        ));
        *music_started = true;
    } else if !audio_settings.music_enabled && *music_started {
        // 停止背景音乐
        for entity in music_query.iter() {
            commands.entity(entity).despawn();
        }
        *music_started = false;
    }
}
