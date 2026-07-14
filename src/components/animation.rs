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
    GroundLight,          // J/L 站立轻攻击
    GroundLightRow(u8),   // Y/U/I/O/P 站立轻攻击第 1-5 行
    AirCombo,             // J/L 空中轻攻击
    AirComboRow(u8),      // 空中连段第 1-5 行
    HeavyRef,             // K 站立重攻击（地面）
    HeavyRefRow(u8),      // Shift+Y/U/I/O/P 站立重攻击第 1-5 行
    UltimateRef,          // K 蹲着重攻击（必杀技）
    UltimateRefRow(u8),   // 奥义第 1-3 行
    MobilityRef,          // Shift+方向 移动攻击
    MobilityRefRow(u8),   // 移动/闪避第 1-4 行
    NinjutsuRef,          // X 忍术投射
    NinjutsuRefRow(u8),   // 忍术投射第 1-4 行
    WeaponProjRef,        // X 蹲下 武器投射
    WeaponProjRefRow(u8), // 武器投影第 1-4 行
    AdvanceRef,           // Shift+V（模块激活中）高级总览
}

#[derive(Component, Debug, Clone, Default)]
pub struct AttackAnimationState {
    pub remaining: f32,
    /// Original action window used to pace all authored animation frames.
    pub duration: f32,
    pub trigger_serial: u32,
    pub style: AttackAnimationStyle,
}

impl AttackAnimationState {
    pub fn trigger(&mut self, duration_secs: f32) {
        self.trigger_with_style(duration_secs, AttackAnimationStyle::Normal);
    }

