//! Explicit frame bounds and foot origins for the repaired Shirou artwork.
//! Generated boards are not assumed to be evenly spaced grids.

use bevy::{prelude::*, sprite::Anchor};
use serde::Deserialize;

use crate::{
    asset_paths,
    components::{AnimationType, AttackAnimationStyle},
    resources::GameConfig,
};

#[derive(Debug, Deserialize)]
pub(crate) struct AuthoredFrame {
    /// Source pixel bounds: left, top, right, bottom (exclusive).
    pub rect: [u32; 4],
    /// Horizontal body origin and foot baseline, in source pixels.
    pub origin: [f32; 2],
}

#[derive(Debug, Deserialize)]
pub(crate) struct AuthoredSheetData {
    pub size: [u32; 2],
    /// Converts source pixels to the existing 256-pixel character canvas.
    pub logical_scale: f32,
    pub frames: Vec<AuthoredFrame>,
}

impl AuthoredSheetData {
    pub fn parse(json: &str) -> Self {
        let data: Self = serde_json::from_str(json).expect("valid authored sprite metadata");
        assert!(data.logical_scale.is_finite() && data.logical_scale > 0.0);
        assert_eq!(data.frames.len(), 16);
        for frame in &data.frames {
            let [left, top, right, bottom] = frame.rect;
            assert!(left < right && top < bottom);
            assert!(right <= data.size[0] && bottom <= data.size[1]);
            assert!(frame.origin.iter().all(|value| value.is_finite()));
        }
        data
    }

    fn layout(&self) -> TextureAtlasLayout {
        let mut layout = TextureAtlasLayout::new_empty(self.size.into());
        for frame in &self.frames {
            let [left, top, right, bottom] = frame.rect;
            layout.add_texture(URect::new(left, top, right, bottom));
        }
        layout
    }
}

pub(crate) struct AuthoredSheet {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
    data: AuthoredSheetData,
}

impl AuthoredSheet {
    fn load(
        path: &'static str,
        metadata: &str,
        server: &AssetServer,
        layouts: &mut Assets<TextureAtlasLayout>,
    ) -> Self {
        let data = AuthoredSheetData::parse(metadata);
        Self {
            image: server.load(path),
            layout: layouts.add(data.layout()),
            data,
        }
    }
}

#[derive(Resource)]
pub struct ShirouSpriteOverrides {
    pub(crate) locomotion: AuthoredSheet,
    ground_attacks: AuthoredSheet,
}

impl ShirouSpriteOverrides {
    pub fn load(server: &AssetServer, layouts: &mut Assets<TextureAtlasLayout>) -> Self {
        Self {
            locomotion: AuthoredSheet::load(
                asset_paths::IMAGE_HF_SHIROU_CORE_SHEET,
                include_str!("../../assets/images/characters/shirou_repaired/locomotion.json"),
                server,
                layouts,
            ),
            ground_attacks: AuthoredSheet::load(
                asset_paths::IMAGE_HF_SHIROU_REPAIRED_GROUND_ATTACKS,
                include_str!("../../assets/images/characters/shirou_repaired/ground_attacks.json"),
                server,
                layouts,
            ),
        }
    }

    pub(crate) fn pose(
        &self,
        animation: &AnimationType,
        style: AttackAnimationStyle,
        atlas_index: usize,
    ) -> Option<AuthoredPose<'_>> {
        let (sheet, index) = if *animation != AnimationType::Attacking {
            (&self.locomotion, atlas_index)
        } else {
            match style {
                AttackAnimationStyle::GroundLightRow(3) => (&self.ground_attacks, atlas_index),
                AttackAnimationStyle::GroundLightRow(4) => (&self.ground_attacks, 8 + atlas_index),
                _ => return None,
            }
        };
        sheet.data.frames.get(index).map(|frame| AuthoredPose {
            sheet,
            frame,
            index,
        })
    }
}

pub(crate) struct AuthoredPose<'a> {
    sheet: &'a AuthoredSheet,
    frame: &'a AuthoredFrame,
    index: usize,
}

impl AuthoredPose<'_> {
    pub fn apply(&self, sprite: &mut Sprite, anchor: &mut Anchor, scale: Vec2, canvas_height: f32) {
        let [left, top, right, bottom] = self.frame.rect;
        let source_size = Vec2::new((right - left) as f32, (bottom - top) as f32);
        let size = source_size * (canvas_height / 256.0 * self.sheet.data.logical_scale) * scale;
        let origin =
            (Vec2::from(self.frame.origin) - Vec2::new(left as f32, top as f32)) / source_size;
        // Anchor is geometry, whereas flip_x only mirrors UVs. Mirror the body
        // origin explicitly so turning does not move the sprite sideways.
        anchor.0 = Vec2::new(
            (origin.x - 0.5) * if sprite.flip_x { -1.0 } else { 1.0 },
            0.5 - origin.y + GameConfig::PLAYER_SIZE.y * 0.5 / size.y,
        );
        sprite.image = self.sheet.image.clone();
        sprite.texture_atlas = Some(TextureAtlas {
            layout: self.sheet.layout.clone(),
            index: self.index,
        });
        sprite.custom_size = Some(size);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_authored_pose_preserves_foot_origin_when_flipped_or_squashed() {
        for json in [
            include_str!("../../assets/images/characters/shirou_repaired/locomotion.json"),
            include_str!("../../assets/images/characters/shirou_repaired/ground_attacks.json"),
        ] {
            let data = AuthoredSheetData::parse(json);
            let sheet = AuthoredSheet {
                image: Handle::default(),
                layout: Handle::default(),
                data,
            };
            for (index, frame) in sheet.data.frames.iter().enumerate() {
                let pose = AuthoredPose {
                    sheet: &sheet,
                    frame,
                    index,
                };
                for flip_x in [false, true] {
                    for scale in [Vec2::ONE, Vec2::new(1.04, 0.94), Vec2::new(0.97, 1.035)] {
                        let mut sprite = Sprite {
                            flip_x,
                            ..default()
                        };
                        let mut anchor = Anchor::default();
                        pose.apply(&mut sprite, &mut anchor, scale, 144.0);
                        let size = sprite.custom_size.unwrap();
                        let [left, top, right, bottom] = frame.rect;
                        let origin = (Vec2::from(frame.origin)
                            - Vec2::new(left as f32, top as f32))
                            / Vec2::new((right - left) as f32, (bottom - top) as f32);
                        let x = (if flip_x { 1.0 - origin.x } else { origin.x } - 0.5 - anchor.0.x)
                            * size.x;
                        let y = (0.5 - origin.y - anchor.0.y) * size.y;
                        assert!(
                            x.abs() < 0.001,
                            "frame {index} moved sideways when mirrored"
                        );
                        assert!(
                            (y + GameConfig::PLAYER_SIZE.y * 0.5).abs() < 0.001,
                            "frame {index} moved its feet when scaled"
                        );
                        assert_eq!(sprite.texture_atlas.unwrap().index, index);
                    }
                }
            }
        }
    }
}
