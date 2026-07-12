//! 樱的逐张图片动画正式实现。
//!
//! HF 士郎使用 `sprite_animation` 中的 TextureAtlas 主链。樱的原画不是规则网格，
//! 因此正式采用逐张图片动画；两种渲染方式共享同一套玩家状态语义。

use crate::{asset_paths, components::*, states::*};
use bevy::image::{ImageLoaderSettings, ImageSampler};
use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct ImageSequenceAnimation {
    frames: Vec<Handle<Image>>,
    current_frame: usize,
    timer: Timer,
    looping: bool,
}

impl ImageSequenceAnimation {
    fn new(frames: Vec<Handle<Image>>, frame_duration: f32, looping: bool) -> Self {
        Self {
            frames,
            current_frame: 0,
            timer: Timer::from_seconds(frame_duration, TimerMode::Repeating),
            looping,
        }
    }

    fn replace_clip(&mut self, frames: Vec<Handle<Image>>, frame_duration: f32, looping: bool) {
        self.frames = frames;
        self.current_frame = 0;
        self.looping = looping;
        self.timer
            .set_duration(std::time::Duration::from_secs_f32(frame_duration));
        self.timer.reset();
        self.timer.unpause();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImageSequenceAnimationType {
    Idle,
    Running,
    Jumping,
    Crouching,
    Attacking,
}

#[derive(Component, Debug)]
pub struct ImageSequenceAnimationState {
    current: ImageSequenceAnimationType,
    idle_frames: Vec<Handle<Image>>,
    running_frames: Vec<Handle<Image>>,
    jumping_frames: Vec<Handle<Image>>,
    crouching_frames: Vec<Handle<Image>>,
}

impl ImageSequenceAnimationState {
    fn clip(&self, animation_type: ImageSequenceAnimationType) -> (&[Handle<Image>], f32, bool) {
        match animation_type {
            ImageSequenceAnimationType::Idle => (&self.idle_frames, 0.15, true),
            ImageSequenceAnimationType::Running => (&self.running_frames, 0.10, true),
            ImageSequenceAnimationType::Jumping => (&self.jumping_frames, 0.12, false),
            ImageSequenceAnimationType::Crouching => (&self.crouching_frames, 0.15, true),
            ImageSequenceAnimationType::Attacking => (&self.idle_frames, 0.15, false),
        }
    }
}

fn load_frames(asset_server: &AssetServer, paths: &[&'static str]) -> Vec<Handle<Image>> {
    paths.iter().map(|path| asset_server.load(*path)).collect()
}

#[derive(Clone, Debug)]
struct SakuraAttackAtlas {
    texture: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
    columns: usize,
    rows: u8,
}

#[derive(Component, Clone, Debug)]
pub struct SakuraAttackAtlases {
    ground_light: SakuraAttackAtlas,
    heavy: SakuraAttackAtlas,
    air_combo: SakuraAttackAtlas,
    mobility: SakuraAttackAtlas,
    ninjutsu: SakuraAttackAtlas,
    ultimate: SakuraAttackAtlas,
    weapon_projection: SakuraAttackAtlas,
}

impl SakuraAttackAtlases {
    fn select(
        &self,
        style: AttackAnimationStyle,
    ) -> (Handle<Image>, Handle<TextureAtlasLayout>, usize, usize) {
        use AttackAnimationStyle::*;

        let (atlas, row) = match style {
            GroundLight => (&self.ground_light, 1),
            GroundLightRow(row) => (&self.ground_light, row),
            AirCombo => (&self.air_combo, 1),
            AirComboRow(row) => (&self.air_combo, row),
            HeavyRef => (&self.heavy, 1),
            HeavyRefRow(row) => (&self.heavy, row),
            UltimateRef => (&self.ultimate, 1),
            UltimateRefRow(row) => (&self.ultimate, row),
            MobilityRef => (&self.mobility, 1),
            MobilityRefRow(row) => (&self.mobility, row),
            NinjutsuRef => (&self.ninjutsu, 1),
            NinjutsuRefRow(row) => (&self.ninjutsu, row),
            WeaponProjRef => (&self.weapon_projection, 1),
            WeaponProjRefRow(row) => (&self.weapon_projection, row),
            OveredgeRelease | OveredgeLight1 | Normal => (&self.ground_light, 1),
            OveredgeLight2 => (&self.ground_light, 3),
            OveredgeLight3 => (&self.ground_light, 5),
            OveredgeHeavy => (&self.heavy, 5),
            AdvanceRef => (&self.ultimate, 3),
        };
        let row = row.clamp(1, atlas.rows) as usize;
        let first_frame = (row - 1) * atlas.columns;
        (
            atlas.texture.clone(),
            atlas.layout.clone(),
            first_frame,
            atlas.columns,
        )
    }
}

#[derive(Component, Debug)]
pub struct SakuraAttackAtlasPlayback {
    trigger_serial: u32,
    first_frame: usize,
    frame_count: usize,
    current_frame: usize,
    timer: Timer,
}

impl Default for SakuraAttackAtlasPlayback {
    fn default() -> Self {
        Self {
            trigger_serial: u32::MAX,
            first_frame: 0,
            frame_count: 1,
            current_frame: 0,
            timer: Timer::from_seconds(0.07, TimerMode::Repeating),
        }
    }
}

fn attack_frame_duration(attack_duration: f32, frame_count: usize) -> f32 {
    let transitions = frame_count.saturating_sub(1).max(1) as f32;
    (attack_duration.max(0.0) / transitions).clamp(1.0 / 120.0, 0.09)
}

impl SakuraAttackAtlasPlayback {
    fn restart(
        &mut self,
        trigger_serial: u32,
        first_frame: usize,
        frame_count: usize,
        attack_duration: f32,
    ) {
        self.trigger_serial = trigger_serial;
        self.first_frame = first_frame;
        self.frame_count = frame_count.max(1);
        self.current_frame = 0;
        let frame_duration = attack_frame_duration(attack_duration, self.frame_count);
        self.timer
            .set_duration(std::time::Duration::from_secs_f32(frame_duration));
        self.timer.reset();
        self.timer.unpause();
    }
}

fn attack_layout(
    texture_atlases: &mut Assets<TextureAtlasLayout>,
    grid: (u32, u32),
) -> Handle<TextureAtlasLayout> {
    texture_atlases.add(TextureAtlasLayout::from_grid(
        UVec2::from(asset_paths::SAKURA_ATTACK_CELL),
        grid.0,
        grid.1,
        None,
        None,
    ))
}

fn attack_atlas(
    asset_server: &AssetServer,
    path: &'static str,
    layout: Handle<TextureAtlasLayout>,
    grid: (u32, u32),
) -> SakuraAttackAtlas {
    SakuraAttackAtlas {
        texture: asset_server
            .load_builder()
            .with_settings(|settings: &mut ImageLoaderSettings| {
                settings.sampler = ImageSampler::nearest();
            })
            .load(path),
        layout,
        columns: grid.0 as usize,
        rows: grid.1 as u8,
    }
}

/// 在樱刚生成时添加逐张图片动画组件。
pub fn setup_sakura_image_sequence_animation(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    player_query: Query<
        Entity,
        (
            Added<Player>,
            Without<crate::systems::sprite_animation::SpriteAnimation>,
        ),
    >,
    character_selection: Res<CharacterSelection>,
) {
    if character_selection.selected_character != CharacterType::Sakura {
        return;
    }

    for entity in player_query.iter() {
        let idle_frames = load_frames(&asset_server, asset_paths::SAKURA_IDLE_FRAMES);
        let running_frames = load_frames(&asset_server, asset_paths::SAKURA_RUNNING_FRAMES);
        let jumping_frames = load_frames(&asset_server, asset_paths::SAKURA_JUMPING_FRAMES);
        let crouching_frames = load_frames(&asset_server, asset_paths::SAKURA_CROUCHING_FRAMES);
        let layout_8x5 = attack_layout(
            &mut texture_atlases,
            asset_paths::SAKURA_ATTACK_GROUND_LIGHT_GRID,
        );
        let layout_8x4 = attack_layout(
            &mut texture_atlases,
            asset_paths::SAKURA_ATTACK_NINJUTSU_GRID,
        );
        let layout_6x4 = attack_layout(
            &mut texture_atlases,
            asset_paths::SAKURA_ATTACK_MOBILITY_GRID,
        );
        let layout_8x3 = attack_layout(
            &mut texture_atlases,
            asset_paths::SAKURA_ATTACK_ULTIMATE_GRID,
        );
        let attack_atlases = SakuraAttackAtlases {
            ground_light: attack_atlas(
                &asset_server,
                asset_paths::IMAGE_SAKURA_ATTACK_GROUND_LIGHT,
                layout_8x5.clone(),
                asset_paths::SAKURA_ATTACK_GROUND_LIGHT_GRID,
            ),
            heavy: attack_atlas(
                &asset_server,
                asset_paths::IMAGE_SAKURA_ATTACK_HEAVY,
                layout_8x5.clone(),
                asset_paths::SAKURA_ATTACK_HEAVY_GRID,
            ),
            air_combo: attack_atlas(
                &asset_server,
                asset_paths::IMAGE_SAKURA_ATTACK_AIR_COMBO,
                layout_8x5,
                asset_paths::SAKURA_ATTACK_AIR_COMBO_GRID,
            ),
            mobility: attack_atlas(
                &asset_server,
                asset_paths::IMAGE_SAKURA_ATTACK_MOBILITY,
                layout_6x4.clone(),
                asset_paths::SAKURA_ATTACK_MOBILITY_GRID,
            ),
            ninjutsu: attack_atlas(
                &asset_server,
                asset_paths::IMAGE_SAKURA_ATTACK_NINJUTSU,
                layout_8x4,
                asset_paths::SAKURA_ATTACK_NINJUTSU_GRID,
            ),
            ultimate: attack_atlas(
                &asset_server,
                asset_paths::IMAGE_SAKURA_ATTACK_ULTIMATE,
                layout_8x3,
                asset_paths::SAKURA_ATTACK_ULTIMATE_GRID,
            ),
            weapon_projection: attack_atlas(
                &asset_server,
                asset_paths::IMAGE_SAKURA_ATTACK_WEAPON_PROJECTION,
                layout_6x4,
                asset_paths::SAKURA_ATTACK_WEAPON_PROJECTION_GRID,
            ),
        };

        commands.entity(entity).insert((
            ImageSequenceAnimation::new(idle_frames.clone(), 0.15, true),
            ImageSequenceAnimationState {
                current: ImageSequenceAnimationType::Idle,
                idle_frames,
                running_frames,
                jumping_frames,
                crouching_frames,
            },
            attack_atlases,
            SakuraAttackAtlasPlayback::default(),
        ));
    }
}

/// 将玩家状态映射到樱的逐帧图片动画，切换时立即显示第一帧。
pub fn update_image_sequence_animation_state(
    mut query: Query<
        (
            &PlayerState,
            &Velocity,
            Option<&AttackAnimationState>,
            Option<&SakuraAttackAtlases>,
            Option<&mut SakuraAttackAtlasPlayback>,
            &mut ImageSequenceAnimationState,
            &mut ImageSequenceAnimation,
            &mut Sprite,
        ),
        With<Player>,
    >,
) {
    for (
        player_state,
        velocity,
        attack_state,
        attack_atlases,
        attack_playback,
        mut state,
        mut animation,
        mut sprite,
    ) in query.iter_mut()
    {
        if let (Some(attack_state), Some(attack_atlases), Some(mut attack_playback)) =
            (attack_state, attack_atlases, attack_playback)
            && attack_state.is_active()
        {
            if state.current != ImageSequenceAnimationType::Attacking
                || attack_playback.trigger_serial != attack_state.trigger_serial
            {
                let (texture, layout, first_frame, frame_count) =
                    attack_atlases.select(attack_state.style);
                sprite.image = texture;
                sprite.texture_atlas = Some(TextureAtlas {
                    layout,
                    index: first_frame,
                });
                attack_playback.restart(
                    attack_state.trigger_serial,
                    first_frame,
                    frame_count,
                    attack_state.remaining,
                );
                animation.timer.pause();
                state.current = ImageSequenceAnimationType::Attacking;
            }
            continue;
        }

        let target = if !player_state.is_grounded {
            ImageSequenceAnimationType::Jumping
        } else if player_state.is_crouching {
            ImageSequenceAnimationType::Crouching
        } else if velocity.x.abs() > 10.0 {
            ImageSequenceAnimationType::Running
        } else {
            ImageSequenceAnimationType::Idle
        };

        if state.current == target {
            continue;
        }

        let (frames, duration, looping) = state.clip(target);
        let frames = frames.to_vec();
        if let Some(first_frame) = frames.first() {
            sprite.image = first_frame.clone();
        }
        sprite.texture_atlas = None;
        animation.replace_clip(frames, duration, looping);
        state.current = target;
    }
}

/// Advances Sakura's current attack row without allowing movement clips to overwrite it.
pub fn advance_sakura_attack_atlas_animations(
    time: Res<Time>,
    mut query: Query<
        (
            &AttackAnimationState,
            &mut SakuraAttackAtlasPlayback,
            &mut Sprite,
        ),
        With<Player>,
    >,
) {
    for (attack_state, mut playback, mut sprite) in query.iter_mut() {
        if !attack_state.is_active() || playback.frame_count <= 1 {
            continue;
        }

        playback.timer.tick(time.delta());
        let completed_intervals = playback.timer.times_finished_this_tick() as usize;
        if completed_intervals == 0 {
            continue;
        }

        playback.current_frame =
            (playback.current_frame + completed_intervals).min(playback.frame_count - 1);
        if let Some(atlas) = sprite.texture_atlas.as_mut() {
            atlas.index = playback.first_frame + playback.current_frame;
        }
    }
}

/// 按时间推进逐张图片动画；卡顿后会补上本帧应跨过的帧数。
pub fn advance_image_sequence_animations(
    time: Res<Time>,
    mut query: Query<(
        &mut ImageSequenceAnimation,
        &ImageSequenceAnimationState,
        &mut Sprite,
    )>,
) {
    for (mut animation, state, mut sprite) in query.iter_mut() {
        if state.current == ImageSequenceAnimationType::Attacking {
            continue;
        }
        if animation.frames.len() <= 1 || animation.timer.is_paused() {
            continue;
        }

        animation.timer.tick(time.delta());
        let completed_intervals = animation.timer.times_finished_this_tick();
        for _ in 0..completed_intervals {
            let is_last_frame = animation.current_frame + 1 >= animation.frames.len();
            if is_last_frame && !animation.looping {
                animation.current_frame = animation.frames.len() - 1;
                animation.timer.pause();
                break;
            }
            animation.current_frame = (animation.current_frame + 1) % animation.frames.len();
        }

        if completed_intervals > 0 {
            sprite.image = animation.frames[animation.current_frame].clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::marker::PhantomData;

    fn image_handle(id: u128) -> Handle<Image> {
        Handle::Uuid(uuid::Uuid::from_u128(id), PhantomData)
    }

    #[test]
    fn state_change_applies_first_frame_immediately() {
        let idle = image_handle(1);
        let running = image_handle(2);
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Update, update_image_sequence_animation_state);
        app.world_mut().spawn((
            Player,
            PlayerState::default(),
            Velocity { x: 20.0, y: 0.0 },
            Sprite {
                image: idle.clone(),
                ..default()
            },
            ImageSequenceAnimation::new(vec![idle.clone()], 0.15, true),
            ImageSequenceAnimationState {
                current: ImageSequenceAnimationType::Idle,
                idle_frames: vec![idle],
                running_frames: vec![running.clone()],
                jumping_frames: vec![image_handle(3)],
                crouching_frames: vec![image_handle(4)],
            },
        ));

        app.update();

        let mut query = app
            .world_mut()
            .query::<(&Sprite, &ImageSequenceAnimationState)>();
        let (sprite, state) = query.single(app.world()).expect("image sequence player");
        assert_eq!(sprite.image, running);
        assert_eq!(state.current, ImageSequenceAnimationType::Running);
    }

    #[test]
    fn short_attacks_can_reach_the_last_atlas_frame() {
        let attack_duration = 0.18;
        let frame_count = 8;
        let transition_duration = attack_frame_duration(attack_duration, frame_count);

        assert!(transition_duration * (frame_count - 1) as f32 <= attack_duration + f32::EPSILON);
    }
}
