//! 音频相关组件
//!
//! 包含音效触发和音频状态管理的组件。

use bevy::prelude::*;

/// 音效类型枚举
///
/// 定义游戏中不同类型的音效。
///
/// # 变体
/// * `Jump` - 跳跃音效
/// * `Land` - 着陆音效
/// * `Footstep` - 脚步音效
#[derive(Debug, Clone, PartialEq)]
pub enum SoundType {
    /// 跳跃音效
    Jump,
    /// 着陆音效
    Land,
    /// 脚步音效
    Footstep,
}

impl SoundType {
    /// 获取音效的默认音量
    ///
    /// # 返回
    /// 音效的默认音量 (0.0-1.0)
    pub fn default_volume(&self) -> f32 {
        match self {
            SoundType::Jump => 0.7,
            SoundType::Land => 0.6,
            SoundType::Footstep => 0.4,
        }
    }

    /// 获取音效的优先级
    ///
    /// 数值越高优先级越高。
    ///
    /// # 返回
    /// 音效优先级
    pub fn priority(&self) -> u8 {
        match self {
            SoundType::Jump => 3,
            SoundType::Land => 2,
            SoundType::Footstep => 1,
        }
    }
}

/// 音效触发组件
///
/// 用于标记需要播放音效的实体。
///
/// # 字段
/// * `sound_type` - 音效类型
/// * `should_play` - 是否应该播放
///
/// # 示例
///
/// ```rust
/// use crate::components::{AudioTrigger, SoundType};
///
/// let trigger = AudioTrigger::new(SoundType::Jump);
/// ```
#[derive(Component, Debug)]
pub struct AudioTrigger {
    pub sound_type: SoundType,
    pub should_play: bool,
}

impl AudioTrigger {
    /// 创建新的音效触发器
    ///
    /// # 参数
    /// * `sound_type` - 音效类型
    ///
    /// # 返回
    /// 新的 AudioTrigger 实例，默认设置为应该播放
    pub fn new(sound_type: SoundType) -> Self {
        Self {
            sound_type,
            should_play: true,
        }
    }

    /// 触发音效播放
    pub fn trigger(&mut self) {
        self.should_play = true;
    }

    /// 重置触发状态
    pub fn reset(&mut self) {
        self.should_play = false;
    }

    /// 检查是否应该播放
    ///
    /// # 返回
    /// 如果应该播放返回 true
    pub fn should_play(&self) -> bool {
        self.should_play
    }
}

/// 音频状态组件
///
/// 跟踪实体的音频播放状态。
///
/// # 字段
/// * `is_playing` - 是否正在播放音频
/// * `last_sound` - 最后播放的音效类型
/// * `play_time` - 播放开始时间
#[derive(Component, Debug)]
pub struct AudioState {
    pub is_playing: bool,
    pub last_sound: Option<SoundType>,
    pub play_time: f32,
}

impl Default for AudioState {
    fn default() -> Self {
        Self {
            is_playing: false,
            last_sound: None,
            play_time: 0.0,
        }
    }
}

impl AudioState {
    /// 创建新的音频状态
    pub fn new() -> Self {
        Self::default()
    }

    /// 开始播放音效
    ///
    /// # 参数
    /// * `sound_type` - 音效类型
    /// * `current_time` - 当前时间
    pub fn start_playing(&mut self, sound_type: SoundType, current_time: f32) {
        self.is_playing = true;
        self.last_sound = Some(sound_type);
        self.play_time = current_time;
    }

    /// 停止播放音效
    pub fn stop_playing(&mut self) {
        self.is_playing = false;
    }

    /// 检查是否可以播放新音效
    ///
    /// 根据音效优先级和播放间隔判断。
    ///
    /// # 参数
    /// * `sound_type` - 要播放的音效类型
    /// * `current_time` - 当前时间
    /// * `min_interval` - 最小播放间隔
    ///
    /// # 返回
    /// 如果可以播放返回 true
    pub fn can_play(&self, sound_type: &SoundType, current_time: f32, min_interval: f32) -> bool {
        if !self.is_playing {
            return true;
        }

        // 检查时间间隔
        if current_time - self.play_time < min_interval {
            return false;
        }

        // 检查优先级
        if let Some(ref last_sound) = self.last_sound {
            sound_type.priority() >= last_sound.priority()
        } else {
            true
        }
    }
}
