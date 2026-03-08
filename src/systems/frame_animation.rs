use crate::{asset_paths, components::*, resources::*, states::*};
use bevy::prelude::*;

type PlayerWithoutAnimationQuery<'w, 's> = Query<
    'w,
    's,
    Entity,
    (
        With<Player>,
        Without<FrameAnimation>,
        Without<crate::systems::sprite_animation::SpriteAnimation>,
    ),
>;

/// 帧动画组件
#[derive(Component, Debug)]
pub struct FrameAnimation {
    pub frames: Vec<Handle<Image>>,
    pub current_frame: usize,
    pub timer: Timer,
    pub is_playing: bool,
    pub loop_animation: bool,
}

impl FrameAnimation {
    pub fn new(frames: Vec<Handle<Image>>, frame_duration: f32, loop_animation: bool) -> Self {
        Self {
            frames,
            current_frame: 0,
            timer: Timer::from_seconds(frame_duration, TimerMode::Repeating),
            is_playing: true,
            loop_animation,
        }
    }

    pub fn play(&mut self) {
        self.is_playing = true;
    }

    pub fn pause(&mut self) {
        self.is_playing = false;
    }

    pub fn reset(&mut self) {
        self.current_frame = 0;
        self.timer.reset();
    }
}

/// 角色动画状态
#[derive(Component, Debug, Clone)]
pub struct CharacterAnimationState {
    pub current_animation: CharacterAnimationType,
    pub idle_frames: Vec<Handle<Image>>,
    pub running_frames: Vec<Handle<Image>>,
    pub jumping_frames: Vec<Handle<Image>>,
    pub crouching_frames: Vec<Handle<Image>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CharacterAnimationType {
    Idle,
    Running,
    Jumping,
    Crouching,
}

/// 加载角色动画帧
pub fn load_character_animations(
    _commands: Commands,
    asset_server: Res<AssetServer>,
    game_assets: Option<ResMut<GameAssets>>,
) {
    crate::debug_log!("🎬 加载角色动画帧...");

    let _shirou_idle_frames: Vec<Handle<Image>> = vec![
        asset_server.load(asset_paths::IMAGE_CHAR_SHIROU_IDLE1),
        asset_server.load(asset_paths::IMAGE_CHAR_SHIROU_IDLE2),
        asset_server.load(asset_paths::IMAGE_CHAR_SHIROU_IDLE3),
    ];

    let _shirou_running_frames: Vec<Handle<Image>> = vec![
        asset_server.load(asset_paths::IMAGE_CHAR_SHIROU_IDLE4),
        asset_server.load(asset_paths::IMAGE_CHAR_SHIROU_IDLE5),
        asset_server.load(asset_paths::IMAGE_CHAR_SHIROU_IDLE6),
        asset_server.load(asset_paths::IMAGE_CHAR_SHIROU_IDLE7),
    ];

    let _shirou_jumping_frames: Vec<Handle<Image>> = vec![
        asset_server.load(asset_paths::IMAGE_CHAR_SHIROU_IDLE8),
        asset_server.load(asset_paths::IMAGE_CHAR_SHIROU_IDLE1), // 复用作为跳跃帧
    ];

    // 加载樱的动画帧
    let _sakura_idle_frames: Vec<Handle<Image>> = vec![
        asset_server.load(asset_paths::IMAGE_CHAR_SAKURA_IDLE01),
        asset_server.load(asset_paths::IMAGE_CHAR_TEACHER_IDLE), // 临时使用
    ];

    // 存储到游戏资源中（如果资源存在）
    if let Some(_assets) = game_assets {
        // 这里可以存储动画帧到资源中，但现在我们先跳过
        crate::debug_log!("✅ 角色动画帧加载完成");
    } else {
        crate::debug_log!("⚠️ GameAssets 资源尚未创建，跳过动画帧存储");
    }
}

/// 更新帧动画系统
pub fn update_frame_animations(
    time: Res<Time>,
    mut query: Query<
        (&mut FrameAnimation, &mut Sprite),
        Without<crate::systems::sprite_animation::SpriteAnimation>,
    >,
) {
    for (mut animation, mut sprite) in query.iter_mut() {
        if !animation.is_playing || animation.frames.is_empty() {
            continue;
        }

        animation.timer.tick(time.delta());

        if animation.timer.just_finished() {
            // 切换到下一帧
            animation.current_frame += 1;

            if animation.current_frame >= animation.frames.len() {
                if animation.loop_animation {
                    animation.current_frame = 0;
                } else {
                    animation.current_frame = animation.frames.len() - 1;
                    animation.is_playing = false;
                }
            }

            // 更新精灵图像
            sprite.image = animation.frames[animation.current_frame].clone();
        }
    }
}

/// 角色动画控制系统
pub fn update_character_animations(
    mut query: Query<
        (
            &mut FrameAnimation,
            &PlayerState,
            &mut CharacterAnimationState,
            Option<&Velocity>,
        ),
        With<Player>,
    >,
    _asset_server: Res<AssetServer>,
) {
    for (mut animation, player_state, mut anim_state, velocity) in query.iter_mut() {
        // 根据玩家状态和速度决定目标动画
        let target_animation = if !player_state.is_grounded {
            CharacterAnimationType::Jumping
        } else if player_state.is_crouching {
            CharacterAnimationType::Crouching
        } else if let Some(vel) = velocity {
            // 根据速度判断是否在跑步（速度阈值：50.0）
            if vel.x.abs() > 50.0 {
                CharacterAnimationType::Running
            } else {
                CharacterAnimationType::Idle
            }
        } else {
            CharacterAnimationType::Idle
        };

        // 如果动画类型改变，切换动画帧
        if anim_state.current_animation != target_animation {
            // 先获取帧数据，避免借用冲突
            let new_frames = match &target_animation {
                CharacterAnimationType::Idle => anim_state.idle_frames.clone(),
                CharacterAnimationType::Running => anim_state.running_frames.clone(),
                CharacterAnimationType::Jumping => anim_state.jumping_frames.clone(),
                CharacterAnimationType::Crouching => anim_state.crouching_frames.clone(),
            };

            let frame_duration = match target_animation {
                CharacterAnimationType::Idle => 0.15,
                CharacterAnimationType::Running => 0.1,
                CharacterAnimationType::Jumping => 0.12,
                CharacterAnimationType::Crouching => 0.15,
            };

            if !new_frames.is_empty() {
                animation.frames = new_frames.clone();
                animation
                    .timer
                    .set_duration(std::time::Duration::from_secs_f32(frame_duration));
                animation.reset();
                animation.play();

                crate::debug_log!(
                    "🎬 切换动画: {:?} ({}帧)",
                    target_animation,
                    new_frames.len()
                );

                // 最后更新状态
                anim_state.current_animation = target_animation;
            }
        }
    }
}

/// 为玩家添加动画组件
pub fn setup_player_animation(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_query: PlayerWithoutAnimationQuery,
    character_selection: Res<CharacterSelection>,
) {
    for entity in player_query.iter() {
        let (idle_frames, running_frames, jumping_frames, crouching_frames) =
            match character_selection.selected_character {
                CharacterType::Shirou1 => {
                    // 使用优化后的 Shirou 动画帧序列
                    let idle: Vec<Handle<Image>> = asset_paths::SHIROU_IDLE_FRAMES
                        .iter()
                        .map(|path| asset_server.load(*path))
                        .collect();
                    let running: Vec<Handle<Image>> = asset_paths::SHIROU_RUNNING_FRAMES
                        .iter()
                        .map(|path| asset_server.load(*path))
                        .collect();
                    let jumping: Vec<Handle<Image>> = asset_paths::SHIROU_JUMPING_FRAMES
                        .iter()
                        .map(|path| asset_server.load(*path))
                        .collect();
                    let crouching: Vec<Handle<Image>> = asset_paths::SHIROU_CROUCHING_FRAMES
                        .iter()
                        .map(|path| asset_server.load(*path))
                        .collect();
                    (idle, running, jumping, crouching)
                }
                CharacterType::Shirou2 => {
                    // 使用优化后的 Sakura 动画帧序列
                    let idle: Vec<Handle<Image>> = asset_paths::SAKURA_IDLE_FRAMES
                        .iter()
                        .map(|path| asset_server.load(*path))
                        .collect();
                    let running: Vec<Handle<Image>> = asset_paths::SAKURA_RUNNING_FRAMES
                        .iter()
                        .map(|path| asset_server.load(*path))
                        .collect();
                    let jumping: Vec<Handle<Image>> = asset_paths::SAKURA_JUMPING_FRAMES
                        .iter()
                        .map(|path| asset_server.load(*path))
                        .collect();
                    let crouching: Vec<Handle<Image>> = asset_paths::SAKURA_CROUCHING_FRAMES
                        .iter()
                        .map(|path| asset_server.load(*path))
                        .collect();
                    (idle, running, jumping, crouching)
                }
            };

        // 记录帧数用于日志
        let idle_count = idle_frames.len();
        let running_count = running_frames.len();
        let jumping_count = jumping_frames.len();
        let crouching_count = crouching_frames.len();

        // 添加帧动画组件（调整帧率为 0.15 秒，让动画更流畅）
        let frame_animation = FrameAnimation::new(idle_frames.clone(), 0.15, true);

        // 添加角色动画状态
        let char_anim_state = CharacterAnimationState {
            current_animation: CharacterAnimationType::Idle,
            idle_frames,
            running_frames,
            jumping_frames,
            crouching_frames,
        };

        commands
            .entity(entity)
            .insert((frame_animation, char_anim_state));

        crate::debug_log!(
            "🎭 为玩家添加动画组件: {:?} (待机: {}帧, 跑步: {}帧, 跳跃: {}帧, 蹲下: {}帧)",
            character_selection.selected_character,
            idle_count,
            running_count,
            jumping_count,
            crouching_count
        );
    }
}

/// 创建动画背景系统（备用 - 目前使用程序化云彩系统）
///
/// 注意：此函数目前未被使用。游戏使用 `background.rs` 中的程序化云彩系统。
/// 如果需要切换到图片背景，可以在 client.rs 中注册此系统。
#[allow(dead_code)]
pub fn setup_animated_background(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 创建动态背景 - 使用所有封面图片
    let background_frames: Vec<Handle<Image>> = asset_paths::UI_COVER_IMAGES
        .iter()
        .map(|path| asset_server.load(*path))
        .collect();

    let background_animation = FrameAnimation::new(background_frames.clone(), 2.0, true);

    commands.spawn((
        Sprite {
            image: background_frames[0].clone(),
            custom_size: Some(Vec2::new(1024.0, 768.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, -10.0)), // 放在最后面
        background_animation,
    ));

    crate::debug_log!("🌅 创建动态背景（图片模式）");
}

/// 动画调试系统
pub fn debug_animations(
    query: Query<(&FrameAnimation, &CharacterAnimationState), With<Player>>,
    mut timer: Local<Timer>,
    time: Res<Time>,
) {
    if timer.duration().is_zero() {
        timer.set_duration(std::time::Duration::from_secs(3));
        timer.set_mode(bevy::time::TimerMode::Repeating);
    }
    timer.tick(time.delta());

    if timer.just_finished() {
        for (animation, char_state) in query.iter() {
            crate::debug_log!(
                "🎬 动画状态: {:?}, 当前帧: {}/{}, 播放中: {}",
                char_state.current_animation,
                animation.current_frame + 1,
                animation.frames.len(),
                animation.is_playing
            );
        }
    }
}
