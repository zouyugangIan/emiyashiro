use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 玩家模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: Uuid,
    pub username: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 游戏会话模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSession {
    pub id: Uuid,
    pub player_id: Uuid,
    pub character_type: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub distance_traveled: f32,
    pub jump_count: i32,
    pub play_time: f32,
    pub score: i32,
}

/// 玩家操作记录模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerAction {
    pub id: Uuid,
    pub session_id: Uuid,
    pub action_type: String,
    pub action_data: Option<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
    pub player_position_x: Option<f32>,
    pub player_position_y: Option<f32>,
}

/// 存档模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveGame {
    pub id: Uuid,
    pub player_id: Uuid,
    pub save_name: String,
    pub game_data: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 游戏数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameData {
    pub character_type: String,
    pub player_position: (f32, f32, f32),
    pub player_velocity: (f32, f32),
    pub camera_position: (f32, f32, f32),
    pub game_stats: GameStatsData,
    pub player_state: PlayerStateData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStatsData {
    pub distance_traveled: f32,
    pub jump_count: u32,
    pub play_time: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStateData {
    pub is_grounded: bool,
    pub is_crouching: bool,
}
