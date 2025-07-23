pub mod models;
pub mod operations;

use sqlx::{PgPool, Row};
use std::env;

/// 数据库连接池
#[derive(Clone)]
pub struct Database {
    pub pool: PgPool,
}

impl Database {
    /// 创建数据库连接
    pub async fn new() -> Result<Self, sqlx::Error> {
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://username:password@localhost/shirou_runner".to_string());
        
        let pool = PgPool::connect(&database_url).await?;
        
        // 创建表
        Self::create_tables(&pool).await?;
        
        Ok(Database { pool })
    }
    
    /// 创建数据库表
    async fn create_tables(pool: &PgPool) -> Result<(), sqlx::Error> {
        // 创建玩家表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS players (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                username VARCHAR(50) UNIQUE NOT NULL,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            )
            "#
        )
        .execute(pool)
        .await?;
        
        // 创建游戏会话表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS game_sessions (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                player_id UUID REFERENCES players(id),
                character_type VARCHAR(20) NOT NULL,
                start_time TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                end_time TIMESTAMP WITH TIME ZONE,
                distance_traveled REAL DEFAULT 0.0,
                jump_count INTEGER DEFAULT 0,
                play_time REAL DEFAULT 0.0,
                score INTEGER DEFAULT 0
            )
            "#
        )
        .execute(pool)
        .await?;
        
        // 创建玩家操作记录表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS player_actions (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                session_id UUID REFERENCES game_sessions(id),
                action_type VARCHAR(20) NOT NULL,
                action_data JSONB,
                timestamp TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                player_position_x REAL,
                player_position_y REAL
            )
            "#
        )
        .execute(pool)
        .await?;
        
        // 创建存档表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS save_games (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                player_id UUID REFERENCES players(id),
                save_name VARCHAR(100) NOT NULL,
                game_data JSONB NOT NULL,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            )
            "#
        )
        .execute(pool)
        .await?;
        
        println!("数据库表创建完成");
        Ok(())
    }
}