    pub fn trigger_with_style(&mut self, duration_secs: f32, style: AttackAnimationStyle) {
        let duration = duration_secs.max(0.0);
        self.remaining = duration;
        self.duration = duration;
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

    /// Reference board 模组使用 reference board 精灵表（Shift+V 未激活）
    pub fn uses_reference_sheet(self) -> bool {
        matches!(
            self,
            AttackAnimationStyle::GroundLight
                | AttackAnimationStyle::GroundLightRow(_)
                | AttackAnimationStyle::AirCombo
                | AttackAnimationStyle::AirComboRow(_)
                | AttackAnimationStyle::HeavyRef
                | AttackAnimationStyle::HeavyRefRow(_)
                | AttackAnimationStyle::UltimateRef
                | AttackAnimationStyle::UltimateRefRow(_)
                | AttackAnimationStyle::MobilityRef
                | AttackAnimationStyle::MobilityRefRow(_)
                | AttackAnimationStyle::NinjutsuRef
                | AttackAnimationStyle::NinjutsuRefRow(_)
                | AttackAnimationStyle::WeaponProjRef
                | AttackAnimationStyle::WeaponProjRefRow(_)
                | AttackAnimationStyle::AdvanceRef
        )
    }

    pub fn reference_row(self) -> Option<u8> {
        match self {
            AttackAnimationStyle::GroundLightRow(row)
            | AttackAnimationStyle::AirComboRow(row)
            | AttackAnimationStyle::HeavyRefRow(row)
            | AttackAnimationStyle::UltimateRefRow(row)
            | AttackAnimationStyle::MobilityRefRow(row)
            | AttackAnimationStyle::NinjutsuRefRow(row)
            | AttackAnimationStyle::WeaponProjRefRow(row) => Some(row),
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
    pub reference_ground_light_row_textures: Vec<Handle<Image>>,
    pub reference_ground_light_layout: Option<Handle<TextureAtlasLayout>>,
    pub reference_ground_light_frame_count: usize,
    pub reference_air_combo_texture: Option<Handle<Image>>,
    pub reference_air_combo_row_textures: Vec<Handle<Image>>,
    pub reference_air_combo_layout: Option<Handle<TextureAtlasLayout>>,
    pub reference_air_combo_frame_count: usize,
    pub reference_heavy_texture: Option<Handle<Image>>,
    pub reference_heavy_row_textures: Vec<Handle<Image>>,
    pub reference_heavy_layout: Option<Handle<TextureAtlasLayout>>,
    pub reference_heavy_frame_count: usize,
    pub reference_ultimate_texture: Option<Handle<Image>>,
    pub reference_ultimate_row_textures: Vec<Handle<Image>>,
    pub reference_ultimate_layout: Option<Handle<TextureAtlasLayout>>,
    pub reference_ultimate_frame_count: usize,
    pub reference_mobility_texture: Option<Handle<Image>>,
    pub reference_mobility_row_textures: Vec<Handle<Image>>,
    pub reference_mobility_layout: Option<Handle<TextureAtlasLayout>>,
    pub reference_mobility_frame_count: usize,
    pub reference_ninjutsu_texture: Option<Handle<Image>>,
    pub reference_ninjutsu_row_textures: Vec<Handle<Image>>,
    pub reference_ninjutsu_layout: Option<Handle<TextureAtlasLayout>>,
    pub reference_ninjutsu_frame_count: usize,
    pub reference_weapon_proj_texture: Option<Handle<Image>>,
    pub reference_weapon_proj_row_textures: Vec<Handle<Image>>,
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
                let reference_sheet =
                    match attack_style {
                        AttackAnimationStyle::GroundLight
                        | AttackAnimationStyle::GroundLightRow(_) => self.reference_row_sheet(
                            attack_style.reference_row(),
                            &self.reference_ground_light_row_textures,
                            self.reference_ground_light_texture.as_ref(),
                            self.reference_ground_light_layout.as_ref(),
                        ),
                        AttackAnimationStyle::AirCombo | AttackAnimationStyle::AirComboRow(_) => {
                            self.reference_row_sheet(
                                attack_style.reference_row(),
                                &self.reference_air_combo_row_textures,
                                self.reference_air_combo_texture.as_ref(),
                                self.reference_air_combo_layout.as_ref(),
                            )
                        }
                        AttackAnimationStyle::HeavyRef | AttackAnimationStyle::HeavyRefRow(_) => {
                            self.reference_row_sheet(
                                attack_style.reference_row(),
                                &self.reference_heavy_row_textures,
                                self.reference_heavy_texture.as_ref(),
                                self.reference_heavy_layout.as_ref(),
                            )
                        }
                        AttackAnimationStyle::UltimateRef
                        | AttackAnimationStyle::UltimateRefRow(_) => self.reference_row_sheet(
                            attack_style.reference_row(),
                            &self.reference_ultimate_row_textures,
                            self.reference_ultimate_texture.as_ref(),
                            self.reference_ultimate_layout.as_ref(),
                        ),
                        AttackAnimationStyle::MobilityRef
                        | AttackAnimationStyle::MobilityRefRow(_) => self.reference_row_sheet(
                            attack_style.reference_row(),
                            &self.reference_mobility_row_textures,
                            self.reference_mobility_texture.as_ref(),
                            self.reference_mobility_layout.as_ref(),
                        ),
                        AttackAnimationStyle::NinjutsuRef
                        | AttackAnimationStyle::NinjutsuRefRow(_) => self.reference_row_sheet(
                            attack_style.reference_row(),
                            &self.reference_ninjutsu_row_textures,
                            self.reference_ninjutsu_texture.as_ref(),
                            self.reference_ninjutsu_layout.as_ref(),
                        ),
                        AttackAnimationStyle::WeaponProjRef
                        | AttackAnimationStyle::WeaponProjRefRow(_) => self.reference_row_sheet(
                            attack_style.reference_row(),
                            &self.reference_weapon_proj_row_textures,
                            self.reference_weapon_proj_texture.as_ref(),
                            self.reference_weapon_proj_layout.as_ref(),
                        ),
                        AttackAnimationStyle::AdvanceRef => self
                            .reference_advance_texture
                            .as_ref()
                            .zip(self.reference_advance_layout.as_ref()),
                        _ => None,
                    };
                if let Some(sheet) = reference_sheet {
                    return Some(sheet);
                }
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

    fn reference_row_sheet<'a>(
        &'a self,
        row: Option<u8>,
        row_textures: &'a [Handle<Image>],
        fallback_texture: Option<&'a Handle<Image>>,
        layout: Option<&'a Handle<TextureAtlasLayout>>,
    ) -> Option<(&'a Handle<Image>, &'a Handle<TextureAtlasLayout>)> {
        let layout = layout?;
        let row_index = row.unwrap_or(1).saturating_sub(1) as usize;
        row_textures
            .get(row_index)
            .or_else(|| row_textures.first())
            .or(fallback_texture)
            .map(|texture| (texture, layout))
    }

    fn reference_sheet_frame_count(
        row_textures: &[Handle<Image>],
        fallback_texture: &Option<Handle<Image>>,
        layout: &Option<Handle<TextureAtlasLayout>>,
        frame_count: usize,
    ) -> Option<usize> {
        let has_texture = !row_textures.is_empty() || fallback_texture.is_some();
        (has_texture && layout.is_some() && frame_count > 0).then_some(frame_count)
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
            GroundLight | GroundLightRow(_) => Self::reference_sheet_frame_count(
                &self.reference_ground_light_row_textures,
                &self.reference_ground_light_texture,
                &self.reference_ground_light_layout,
                self.reference_ground_light_frame_count,
            ),
            AirCombo | AirComboRow(_) => Self::reference_sheet_frame_count(
                &self.reference_air_combo_row_textures,
                &self.reference_air_combo_texture,
                &self.reference_air_combo_layout,
                self.reference_air_combo_frame_count,
            ),
            HeavyRef | HeavyRefRow(_) => Self::reference_sheet_frame_count(
                &self.reference_heavy_row_textures,
                &self.reference_heavy_texture,
                &self.reference_heavy_layout,
                self.reference_heavy_frame_count,
            ),
            UltimateRef | UltimateRefRow(_) => Self::reference_sheet_frame_count(
                &self.reference_ultimate_row_textures,
                &self.reference_ultimate_texture,
                &self.reference_ultimate_layout,
                self.reference_ultimate_frame_count,
            ),
            MobilityRef | MobilityRefRow(_) => Self::reference_sheet_frame_count(
                &self.reference_mobility_row_textures,
                &self.reference_mobility_texture,
                &self.reference_mobility_layout,
                self.reference_mobility_frame_count,
            ),
            NinjutsuRef | NinjutsuRefRow(_) => Self::reference_sheet_frame_count(
                &self.reference_ninjutsu_row_textures,
                &self.reference_ninjutsu_texture,
                &self.reference_ninjutsu_layout,
                self.reference_ninjutsu_frame_count,
            ),
            WeaponProjRef | WeaponProjRefRow(_) => Self::reference_sheet_frame_count(
                &self.reference_weapon_proj_row_textures,
                &self.reference_weapon_proj_texture,
                &self.reference_weapon_proj_layout,
                self.reference_weapon_proj_frame_count,
            ),
            AdvanceRef => (self.reference_advance_texture.is_some()
                && self.reference_advance_layout.is_some()
                && self.reference_advance_frame_count > 0)
                .then_some(self.reference_advance_frame_count),
            Normal => None,
        }
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
