use bevy::prelude::*;
use crate::{
    components::*,
    resources::*,
    states::CharacterType,
};

/// 精灵动画组件
#[derive(Component, Debug)]
pub struct SpriteAnimation {
    pub current_animation: AnimationType,
    pub frame_timer: Timer,
    pub current_frame: usize,
    pub animations: std::collections::HashMap<AnimationType, AnimationClip>,
}

/// 动画片段定义
#[derive(Debug, Clone)]
pub struct AnimationClip {
    pub frames: Vec<usize>,           // 帧索引
    pub frame_duration: f32,          // 每帧持续时间
    pub looping: bool,                // 是否循环
    pub texture_atlas_handle: Handle<TextureAtlasLayout>, // 纹理图集句柄
}

impl Default for SpriteAnimation {
    fn default() -> Self {
        Self {
            current_animation: AnimationType::Idle,
            frame_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            current_frame: 0,
            animations: std::collections::HashMap::new(),
        }
    }
}

/// 创建角色动画系统
pub fn setup_character_animations(
    _commands: Commands,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
    mut game_assets: ResMut<GameAssets>,
) {
    // 创建士郎的精灵表动画
    let shirou_texture = asset_server.load("images/characters/shirou_spritesheet.png");
    let shirou_atlas_layout = TextureAtlasLayout::from_grid(
        UVec2::new(64, 64), // 每帧大小
        8, 4,               // 8列4行
        None, None
    );
    let shirou_atlas_handle = texture_atlases.add(shirou_atlas_layout);
    
    // 创建樱的精灵表动画
    let sakura_texture = asset_server.load("images/characters/sakura_spritesheet.png");
    let sakura_atlas_layout = TextureAtlasLayout::from_grid(
        UVec2::new(64, 64), // 每帧大小
        8, 4,               // 8列4行
        None, None
    );
    let sakura_atlas_handle = texture_atlases.add(sakura_atlas_layout);
    
    // 存储到游戏资源中
    game_assets.shirou_atlas = Some(shirou_atlas_handle.clone());
    game_assets.sakura_atlas = Some(sakura_atlas_handle.clone());
    game_assets.shirou_spritesheet = Some(shirou_texture);
    game_assets.sakura_spritesheet = Some(sakura_texture);
    
    println!("🎭 角色动画系统初始化完成");
}

/// 创建角色动画组件
pub fn create_character_animation(character_type: &CharacterType) -> SpriteAnimation {
    let mut animations = std::collections::HashMap::new();
    
    // 定义不同的动画片段
    match character_type {
        CharacterType::Shirou1 => {
            // 士郎的动画定义
            animations.insert(AnimationType::Idle, AnimationClip {
                frames: vec![0, 1, 2, 3],
                frame_duration: 0.2,
                looping: true,
                texture_atlas_handle: Handle::default(), // 稍后设置
            });
            
            animations.insert(AnimationType::Running, AnimationClip {
                frames: vec![8, 9, 10, 11, 12, 13],
                frame_duration: 0.1,
                looping: true,
                texture_atlas_handle: Handle::default(),
            });
            
            animations.insert(AnimationType::Jumping, AnimationClip {
                frames: vec![16, 17, 18],
                frame_duration: 0.15,
                looping: false,
                texture_atlas_handle: Handle::default(),
            });
            
            animations.insert(AnimationType::Crouching, AnimationClip {
                frames: vec![24, 25],
                frame_duration: 0.2,
                looping: true,
                texture_atlas_handle: Handle::default(),
            });
        }
        CharacterType::Shirou2 => {
            // 樱的动画定义（使用不同的帧）
            animations.insert(AnimationType::Idle, AnimationClip {
                frames: vec![4, 5, 6, 7],
                frame_duration: 0.25,
                looping: true,
                texture_atlas_handle: Handle::default(),
            });
            
            animations.insert(AnimationType::Running, AnimationClip {
                frames: vec![14, 15, 16, 17, 18, 19],
                frame_duration: 0.12,
                looping: true,
                texture_atlas_handle: Handle::default(),
            });
            
            animations.insert(AnimationType::Jumping, AnimationClip {
                frames: vec![20, 21, 22],
                frame_duration: 0.18,
                looping: false,
                texture_atlas_handle: Handle::default(),
            });
            
            animations.insert(AnimationType::Crouching, AnimationClip {
                frames: vec![26, 27],
                frame_duration: 0.25,
                looping: true,
                texture_atlas_handle: Handle::default(),
            });
        }
    }
    
    SpriteAnimation {
        current_animation: AnimationType::Idle,
        frame_timer: Timer::from_seconds(0.2, TimerMode::Repeating),
        current_frame: 0,
        animations,
    }
}

