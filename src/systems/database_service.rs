use crate::{resources::*, states::*};
use bevy::prelude::*;

#[cfg(feature = "server")]
use crate::database::*;

/// 数据库服务系统
#[derive(Resource, Default)]
pub struct DatabaseService {
    #[cfg(feature = "server")]
    pub database: Option<Database>,
    pub is_connected: bool,
}

/// 玩家记录结构
#[derive(Debug, Clone)]
pub struct PlayerRecord {
    pub id: uuid::Uuid,
    pub name: String,
    pub character_type: CharacterType,
    pub best_distance: f32,
    pub total_jumps: i32,
    pub total_play_time: f32,
    pub games_played: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// 游戏会话记录
#[derive(Debug, Clone)]
pub struct GameSession {
    pub id: uuid::Uuid,
    pub player_id: uuid::Uuid,
    pub distance_traveled: f32,
    pub jumps_count: i32,
    pub play_time: f32,
    pub character_used: CharacterType,
    pub session_date: chrono::DateTime<chrono::Utc>,
}

/// 初始化数据库连接
pub async fn initialize_database() -> Result<DatabaseService, Box<dyn std::error::Error>> {
    #[cfg(feature = "server")]
    {
        crate::debug_log!("🗄️ 正在连接数据库...");
        match Database::new().await {
            Ok(db) => {
                crate::debug_log!("✅ 数据库连接成功！");
                Ok(DatabaseService {
                    database: Some(db),
                    is_connected: true,
                })
            }
            Err(e) => {
                crate::debug_log!("❌ 数据库连接失败: {}", e);
                Ok(DatabaseService::default())
            }
        }
    }

    #[cfg(not(feature = "server"))]
    {
        // Client always returns disconnected/mock service
        Ok(DatabaseService::default())
    }
}

/// 保存玩家记录到数据库
#[allow(clippy::needless_return)]
pub fn save_player_to_database(
    _game_stats: Res<GameStats>,
    _character_selection: Res<CharacterSelection>,
    database_service: ResMut<DatabaseService>,
    _current_session: ResMut<CurrentSession>,
) {
    if !database_service.is_connected {
        return;
    }

    #[cfg(feature = "server")]
    {
        crate::debug_log!("💾 [Server] 保存玩家数据到数据库...");
        // Real DB logic here
    }
}

/// 从数据库加载玩家记录
#[allow(clippy::needless_return)]
pub fn load_player_from_database(
    database_service: ResMut<DatabaseService>,
    _character_selection: ResMut<CharacterSelection>,
) {
    if !database_service.is_connected {
        return;
    }
    #[cfg(feature = "server")]
    crate::debug_log!("📂 [Server] 从数据库加载玩家数据...");
}

/// 获取排行榜数据
pub fn get_leaderboard(database_service: Res<DatabaseService>) -> Vec<PlayerRecord> {
    if !database_service.is_connected {
        return vec![];
    }

    // Return mock data for now
    vec![]
}

/// 数据库统计系统
#[cfg(feature = "server")]
pub fn database_stats_system(
    database_service: Res<DatabaseService>,
    mut timer: Local<Timer>,
    time: Res<Time>,
) {
    if !database_service.is_connected {
        return;
    }

    if timer.duration().is_zero() {
        timer.set_duration(std::time::Duration::from_secs(60));
        timer.set_mode(bevy::time::TimerMode::Repeating);
    }
    timer.tick(time.delta());

    if timer.just_finished() {
        crate::debug_log!("📊 [Server] 数据库统计...");
    }
}

#[cfg(not(feature = "server"))]
#[allow(clippy::needless_return)]
pub fn database_stats_system(
    database_service: Res<DatabaseService>,
    mut _timer: Local<Timer>,
    _time: Res<Time>,
) {
    if !database_service.is_connected {
        return;
    }
}

/// 清理旧的游戏会话数据
#[cfg(feature = "server")]
pub fn cleanup_old_sessions(
    database_service: Res<DatabaseService>,
    mut timer: Local<Timer>,
    time: Res<Time>,
) {
    if !database_service.is_connected {
        return;
    }

    if timer.duration().is_zero() {
        timer.set_duration(std::time::Duration::from_secs(24 * 60 * 60));
        timer.set_mode(bevy::time::TimerMode::Repeating);
    }
    timer.tick(time.delta());

    if timer.just_finished() {
        crate::debug_log!("🧹 [Server] 清理旧数据...");
    }
}

#[cfg(not(feature = "server"))]
#[allow(clippy::needless_return)]
pub fn cleanup_old_sessions(
    database_service: Res<DatabaseService>,
    mut _timer: Local<Timer>,
    _time: Res<Time>,
) {
    if !database_service.is_connected {
        return;
    }
}
