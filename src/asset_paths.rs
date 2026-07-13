// FONT ASSETS
pub const FONT_FIRA_SANS: &str = "fonts/FiraSans-Bold.ttf";

// IMAGE ASSETS - UI COVERS (主界面封面图片)
pub const IMAGE_UI_COVER01: &str = "images/ui/cover01.png";
pub const IMAGE_UI_COVER02: &str = "images/ui/cover02.png";
pub const IMAGE_UI_COVER03: &str = "images/ui/cover03.png";
pub const IMAGE_UI_COVER04: &str = "images/ui/cover04.png";
pub const IMAGE_UI_COVER05: &str = "images/ui/cover05.jpeg";
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
pub const IMAGE_UI_COVER18: &str = "images/ui/cover18.png";
pub const IMAGE_UI_VOLUME_ICON: &str = "images/ui/volume_icon.png";
pub const IMAGE_UI_VOLUME_MUTED_ICON: &str = "images/ui/volume_muted_icon.png";

// 所有封面图片的数组，用于轮流显示
pub const UI_COVER_IMAGES: &[&str] = &[
    IMAGE_UI_COVER01,
    IMAGE_UI_COVER02,
    IMAGE_UI_COVER03,
    IMAGE_UI_COVER04,
    IMAGE_UI_COVER05,
    IMAGE_UI_COVER06,
    IMAGE_UI_COVER07,
    IMAGE_UI_COVER08,
    IMAGE_UI_COVER09,
    IMAGE_UI_COVER10,
    IMAGE_UI_COVER11,
    IMAGE_UI_COVER12,
    IMAGE_UI_COVER13,
    IMAGE_UI_COVER14,
    IMAGE_UI_COVER15,
    IMAGE_UI_COVER16,
    IMAGE_UI_COVER17,
    IMAGE_UI_COVER18,
];

// CHARACTER ASSETS - HEAVEN'S FEEL SHIROU
pub const IMAGE_HF_SHIROU_IDLE: &str = "images/characters/hf_idle.png";

// CHARACTER ASSETS - SAKURA (2P角色精灵动画)
pub const IMAGE_CHAR_SAKURA_IDLE01: &str = "images/characters/sakura_jk/base_frames/f01.png";
pub const IMAGE_CHAR_SAKURA_IDLE02: &str = "images/characters/sakura_idle02.png";
pub const IMAGE_CHAR_SAKURA_IDLE03: &str = "images/characters/sakura_idle03.png";
pub const IMAGE_CHAR_SAKURA_IDLE04: &str = "images/characters/sakura_idle04.png";
pub const IMAGE_CHAR_SAKURA_IDLE05: &str = "images/characters/sakura_idle05.png";
pub const IMAGE_CHAR_SAKURA_IDLE06: &str = "images/characters/sakura_idle06.png";
pub const IMAGE_CHAR_SAKURA_IDLE07: &str = "images/characters/sakura_idle07.png";
pub const IMAGE_CHAR_SAKURA_IDLE08: &str = "images/characters/sakura_idle08.png";
pub const IMAGE_CHAR_SAKURA_IDLE09: &str = "images/characters/sakura_idle9.jpg"; // 注意：实际文件名是 sakura_idle9.jpg
pub const IMAGE_CHAR_SAKURA_IDLE10: &str = "images/characters/sakura_idle10.png";
pub const IMAGE_CHAR_SAKURA_IDLE11: &str = "images/characters/sakura_idle11.png";
pub const IMAGE_CHAR_SAKURA_IDLE13: &str = "images/characters/sakura_idle13.jpg";
pub const IMAGE_CHAR_SAKURA_IDLE14: &str = "images/characters/sakura_idle14.png";

// Sakura JK redesign: every gameplay state switches standalone 256x256 images.
pub const SAKURA_IDLE_FRAMES: &[&str] = &[
    "images/characters/sakura_jk/base_frames/f01.png",
    "images/characters/sakura_jk/base_frames/f02.png",
    "images/characters/sakura_jk/base_frames/f01.png",
];

// Sakura 跑步动画帧
pub const SAKURA_RUNNING_FRAMES: &[&str] = &[
    "images/characters/sakura_jk/base_frames/f03.png",
    "images/characters/sakura_jk/base_frames/f04.png",
];

