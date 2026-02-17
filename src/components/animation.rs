//! 动画相关组件
//!
//! 包含角色动画和动画资源管理的组件。

use bevy::prelude::*;

/// 动画类型枚举
///
/// 定义游戏中不同类型的角色动画。
///
/// # 变体
/// * `Idle` - 待机动画
/// * `Running` - 跑步动画
/// * `Jumping` - 跳跃动画
/// * `Crouching` - 蹲下动画
/// * `Landing` - 着陆动画
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Deserialize)]
pub enum AnimationType {
    Idle,
    Running,
    Jumping,
    Crouching,
    Landing,
}

impl AnimationType {
    /// 获取动画的默认帧持续时间
    ///
    /// # 返回
    /// 动画帧的持续时间（秒）
    pub fn frame_duration(&self) -> f32 {
        match self {
            AnimationType::Idle => 0.2,
            AnimationType::Running => 0.1,
            AnimationType::Jumping => 0.15,
            AnimationType::Crouching => 0.1,
            AnimationType::Landing => 0.08,
        }
    }

    /// 检查动画是否循环播放
    ///
    /// # 返回
    /// 如果动画循环播放返回 true
    pub fn is_looping(&self) -> bool {
        match self {
            AnimationType::Idle | AnimationType::Running | AnimationType::Crouching => true,
            AnimationType::Jumping | AnimationType::Landing => false,
        }
    }
}

/// 角色动画组件
///
/// 管理角色的当前动画状态和播放进度。
///
/// # 字段
/// * `current_animation` - 当前播放的动画类型
/// * `frame_timer` - 帧切换计时器
/// * `current_frame` - 当前帧索引
///
/// # 示例
///
/// ```rust
/// use s_emiyashiro::components::{AnimationType, CharacterAnimation};
///
/// let mut animation = CharacterAnimation::new(AnimationType::Running);
/// animation.set_animation(AnimationType::Jumping);
/// ```
#[derive(Component, Debug)]
pub struct CharacterAnimation {
    pub current_animation: AnimationType,
    pub frame_timer: Timer,
    pub current_frame: usize,
}

impl Default for CharacterAnimation {
    fn default() -> Self {
        Self::new(AnimationType::Idle)
    }
}

impl CharacterAnimation {
    /// 创建新的角色动画组件
    ///
    /// # 参数
    /// * `animation_type` - 初始动画类型
    ///
    /// # 返回
    /// 新的 CharacterAnimation 实例
    pub fn new(animation_type: AnimationType) -> Self {
        let duration = animation_type.frame_duration();
        Self {
            current_animation: animation_type,
            frame_timer: Timer::from_seconds(duration, TimerMode::Repeating),
            current_frame: 0,
        }
    }

    /// 设置新的动画类型
    ///
    /// # 参数
    /// * `animation_type` - 新的动画类型
    pub fn set_animation(&mut self, animation_type: AnimationType) {
        if self.current_animation != animation_type {
            self.current_animation = animation_type.clone();
            self.current_frame = 0;
            let duration = animation_type.frame_duration();
            self.frame_timer
                .set_duration(std::time::Duration::from_secs_f32(duration));
            self.frame_timer.reset();
        }
    }

    /// 更新动画状态
    ///
    /// # 参数
    /// * `delta_time` - 时间增量
    /// * `frame_count` - 总帧数
    ///
    /// # 返回
    /// 如果切换到新帧返回 true
    pub fn update(&mut self, delta_time: std::time::Duration, frame_count: usize) -> bool {
        self.frame_timer.tick(delta_time);

        if self.frame_timer.just_finished() && frame_count > 0 {
            if self.current_animation.is_looping() {
                self.current_frame = (self.current_frame + 1) % frame_count;
            } else {
                self.current_frame = (self.current_frame + 1).min(frame_count - 1);
            }
            return true;
        }

        false
    }

    /// 检查动画是否完成
    ///
    /// 只对非循环动画有效。
    ///
    /// # 参数
    /// * `frame_count` - 总帧数
    ///
    /// # 返回
    /// 如果动画完成返回 true
    pub fn is_finished(&self, frame_count: usize) -> bool {
        !self.current_animation.is_looping() && self.current_frame >= frame_count.saturating_sub(1)
    }
}

/// 动画资源组件
///
/// 存储不同动画类型的帧资源句柄。
///
/// # 字段
/// * `idle_frames` - 待机动画帧
/// * `running_frames` - 跑步动画帧
/// * `jumping_frames` - 跳跃动画帧
/// * `crouching_frames` - 蹲下动画帧
#[derive(Component, Debug)]
pub struct AnimationFrames {
    pub idle_frames: Vec<Handle<Image>>,
    pub running_frames: Vec<Handle<Image>>,
    pub jumping_frames: Vec<Handle<Image>>,
    pub crouching_frames: Vec<Handle<Image>>,
}

impl AnimationFrames {
    /// 创建新的动画资源组件
    pub fn new() -> Self {
        Self {
            idle_frames: Vec::new(),
            running_frames: Vec::new(),
            jumping_frames: Vec::new(),
            crouching_frames: Vec::new(),
        }
    }

    /// 根据动画类型获取对应的帧列表
    ///
    /// # 参数
    /// * `animation_type` - 动画类型
    ///
    /// # 返回
    /// 对应动画类型的帧列表引用
    pub fn get_frames(&self, animation_type: &AnimationType) -> &Vec<Handle<Image>> {
        match animation_type {
            AnimationType::Idle => &self.idle_frames,
            AnimationType::Running => &self.running_frames,
            AnimationType::Jumping => &self.jumping_frames,
            AnimationType::Crouching => &self.crouching_frames,
            AnimationType::Landing => &self.idle_frames, // 使用待机帧作为着陆帧
        }
    }

    /// 根据动画类型获取对应的帧列表（可变引用）
    ///
    /// # 参数
    /// * `animation_type` - 动画类型
    ///
    /// # 返回
    /// 对应动画类型的帧列表可变引用
    pub fn get_frames_mut(&mut self, animation_type: &AnimationType) -> &mut Vec<Handle<Image>> {
        match animation_type {
            AnimationType::Idle => &mut self.idle_frames,
            AnimationType::Running => &mut self.running_frames,
            AnimationType::Jumping => &mut self.jumping_frames,
            AnimationType::Crouching => &mut self.crouching_frames,
            AnimationType::Landing => &mut self.idle_frames,
        }
    }
}

impl Default for AnimationFrames {
    fn default() -> Self {
        Self::new()
    }
}
