//! 角色動畫系統測試
//!
//! 測試角色動畫幀的配置和訪問接口

#[cfg(test)]
mod tests {
    use crate::asset_paths;

    #[test]
    fn test_shirou_idle_frames_not_empty() {
        // 驗證 Shirou 待機動畫幀不為空
        assert!(!asset_paths::SHIROU_IDLE_FRAMES.is_empty(), "Shirou 待機動畫幀不應為空");
        println!("✅ Shirou 待機動畫幀數: {}", asset_paths::SHIROU_IDLE_FRAMES.len());
    }

    #[test]
    fn test_shirou_running_frames_not_empty() {
        // 驗證 Shirou 跑步動畫幀不為空
        assert!(!asset_paths::SHIROU_RUNNING_FRAMES.is_empty(), "Shirou 跑步動畫幀不應為空");
        println!("✅ Shirou 跑步動畫幀數: {}", asset_paths::SHIROU_RUNNING_FRAMES.len());
    }

    #[test]
    fn test_shirou_jumping_frames_not_empty() {
        // 驗證 Shirou 跳躍動畫幀不為空
        assert!(!asset_paths::SHIROU_JUMPING_FRAMES.is_empty(), "Shirou 跳躍動畫幀不應為空");
        println!("✅ Shirou 跳躍動畫幀數: {}", asset_paths::SHIROU_JUMPING_FRAMES.len());
    }

    #[test]
    fn test_sakura_idle_frames_not_empty() {
        // 驗證 Sakura 待機動畫幀不為空
        assert!(!asset_paths::SAKURA_IDLE_FRAMES.is_empty(), "Sakura 待機動畫幀不應為空");
        println!("✅ Sakura 待機動畫幀數: {}", asset_paths::SAKURA_IDLE_FRAMES.len());
    }

    #[test]
    fn test_sakura_running_frames_not_empty() {
        // 驗證 Sakura 跑步動畫幀不為空
        assert!(!asset_paths::SAKURA_RUNNING_FRAMES.is_empty(), "Sakura 跑步動畫幀不應為空");
        println!("✅ Sakura 跑步動畫幀數: {}", asset_paths::SAKURA_RUNNING_FRAMES.len());
    }

    #[test]
    fn test_animation_frame_paths_format() {
        // 驗證所有 Shirou 動畫幀路徑格式正確
        for path in asset_paths::SHIROU_IDLE_FRAMES {
            assert!(path.starts_with("images/characters/shirou_"), 
                "Shirou 動畫幀路徑應以 'images/characters/shirou_' 開頭: {}", path);
        }

        // 驗證所有 Sakura 動畫幀路徑格式正確
        for path in asset_paths::SAKURA_IDLE_FRAMES {
            assert!(path.starts_with("images/characters/sakura_"), 
                "Sakura 動畫幀路徑應以 'images/characters/sakura_' 開頭: {}", path);
        }
        
        println!("✅ 所有動畫幀路徑格式正確");
    }

    #[test]
    fn test_shirou_animation_has_ping_pong_effect() {
        // 驗證 Shirou 待機動畫使用了乒乓效果（首尾相同）
        let frames = asset_paths::SHIROU_IDLE_FRAMES;
        assert!(frames.len() >= 3, "乒乓動畫至少需要 3 幀");
        assert_eq!(frames[0], frames[frames.len() - 1], 
            "乒乓動畫的首尾幀應該相同");
        println!("✅ Shirou 待機動畫使用乒乓循環效果");
    }

    #[test]
    fn test_sakura_animation_has_ping_pong_effect() {
        // 驗證 Sakura 待機動畫使用了乒乓效果
        let frames = asset_paths::SAKURA_IDLE_FRAMES;
        assert!(frames.len() >= 3, "乒乓動畫至少需要 3 幀");
        assert_eq!(frames[0], frames[frames.len() - 1], 
            "乒乓動畫的首尾幀應該相同");
        println!("✅ Sakura 待機動畫使用乒乓循環效果");
    }

    #[test]
    fn test_backward_compatibility() {
        // 驗證向後兼容性：舊的動畫幀數組仍然可用
        assert!(!asset_paths::SHIROU_ANIMATION_FRAMES.is_empty());
        assert!(!asset_paths::SAKURA_ANIMATION_FRAMES.is_empty());
        println!("✅ 向後兼容的動畫幀數組可用");
    }

    #[test]
    fn test_all_animation_types_have_frames() {
        // 驗證所有動畫類型都有對應的幀
        let shirou_animations = vec![
            ("待機", asset_paths::SHIROU_IDLE_FRAMES),
            ("跑步", asset_paths::SHIROU_RUNNING_FRAMES),
            ("跳躍", asset_paths::SHIROU_JUMPING_FRAMES),
            ("蹲下", asset_paths::SHIROU_CROUCHING_FRAMES),
        ];

        for (name, frames) in shirou_animations {
            assert!(!frames.is_empty(), "Shirou {} 動畫幀不應為空", name);
            println!("✅ Shirou {} 動畫: {} 幀", name, frames.len());
        }

        let sakura_animations = vec![
            ("待機", asset_paths::SAKURA_IDLE_FRAMES),
            ("跑步", asset_paths::SAKURA_RUNNING_FRAMES),
            ("跳躍", asset_paths::SAKURA_JUMPING_FRAMES),
            ("蹲下", asset_paths::SAKURA_CROUCHING_FRAMES),
        ];

        for (name, frames) in sakura_animations {
            assert!(!frames.is_empty(), "Sakura {} 動畫幀不應為空", name);
            println!("✅ Sakura {} 動畫: {} 幀", name, frames.len());
        }
    }
}
