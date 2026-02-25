use bevy::prelude::*;
use redis::{Client, Commands, Connection, RedisError};
use std::env;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::time::Duration;

#[derive(Clone, Debug)]
struct RedisWriteConfig {
    max_retries: u32,
    retry_backoff_ms: u64,
}

impl RedisWriteConfig {
    fn from_env() -> Self {
        let max_retries = env::var("REDIS_WRITE_RETRY_MAX")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(3);
        let retry_backoff_ms = env::var("REDIS_WRITE_RETRY_BACKOFF_MS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(20);
        Self {
            max_retries,
            retry_backoff_ms,
        }
    }
}

#[derive(Clone, Default)]
struct RedisWriteMetrics {
    queued_batches: Arc<AtomicU64>,
    processed_batches: Arc<AtomicU64>,
    dropped_batches: Arc<AtomicU64>,
    failed_batches: Arc<AtomicU64>,
    retry_attempts: Arc<AtomicU64>,
}

#[derive(Debug, Clone, Default)]
pub struct RedisWriteMetricsSnapshot {
    pub queued_batches: u64,
    pub processed_batches: u64,
    pub dropped_batches: u64,
    pub failed_batches: u64,
    pub retry_attempts: u64,
    pub estimated_pending_batches: u64,
}

#[derive(Resource)]
pub struct RedisManager {
    client: Client,
    write_tx: SyncSender<Vec<(String, String)>>,
    metrics: RedisWriteMetrics,
}

impl RedisManager {
    pub fn new() -> Result<Self, RedisError> {
        let redis_url =
            env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379/".to_string());
        let client = Client::open(redis_url)?;
        let config = RedisWriteConfig::from_env();
        let metrics = RedisWriteMetrics::default();

        let (write_tx, write_rx) = mpsc::sync_channel::<Vec<(String, String)>>(128);
        let worker_client = client.clone();
        let worker_metrics = metrics.clone();
        let worker_config = config.clone();
        std::thread::Builder::new()
            .name("redis-write-worker".to_string())
            .spawn(move || {
                Self::run_write_worker(worker_client, write_rx, worker_config, worker_metrics)
            })
            .map_err(|_| {
                RedisError::from((
                    redis::ErrorKind::Io,
                    "Failed to spawn Redis write worker thread",
                ))
            })?;

        Ok(Self {
            client,
            write_tx,
            metrics,
        })
    }

    fn run_write_worker(
        client: Client,
        write_rx: Receiver<Vec<(String, String)>>,
        config: RedisWriteConfig,
        metrics: RedisWriteMetrics,
    ) {
        let mut connection = Self::connect(&client);
        while let Ok(batch) = write_rx.recv() {
            if batch.is_empty() {
                continue;
            }

            let mut success = false;
            for attempt in 0..=config.max_retries {
                if connection.is_none() {
                    connection = Self::connect(&client);
                }

                if let Some(conn) = connection.as_mut() {
                    if Self::write_batch(conn, &batch).is_ok() {
                        success = true;
                        break;
                    }
                    connection = None;
                }

                if attempt < config.max_retries {
                    metrics.retry_attempts.fetch_add(1, Ordering::Relaxed);
                    std::thread::sleep(Duration::from_millis(config.retry_backoff_ms));
                }
            }

            if !success {
                metrics.failed_batches.fetch_add(1, Ordering::Relaxed);
            }

            metrics.processed_batches.fetch_add(1, Ordering::Relaxed);
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
    /// Returns an error if queue is full/disconnected; caller should rely on worker retries.
    pub fn set_many(&self, entries: &[(String, String)]) -> Result<(), RedisError> {
        if entries.is_empty() {
            return Ok(());
        }

        match self.write_tx.try_send(entries.to_vec()) {
            Ok(()) => {
                self.metrics.queued_batches.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
            Err(TrySendError::Full(_)) => {
                self.metrics.dropped_batches.fetch_add(1, Ordering::Relaxed);
                Err(RedisError::from((
                    redis::ErrorKind::Io,
                    "Redis write queue is full",
                )))
            }
            Err(TrySendError::Disconnected(_)) => {
                self.metrics.dropped_batches.fetch_add(1, Ordering::Relaxed);
                Err(RedisError::from((
                    redis::ErrorKind::Io,
                    "Redis write queue is disconnected",
                )))
            }
        }
    }

    pub fn get_key(&self, key: &str) -> Result<String, RedisError> {
        let mut con = self.client.get_connection()?;
        con.get(key)
    }

    pub fn metrics_snapshot(&self) -> RedisWriteMetricsSnapshot {
        let queued_batches = self.metrics.queued_batches.load(Ordering::Relaxed);
        let processed_batches = self.metrics.processed_batches.load(Ordering::Relaxed);
        let dropped_batches = self.metrics.dropped_batches.load(Ordering::Relaxed);
        let failed_batches = self.metrics.failed_batches.load(Ordering::Relaxed);
        let retry_attempts = self.metrics.retry_attempts.load(Ordering::Relaxed);
        let estimated_pending_batches = queued_batches.saturating_sub(processed_batches);

        RedisWriteMetricsSnapshot {
            queued_batches,
            processed_batches,
            dropped_batches,
            failed_batches,
            retry_attempts,
            estimated_pending_batches,
        }
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
