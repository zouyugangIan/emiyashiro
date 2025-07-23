use super::{Database, models::*};
use sqlx::Row;
use uuid::Uuid;
use chrono::Utc;

impl Database {
    /// 创建或获取玩家
    pub async fn get_or_create_player(&self, username: &str) -> Result<Player, sqlx::Error> {
        // 尝试获取现有玩家
        if let Ok(player) = self.get_player_by_username(username).await {
            return Ok(player);
        }
        
        // 创建新玩家
        let row = sqlx::query(
            "INSERT INTO players (username) VALUES ($1) RETURNING id, username, created_at, updated_at"
        )
        .bind(username)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(Player {
            id: row.get("id"),
            username: row.get("username"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }
    
    /// 根据用户名获取玩家
    pub async fn get_player_by_username(&self, username: &str) -> Result<Player, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, username, created_at, updated_at FROM players WHERE username = $1"
        )
        .bind(username)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(Player {
            id: row.get("id"),
            username: row.get("username"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }
    
    /// 创建游戏会话
    pub async fn create_game_session(&self, player_id: Uuid, character_type: &str) -> Result<GameSession, sqlx::Error> {
        let row = sqlx::query(
            r#"
            INSERT INTO game_sessions (player_id, character_type) 
            VALUES ($1, $2) 
            RETURNING id, player_id, character_type, start_time, end_time, distance_traveled, jump_count, play_time, score
            "#
        )
        .bind(player_id)
        .bind(character_type)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(GameSession {
            id: row.get("id"),
            player_id: row.get("player_id"),
            character_type: row.get("character_type"),
            start_time: row.get("start_time"),
            end_time: row.get("end_time"),
            distance_traveled: row.get("distance_traveled"),
            jump_count: row.get("jump_count"),
            play_time: row.get("play_time"),
            score: row.get("score"),
        })
    }
    
    /// 更新游戏会话
    pub async fn update_game_session(&self, session_id: Uuid, distance: f32, jumps: i32, play_time: f32, score: i32) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE game_sessions 
            SET distance_traveled = $2, jump_count = $3, play_time = $4, score = $5, end_time = NOW()
            WHERE id = $1
            "#
        )
        .bind(session_id)
        .bind(distance)
        .bind(jumps)
        .bind(play_time)
        .bind(score)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    /// 记录玩家操作
    pub async fn log_player_action(
        &self, 
        session_id: Uuid, 
        action_type: &str, 
        action_data: Option<serde_json::Value>,
        player_pos: Option<(f32, f32)>
    ) -> Result<(), sqlx::Error> {
        let (pos_x, pos_y) = player_pos.unwrap_or((0.0, 0.0));
        
        sqlx::query(
            r#"
            INSERT INTO player_actions (session_id, action_type, action_data, player_position_x, player_position_y)
            VALUES ($1, $2, $3, $4, $5)
            "#
        )
        .bind(session_id)
        .bind(action_type)
        .bind(action_data)
        .bind(pos_x)
        .bind(pos_y)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    /// 保存游戏存档
    pub async fn save_game(&self, player_id: Uuid, save_name: &str, game_data: &GameData) -> Result<Uuid, sqlx::Error> {
        let game_data_json = serde_json::to_value(game_data).unwrap();
        
        let row = sqlx::query(
            r#"
            INSERT INTO save_games (player_id, save_name, game_data)
            VALUES ($1, $2, $3)
            ON CONFLICT (player_id, save_name) 
            DO UPDATE SET game_data = $3, updated_at = NOW()
            RETURNING id
            "#
        )
        .bind(player_id)
        .bind(save_name)
        .bind(game_data_json)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(row.get("id"))
    }
    
    /// 加载游戏存档
    pub async fn load_game(&self, player_id: Uuid, save_name: &str) -> Result<GameData, sqlx::Error> {
        let row = sqlx::query(
            "SELECT game_data FROM save_games WHERE player_id = $1 AND save_name = $2"
        )
        .bind(player_id)
        .bind(save_name)
        .fetch_one(&self.pool)
        .await?;
        
        let game_data_json: serde_json::Value = row.get("game_data");
        let game_data: GameData = serde_json::from_value(game_data_json).unwrap();
        
        Ok(game_data)
    }
    
    /// 获取玩家的所有存档
    pub async fn get_player_saves(&self, player_id: Uuid) -> Result<Vec<SaveGame>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, player_id, save_name, game_data, created_at, updated_at FROM save_games WHERE player_id = $1 ORDER BY updated_at DESC"
        )
        .bind(player_id)
        .fetch_all(&self.pool)
        .await?;
        
        let saves = rows.into_iter().map(|row| SaveGame {
            id: row.get("id"),
            player_id: row.get("player_id"),
            save_name: row.get("save_name"),
            game_data: row.get("game_data"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }).collect();
        
        Ok(saves)
    }
}