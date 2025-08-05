use serde_json;
use std::fs;

// 复制相关的结构体定义进行测试
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct SaveFileData {
    pub version: String,
    pub metadata: SaveFileMetadata,
    pub game_state: CompleteGameState,
    pub checksum: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct SaveFileMetadata {
    pub name: String,
    pub score: u32,
    pub distance: f32,
    pub play_time: f32,
    pub save_timestamp: chrono::DateTime<chrono::Utc>,
    pub file_path: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct CompleteGameState {
    pub player_position: Vec3,
    pub player_velocity: Velocity,
    pub player_grounded: bool,
    pub player_crouching: bool,
    pub player_animation_state: String,
    pub camera_position: Vec3,
    pub camera_target: Vec3,
    pub score: u32,
    pub distance_traveled: f32,
    pub jump_count: u32,
    pub play_time: f32,
    pub selected_character: CharacterType,
    pub player_count: PlayerCount,
    pub music_position: f32,
    pub music_playing: bool,
    pub audio_volume: f32,
    pub entities_snapshot: Vec<EntitySnapshot>,
    pub save_timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum CharacterType {
    #[default]
    Shirou1,
    Shirou2,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default)]
pub enum PlayerCount {
    #[default]
    Single,
    Double,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct EntitySnapshot {
    // 空结构体用于测试
}

fn main() {
    let json_data = fs::read_to_string("saves/test_save.json").expect("Failed to read file");
    
    match serde_json::from_str::<SaveFileData>(&json_data) {
        Ok(save_data) => {
            println!("✅ 成功解析存档文件!");
            println!("版本: {}", save_data.version);
            println!("存档名: {}", save_data.metadata.name);
            println!("分数: {}", save_data.metadata.score);
        }
        Err(e) => {
            println!("❌ 解析失败: {}", e);
            
            // 尝试逐步解析来找出问题
            let json_value: serde_json::Value = serde_json::from_str(&json_data).unwrap();
            println!("JSON 结构:");
            println!("{:#}", json_value);
        }
    }
}