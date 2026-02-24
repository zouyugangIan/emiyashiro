use bevy::prelude::*;

// Vec3 序列化支持
mod vec3_serde {
    use bevy::prelude::Vec3;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Serialize, Deserialize)]
    struct Vec3Helper {
        x: f32,
        y: f32,
        z: f32,
    }

    pub fn serialize<S>(vec: &Vec3, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let helper = Vec3Helper {
            x: vec.x,
            y: vec.y,
            z: vec.z,
        };
        helper.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec3, D::Error>
    where
        D: Deserializer<'de>,
    {
        let helper = Vec3Helper::deserialize(deserializer)?;
        Ok(Vec3::new(helper.x, helper.y, helper.z))
    }
}

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
    // UI封面图片集合（用于轮换显示）
    pub cover_textures: Vec<Handle<Image>>,
    pub current_cover_index: usize,

    // 角色动画帧集合
    pub shirou_animation_frames: Vec<Handle<Image>>,
    pub sakura_animation_frames: Vec<Handle<Image>>,
    pub current_shirou_frame: usize,
    pub current_sakura_frame: usize,

    pub font: Handle<Font>,
    // 精灵表资源
    pub shirou_spritesheet: Option<Handle<Image>>,
    pub shirou_spritesheet_run: Option<Handle<Image>>,
    pub shirou_spritesheet_attack: Option<Handle<Image>>,
    pub sakura_spritesheet: Option<Handle<Image>>,
    pub shirou_atlas: Option<Handle<TextureAtlasLayout>>,
    pub shirou_atlas_run: Option<Handle<TextureAtlasLayout>>,
    pub shirou_atlas_attack: Option<Handle<TextureAtlasLayout>>,
    pub sakura_atlas: Option<Handle<TextureAtlasLayout>>,
    // 音效资源
    pub jump_sound: Handle<AudioSource>,
    pub land_sound: Handle<AudioSource>,
    pub footstep_sound: Handle<AudioSource>,
    // 背景音乐
    pub menu_music: Handle<AudioSource>,
    pub game_music: Handle<AudioSource>,
    pub game_whyifight_music: Handle<AudioSource>, // 第一首游戏音乐
    pub background_music: Handle<AudioSource>,
}

impl GameAssets {
    /// 获取当前封面图片
    pub fn get_current_cover(&self) -> Handle<Image> {
        self.cover_textures[self.current_cover_index].clone()
    }

    /// 切换到下一张封面
    pub fn next_cover(&mut self) -> Handle<Image> {
        self.current_cover_index = (self.current_cover_index + 1) % self.cover_textures.len();
        self.get_current_cover()
    }

    /// 获取当前 Shirou 动画帧
    pub fn get_current_shirou_frame(&self) -> Handle<Image> {
        self.shirou_animation_frames[self.current_shirou_frame].clone()
    }

    /// 切换到下一个 Shirou 动画帧
    pub fn next_shirou_frame(&mut self) -> Handle<Image> {
        self.current_shirou_frame =
            (self.current_shirou_frame + 1) % self.shirou_animation_frames.len();
        self.get_current_shirou_frame()
    }

    /// 获取当前 Sakura 动画帧
    pub fn get_current_sakura_frame(&self) -> Handle<Image> {
        self.sakura_animation_frames[self.current_sakura_frame].clone()
    }

    /// 切换到下一个 Sakura 动画帧
    pub fn next_sakura_frame(&mut self) -> Handle<Image> {
        self.current_sakura_frame =
            (self.current_sakura_frame + 1) % self.sakura_animation_frames.len();
        self.get_current_sakura_frame()
    }
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
#[derive(Resource, Default)]
pub struct DatabaseResource {
    pub database: Option<crate::database::Database>,
}

/// 当前游戏会话资源
#[derive(Resource, Default)]
pub struct CurrentSession {
    pub session_id: Option<uuid::Uuid>,
    pub player_id: Option<uuid::Uuid>,
}

fn default_save_file_version() -> String {
    "2.0".to_string()
}

/// 新的存档文件格式 - 包含元数据、游戏状态和校验和
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct SaveFileData {
    #[serde(default = "default_save_file_version")]
    pub version: String,
    pub metadata: SaveFileMetadata,
    pub game_state: CompleteGameState,
    pub checksum: String,
}

impl SaveFileData {
    pub fn new(metadata: SaveFileMetadata, game_state: CompleteGameState) -> Self {
        let mut data = Self {
            version: "2.0".to_string(),
            metadata,
            game_state,
            checksum: String::new(),
        };
        // Calculate checksum after creating the struct
        data.checksum = Self::calculate_checksum_for(&data);
        data
    }

