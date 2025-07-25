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
    // 精灵表资源
    pub shirou_spritesheet: Option<Handle<Image>>,
    pub sakura_spritesheet: Option<Handle<Image>>,
    pub shirou_atlas: Option<Handle<TextureAtlasLayout>>,
    pub sakura_atlas: Option<Handle<TextureAtlasLayout>>,
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

/// 存档数据结构
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct SaveData {
    pub player_name: String,
    pub selected_character: crate::states::CharacterType,
    pub best_distance: f32,
    pub total_jumps: u32,
    pub total_play_time: f32,
    pub save_time: chrono::DateTime<chrono::Utc>,
}

impl Default for SaveData {
    fn default() -> Self {
        Self {
            player_name: "士郎".to_string(),
            selected_character: crate::states::CharacterType::Shirou1,
            best_distance: 0.0,
            total_jumps: 0,
            total_play_time: 0.0,
            save_time: chrono::Utc::now(),
        }
    }
}

/// 存档管理资源
#[derive(Resource, Default)]
pub struct SaveManager {
    pub current_save: Option<SaveData>,
    pub save_file_path: String,
}

impl SaveManager {
    pub fn new() -> Self {
        Self {
            current_save: None,
            save_file_path: "save_data.json".to_string(),
        }
    }
}

/// 完整游戏状态 - 用于暂停存档系统
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct CompleteGameState {
    // Player state
    pub player_position: Vec3,
    pub player_velocity: crate::components::Velocity,
    pub player_grounded: bool,
    pub player_crouching: bool,
    
    // Camera state
    pub camera_position: Vec3,
    
    // Game metrics
    pub score: u32,
    pub distance_traveled: f32,
    pub jump_count: u32,
    pub play_time: f32,
    
    // Character selection
    pub selected_character: crate::states::CharacterType,
    
    // Timestamp
    pub save_timestamp: chrono::DateTime<chrono::Utc>,
}

impl Default for CompleteGameState {
    fn default() -> Self {
        Self {
            player_position: Vec3::ZERO,
            player_velocity: crate::components::Velocity { x: 0.0, y: 0.0 },
            player_grounded: true,
            player_crouching: false,
            camera_position: Vec3::ZERO,
            score: 0,
            distance_traveled: 0.0,
            jump_count: 0,
            play_time: 0.0,
            selected_character: crate::states::CharacterType::Shirou1,
            save_timestamp: chrono::Utc::now(),
        }
    }
}

/// 存档文件管理器
#[derive(Resource)]
pub struct SaveFileManager {
    pub save_directory: String,
    pub save_files: Vec<SaveFileMetadata>,
    pub current_save_name: Option<String>,
}

impl SaveFileManager {
    pub fn new() -> Self {
        Self {
            save_directory: "saves".to_string(),
            save_files: Vec::new(),
            current_save_name: None,
        }
    }
}

impl Default for SaveFileManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 存档文件元数据
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct SaveFileMetadata {
    pub name: String,
    pub score: u32,
    pub distance: f32,
    pub play_time: f32,
    pub save_timestamp: chrono::DateTime<chrono::Utc>,
    pub file_path: String,
}

/// 暂停管理器
#[derive(Resource, Default)]
pub struct PauseManager {
    pub is_paused: bool,
    pub preserved_state: Option<CompleteGameState>,
    pub pause_timestamp: Option<std::time::Instant>,
}

impl PauseManager {
    pub fn new() -> Self {
        Self {
            is_paused: false,
            preserved_state: None,
            pause_timestamp: None,
        }
    }
    
    pub fn pause_game(&mut self, state: CompleteGameState) {
        self.is_paused = true;
        self.preserved_state = Some(state);
        self.pause_timestamp = Some(std::time::Instant::now());
    }
    
    pub fn resume_game(&mut self) -> Option<CompleteGameState> {
        self.is_paused = false;
        self.pause_timestamp = None;
        self.preserved_state.take()
    }
}

/// 音频状态管理器
#[derive(Resource, Default)]
pub struct AudioStateManager {
    pub music_playing: bool,
    pub music_volume: f32,
    pub sfx_enabled: bool,
}

impl AudioStateManager {
    pub fn new() -> Self {
        Self {
            music_playing: false,
            music_volume: 0.5,
            sfx_enabled: true,
        }
    }
}