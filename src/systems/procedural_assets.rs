use bevy::prelude::*;

/// 程序化生成游戏素材
pub fn generate_simple_assets(_commands: Commands, mut images: ResMut<Assets<Image>>) {
    // 生成简单的角色纹理
    let character_image = create_character_texture();
    let _character_handle = images.add(character_image);

    // 生成地面纹理
    let ground_image = create_ground_texture();
    let _ground_handle = images.add(ground_image);

    // 生成背景纹理
    let background_image = create_background_texture();
    let _background_handle = images.add(background_image);

    println!("🎨 程序化素材生成完成！");
}

/// 创建简单的角色纹理
fn create_character_texture() -> Image {
    let width = 32;
    let height = 48;
    let mut data = vec![0u8; (width * height * 4) as usize];

    // 创建一个简单的人形轮廓
    for y in 0..height {
        for x in 0..width {
            let index = ((y * width + x) * 4) as usize;

            // 头部 (上1/4)
            if y < height / 4 && x >= width / 4 && x < 3 * width / 4 {
                data[index] = 255; // R - 肉色
                data[index + 1] = 220; // G
                data[index + 2] = 177; // B
                data[index + 3] = 255; // A
            }
            // 身体 (中间1/2)
            else if y >= height / 4 && y < 3 * height / 4 && x >= width / 3 && x < 2 * width / 3 {
                data[index] = 100; // R - 蓝色衣服
                data[index + 1] = 150; // G
                data[index + 2] = 255; // B
                data[index + 3] = 255; // A
            }
            // 腿部 (下1/4)
            else if y >= 3 * height / 4
                && ((x >= width / 3 && x < width / 2) || (x >= width / 2 && x < 2 * width / 3))
            {
                data[index] = 50; // R - 深蓝色裤子
                data[index + 1] = 50; // G
                data[index + 2] = 150; // B
                data[index + 3] = 255; // A
            }
            // 透明背景
            else {
                data[index + 3] = 0; // 透明
            }
        }
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
        Default::default(),
    )
}

/// 创建地面纹理
fn create_ground_texture() -> Image {
    let width = 64;
    let height = 32;
    let mut data = vec![0u8; (width * height * 4) as usize];

    for y in 0..height {
        for x in 0..width {
            let index = ((y * width + x) * 4) as usize;

            // 创建草地效果
            let grass_green = if (x + y) % 4 == 0 { 100 } else { 80 };
            data[index] = 34; // R - 深绿
            data[index + 1] = grass_green; // G - 变化的绿色
            data[index + 2] = 34; // B
            data[index + 3] = 255; // A
        }
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
        Default::default(),
    )
}

/// 创建背景纹理
fn create_background_texture() -> Image {
    let width = 256;
    let height = 192;
    let mut data = vec![0u8; (width * height * 4) as usize];

    for y in 0..height {
        for x in 0..width {
            let index = ((y * width + x) * 4) as usize;

            // 创建天空渐变效果
            let sky_blue = 135 + ((height - y) * 120 / height) as u8;
            data[index] = 135; // R - 天蓝
            data[index + 1] = 206; // G
            data[index + 2] = sky_blue; // B - 渐变蓝色
            data[index + 3] = 255; // A
        }
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
        Default::default(),
    )
}
