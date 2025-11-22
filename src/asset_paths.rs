// FONT ASSETS
pub const FONT_FIRA_SANS: &str = "fonts/FiraSans-Bold.ttf";

// IMAGE ASSETS - UI COVERS (主界面封面图片)
pub const IMAGE_UI_COVER01: &str = "images/ui/cove01.png";
pub const IMAGE_UI_COVER02: &str = "images/ui/cover02.png";
pub const IMAGE_UI_COVER03: &str = "images/ui/cover03.png";
pub const IMAGE_UI_COVER04: &str = "images/ui/cover04.png";
pub const IMAGE_UI_COVER05: &str = "images/ui/cover05.png";
pub const IMAGE_UI_COVER06: &str = "images/ui/cover06.png";
pub const IMAGE_UI_COVER07: &str = "images/ui/cover07.png";
pub const IMAGE_UI_COVER08: &str = "images/ui/cover08.png";
pub const IMAGE_UI_COVER09: &str = "images/ui/cover09.png";
pub const IMAGE_UI_COVER10: &str = "images/ui/cover10.jpg";
pub const IMAGE_UI_COVER11: &str = "images/ui/cover11.jpg";
pub const IMAGE_UI_COVER12: &str = "images/ui/cover12.jpg";
pub const IMAGE_UI_COVER13: &str = "images/ui/cover13.png";
pub const IMAGE_UI_COVER14: &str = "images/ui/cover14.png";
pub const IMAGE_UI_COVER15: &str = "images/ui/cover15.png";
pub const IMAGE_UI_COVER16: &str = "images/ui/cover16.png";
pub const IMAGE_UI_COVER17: &str = "images/ui/cover17.png";
pub const IMAGE_UI_COVER18: &str = "images/ui/cover18.jpg";
pub const IMAGE_UI_COVER00: &str = "images/ui/cover00.jpeg";

// 所有封面图片的数组，用于轮流显示
pub const UI_COVER_IMAGES: &[&str] = &[
    IMAGE_UI_COVER00, IMAGE_UI_COVER01, IMAGE_UI_COVER02, IMAGE_UI_COVER03,
    IMAGE_UI_COVER04, IMAGE_UI_COVER05, IMAGE_UI_COVER06, IMAGE_UI_COVER07,
    IMAGE_UI_COVER08, IMAGE_UI_COVER09, IMAGE_UI_COVER10, IMAGE_UI_COVER11,
    IMAGE_UI_COVER12, IMAGE_UI_COVER13, IMAGE_UI_COVER14, IMAGE_UI_COVER15,
    IMAGE_UI_COVER16, IMAGE_UI_COVER17, IMAGE_UI_COVER18,
];

// CHARACTER ASSETS - SHIROU (1P角色精灵动画)
pub const IMAGE_CHAR_SHIROU_IDLE1: &str = "images/characters/shirou_idle1.jpg";
pub const IMAGE_CHAR_SHIROU_IDLE2: &str = "images/characters/shirou_idle2.jpg";
pub const IMAGE_CHAR_SHIROU_IDLE3: &str = "images/characters/shirou_idle3.jpg";
pub const IMAGE_CHAR_SHIROU_IDLE4: &str = "images/characters/shirou_idle4.png";
pub const IMAGE_CHAR_SHIROU_IDLE5: &str = "images/characters/shirou_idle5.png";
pub const IMAGE_CHAR_SHIROU_IDLE6: &str = "images/characters/shirou_idle6.png";
pub const IMAGE_CHAR_SHIROU_IDLE7: &str = "images/characters/shirou_idle7.png";
pub const IMAGE_CHAR_SHIROU_IDLE8: &str = "images/characters/shirou_idle8.png";
pub const IMAGE_CHAR_SHIROU_IDLE10: &str = "images/characters/shirou_idle10.png";
pub const IMAGE_CHAR_SHIROU_IDLE11: &str = "images/characters/shirou_idle11.png";
pub const IMAGE_CHAR_SHIROU_IDLE12: &str = "images/characters/shirou_idle12.png";
pub const IMAGE_CHAR_SHIROU_IDLE13: &str = "images/characters/shirou_idle13.png";
pub const IMAGE_CHAR_SHIROU_IDLE14: &str = "images/characters/shirou_idle14.png";

