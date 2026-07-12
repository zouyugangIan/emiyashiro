use futures_lite::StreamExt;
use lapin::{Connection, ConnectionProperties, options::*, types::FieldTable};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::{env, error::Error, time::Duration};
use uuid::Uuid;

const SAVE_QUEUE: &str = "q_save_game";
const DEFAULT_RABBITMQ_URL: &str = "amqp://guest:guest@127.0.0.1:5672/%2f";

type WorkerResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

/// 存档任务消息。
#[derive(Debug, Serialize, Deserialize)]
pub struct SaveGameTask {
    pub player_id: Uuid,
    pub save_name: String,
    pub game_data: serde_json::Value,
}

/// 运行带自动重连的 RabbitMQ 存档消费者。
pub async fn run_save_worker(pool: PgPool) {
    let rabbitmq_url = env::var("RABBITMQ_URL").unwrap_or_else(|_| DEFAULT_RABBITMQ_URL.to_owned());

    loop {
        if let Err(error) = consume_save_queue(&pool, &rabbitmq_url).await {
            bevy::log::error!("Save worker stopped: {error}; retrying in 5 seconds");
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

async fn consume_save_queue(pool: &PgPool, rabbitmq_url: &str) -> WorkerResult<()> {
    let connection = Connection::connect(rabbitmq_url, ConnectionProperties::default()).await?;
    let channel = connection.create_channel().await?;
    channel
        .queue_declare(
            SAVE_QUEUE.into(),
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;
    channel.basic_qos(1, BasicQosOptions::default()).await?;

    let mut consumer = channel
        .basic_consume(
            SAVE_QUEUE.into(),
            "".into(),
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    bevy::log::info!("Save worker is consuming queue {SAVE_QUEUE}");

    while let Some(delivery) = consumer.next().await {
        let delivery = delivery?;
        let task = match serde_json::from_slice::<SaveGameTask>(&delivery.data) {
            Ok(task) => task,
            Err(error) => {
                bevy::log::warn!("Discarding malformed save task: {error}");
                delivery.ack(BasicAckOptions::default()).await?;
                continue;
            }
        };

        let save_result = sqlx::query(
            r#"
            INSERT INTO save_games (player_id, save_name, game_data)
            VALUES ($1, $2, $3)
            ON CONFLICT (player_id, save_name)
            DO UPDATE SET game_data = EXCLUDED.game_data, updated_at = NOW()
            "#,
        )
        .bind(task.player_id)
        .bind(&task.save_name)
        .bind(&task.game_data)
        .execute(pool)
        .await;

        match save_result {
            Ok(_) => {
                delivery.ack(BasicAckOptions::default()).await?;
            }
            Err(error) => {
                let requeue = should_requeue_database_error(&error);
                bevy::log::error!(
                    "Failed to persist save task for player {}; requeue={requeue}: {error}",
                    task.player_id,
                );
                delivery
                    .nack(BasicNackOptions {
                        requeue,
                        ..Default::default()
                    })
                    .await?;
            }
        }
    }

    Err("RabbitMQ save consumer ended unexpectedly".into())
}

fn should_requeue_database_error(error: &sqlx::Error) -> bool {
    match error {
        sqlx::Error::Io(_)
        | sqlx::Error::Tls(_)
        | sqlx::Error::PoolTimedOut
        | sqlx::Error::PoolClosed
        | sqlx::Error::WorkerCrashed => true,
        sqlx::Error::Database(database_error) => database_error.code().is_some_and(|code| {
            // PostgreSQL SQLSTATE classes for connection, transaction rollback,
            // resource/lock pressure, operator intervention and system errors.
            ["08", "40", "53", "55", "57", "58"]
                .iter()
                .any(|prefix| code.starts_with(prefix))
        }),
        _ => false,
    }
}

/// 发布持久化存档任务并等待 broker 确认。
pub async fn publish_save_task(task: SaveGameTask) -> WorkerResult<()> {
    let rabbitmq_url = env::var("RABBITMQ_URL").unwrap_or_else(|_| DEFAULT_RABBITMQ_URL.to_owned());
    let connection = Connection::connect(&rabbitmq_url, ConnectionProperties::default()).await?;
    let channel = connection.create_channel().await?;

    channel
        .queue_declare(
            SAVE_QUEUE.into(),
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;

    let payload = serde_json::to_vec(&task)?;
    channel
        .basic_publish(
            "".into(),
            SAVE_QUEUE.into(),
            BasicPublishOptions::default(),
            &payload,
            lapin::BasicProperties::default().with_delivery_mode(2),
        )
        .await?
        .await?;

    Ok(())
}
