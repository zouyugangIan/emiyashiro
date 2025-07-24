use bevy::prelude::*;

/// 图像处理工具
pub struct ImageProcessor;

impl ImageProcessor {
    /// 创建简单的角色精灵
    pub fn create_character_sprite(
        width: u32,
        height: u32,
        character_type: &str,
    ) -> Image {
        let mut data = vec![0u8; (width * height * 4) as usize];
        
        match character_type {
            "shirou" => Self::draw_shirou_sprite(&mut data, width, height),
            "sakura" => Self::draw_sakura_sprite(&mut data, width, height),
            _ => Self::draw_default_sprite(&mut data, width, height),
        }
        
        Image::new(
            bevy::render::render_resource::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            bevy::render::render_resource::TextureDimension::D2,
            data,
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
            bevy::render::render_asset::RenderAssetUsages::RENDER_WORLD,
        )
    }
    
    /// 绘制士郎精灵
    fn draw_shirou_sprite(data: &mut [u8], width: u32, height: u32) {
        for y in 0..height {
            for x in 0..width {
                let index = ((y * width + x) * 4) as usize;
                
                // 创建一个简单的人形轮廓 - 士郎（蓝色主题）
                if Self::is_in_character_shape(x, y, width, height) {
                    // 头部
                    if y < height / 4 {
                        data[index] = 255;     // R - 肉色
                        data[index + 1] = 220; // G
                        data[index + 2] = 177; // B
                        data[index + 3] = 255; // A
                    }
                    // 身体 - 蓝色衣服
                    else if y < 3 * height / 4 {
                        data[index] = 70;      // R - 深蓝
                        data[index + 1] = 130; // G
                        data[index + 2] = 180; // B
                        data[index + 3] = 255; // A
                    }
                    // 腿部 - 深色裤子
                    else {
                        data[index] = 40;      // R
                        data[index + 1] = 40;  // G
                        data[index + 2] = 80;  // B
                        data[index + 3] = 255; // A
                    }
                } else {
                    data[index + 3] = 0; // 透明背景
                }
            }
        }
    }
    
    /// 绘制樱精灵
    fn draw_sakura_sprite(data: &mut [u8], width: u32, height: u32) {
        for y in 0..height {
            for x in 0..width {
                let index = ((y * width + x) * 4) as usize;
                
                // 创建一个简单的人形轮廓 - 樱（粉色主题）
                if Self::is_in_character_shape(x, y, width, height) {
                    // 头部
                    if y < height / 4 {
                        data[index] = 255;     // R - 肉色
                        data[index + 1] = 220; // G
                        data[index + 2] = 177; // B
                        data[index + 3] = 255; // A
                    }
                    // 身体 - 粉色衣服
                    else if y < 3 * height / 4 {
                        data[index] = 255;     // R - 粉色
                        data[index + 1] = 182; // G
                        data[index + 2] = 193; // B
                        data[index + 3] = 255; // A
                    }
                    // 腿部 - 深色裙子
                    else {
                        data[index] = 139;     // R
                        data[index + 1] = 69;  // G
                        data[index + 2] = 19;  // B
                        data[index + 3] = 255; // A
                    }
                } else {
                    data[index + 3] = 0; // 透明背景
                }
            }
        }
    }
    
    /// 绘制默认精灵
    fn draw_default_sprite(data: &mut [u8], width: u32, height: u32) {
        for y in 0..height {
            for x in 0..width {
                let index = ((y * width + x) * 4) as usize;
                
                if Self::is_in_character_shape(x, y, width, height) {
                    data[index] = 128;     // R - 灰色
                    data[index + 1] = 128; // G
                    data[index + 2] = 128; // B
                    data[index + 3] = 255; // A
                } else {
                    data[index + 3] = 0; // 透明背景
                }
            }
        }
    }
    
    /// 判断像素是否在角色形状内
    fn is_in_character_shape(x: u32, y: u32, width: u32, height: u32) -> bool {
        let center_x = width / 2;
        let char_width = width / 3;
        
        // 头部 (圆形)
        if y < height / 4 {
            let head_radius = char_width / 3;
            let dx = (x as i32 - center_x as i32).abs() as u32;
            let dy = (y as i32 - (height / 8) as i32).abs() as u32;
            return dx * dx + dy * dy < head_radius * head_radius;
        }
        
        // 身体 (矩形)
        if y >= height / 4 && y < 3 * height / 4 {
            return x >= center_x - char_width / 2 && x <= center_x + char_width / 2;
        }
        
        // 腿部 (两个矩形)
        if y >= 3 * height / 4 {
            let leg_width = char_width / 3;
            let left_leg = x >= center_x - char_width / 2 && x <= center_x - char_width / 6;
            let right_leg = x >= center_x + char_width / 6 && x <= center_x + char_width / 2;
            return left_leg || right_leg;
        }
        
        false
    }
    
    /// 创建动画帧序列
    pub fn create_animation_frames(
        character_type: &str,
        frame_count: u32,
        width: u32,
        height: u32,
    ) -> Vec<Image> {
        let mut frames = Vec::new();
        
        for i in 0..frame_count {
            let mut image = Self::create_character_sprite(width, height, character_type);
            
            // 为每一帧添加轻微的变化（简单的摆动效果）
            if let Some(data) = image.data.as_mut() {
                let offset = ((i as f32 * std::f32::consts::PI * 2.0 / frame_count as f32).sin() * 2.0) as i32;
                Self::apply_sway_effect(data, width, height, offset);
            }
            
            frames.push(image);
        }
        
        frames
    }
    
    /// 应用摆动效果
    fn apply_sway_effect(data: &mut [u8], width: u32, height: u32, offset: i32) {
        // 这里可以实现简单的像素偏移来创建摆动效果
        // 为了简化，我们只是调整一些像素的颜色来模拟动画
        for y in 0..height {
            for x in 0..width {
                let index = ((y * width + x) * 4) as usize;
                if data[index + 3] > 0 { // 如果不是透明像素
                    // 添加轻微的颜色变化来模拟动画
                    let variation = (offset.abs() * 10) as u8;
                    data[index] = data[index].saturating_add(variation / 3);
                    data[index + 1] = data[index + 1].saturating_add(variation / 4);
                    data[index + 2] = data[index + 2].saturating_add(variation / 5);
                }
            }
        }
    }
}

/// 程序化生成角色精灵系统
pub fn generate_character_sprites(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    println!("🎨 生成程序化角色精灵...");
    
    // 生成士郎精灵
    let shirou_sprite = ImageProcessor::create_character_sprite(64, 96, "shirou");
    let shirou_handle = images.add(shirou_sprite);
    
    // 生成樱精灵
    let sakura_sprite = ImageProcessor::create_character_sprite(64, 96, "sakura");
    let sakura_handle = images.add(sakura_sprite);
    
    // 生成动画帧
    let shirou_frames = ImageProcessor::create_animation_frames("shirou", 4, 64, 96);
    let sakura_frames = ImageProcessor::create_animation_frames("sakura", 4, 64, 96);
    
    println!("✅ 程序化精灵生成完成");
    println!("   - 士郎精灵: 64x96 像素");
    println!("   - 樱精灵: 64x96 像素");
    println!("   - 动画帧: 每个角色 4 帧");
}