    fn calculate_checksum_for(data: &SaveFileData) -> String {
        use crate::systems::shared_utils::calculate_checksum;

        let mut temp_data = data.clone();
        temp_data.checksum = String::new();
        if let Ok(json) = serde_json::to_string_pretty(&temp_data) {
            calculate_checksum(json.as_bytes())
        } else {
            String::new()
        }
    }

    pub fn verify_checksum(&self) -> bool {
        let calculated = Self::calculate_checksum_for(self);
        calculated == self.checksum
    }
}

/// 存档管理资源
#[derive(Resource, Default)]
pub struct SaveManager {
    pub current_save: Option<SaveFileData>,
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
    #[serde(with = "vec3_serde")]
    pub player_position: Vec3,
    pub player_velocity: crate::components::Velocity,
    pub player_grounded: bool,
    pub player_crouching: bool,
    pub player_animation_state: String, // 当前动画状态

    // Camera state
    #[serde(with = "vec3_serde")]
    pub camera_position: Vec3,
    #[serde(with = "vec3_serde")]
    pub camera_target: Vec3,

    // Game metrics
    pub score: u32,
    pub distance_traveled: f32,
    pub jump_count: u32,
    pub play_time: f32,

    // Character selection and player count
    pub selected_character: crate::states::CharacterType,
    pub player_count: PlayerCount,

    // Audio state
    pub music_position: f32,
    pub music_playing: bool,
    pub audio_volume: f32,

    // Game entities state (for future expansion)
    pub entities_snapshot: Vec<EntitySnapshot>,

    // Timestamp
    pub save_timestamp: chrono::DateTime<chrono::Utc>,
}

/// 玩家数量枚举
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default)]
pub enum PlayerCount {
    #[default]
    Single, // 1P
    Double, // 2P
}

impl PlayerCount {
    pub fn to_display_string(&self) -> &'static str {
        match self {
            PlayerCount::Single => "1P",
            PlayerCount::Double => "2P",
        }
    }
}

/// 实体快照 - 用于保存游戏实体状态
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct EntitySnapshot {
    pub entity_type: EntityType,
    pub position: Vec3,
    pub velocity: Option<crate::components::Velocity>,
    pub active: bool,
}

/// 实体类型枚举
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub enum EntityType {
    Player,
    Ground,
    Obstacle,
    Collectible,
    Effect,
    Camera,
}

impl Default for CompleteGameState {
    fn default() -> Self {
        Self {
            player_position: Vec3::ZERO,
            player_velocity: crate::components::Velocity { x: 0.0, y: 0.0 },
            player_grounded: true,
            player_crouching: false,
            player_animation_state: "idle".to_string(),
            camera_position: Vec3::ZERO,
            camera_target: Vec3::ZERO,
            score: 0,
            distance_traveled: 0.0,
            jump_count: 0,
            play_time: 0.0,
            selected_character: crate::states::CharacterType::Shirou1,
            player_count: PlayerCount::Single,
            music_position: 0.0,
            music_playing: false,
            audio_volume: 0.5,
            entities_snapshot: Vec::new(),
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
    pub selected_save_index: Option<usize>,
}

impl SaveFileManager {
    pub fn new() -> Self {
        Self {
            save_directory: "saves".to_string(),
            save_files: Vec::new(),
            current_save_name: None,
            selected_save_index: None,
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
    #[serde(default)]
    pub selected_character: crate::states::CharacterType,
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

    pub fn clear_pause_state(&mut self) {
        self.is_paused = false;
        self.preserved_state = None;
        self.pause_timestamp = None;
    }

    pub fn resume_game(&mut self) -> Option<CompleteGameState> {
        let preserved_state = self.preserved_state.take();
        self.is_paused = false;
        self.pause_timestamp = None;
        preserved_state
    }
}

/// 音频状态管理器
#[derive(Resource)]
pub struct AudioStateManager {
    pub music_playing: bool,
    pub music_volume: f32,
    pub music_position: f32,
    pub sfx_enabled: bool,
}

impl Default for AudioStateManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioStateManager {
    pub fn new() -> Self {
        Self {
            music_playing: false,
            music_volume: 0.5,
            music_position: 0.0,
            sfx_enabled: true,
        }
    }

    pub fn set_music_playing(&mut self, playing: bool) {
        self.music_playing = playing;
    }

    pub fn set_music_position(&mut self, position: f32) {
        self.music_position = position;
    }
}
