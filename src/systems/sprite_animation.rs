use crate::{
    components::{
        animation_data::{AnimationDataMap, CharacterAnimationData},
        *,
    },
    resources::GameAssets,
};
use bevy::prelude::*;
use std::{fs, collections::HashMap};

/// 精灵动画组件
#[derive(Component, Debug)]
pub struct SpriteAnimation {
    pub current_animation: AnimationType,
    pub frame_timer: Timer,
    pub current_frame: usize,
    /// Holds the animation data cloned from the central resource.
    pub animations: HashMap<AnimationType, AnimationClipData>,
}

impl Default for SpriteAnimation {
    fn default() -> Self {
        Self {
            current_animation: AnimationType::Idle,
            frame_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            current_frame: 0,
            animations: HashMap::new(),
        }
    }
}

/// 在启动时加载所有动画配置文件
pub fn load_animation_data() -> AnimationDataMap {
    let mut animation_map = AnimationDataMap::default();
    
    // Check if directory exists before reading
    if let Ok(paths) = fs::read_dir("assets/animations") {
        for path in paths {
            let path = path.expect("Failed to read path").path();
            if path.extension().and_then(|s| s.to_str()) == Some("ron") {
                let character_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap()
                    .to_string();
                let ron_string = fs::read_to_string(&path).expect("Failed to read animation file");
                let anim_data: CharacterAnimationData = ron::from_str(&ron_string)
                    .expect(&format!("Failed to parse RON for {}", character_name));
                
                animation_map.0.insert(character_name, anim_data);
            }
        }
        
        println!("📂 Loaded {} character animation profiles.", animation_map.0.len());
    } else {
        println!("⚠️ Warning: assets/animations directory not found, using empty animation map");
    }
    
    animation_map
}


/// 创建角色动画组件
pub fn create_character_animation(
    anim_data_map: &Res<AnimationDataMap>,
    character_name: &str,
) -> SpriteAnimation {
    let character_data = anim_data_map
        .0
        .get(character_name)
        .expect(&format!("Animation data for '{}' not found", character_name));

    let initial_clip = character_data
        .animations
        .get(&AnimationType::Idle)
        .expect("Idle animation not found for character");

    SpriteAnimation {
        current_animation: AnimationType::Idle,
        frame_timer: Timer::from_seconds(initial_clip.frame_duration, TimerMode::Repeating),
        current_frame: 0,
        animations: character_data.animations.clone(),
    }
}

/// 更新精灵动画系统
pub fn update_sprite_animations(
    time: Res<Time>,
    mut query: Query<(&mut SpriteAnimation, &mut Sprite)>,
) {
    for (mut animation, mut sprite) in query.iter_mut() {
        // Prepare to allow disjoint borrows if needed, though with the split logic below it's less critical.
        let animation = &mut *animation; 
        
        animation.frame_timer.tick(time.delta());

        if animation.frame_timer.just_finished() {
            // 1. Calculate next state (Read Phase)
            // Clone current_animation key to avoid holding borrow on animation field if needed
            let current_anim_key = animation.current_animation.clone();
            
            let update_data = if let Some(clip) = animation.animations.get(&current_anim_key) {
                let frame_count = clip.frames.len();
                if frame_count == 0 { None } 
                else {
                    // Advance frame
                    let next_frame = if clip.looping {
                        (animation.current_frame + 1) % frame_count
                    } else {
                        (animation.current_frame + 1).min(frame_count - 1)
                    };
                    
                    // Get atlas index
                    let atlas_index = clip.frames.get(next_frame).cloned();
                    
                    Some((next_frame, atlas_index))
                }
            } else {
                None
            };

            // 2. Apply updates (Write Phase)
            if let Some((next_frame, atlas_idx_opt)) = update_data {
                animation.current_frame = next_frame;
                
                if let Some(atlas_idx) = atlas_idx_opt {
                     if let Some(ref mut atlas) = sprite.texture_atlas {
                        atlas.index = atlas_idx;
                    }
                }
            }
        }
    }
}

/// 根据玩家状态更新动画
pub fn update_character_animation_state(
    mut query: Query<(&mut SpriteAnimation, &mut Sprite, &PlayerState, &Velocity), With<Player>>,
    game_assets: Option<Res<GameAssets>>,
) {
    for (mut animation, mut sprite, player_state, velocity) in query.iter_mut() {
        let new_animation = if !player_state.is_grounded {
            AnimationType::Jumping
        } else if player_state.is_crouching {
            AnimationType::Crouching
        } else if velocity.x.abs() > 10.0 {
            AnimationType::Running
        } else {
            AnimationType::Idle
        };

        // 只有当动画类型改变时才切换
        if animation.current_animation != new_animation {
            // Get the frame duration first to avoid borrow issues
            let frame_duration = animation.animations.get(&new_animation)
                .map(|clip| clip.frame_duration);
            
            if let Some(duration) = frame_duration {
                animation.current_animation = new_animation.clone();
                animation.current_frame = 0;
                animation
                    .frame_timer
                    .set_duration(std::time::Duration::from_secs_f32(duration));
                animation.frame_timer.reset();
                println!("🎭 切换动画: {:?}", new_animation);
                
                // === Dynamic Layout Switching for Mixed-Grid Sprite Sheets ===
                if let Some(assets) = &game_assets {
                    if let Some(ref mut atlas) = sprite.texture_atlas {
                        // Check if we need the 5-column layout (Run) or 4-column (Rest)
                        if new_animation == AnimationType::Running {
                            if let Some(run_layout) = &assets.shirou_atlas_run {
                                atlas.layout = run_layout.clone();
                            }
                        } else {
                            if let Some(std_layout) = &assets.shirou_atlas {
                                atlas.layout = std_layout.clone();
                            }
                        }
                    }
                }
            }
        }
    }
}