// Sakura 跳跃动画帧
pub const SAKURA_JUMPING_FRAMES: &[&str] = &[
    "images/characters/sakura_jk/base_frames/f05.png",
    "images/characters/sakura_jk/base_frames/f06.png",
];

// Sakura 蹲下动画帧
pub const SAKURA_CROUCHING_FRAMES: &[&str] = &[
    "images/characters/sakura_jk/base_frames/f07.png",
    "images/characters/sakura_jk/base_frames/f08.png",
];

// Sakura 2P attack source atlases. Runtime playback uses the standalone images
// generated under `SAKURA_ATTACK_FRAME_ROOT`, never a TextureAtlas.
pub const IMAGE_SAKURA_ATTACK_GROUND_LIGHT: &str =
    "images/characters/sakura_attack/sakura_attack_ground_light.png";
pub const IMAGE_SAKURA_ATTACK_HEAVY: &str =
    "images/characters/sakura_attack/sakura_attack_heavy.png";
pub const IMAGE_SAKURA_ATTACK_AIR_COMBO: &str =
    "images/characters/sakura_attack/sakura_attack_air_combo.png";
pub const IMAGE_SAKURA_ATTACK_MOBILITY: &str =
    "images/characters/sakura_attack/sakura_attack_mobility.png";
pub const IMAGE_SAKURA_ATTACK_NINJUTSU: &str =
    "images/characters/sakura_attack/sakura_attack_ninjutsu.png";
pub const IMAGE_SAKURA_ATTACK_ULTIMATE: &str =
    "images/characters/sakura_attack/sakura_attack_ultimate.png";
pub const IMAGE_SAKURA_ATTACK_WEAPON_PROJECTION: &str =
    "images/characters/sakura_attack/sakura_attack_weapon_projection.png";
pub const SAKURA_ATTACK_CELL: (u32, u32) = (256, 256);
pub const SAKURA_ATTACK_GROUND_LIGHT_GRID: (u32, u32) = (8, 5);
pub const SAKURA_ATTACK_HEAVY_GRID: (u32, u32) = (8, 5);
pub const SAKURA_ATTACK_AIR_COMBO_GRID: (u32, u32) = (8, 5);
pub const SAKURA_ATTACK_MOBILITY_GRID: (u32, u32) = (6, 4);
pub const SAKURA_ATTACK_NINJUTSU_GRID: (u32, u32) = (8, 4);
pub const SAKURA_ATTACK_ULTIMATE_GRID: (u32, u32) = (8, 3);
pub const SAKURA_ATTACK_WEAPON_PROJECTION_GRID: (u32, u32) = (6, 4);
pub const SAKURA_ATTACK_FRAME_ROOT: &str = "images/characters/sakura_attack/frames";
pub const SAKURA_ATTACK_GROUND_LIGHT_GROUP: &str = "ground_light";
pub const SAKURA_ATTACK_HEAVY_GROUP: &str = "heavy";
pub const SAKURA_ATTACK_AIR_COMBO_GROUP: &str = "air_combo";
pub const SAKURA_ATTACK_MOBILITY_GROUP: &str = "mobility";
pub const SAKURA_ATTACK_NINJUTSU_GROUP: &str = "ninjutsu_projectiles";
pub const SAKURA_ATTACK_ULTIMATE_GROUP: &str = "ultimate";
pub const SAKURA_ATTACK_WEAPON_PROJECTION_GROUP: &str = "weapon_projection";

pub fn sakura_attack_frame_path(group: &str, row: u8, frame: usize) -> String {
    format!("{SAKURA_ATTACK_FRAME_ROOT}/{group}/r{row:02}_f{frame:02}.png")
}