// Shirou 待機動畫幀（使用乒乓循環製造流暢效果）
pub const SHIROU_IDLE_FRAMES: &[&str] = &[
    IMAGE_CHAR_SHIROU_IDLE1,
    IMAGE_CHAR_SHIROU_IDLE2,
    IMAGE_CHAR_SHIROU_IDLE3,
    IMAGE_CHAR_SHIROU_IDLE2,  // 反向回到中間幀
    IMAGE_CHAR_SHIROU_IDLE1,  // 形成循環
];

// Shirou 跑步動畫幀（使用重複幀延長動作）
pub const SHIROU_RUNNING_FRAMES: &[&str] = &[
    IMAGE_CHAR_SHIROU_IDLE4,
    IMAGE_CHAR_SHIROU_IDLE5,
    IMAGE_CHAR_SHIROU_IDLE6,
    IMAGE_CHAR_SHIROU_IDLE7,
    IMAGE_CHAR_SHIROU_IDLE6,  // 重複關鍵幀
    IMAGE_CHAR_SHIROU_IDLE5,  // 反向循環
];

// Shirou 跳躍動畫幀
pub const SHIROU_JUMPING_FRAMES: &[&str] = &[
    IMAGE_CHAR_SHIROU_IDLE8,
    IMAGE_CHAR_SHIROU_IDLE10,
    IMAGE_CHAR_SHIROU_IDLE11,
];

// Shirou 蹲下動畫幀
pub const SHIROU_CROUCHING_FRAMES: &[&str] = &[
    IMAGE_CHAR_SHIROU_IDLE12,
    IMAGE_CHAR_SHIROU_IDLE13,
    IMAGE_CHAR_SHIROU_IDLE14,
];

// Shirou所有动画帧的数组（向後兼容）
pub const SHIROU_ANIMATION_FRAMES: &[&str] = &[
    IMAGE_CHAR_SHIROU_IDLE1, IMAGE_CHAR_SHIROU_IDLE2, IMAGE_CHAR_SHIROU_IDLE3,
    IMAGE_CHAR_SHIROU_IDLE4, IMAGE_CHAR_SHIROU_IDLE5, IMAGE_CHAR_SHIROU_IDLE6,
    IMAGE_CHAR_SHIROU_IDLE7, IMAGE_CHAR_SHIROU_IDLE8, IMAGE_CHAR_SHIROU_IDLE10,
    IMAGE_CHAR_SHIROU_IDLE11, IMAGE_CHAR_SHIROU_IDLE12, IMAGE_CHAR_SHIROU_IDLE13,
    IMAGE_CHAR_SHIROU_IDLE14,
];

// CHARACTER ASSETS - SAKURA (2P角色精灵动画)
pub const IMAGE_CHAR_SAKURA_IDLE1: &str = "images/characters/sakura_idle1.jpg";
pub const IMAGE_CHAR_SAKURA_IDLE01: &str = "images/characters/sakura_idle01.png";
pub const IMAGE_CHAR_SAKURA_IDLE02: &str = "images/characters/sakura_idle02.png";
pub const IMAGE_CHAR_SAKURA_IDLE03: &str = "images/characters/sakura_idle03.png";
pub const IMAGE_CHAR_SAKURA_IDLE04: &str = "images/characters/sakura_idle04.png";
pub const IMAGE_CHAR_SAKURA_IDLE05: &str = "images/characters/sakura_idle05.png";
pub const IMAGE_CHAR_SAKURA_IDLE06: &str = "images/characters/sakura_idle06.png";
pub const IMAGE_CHAR_SAKURA_IDLE07: &str = "images/characters/sakura_idle07.png";
pub const IMAGE_CHAR_SAKURA_IDLE08: &str = "images/characters/sakura_idle08.png";
pub const IMAGE_CHAR_SAKURA_IDLE09: &str = "images/characters/sakura_idle09.png";
pub const IMAGE_CHAR_SAKURA_IDLE10: &str = "images/characters/sakura_idle10.png";
pub const IMAGE_CHAR_SAKURA_IDLE11: &str = "images/characters/sakura_idle11.png";
pub const IMAGE_CHAR_SAKURA_IDLE13: &str = "images/characters/sakura_idle13.jpg";
pub const IMAGE_CHAR_SAKURA_IDLE_13: &str = "images/characters/sakura_idle_13.png";
pub const IMAGE_CHAR_SAKURA_IDLE14: &str = "images/characters/sakura_idle14.png";

