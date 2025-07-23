use bevy::prelude::*;

/// 游戏常量配置
pub struct GameConfig;

impl GameConfig {
    // 物理常量
    pub const GRAVITY: f32 = 800.0;
    pub const JUMP_VELOCITY: f32 = 400.0;
    pub const MOVE_SPEED: f32 = 250.0;
    pub const GROUND_LEVEL: f32 = -240.0;
    
    // 摄像机设置
    pub const CAMERA_FOLLOW_SPEED: f32 = 2.0;
    pub const CAMERA_OFFSET: f32 = 200.0;
    pub const CAMERA_IDLE_SPEED: f32 = 50.0;
    
    // 玩家设置
    pub const PLAYER_SIZE: Vec2 = Vec2::new(40.0, 60.0);
    pub const PLAYER_CROUCH_SIZE: Vec2 = Vec2::new(40.0, 30.0);
    pub const PLAYER_START_POS: Vec3 = Vec3::new(-400.0, -240.0, 1.0);
    
    // 地面设置
    pub const GROUND_SIZE: Vec2 = Vec2::new(2000.0, 50.0);
    pub const GROUND_POS: Vec3 = Vec3::new(0.0, -300.0, 0.0);
    pub const GROUND_COLOR: Color = Color::srgb(0.3, 0.3, 0.3);
}

/// 游戏资源句柄
#[derive(Resource)]
pub struct GameAssets {
    pub cover_texture: Handle<Image>,
    pub cover2_texture: Handle<Image>,
    pub shirou1_texture: Handle<Image>,
    pub shirou2_texture: Handle<Image>,
    pub font: Handle<Font>,
    // 音效资源
    pub jump_sound: Handle<AudioSource>,
    pub land_sound: Handle<AudioSource>,
    pub footstep_sound: Handle<AudioSource>,
    // 背景音乐
    pub menu_music: Handle<AudioSource>,
    pub game_music: Handle<AudioSource>,
    pub background_music: Handle<AudioSource>,
}

/// 音效设置资源
#[derive(Resource)]
pub struct AudioSettings {
    pub master_volume: f32,
    pub sfx_volume: f32,
    pub music_volume: f32,
    pub music_enabled: bool,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            sfx_volume: 0.7,
            music_volume: 0.5,
            music_enabled: true,
        }
    }
}

/// 游戏统计资源
#[derive(Resource, Default)]
pub struct GameStats {
    pub distance_traveled: f32,
    pub jump_count: u32,
    pub play_time: f32,
}

/// 数据库资源
#[derive(Resource)]
pub struct DatabaseResource {
    pub database: Option<crate::database::Database>,
}

impl Default for DatabaseResource {
    fn default() -> Self {
        Self { database: None }
    }
}

/// 当前游戏会话资源
#[derive(Resource, Default)]
pub struct CurrentSession {
    pub session_id: Option<uuid::Uuid>,
    pub player_id: Option<uuid::Uuid>,
}