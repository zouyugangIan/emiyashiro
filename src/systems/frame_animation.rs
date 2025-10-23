use crate::{components::*, resources::*, states::*};
use bevy::prelude::*;

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
    println!("🎬 加载角色动画帧...");

    // 加载士郎的动画帧
    let _shirou_idle_frames: Vec<Handle<Image>> = vec![
        asset_server.load("images/characters/shirou_idle1.jpg"),
        asset_server.load("images/characters/shirou_idle2.jpg"),
        asset_server.load("images/characters/shirou_idle3.jpg"),
    ];

    let _shirou_running_frames: Vec<Handle<Image>> = vec![
        asset_server.load("images/characters/shirou_idle4.png"),
        asset_server.load("images/characters/shirou_idle5.png"),
        asset_server.load("images/characters/shirou_idle6.png"),
        asset_server.load("images/characters/shirou_idle7.png"),
    ];

    let _shirou_jumping_frames: Vec<Handle<Image>> = vec![
        asset_server.load("images/characters/shirou_idle8.png"),
        asset_server.load("images/characters/shirou_idle1.jpg"), // 复用作为跳跃帧
    ];

    // 加载樱的动画帧
    let _sakura_idle_frames: Vec<Handle<Image>> = vec![
        asset_server.load("images/characters/sakura_idle1.jpg"),
        asset_server.load("images/characters/teacher_idle.jpg"), // 临时使用
    ];

    // 存储到游戏资源中（如果资源存在）
    if let Some(_assets) = game_assets {
        // 这里可以存储动画帧到资源中，但现在我们先跳过
        println!("✅ 角色动画帧加载完成");
    } else {
        println!("⚠️ GameAssets 资源尚未创建，跳过动画帧存储");
    }
}

/// 更新帧动画系统
pub fn update_frame_animations(
    time: Res<Time>,
    mut query: Query<(&mut FrameAnimation, &mut Sprite)>,
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
    mut query: Query<(&mut FrameAnimation, &PlayerState, &CharacterAnimationState), With<Player>>,
    _asset_server: Res<AssetServer>,
) {
    for (mut animation, player_state, anim_state) in query.iter_mut() {
        let target_animation = if !player_state.is_grounded {
            CharacterAnimationType::Jumping
        } else if player_state.is_crouching {
            CharacterAnimationType::Crouching
        } else if player_state.is_grounded {
            // 这里可以根据速度判断是否在跑步
            CharacterAnimationType::Idle
        } else {
            CharacterAnimationType::Idle
        };

        // 如果动画类型改变，切换动画帧
        if anim_state.current_animation != target_animation {
            let new_frames = match target_animation {
                CharacterAnimationType::Idle => &anim_state.idle_frames,
                CharacterAnimationType::Running => &anim_state.running_frames,
                CharacterAnimationType::Jumping => &anim_state.jumping_frames,
                CharacterAnimationType::Crouching => &anim_state.crouching_frames,
            };

            if !new_frames.is_empty() {
                animation.frames = new_frames.clone();
                animation.reset();
                animation.play();
            }
        }
    }
}

/// 为玩家添加动画组件
pub fn setup_player_animation(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_query: Query<Entity, (With<Player>, Without<FrameAnimation>)>,
    character_selection: Res<CharacterSelection>,
) {
    for entity in player_query.iter() {
        let (idle_frames, running_frames, jumping_frames, crouching_frames) =
            match character_selection.selected_character {
                CharacterType::Shirou1 => {
                    let idle = vec![
                        asset_server.load("images/characters/shirou_idle1.jpg"),
                        asset_server.load("images/characters/shirou_idle2.jpg"),
                        asset_server.load("images/characters/shirou_idle3.jpg"),
                    ];
                    let running = vec![
                        asset_server.load("images/characters/shirou_idle4.png"),
                        asset_server.load("images/characters/shirou_idle5.png"),
                        asset_server.load("images/characters/shirou_idle6.png"),
                        asset_server.load("images/characters/shirou_idle7.png"),
                    ];
                    let jumping = vec![asset_server.load("images/characters/shirou_idle8.png")];
                    let crouching = vec![asset_server.load("images/characters/shirou_idle3.jpg")];
                    (idle, running, jumping, crouching)
                }
                CharacterType::Shirou2 => {
                    let idle = vec![
                        asset_server.load("images/characters/sakura_idle1.jpg"),
                        asset_server.load("images/characters/teacher_idle.jpg"),
                    ];
                    let running = idle.clone();
                    let jumping = idle.clone();
                    let crouching = idle.clone();
                    (idle, running, jumping, crouching)
                }
            };

        // 添加帧动画组件
        let frame_animation = FrameAnimation::new(idle_frames.clone(), 0.3, true);

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

        println!(
            "🎭 为玩家添加动画组件: {:?}",
            character_selection.selected_character
        );
    }
}

/// 创建动画背景系统
pub fn setup_animated_background(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 创建动态背景
    let background_frames = vec![
        asset_server.load("images/ui/cover1.jpg"),
        asset_server.load("images/ui/cover2.jpg"),
        asset_server.load("images/ui/cover3.jpeg"),
        asset_server.load("images/ui/cover4.jpg"),
    ];

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

    println!("🌅 创建动态背景");
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
            println!(
                "🎬 动画状态: {:?}, 当前帧: {}/{}, 播放中: {}",
                char_state.current_animation,
                animation.current_frame + 1,
                animation.frames.len(),
                animation.is_playing
            );
        }
    }
}