// Sakura 待機動畫幀（使用乒乓循環製造流暢效果）
pub const SAKURA_IDLE_FRAMES: &[&str] = &[
    IMAGE_CHAR_SAKURA_IDLE01,
    IMAGE_CHAR_SAKURA_IDLE02,
    IMAGE_CHAR_SAKURA_IDLE03,
    IMAGE_CHAR_SAKURA_IDLE04,
    IMAGE_CHAR_SAKURA_IDLE03,  // 反向
    IMAGE_CHAR_SAKURA_IDLE02,
    IMAGE_CHAR_SAKURA_IDLE01,
];

// Sakura 跑步動畫幀
pub const SAKURA_RUNNING_FRAMES: &[&str] = &[
    IMAGE_CHAR_SAKURA_IDLE05,
    IMAGE_CHAR_SAKURA_IDLE06,
    IMAGE_CHAR_SAKURA_IDLE07,
    IMAGE_CHAR_SAKURA_IDLE08,
    IMAGE_CHAR_SAKURA_IDLE07,  // 重複關鍵幀
    IMAGE_CHAR_SAKURA_IDLE06,
];

// Sakura 跳躍動畫幀
pub const SAKURA_JUMPING_FRAMES: &[&str] = &[
    IMAGE_CHAR_SAKURA_IDLE09,
    IMAGE_CHAR_SAKURA_IDLE10,
    IMAGE_CHAR_SAKURA_IDLE11,
];

// Sakura 蹲下動畫幀
pub const SAKURA_CROUCHING_FRAMES: &[&str] = &[
    IMAGE_CHAR_SAKURA_IDLE_13,
    IMAGE_CHAR_SAKURA_IDLE14,
];

// Sakura所有动画帧的数组（向後兼容）
pub const SAKURA_ANIMATION_FRAMES: &[&str] = &[
    IMAGE_CHAR_SAKURA_IDLE1, IMAGE_CHAR_SAKURA_IDLE01, IMAGE_CHAR_SAKURA_IDLE02,
    IMAGE_CHAR_SAKURA_IDLE03, IMAGE_CHAR_SAKURA_IDLE04, IMAGE_CHAR_SAKURA_IDLE05,
    IMAGE_CHAR_SAKURA_IDLE06, IMAGE_CHAR_SAKURA_IDLE07, IMAGE_CHAR_SAKURA_IDLE08,
    IMAGE_CHAR_SAKURA_IDLE09, IMAGE_CHAR_SAKURA_IDLE10, IMAGE_CHAR_SAKURA_IDLE11,
    IMAGE_CHAR_SAKURA_IDLE13, IMAGE_CHAR_SAKURA_IDLE_13, IMAGE_CHAR_SAKURA_IDLE14,
];

// OTHER CHARACTER ASSETS
pub const IMAGE_CHAR_TEACHER_IDLE: &str = "images/characters/teacher_idle.jpg";

pub const IMAGE_CHAR_SHIROU_SPRITESHEET: &str = "images/characters/shirou_spritesheet.png";
pub const IMAGE_CHAR_SAKURA_SPRITESHEET: &str = "images/characters/sakura_spritesheet.png";

// SOUND ASSETS
pub const SOUND_JUMP: &str = "sounds/jump.ogg";
pub const SOUND_LAND: &str = "sounds/land.ogg";
pub const SOUND_MENU_MUSIC: &str = "sounds/menu.ogg";
pub const SOUND_GAME_MUSIC: &str = "sounds/game.ogg";
pub const SOUND_GAME_WHY_I_FIGHT_MUSIC: &str = "sounds/game-whyIfight.ogg";
pub const SOUND_FOOTSTEP: &str = "sounds/footstep.ogg";
pub const SOUND_BACKGROUND_MUSIC: &str = "sounds/background.ogg";
