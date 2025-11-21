use redis::{Client, Commands, Connection, RedisError};
use std::env;
use bevy::prelude::*;

#[derive(Resource)]
pub struct RedisManager {
    client: Client,
}

impl RedisManager {
    pub fn new() -> Result<Self, RedisError> {
        let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379/".to_string());
        let client = Client::open(redis_url)?;
        Ok(Self { client })
    }

    pub fn get_connection(&self) -> Result<Connection, RedisError> {
        self.client.get_connection()
    }

    pub fn set_key(&self, key: &str, value: &str) -> Result<(), RedisError> {
        let mut con = self.get_connection()?;
        con.set(key, value)
    }

    pub fn get_key(&self, key: &str) -> Result<String, RedisError> {
        let mut con = self.get_connection()?;
        con.get(key)
    }
}

pub struct RedisPlugin;

impl Plugin for RedisPlugin {
    fn build(&self, app: &mut App) {
        match RedisManager::new() {
            Ok(manager) => {
                app.insert_resource(manager);
                println!("Redis connected successfully");
            }
            Err(e) => {
                eprintln!("Failed to connect to Redis: {}", e);
            }
        }
    }
}
