use bevy::prelude::*;

/// 精灵表生成工具
pub struct SpritesheetGenerator;

impl SpritesheetGenerator {
    /// 从现有图片生成精灵表
    pub fn create_from_images(
        _images: Vec<Handle<Image>>,
        _frame_size: UVec2,
        _columns: u32,
    ) -> Result<Handle<Image>, String> {
        // 这里应该实现从多个图片合成精灵表的逻辑
        // 由于需要访问图片数据，这在实际项目中需要异步处理
        Err("需要实现图片合成逻辑".to_string())
    }

    /// 生成角色动画配置
    pub fn generate_character_config(character_name: &str) -> String {
        format!(
            r#"
# {character_name} 角色动画配置

## 精灵表布局
- 尺寸: 512x256 (8列 x 4行)
- 每帧: 64x64 像素
- 总帧数: 32帧

## 动画定义

### 待机动画 (Idle)
- 帧范围: 0-3
- 帧率: 5 FPS (每帧0.2秒)
- 循环: 是

### 跑步动画 (Running)
- 帧范围: 8-13
- 帧率: 10 FPS (每帧0.1秒)
- 循环: 是

### 跳跃动画 (Jumping)
- 帧范围: 16-18
- 帧率: 6.67 FPS (每帧0.15秒)
- 循环: 否

### 蹲下动画 (Crouching)
- 帧范围: 24-25
- 帧率: 5 FPS (每帧0.2秒)
- 循环: 是

## 制作建议

1. **使用 Aseprite**:
   - 创建 512x256 画布
   - 设置网格为 64x64
   - 每个网格绘制一帧动画

2. **动画原则**:
   - 待机: 轻微的呼吸动作
   - 跑步: 腿部交替，手臂摆动
   - 跳跃: 起跳-空中-着地
   - 蹲下: 身体压低，保持平衡

3. **导出设置**:
   - 格式: PNG
   - 透明背景
   - 无压缩
"#,
            character_name = character_name
        )
    }

    /// 生成精灵表模板
    pub fn generate_template_guide() -> String {
        r#"
# 精灵表制作指南

## 工具推荐

### 免费工具
1. **GIMP** - 功能强大的免费图像编辑器
2. **Krita** - 专业的免费绘画软件
3. **Piskel** - 在线像素艺术工具
4. **GraphicsGale** - 像素动画专用工具

### 付费工具
1. **Aseprite** - 最佳的像素艺术和动画工具
2. **Photoshop** - 专业图像处理
3. **Clip Studio Paint** - 动画制作

## 制作流程

### 1. 规划阶段
- 确定角色设计
- 规划动画动作
- 设计精灵表布局

### 2. 绘制阶段
- 创建基础角色设计
- 绘制关键帧
- 添加中间帧
- 调整动画流畅度

### 3. 导出阶段
- 统一帧尺寸
- 保持透明背景
- 导出为PNG格式

## 动画技巧

### 待机动画
- 添加轻微的上下浮动
- 眨眼动作
- 衣服或头发的轻微摆动

### 跑步动画
- 腿部交替运动
- 手臂自然摆动
- 身体重心变化
- 至少6-8帧

### 跳跃动画
- 起跳准备姿势
- 空中姿态
- 着地缓冲
- 3-4帧足够

### 蹲下动画
- 身体下压
- 重心降低
- 可以是静态或轻微摆动

## 文件命名规范
- character_name_spritesheet.png
- 例如: shirou_spritesheet.png, sakura_spritesheet.png
"#
        .to_string()
    }
}

/// 精灵表验证工具
pub struct SpritesheetValidator;

impl SpritesheetValidator {
    /// 验证精灵表格式
    pub fn validate_spritesheet(
        image_size: UVec2,
        frame_size: UVec2,
        expected_frames: u32,
    ) -> Result<(), String> {
        let columns = image_size.x / frame_size.x;
        let rows = image_size.y / frame_size.y;
        let total_frames = columns * rows;

        if total_frames < expected_frames {
            return Err(format!(
                "精灵表帧数不足: 需要{}帧，实际{}帧",
                expected_frames, total_frames
            ));
        }

        if !image_size.x.is_multiple_of(frame_size.x) || !image_size.y.is_multiple_of(frame_size.y)
        {
            return Err("精灵表尺寸不能被帧尺寸整除".to_string());
        }

        Ok(())
    }

    /// 生成精灵表信息
    pub fn analyze_spritesheet(image_size: UVec2, frame_size: UVec2) -> String {
        let columns = image_size.x / frame_size.x;
        let rows = image_size.y / frame_size.y;
        let total_frames = columns * rows;

        format!(
            "精灵表分析:\n\
            - 总尺寸: {}x{}\n\
            - 帧尺寸: {}x{}\n\
            - 布局: {}列 x {}行\n\
            - 总帧数: {}帧",
            image_size.x, image_size.y, frame_size.x, frame_size.y, columns, rows, total_frames
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spritesheet_validation() {
        // 测试有效的精灵表
        assert!(
            SpritesheetValidator::validate_spritesheet(
                UVec2::new(512, 256),
                UVec2::new(64, 64),
                32
            )
            .is_ok()
        );

        // 测试无效的精灵表
        assert!(
            SpritesheetValidator::validate_spritesheet(
                UVec2::new(500, 250),
                UVec2::new(64, 64),
                32
            )
            .is_err()
        );
    }

    #[test]
    fn test_character_config_generation() {
        let config = SpritesheetGenerator::generate_character_config("TestCharacter");
        assert!(config.contains("TestCharacter"));
        assert!(config.contains("待机动画"));
        assert!(config.contains("跑步动画"));
    }
}
