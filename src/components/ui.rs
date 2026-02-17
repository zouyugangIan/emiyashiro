//! UI相关组件
//!
//! 包含用户界面元素的组件定义。

use crate::states::CharacterType;
use bevy::prelude::*;

/// UI组件标记
///
/// 用于标识菜单UI实体。
#[derive(Component, Debug)]
pub struct MenuUI;

/// 开始按钮组件
///
/// 标识主菜单中的开始游戏按钮。
#[derive(Component, Debug)]
pub struct StartButton;

/// 存档按钮组件
///
/// 标识主菜单中的存档相关按钮。
#[derive(Component, Debug)]
pub struct SaveButton;

/// 角色选择按钮组件
///
/// 包含角色类型信息的按钮组件。
///
/// # 字段
/// * `character_type` - 按钮对应的角色类型
#[derive(Component, Debug)]
pub struct CharacterSelectButton {
    pub character_type: CharacterType,
}

impl CharacterSelectButton {
    /// 创建新的角色选择按钮
    ///
    /// # 参数
    /// * `character_type` - 角色类型
    ///
    /// # 返回
    /// 新的 CharacterSelectButton 实例
    pub fn new(character_type: CharacterType) -> Self {
        Self { character_type }
    }
}

/// 封面图片1组件
///
/// 标识主菜单的第一张封面图片。
///
/// 此组件用于UI节点（Node + ImageNode），实现响应式布局。
/// 封面图片使用绝对定位填充整个窗口（100% 宽度和高度）。
///
/// # 使用
/// 与 `Node`, `ImageNode`, `BackgroundColor`, 和 `CoverFadeState` 组件配合使用。
#[derive(Component, Debug)]
pub struct CoverImage1;

/// 封面图片2组件
///
/// 标识主菜单的第二张封面图片。
///
/// 此组件用于UI节点（Node + ImageNode），实现响应式布局。
/// 封面图片使用绝对定位填充整个窗口（100% 宽度和高度）。
///
/// # 使用
/// 与 `Node`, `ImageNode`, `BackgroundColor`, 和 `CoverFadeState` 组件配合使用。
#[derive(Component, Debug)]
pub struct CoverImage2;

/// 封面渐变状态组件
///
/// 控制封面图片的渐变动画效果。
///
/// 此组件与 `BackgroundColor` 配合使用，通过修改 UI 节点的背景颜色透明度
/// 来实现平滑的渐变动画效果。动画使用 smoothstep 缓动函数，
/// 在 15 秒的周期内实现两张封面图片的互补渐变。
///
/// # 字段
/// * `alpha` - 当前透明度 (0.0-1.0)
/// * `fade_direction` - 渐变方向 (1.0 为第一张图片, -1.0 为第二张图片)
///
/// # 示例
///
/// ```rust
/// use s_emiyashiro::components::CoverFadeState;
///
/// // 第一张图片从完全不透明开始
/// let fade_state_1 = CoverFadeState::default();
///
/// // 第二张图片从透明开始
/// let fade_state_2 = CoverFadeState::new(0.0, -1.0);
/// ```
#[derive(Component, Debug, Clone)]
pub struct CoverFadeState {
    pub alpha: f32,
    pub fade_direction: f32,
}

impl Default for CoverFadeState {
    fn default() -> Self {
        Self {
            alpha: 1.0,
            fade_direction: -1.0,
        }
    }
}

impl CoverFadeState {
    /// 创建新的渐变状态
    ///
    /// # 参数
    /// * `alpha` - 初始透明度
    /// * `fade_direction` - 渐变方向
    ///
    /// # 返回
    /// 新的 CoverFadeState 实例
    pub fn new(alpha: f32, fade_direction: f32) -> Self {
        Self {
            alpha,
            fade_direction,
        }
    }

    /// 更新渐变状态
    ///
    /// # 参数
    /// * `delta_time` - 时间增量
    /// * `fade_speed` - 渐变速度
    pub fn update(&mut self, delta_time: f32, fade_speed: f32) {
        self.alpha += self.fade_direction * fade_speed * delta_time;
        self.alpha = self.alpha.clamp(0.0, 1.0);

        // 到达边界时反转方向
        if self.alpha <= 0.0 || self.alpha >= 1.0 {
            self.fade_direction *= -1.0;
        }
    }
}