pub const IMAGE_HF_SHIROU_CORE_SHEET: &str = "images/characters/hf_shirou_core_sheet.png";
pub const IMAGE_HF_SHIROU_RUN_SHEET: &str = "images/characters/hf_shirou_run_sheet.png";
pub const IMAGE_HF_SHIROU_ATTACK_SHEET: &str = "images/characters/hf_shirou_attack_sheet.png";
pub const HF_SHIROU_CORE_COLS: u32 = 4;
pub const HF_SHIROU_CORE_ROWS: u32 = 2;
pub const HF_SHIROU_CORE_FRAME_COUNT: usize = (HF_SHIROU_CORE_COLS * HF_SHIROU_CORE_ROWS) as usize;
pub const HF_SHIROU_RUN_COLS: u32 = 5;
pub const HF_SHIROU_RUN_FRAME_COUNT: usize = HF_SHIROU_RUN_COLS as usize;
pub const HF_SHIROU_ATTACK_COLS: u32 = 4;
pub const HF_SHIROU_ATTACK_FRAME_COUNT: usize = HF_SHIROU_ATTACK_COLS as usize;
pub const IMAGE_HF_SHIROU_OVEREDGE_LIGHT_ATTACK_SHEET: &str =
    "images/characters/hf_shirou_overedge_light_combo_sheet.png";
pub const IMAGE_HF_SHIROU_OVEREDGE_HEAVY_ATTACK_SHEET: &str =
    "images/characters/hf_shirou_overedge_heavy_combo_sheet.png";
pub const HF_SHIROU_OVEREDGE_LIGHT_ATTACK_FRAME_COUNT: usize = 11;
pub const HF_SHIROU_OVEREDGE_RELEASE_FRAME_COUNT: usize = 3;
pub const HF_SHIROU_OVEREDGE_LIGHT_ATTACK_SEGMENT_FRAME_COUNT: usize = 3;
pub const HF_SHIROU_OVEREDGE_HEAVY_ATTACK_FRAME_COUNT: usize = 17;
pub const HF_SHIROU_OVEREDGE_ATTACK_FRAME_DURATION_SECS: f32 = 0.07;
pub const IMAGE_HF_SHIROU_ATTACK_MODULES_OVERVIEW: &str =
    "images/characters/reference/hf_shirou_attack_modules_overview.png";
pub const IMAGE_HF_SHIROU_ADVANCED_ATTACK_MODULES_OVERVIEW: &str =
    "images/characters/reference/hf_shirou_advanced_attack_modules_overview.png";
pub const IMAGE_HF_SHIROU_ATTACK_GROUND_LIGHT_REFERENCE: &str =
    "images/characters/reference/hf_shirou_attack_ground_light_reference.png";
pub const IMAGE_HF_SHIROU_ATTACK_HEAVY_REFERENCE: &str =
    "images/characters/reference/hf_shirou_attack_heavy_reference.png";
pub const IMAGE_HF_SHIROU_ATTACK_AIR_COMBO_REFERENCE: &str =
    "images/characters/reference/hf_shirou_attack_air_combo_reference.png";
pub const IMAGE_HF_SHIROU_ATTACK_MOBILITY_REFERENCE: &str =
    "images/characters/reference/hf_shirou_attack_mobility_reference.png";
pub const IMAGE_HF_SHIROU_ATTACK_NINJUTSU_PROJECTILES_REFERENCE: &str =
    "images/characters/reference/hf_shirou_attack_ninjutsu_projectiles_reference.png";
pub const IMAGE_HF_SHIROU_ATTACK_ULTIMATE_REFERENCE: &str =
    "images/characters/reference/hf_shirou_attack_ultimate_reference.png";
pub const IMAGE_HF_SHIROU_ATTACK_WEAPON_PROJECTION_REFERENCE: &str =
    "images/characters/reference/hf_shirou_attack_weapon_projection_reference.png";
pub const IMAGE_HF_SHIROU_ATTACK_GROUND_LIGHT_ROW_SHEETS: [&str; 5] = [
    "images/characters/reference/v2_generated/rows/ground_light_v2_r01.png",
    "images/characters/reference/v2_generated/rows/ground_light_v2_r02.png",
    "images/characters/reference/v2_generated/rows/ground_light_v2_r03.png",
    "images/characters/reference/v2_generated/rows/ground_light_v2_r04.png",
    "images/characters/reference/v2_generated/rows/ground_light_v2_r05.png",
];

