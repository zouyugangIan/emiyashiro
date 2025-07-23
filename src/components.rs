use bevy::prelude::*;

/// 玩家组件标记
#[derive(Component)]
pub struct Player;

/// 速度组件
#[derive(Component)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

/// 玩家状态组件
#[derive(Component)]
pub struct PlayerState {
    pub is_grounded: bool,
    pub is_crouching: bool,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            is_grounded: true,
            is_crouching: false,
        }
    }
}

/// 地面组件标记
#[derive(Component)]
pub struct Ground;

/// UI组件标记
#[derive(Component)]
pub struct MenuUI;

/// 开始按钮组件
#[derive(Component)]
pub struct StartButton;

/// 存档按钮组件
#[derive(Component)]
pub struct SaveButton;

/// 角色选择按钮组件
#[derive(Component)]
pub struct CharacterSelectButton {
    pub character_type: crate::states::CharacterType,
}

/// 封面图片1组件
#[derive(Component)]
pub struct CoverImage1;

/// 封面图片2组件
#[derive(Component)]
pub struct CoverImage2;

/// 封面渐变状态组件
#[derive(Component)]
pub struct CoverFadeState {
    pub alpha: f32,
    pub fade_direction: f32,
    pub fade_speed: f32,
}

impl Default for CoverFadeState {
    fn default() -> Self {
        Self {
            alpha: 1.0,
            fade_direction: -1.0,
            fade_speed: 0.5,
        }
    }
}

/// 角色动画组件
#[derive(Component)]
pub struct CharacterAnimation {
    pub current_animation: AnimationType,
    pub frame_timer: Timer,
    pub current_frame: usize,
}

impl Default for CharacterAnimation {
    fn default() -> Self {
        Self {
            current_animation: AnimationType::Idle,
            frame_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            current_frame: 0,
        }
    }
}

/// 动画类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum AnimationType {
    Idle,
    Running,
    Jumping,
    Crouching,
    Landing,
}

/// 动画资源组件
#[derive(Component)]
pub struct AnimationFrames {
    pub idle_frames: Vec<Handle<Image>>,
    pub running_frames: Vec<Handle<Image>>,
    pub jumping_frames: Vec<Handle<Image>>,
    pub crouching_frames: Vec<Handle<Image>>,
}

/// 音效触发组件
#[derive(Component)]
pub struct AudioTrigger {
    pub sound_type: SoundType,
    pub should_play: bool,
}

/// 音效类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum SoundType {
    Jump,
    Land,
    Footstep,
    Collect,
    Hit,
}