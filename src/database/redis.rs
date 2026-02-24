use bevy::prelude::*;
use redis::{Client, Commands, Connection, RedisError};
use std::env;
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};

#[derive(Resource)]
pub struct RedisManager {
    client: Client,
    write_tx: SyncSender<Vec<(String, String)>>,
}

impl RedisManager {
    pub fn new() -> Result<Self, RedisError> {
        let redis_url =
            env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379/".to_string());
        let client = Client::open(redis_url)?;

        let (write_tx, write_rx) = mpsc::sync_channel::<Vec<(String, String)>>(128);
        let worker_client = client.clone();
        std::thread::Builder::new()
            .name("redis-write-worker".to_string())
            .spawn(move || Self::run_write_worker(worker_client, write_rx))
            .map_err(|_| {
                RedisError::from((
                    redis::ErrorKind::IoError,
                    "Failed to spawn Redis write worker thread",
                ))
            })?;

        Ok(Self { client, write_tx })
    }

    fn run_write_worker(client: Client, write_rx: Receiver<Vec<(String, String)>>) {
        let mut connection = Self::connect(&client);
        while let Ok(batch) = write_rx.recv() {
            if batch.is_empty() {
                continue;
            }

            if connection.is_none() {
                connection = Self::connect(&client);
            }

            if let Some(conn) = connection.as_mut()
                && Self::write_batch(conn, &batch).is_ok()
            {
                continue;
            }

            connection = Self::connect(&client);
            if let Some(conn) = connection.as_mut() {
                let _ = Self::write_batch(conn, &batch);
            }
        }
    }

    fn connect(client: &Client) -> Option<Connection> {
        match client.get_connection() {
            Ok(connection) => Some(connection),
            Err(error) => {
                eprintln!("Redis worker failed to connect: {}", error);
                None
            }
        }
    }

    fn write_batch(
        connection: &mut Connection,
        entries: &[(String, String)],
    ) -> Result<(), RedisError> {
        let mut pipeline = redis::pipe();
        for (key, value) in entries {
            pipeline.cmd("SET").arg(key).arg(value).ignore();
        }
        pipeline.query::<()>(connection)
    }

    fn set_many_sync(&self, entries: &[(String, String)]) -> Result<(), RedisError> {
        if entries.is_empty() {
            return Ok(());
        }

        let mut connection = self.client.get_connection()?;
        Self::write_batch(&mut connection, entries)
    }

    pub fn set_key(&self, key: &str, value: &str) -> Result<(), RedisError> {
        self.set_many_sync(&[(key.to_string(), value.to_string())])
    }

    /// Queue batched writes to a background worker to avoid main-thread blocking.
    /// Falls back to sync write if queue is disconnected.
    pub fn set_many(&self, entries: &[(String, String)]) -> Result<(), RedisError> {
        if entries.is_empty() {
            return Ok(());
        }

        match self.write_tx.try_send(entries.to_vec()) {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(_)) => Err(RedisError::from((
                redis::ErrorKind::IoError,
                "Redis write queue is full",
            ))),
            Err(TrySendError::Disconnected(batch)) => self.set_many_sync(&batch),
        }
    }

    pub fn get_key(&self, key: &str) -> Result<String, RedisError> {
        let mut con = self.client.get_connection()?;
        con.get(key)
    }
}

pub struct RedisPlugin;

impl Plugin for RedisPlugin {
    fn build(&self, app: &mut App) {
        match RedisManager::new() {
            Ok(manager) => {
                app.insert_resource(manager);
                info!("Redis connected successfully");
            }
            Err(e) => {
                warn!("Failed to connect to Redis: {}", e);
            }
        }
    }
}