/// 更新精灵动画系统
pub fn update_sprite_animations(
    time: Res<Time>,
    mut query: Query<(&mut SpriteAnimation, &mut Sprite)>,
) {
    for (mut animation, mut _sprite) in query.iter_mut() {
        animation.frame_timer.tick(time.delta());
        
        if animation.frame_timer.just_finished() {
            if let Some(clip) = animation.animations.get(&animation.current_animation) {
                if clip.looping {
                    animation.current_frame = (animation.current_frame + 1) % clip.frames.len();
                } else {
                    animation.current_frame = (animation.current_frame + 1).min(clip.frames.len() - 1);
                }
                
                // 更新纹理图集索引（暂时注释掉，需要实际的纹理图集）
                // if let Some(frame_index) = clip.frames.get(animation.current_frame) {
                //     atlas.index = *frame_index;
                // }
            }
        }
    }
}

/// 根据玩家状态更新动画
pub fn update_character_animation_state(
    mut query: Query<(&mut SpriteAnimation, &PlayerState, &Velocity), With<Player>>,
) {
    for (mut animation, player_state, velocity) in query.iter_mut() {
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
            // 先获取动画片段信息
            let frame_duration = if let Some(clip) = animation.animations.get(&new_animation) {
                clip.frame_duration
            } else {
                0.2 // 默认帧持续时间
            };
            
            // 更新动画状态
            animation.current_animation = new_animation.clone();
            animation.current_frame = 0;
            animation.frame_timer.set_duration(std::time::Duration::from_secs_f32(frame_duration));
            animation.frame_timer.reset();
            
            println!("🎭 切换动画: {:?}", new_animation);
        }
    }
}

/// 程序化生成简单的精灵表
pub fn generate_simple_spritesheet(
    _commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    // 生成一个简单的8x4精灵表
    let width = 512; // 8 * 64
    let height = 256; // 4 * 64
    let mut data = vec![0u8; (width * height * 4) as usize];
    
    // 为每个64x64的帧生成不同的颜色和形状
    for row in 0..4 {
        for col in 0..8 {
            let frame_index = row * 8 + col;
            let base_x = col * 64;
            let base_y = row * 64;
            
            // 根据帧索引生成不同的角色姿态
            generate_character_frame(&mut data, base_x, base_y, frame_index, width);
        }
    }
    
    let image = Image::new(
        bevy::render::render_resource::Extent3d {
            width: width as u32,
            height: height as u32,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        data,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
    );
    
    let _texture_handle = images.add(image);
    
    // 创建纹理图集布局
    let atlas_layout = TextureAtlasLayout::from_grid(
        UVec2::new(64, 64),
        8, 4,
        None, None
    );
    let _atlas_handle = texture_atlases.add(atlas_layout);
    
    println!("🎨 程序化精灵表生成完成: 8x4 = 32帧");
}

/// 生成单个角色帧
fn generate_character_frame(data: &mut [u8], base_x: usize, base_y: usize, frame_index: usize, width: usize) {
    let frame_width = 64;
    let frame_height = 64;
    
    // 根据帧索引决定角色姿态和颜色
    let (body_color, pose_offset) = match frame_index % 8 {
        0..=3 => ([255, 200, 150, 255], 0),      // 待机动画 - 肉色
        4..=7 => ([200, 150, 255, 255], 0),      // 待机动画 - 紫色（樱）
        8..=13 => ([100, 150, 255, 255], frame_index % 4), // 跑步动画 - 蓝色
        14..=19 => ([255, 150, 200, 255], frame_index % 4), // 跑步动画 - 粉色
        20..=22 => ([255, 255, 100, 255], 2),    // 跳跃动画 - 黄色
        23..=25 => ([100, 255, 100, 255], 2),    // 跳跃动画 - 绿色
        26..=27 => ([255, 100, 100, 255], 0),    // 蹲下动画 - 红色
        _ => ([150, 150, 150, 255], 0),          // 默认 - 灰色
    };
    
    // 绘制简单的人形
    for y in 0..frame_height {
        for x in 0..frame_width {
            let pixel_x = base_x + x;
            let pixel_y = base_y + y + pose_offset.max(0) as usize;
            
            if pixel_x >= width || pixel_y * width + pixel_x >= data.len() / 4 {
                continue;
            }
            
            let index = ((pixel_y * width + pixel_x) * 4) as usize;
            
            // 绘制简单的人形轮廓
            let should_draw = match y {
                0..=15 if x >= 24 && x <= 40 => true,  // 头部
                16..=40 if x >= 20 && x <= 44 => true, // 身体
                41..=63 if (x >= 20 && x <= 28) || (x >= 36 && x <= 44) => true, // 腿部
                _ => false,
            };
            
            if should_draw && index + 3 < data.len() {
                data[index] = body_color[0];     // R
                data[index + 1] = body_color[1]; // G
                data[index + 2] = body_color[2]; // B
                data[index + 3] = body_color[3]; // A
            }
        }
    }
}