// Reference attack atlas parameters. The generated reference actions are
// production 256px RGBA frames with no board label/header offset.
pub const REFERENCE_BOARD_GROUND_LIGHT_COLS: u32 = 8;
pub const REFERENCE_BOARD_GROUND_LIGHT_ROWS: u32 = 5;
pub const REFERENCE_BOARD_GROUND_LIGHT_CELL: (u32, u32) = (256, 256);
pub const REFERENCE_BOARD_GROUND_LIGHT_OFFSET: (u32, u32) = (0, 0);
pub const REFERENCE_BOARD_HEAVY_COLS: u32 = 8;
pub const REFERENCE_BOARD_HEAVY_ROWS: u32 = 5;
pub const REFERENCE_BOARD_HEAVY_CELL: (u32, u32) = (256, 256);
pub const REFERENCE_BOARD_HEAVY_OFFSET: (u32, u32) = (0, 0);
pub const REFERENCE_BOARD_AIR_COMBO_COLS: u32 = 8;
pub const REFERENCE_BOARD_AIR_COMBO_ROWS: u32 = 5;
pub const REFERENCE_BOARD_AIR_COMBO_CELL: (u32, u32) = (256, 256);
pub const REFERENCE_BOARD_AIR_COMBO_OFFSET: (u32, u32) = (0, 0);
pub const REFERENCE_BOARD_MOBILITY_COLS: u32 = 6;
pub const REFERENCE_BOARD_MOBILITY_ROWS: u32 = 4;
pub const REFERENCE_BOARD_MOBILITY_CELL: (u32, u32) = (256, 256);
pub const REFERENCE_BOARD_MOBILITY_OFFSET: (u32, u32) = (0, 0);
pub const REFERENCE_BOARD_NINJUTSU_COLS: u32 = 8;
pub const REFERENCE_BOARD_NINJUTSU_ROWS: u32 = 4;
pub const REFERENCE_BOARD_NINJUTSU_CELL: (u32, u32) = (256, 256);
pub const REFERENCE_BOARD_NINJUTSU_OFFSET: (u32, u32) = (0, 0);
pub const REFERENCE_BOARD_ULTIMATE_COLS: u32 = 8;
pub const REFERENCE_BOARD_ULTIMATE_ROWS: u32 = 3;
pub const REFERENCE_BOARD_ULTIMATE_CELL: (u32, u32) = (256, 256);
pub const REFERENCE_BOARD_ULTIMATE_OFFSET: (u32, u32) = (0, 0);
pub const REFERENCE_BOARD_WEAPON_PROJ_COLS: u32 = 6;
pub const REFERENCE_BOARD_WEAPON_PROJ_ROWS: u32 = 4;
pub const REFERENCE_BOARD_WEAPON_PROJ_CELL: (u32, u32) = (256, 256);
pub const REFERENCE_BOARD_WEAPON_PROJ_OFFSET: (u32, u32) = (0, 0);
pub const REFERENCE_BOARD_OVERVIEW_COLS: u32 = 8;
pub const REFERENCE_BOARD_OVERVIEW_ROWS: u32 = 5;
pub const REFERENCE_BOARD_OVERVIEW_CELL: (u32, u32) = (256, 256);
pub const REFERENCE_BOARD_ADVANCED_OVERVIEW_COLS: u32 = 6;
pub const REFERENCE_BOARD_ADVANCED_OVERVIEW_ROWS: u32 = 4;
pub const REFERENCE_BOARD_ADVANCED_OVERVIEW_CELL: (u32, u32) = (256, 256);

// CLOUD ASSETS (背景云彩图片)
pub const IMAGE_CLOUD_01: &str = "images/cloud/cloud_soft_01.png";
pub const IMAGE_CLOUD_02: &str = "images/cloud/cloud_soft_02.png";

// 所有云彩图片的数组
pub const CLOUD_IMAGES: &[&str] = &[IMAGE_CLOUD_01, IMAGE_CLOUD_02];

// SOUND ASSETS
pub const SOUND_JUMP: &str = "sounds/jump.ogg";
pub const SOUND_LAND: &str = "sounds/land.ogg";
pub const SOUND_MENU_MUSIC: &str = "sounds/menu.ogg";
pub const SOUND_GAME_MUSIC: &str = "sounds/game.ogg";
pub const SOUND_GAME_WHY_I_FIGHT_MUSIC: &str = "sounds/game-whyIfight.ogg";
pub const SOUND_FOOTSTEP: &str = "sounds/footstep.ogg";
pub const SOUND_BACKGROUND_MUSIC: &str = "sounds/background.ogg";
