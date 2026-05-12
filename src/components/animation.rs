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
/// * `Attacking` - 攻击动画
/// * `Jumping` - 跳跃动画
/// * `Crouching` - 蹲下动画
/// * `Landing` - 着陆动画
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Deserialize)]
pub enum AnimationType {
    Idle,
    Running,
    Attacking,
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
            AnimationType::Attacking => 0.08,
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
            AnimationType::Attacking | AnimationType::Jumping | AnimationType::Landing => false,
        }
    }

    pub fn sprite_sheet_kind(&self) -> SpriteSheetKind {
        match self {
            AnimationType::Running => SpriteSheetKind::Running,
            AnimationType::Attacking => SpriteSheetKind::Attacking,
            AnimationType::Idle
            | AnimationType::Jumping
            | AnimationType::Crouching
            | AnimationType::Landing => SpriteSheetKind::Core,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpriteSheetKind {
    Core,
    Running,
    Attacking,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AttackAnimationStyle {
    #[default]
    Normal,
    // --- Overedge 模组（Shift+V 激活后使用 overedge 精灵表）---
    OveredgeRelease,
    OveredgeLight1,
    OveredgeLight2,
    OveredgeLight3,
    OveredgeHeavy,
    // --- Reference Board 模组（Shift+V 未激活时使用 reference board 精灵表）---
    GroundLight,        // J/L 站立轻攻击
    GroundLightRow(u8), // Y/U/I/O/P 站立轻攻击第 1-5 行
    AirCombo,           // J/L 空中轻攻击
    HeavyRef,           // K 站立重攻击（地面）
    HeavyRefRow(u8),    // Shift+Y/U/I/O/P 站立重攻击第 1-5 行
    UltimateRef,        // K 蹲着重攻击（必杀技）
    MobilityRef,        // Shift+方向 移动攻击
    NinjutsuRef,        // X 忍术投射
    WeaponProjRef,      // X 蹲下 武器投射
    AdvanceRef,         // Shift+V（模块激活中）高级总览
}

#[derive(Component, Debug, Clone, Default)]
pub struct AttackAnimationState {
    pub remaining: f32,
    pub trigger_serial: u32,
    pub style: AttackAnimationStyle,
}

impl AttackAnimationState {
    pub fn trigger(&mut self, duration_secs: f32) {
        self.trigger_with_style(duration_secs, AttackAnimationStyle::Normal);
    }

    pub fn trigger_with_style(&mut self, duration_secs: f32, style: AttackAnimationStyle) {
        self.remaining = duration_secs.max(0.0);
        self.trigger_serial = self.trigger_serial.wrapping_add(1);
        self.style = style;
    }

    pub fn tick(&mut self, delta_secs: f32) {
        self.remaining = (self.remaining - delta_secs).max(0.0);
    }

    pub fn is_active(&self) -> bool {
        self.remaining > 0.0
    }
}

impl AttackAnimationStyle {
    pub fn is_overedge_light(self) -> bool {
        matches!(
            self,
            AttackAnimationStyle::OveredgeLight1
                | AttackAnimationStyle::OveredgeLight2
                | AttackAnimationStyle::OveredgeLight3
        )
    }

    /// Reference board 模组使用 overedge 精灵表（Shift+V 激活）
    pub fn uses_overedge_sheet(self) -> bool {
        matches!(
            self,
            AttackAnimationStyle::OveredgeRelease
                | AttackAnimationStyle::OveredgeLight1
                | AttackAnimationStyle::OveredgeLight2
                | AttackAnimationStyle::OveredgeLight3
                | AttackAnimationStyle::OveredgeHeavy
        )
    }

    /// Reference board 模组使用 reference board 精灵表（Shift+V 未激活）
    pub fn uses_reference_sheet(self) -> bool {
        matches!(
            self,
            AttackAnimationStyle::GroundLight
                | AttackAnimationStyle::GroundLightRow(_)
                | AttackAnimationStyle::AirCombo
                | AttackAnimationStyle::HeavyRef
                | AttackAnimationStyle::HeavyRefRow(_)
                | AttackAnimationStyle::UltimateRef
                | AttackAnimationStyle::MobilityRef
                | AttackAnimationStyle::NinjutsuRef
                | AttackAnimationStyle::WeaponProjRef
                | AttackAnimationStyle::AdvanceRef
        )
    }

    pub fn reference_row(self) -> Option<u8> {
        match self {
            AttackAnimationStyle::GroundLightRow(row) | AttackAnimationStyle::HeavyRefRow(row) => {
                Some(row)
            }
            _ => None,
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct SpriteAnimationSheets {
    pub core_texture: Handle<Image>,
    pub core_layout: Handle<TextureAtlasLayout>,
    pub running_texture: Handle<Image>,
    pub running_layout: Handle<TextureAtlasLayout>,
    pub attacking_texture: Handle<Image>,
    pub attacking_layout: Handle<TextureAtlasLayout>,
    // --- Overedge 精灵表（Shift+V 激活后使用）---
    pub overedge_light_attacking_texture: Option<Handle<Image>>,
    pub overedge_light_attacking_layout: Option<Handle<TextureAtlasLayout>>,
    pub overedge_light_attacking_frame_count: usize,
    pub overedge_heavy_attacking_texture: Option<Handle<Image>>,
    pub overedge_heavy_attacking_layout: Option<Handle<TextureAtlasLayout>>,
    pub overedge_heavy_attacking_frame_count: usize,
    // --- Reference Board 精灵表（Shift+V 未激活时使用）---
    pub reference_ground_light_texture: Option<Handle<Image>>,
    pub reference_ground_light_layout: Option<Handle<TextureAtlasLayout>>,
    pub reference_ground_light_frame_count: usize,
    pub reference_air_combo_texture: Option<Handle<Image>>,
    pub reference_air_combo_layout: Option<Handle<TextureAtlasLayout>>,
    pub reference_air_combo_frame_count: usize,
    pub reference_heavy_texture: Option<Handle<Image>>,
    pub reference_heavy_layout: Option<Handle<TextureAtlasLayout>>,
    pub reference_heavy_frame_count: usize,
    pub reference_ultimate_texture: Option<Handle<Image>>,
    pub reference_ultimate_layout: Option<Handle<TextureAtlasLayout>>,
    pub reference_ultimate_frame_count: usize,
    pub reference_mobility_texture: Option<Handle<Image>>,
    pub reference_mobility_layout: Option<Handle<TextureAtlasLayout>>,
    pub reference_mobility_frame_count: usize,
    pub reference_ninjutsu_texture: Option<Handle<Image>>,
    pub reference_ninjutsu_layout: Option<Handle<TextureAtlasLayout>>,
    pub reference_ninjutsu_frame_count: usize,
    pub reference_weapon_proj_texture: Option<Handle<Image>>,
    pub reference_weapon_proj_layout: Option<Handle<TextureAtlasLayout>>,
    pub reference_weapon_proj_frame_count: usize,
    pub reference_advance_texture: Option<Handle<Image>>,
    pub reference_advance_layout: Option<Handle<TextureAtlasLayout>>,
    pub reference_advance_frame_count: usize,
}

impl SpriteAnimationSheets {
    pub fn select_sheet(
        &self,
        animation_type: &AnimationType,
    ) -> (&Handle<Image>, &Handle<TextureAtlasLayout>) {
        match animation_type.sprite_sheet_kind() {
            SpriteSheetKind::Core => (&self.core_texture, &self.core_layout),
            SpriteSheetKind::Running => (&self.running_texture, &self.running_layout),
            SpriteSheetKind::Attacking => (&self.attacking_texture, &self.attacking_layout),
        }
    }

    pub fn select_sheet_for_attack_style(
        &self,
        animation_type: &AnimationType,
        attack_style: AttackAnimationStyle,
    ) -> Option<(&Handle<Image>, &Handle<TextureAtlasLayout>)> {
        if *animation_type == AnimationType::Attacking {
            // Reference board 模组优先
            if attack_style.uses_reference_sheet() {
                return match attack_style {
                    AttackAnimationStyle::GroundLight | AttackAnimationStyle::GroundLightRow(_) => {
                        Some((
                            self.reference_ground_light_texture.as_ref()?,
                            self.reference_ground_light_layout.as_ref()?,
                        ))
                    }
                    AttackAnimationStyle::AirCombo => Some((
                        self.reference_air_combo_texture.as_ref()?,
                        self.reference_air_combo_layout.as_ref()?,
                    )),
                    AttackAnimationStyle::HeavyRef | AttackAnimationStyle::HeavyRefRow(_) => {
                        Some((
                            self.reference_heavy_texture.as_ref()?,
                            self.reference_heavy_layout.as_ref()?,
                        ))
                    }
                    AttackAnimationStyle::UltimateRef => Some((
                        self.reference_ultimate_texture.as_ref()?,
                        self.reference_ultimate_layout.as_ref()?,
                    )),
                    AttackAnimationStyle::MobilityRef => Some((
                        self.reference_mobility_texture.as_ref()?,
                        self.reference_mobility_layout.as_ref()?,
                    )),
                    AttackAnimationStyle::NinjutsuRef => Some((
                        self.reference_ninjutsu_texture.as_ref()?,
                        self.reference_ninjutsu_layout.as_ref()?,
                    )),
                    AttackAnimationStyle::WeaponProjRef => Some((
                        self.reference_weapon_proj_texture.as_ref()?,
                        self.reference_weapon_proj_layout.as_ref()?,
                    )),
                    AttackAnimationStyle::AdvanceRef => Some((
                        self.reference_advance_texture.as_ref()?,
                        self.reference_advance_layout.as_ref()?,
                    )),
                    _ => None,
                };
            }
            // Overedge 模组
            match attack_style {
                AttackAnimationStyle::OveredgeRelease
                | AttackAnimationStyle::OveredgeLight1
                | AttackAnimationStyle::OveredgeLight2
                | AttackAnimationStyle::OveredgeLight3 => {
                    if let (Some(texture), Some(layout)) = (
                        self.overedge_light_attacking_texture.as_ref(),
                        self.overedge_light_attacking_layout.as_ref(),
                    ) {
                        return Some((texture, layout));
                    }
                }
                AttackAnimationStyle::OveredgeHeavy => {
                    if let (Some(texture), Some(layout)) = (
                        self.overedge_heavy_attacking_texture.as_ref(),
                        self.overedge_heavy_attacking_layout.as_ref(),
                    ) {
                        return Some((texture, layout));
                    }
                }
                AttackAnimationStyle::Normal => {}
                _ => {}
            }
        }

        Some(self.select_sheet(animation_type))
    }

    pub fn attacking_frame_count(&self, attack_style: AttackAnimationStyle) -> Option<usize> {
        use AttackAnimationStyle::*;
        match attack_style {
            OveredgeRelease | OveredgeLight1 | OveredgeLight2 | OveredgeLight3 => {
                (self.overedge_light_attacking_texture.is_some()
                    && self.overedge_light_attacking_layout.is_some()
                    && self.overedge_light_attacking_frame_count > 0)
                    .then_some(self.overedge_light_attacking_frame_count)
            }
            OveredgeHeavy => (self.overedge_heavy_attacking_texture.is_some()
                && self.overedge_heavy_attacking_layout.is_some()
                && self.overedge_heavy_attacking_frame_count > 0)
                .then_some(self.overedge_heavy_attacking_frame_count),
            GroundLight | GroundLightRow(_) => (self.reference_ground_light_texture.is_some()
                && self.reference_ground_light_layout.is_some()
                && self.reference_ground_light_frame_count > 0)
                .then_some(self.reference_ground_light_frame_count),
            AirCombo => (self.reference_air_combo_texture.is_some()
                && self.reference_air_combo_layout.is_some()
                && self.reference_air_combo_frame_count > 0)
                .then_some(self.reference_air_combo_frame_count),
            HeavyRef | HeavyRefRow(_) => (self.reference_heavy_texture.is_some()
                && self.reference_heavy_layout.is_some()
                && self.reference_heavy_frame_count > 0)
                .then_some(self.reference_heavy_frame_count),
            UltimateRef => (self.reference_ultimate_texture.is_some()
                && self.reference_ultimate_layout.is_some()
                && self.reference_ultimate_frame_count > 0)
                .then_some(self.reference_ultimate_frame_count),
            MobilityRef => (self.reference_mobility_texture.is_some()
                && self.reference_mobility_layout.is_some()
                && self.reference_mobility_frame_count > 0)
                .then_some(self.reference_mobility_frame_count),
            NinjutsuRef => (self.reference_ninjutsu_texture.is_some()
                && self.reference_ninjutsu_layout.is_some()
                && self.reference_ninjutsu_frame_count > 0)
                .then_some(self.reference_ninjutsu_frame_count),
            WeaponProjRef => (self.reference_weapon_proj_texture.is_some()
                && self.reference_weapon_proj_layout.is_some()
                && self.reference_weapon_proj_frame_count > 0)
                .then_some(self.reference_weapon_proj_frame_count),
            AdvanceRef => (self.reference_advance_texture.is_some()
                && self.reference_advance_layout.is_some()
                && self.reference_advance_frame_count > 0)
                .then_some(self.reference_advance_frame_count),
            Normal => None,
        }
    }

    /// 兼容旧方法（overedge 特化）
    pub fn overedge_attacking_frame_count(
        &self,
        attack_style: AttackAnimationStyle,
    ) -> Option<usize> {
        self.attacking_frame_count(attack_style)
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
/// use emiyashiro::components::{AnimationType, CharacterAnimation};
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
            AnimationType::Attacking => &self.running_frames,
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
            AnimationType::Attacking => &mut self.running_frames,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn animation_types_map_to_expected_sheet_kind() {
        assert_eq!(
            AnimationType::Idle.sprite_sheet_kind(),
            SpriteSheetKind::Core
        );
        assert_eq!(
            AnimationType::Jumping.sprite_sheet_kind(),
            SpriteSheetKind::Core
        );
        assert_eq!(
            AnimationType::Crouching.sprite_sheet_kind(),
            SpriteSheetKind::Core
        );
        assert_eq!(
            AnimationType::Landing.sprite_sheet_kind(),
            SpriteSheetKind::Core
        );
        assert_eq!(
            AnimationType::Running.sprite_sheet_kind(),
            SpriteSheetKind::Running
        );
        assert_eq!(
            AnimationType::Attacking.sprite_sheet_kind(),
            SpriteSheetKind::Attacking
        );
    }
}
