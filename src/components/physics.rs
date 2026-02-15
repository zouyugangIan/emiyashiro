//! 物理相关组件
//!
//! 包含物理模拟所需的组件，如速度、碰撞等。

use bevy::prelude::*;

/// 速度组件
///
/// 存储实体的二维速度信息，用于物理系统计算移动。
///
/// # 字段
/// * `x` - 水平速度（像素/秒）
/// * `y` - 垂直速度（像素/秒）
///
/// # 示例
///
/// ```rust
/// use crate::components::Velocity;
///
/// let velocity = Velocity::new(100.0, 200.0);
/// let speed = velocity.length();
/// println!("速度大小: {}", speed);
/// ```
#[derive(Component, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

impl Velocity {
    /// 创建新的速度组件
    ///
    /// # 参数
    /// * `x` - 水平速度
    /// * `y` - 垂直速度
    ///
    /// # 返回
    /// 新的 Velocity 实例
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// 创建零速度
    ///
    /// # 返回
    /// 速度为 (0, 0) 的 Velocity 实例
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    /// 获取速度的长度（标量）
    ///
    /// # 返回
    /// 速度向量的长度
    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// 获取速度的平方长度
    ///
    /// 用于性能优化，避免开方运算。
    ///
    /// # 返回
    /// 速度向量长度的平方
    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    /// 归一化速度向量
    ///
    /// # 返回
    /// 单位长度的速度向量
    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > 0.0 {
            Self {
                x: self.x / len,
                y: self.y / len,
            }
        } else {
            Self::zero()
        }
    }

    /// 限制速度大小
    ///
    /// # 参数
    /// * `max_length` - 最大速度
    ///
    /// # 返回
    /// 限制后的速度向量
    pub fn clamp_length(&self, max_length: f32) -> Self {
        let len = self.length();
        if len > max_length {
            let scale = max_length / len;
            Self {
                x: self.x * scale,
                y: self.y * scale,
            }
        } else {
            self.clone()
        }
    }
}

impl Default for Velocity {
    fn default() -> Self {
        Self::zero()
    }
}

/// 地面组件标记
///
/// 用于标识地面实体，供碰撞检测系统使用。
#[derive(Component, Debug)]
pub struct Ground;
