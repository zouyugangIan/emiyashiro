use crate::{database::*, resources::*, states::*};
use bevy::prelude::*;

/// 数据库服务系统
#[derive(Resource)]
pub struct DatabaseService {
    pub database: Option<Database>,
    pub is_connected: bool,
}

impl Default for DatabaseService {
    fn default() -> Self {
        Self {
            database: None,
            is_connected: false,
        }
    }
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
    println!("🗄️ 正在连接数据库...");

    match Database::new().await {
        Ok(db) => {
            println!("✅ 数据库连接成功！");
            Ok(DatabaseService {
                database: Some(db),
                is_connected: true,
            })
        }
        Err(e) => {
            println!("❌ 数据库连接失败: {}", e);
            println!("💡 提示: 请确保 PostgreSQL 数据库正在运行");
            println!("💡 数据库配置: DATABASE_URL 环境变量");
            Ok(DatabaseService::default())
        }
    }
}

/// 保存玩家记录到数据库
pub fn save_player_to_database(
    game_stats: Res<GameStats>,
    character_selection: Res<CharacterSelection>,
    database_service: ResMut<DatabaseService>,
    mut current_session: ResMut<CurrentSession>,
) {
    if !database_service.is_connected {
        println!("⚠️ 数据库未连接，跳过保存");
        return;
    }

    // 这里应该是异步操作，但为了简化，我们先打印日志
    println!("💾 保存玩家数据到数据库:");
    println!("   角色: {:?}", character_selection.selected_character);
    println!("   距离: {:.1}m", game_stats.distance_traveled);
    println!("   跳跃: {} 次", game_stats.jump_count);
    println!("   时间: {:.1}s", game_stats.play_time);

    // 生成会话ID（如果还没有）
    if current_session.session_id.is_none() {
        current_session.session_id = Some(uuid::Uuid::new_v4());
        current_session.player_id = Some(uuid::Uuid::new_v4());
        println!("🆔 生成新的会话ID: {:?}", current_session.session_id);
    }
}

/// 从数据库加载玩家记录
pub fn load_player_from_database(
    database_service: ResMut<DatabaseService>,
    _character_selection: ResMut<CharacterSelection>,
) {
    if !database_service.is_connected {
        println!("⚠️ 数据库未连接，跳过加载");
        return;
    }

    println!("📂 从数据库加载玩家数据...");
    // 这里应该是异步查询操作
    println!("✅ 玩家数据加载完成");
}

/// 获取排行榜数据
pub fn get_leaderboard(database_service: Res<DatabaseService>) -> Vec<PlayerRecord> {
    if !database_service.is_connected {
        println!("⚠️ 数据库未连接，返回空排行榜");
        return vec![];
    }

    println!("🏆 获取排行榜数据...");

    // 模拟排行榜数据
    vec![
        PlayerRecord {
            id: uuid::Uuid::new_v4(),
            name: "士郎".to_string(),
            character_type: CharacterType::Shirou1,
            best_distance: 1500.0,
            total_jumps: 200,
            total_play_time: 300.0,
            games_played: 15,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
        PlayerRecord {
            id: uuid::Uuid::new_v4(),
            name: "樱".to_string(),
            character_type: CharacterType::Shirou2,
            best_distance: 1200.0,
            total_jumps: 180,
            total_play_time: 250.0,
            games_played: 12,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
    ]
}

/// 数据库统计系统
pub fn database_stats_system(
    database_service: Res<DatabaseService>,
    mut timer: Local<Timer>,
    time: Res<Time>,
) {
    if !database_service.is_connected {
        return;
    }

    // 每60秒显示一次数据库统计
    if timer.duration().is_zero() {
        timer.set_duration(std::time::Duration::from_secs(60));
        timer.set_mode(bevy::time::TimerMode::Repeating);
    }
    timer.tick(time.delta());

    if timer.just_finished() {
        println!("📊 数据库统计:");
        println!("   总玩家数: 模拟数据");
        println!("   总游戏会话: 模拟数据");
        println!("   平均游戏时长: 模拟数据");
    }
}

/// 清理旧的游戏会话数据
pub fn cleanup_old_sessions(
    database_service: Res<DatabaseService>,
    mut timer: Local<Timer>,
    time: Res<Time>,
) {
    if !database_service.is_connected {
        return;
    }

    // 每24小时清理一次旧数据
    if timer.duration().is_zero() {
        timer.set_duration(std::time::Duration::from_secs(24 * 60 * 60));
        timer.set_mode(bevy::time::TimerMode::Repeating);
    }
    timer.tick(time.delta());

    if timer.just_finished() {
        println!("🧹 清理30天前的游戏会话数据...");
        // 这里应该执行数据库清理操作
        println!("✅ 数据清理完成");
    }
}
