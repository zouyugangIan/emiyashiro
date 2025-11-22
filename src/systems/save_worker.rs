use sqlx::PgPool;
use lapin::{
    options::*, types::FieldTable, Connection, ConnectionProperties,
};
use serde::{Deserialize, Serialize};
use std::env;

/// 存档任务消息
#[derive(Debug, Serialize, Deserialize)]
pub struct SaveGameTask {
    pub player_id: String,
    pub save_name: String,
    pub game_data: serde_json::Value,
}

/// 异步存档消费者 (Save Worker)
/// 从 RabbitMQ 队列 `q_save_game` 消费存档任务，并写入 Postgres
pub async fn run_save_worker(pool: PgPool) {
    println!("Starting Save Worker...");

    let rabbitmq_url = env::var("RABBITMQ_URL")
        .unwrap_or_else(|_| "amqp://guest:guest@127.0.0.1:5672/%2f".to_string());

    loop {
        match Connection::connect(&rabbitmq_url, ConnectionProperties::default()).await {
            Ok(conn) => {
                println!("Save Worker connected to RabbitMQ");
                
                let channel = conn.create_channel().await.expect("Failed to create channel");
                
                // 声明队列
                let _queue = channel
                    .queue_declare(
                        "q_save_game",
                        QueueDeclareOptions::default(),
                        FieldTable::default(),
                    )
                    .await
                    .expect("Failed to declare queue");

                println!("Save Worker listening on queue: q_save_game");

                let mut consumer = channel
                    .basic_consume(
                        "q_save_game",
                        "save_worker",
                        BasicConsumeOptions::default(),
                        FieldTable::default(),
                    )
                    .await
                    .expect("Failed to create consumer");

                while let Some(delivery) = futures_lite::StreamExt::next(&mut consumer).await {
                    if let Ok(delivery) = delivery {
                        match serde_json::from_slice::<SaveGameTask>(&delivery.data) {
                            Ok(task) => {
                                println!("Processing save task for player: {}", task.player_id);
                                
                                // 保存到数据库
                                let result = sqlx::query(
                                    r#"
                                    INSERT INTO save_games (player_id, save_name, game_data, created_at, updated_at)
                                    VALUES ($1::uuid, $2, $3, NOW(), NOW())
                                    ON CONFLICT (player_id, save_name) 
                                    DO UPDATE SET game_data = $3, updated_at = NOW()
                                    "#
                                )
                                .bind(&task.player_id)
                                .bind(&task.save_name)
                                .bind(&task.game_data)
                                .execute(&pool)
                                .await;

                                match result {
                                    Ok(_) => {
                                        println!("Save task completed for player: {}", task.player_id);
                                        delivery
                                            .ack(BasicAckOptions::default())
                                            .await
                                            .expect("Failed to ack");
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to save to database: {}", e);
                                        // Nack and requeue
                                        delivery
                                            .nack(BasicNackOptions { requeue: true, ..Default::default() })
                                            .await
                                            .expect("Failed to nack");
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to deserialize save task: {}", e);
                                // Ack to remove invalid message
                                delivery
                                    .ack(BasicAckOptions::default())
                                    .await
                                    .expect("Failed to ack");
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to connect to RabbitMQ: {}. Retrying in 5s...", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    }
}

/// 发布存档任务到 RabbitMQ
pub async fn publish_save_task(task: SaveGameTask) -> Result<(), Box<dyn std::error::Error>> {
    let rabbitmq_url = env::var("RABBITMQ_URL")
        .unwrap_or_else(|_| "amqp://guest:guest@127.0.0.1:5672/%2f".to_string());

    let conn = Connection::connect(&rabbitmq_url, ConnectionProperties::default()).await?;
    let channel = conn.create_channel().await?;

    channel
        .queue_declare(
            "q_save_game",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    let payload = serde_json::to_vec(&task)?;

    channel
        .basic_publish(
            "",
            "q_save_game",
            BasicPublishOptions::default(),
            &payload,
            lapin::BasicProperties::default(),
        )
        .await?;

    println!("Published save task for player: {}", task.player_id);
    Ok(())
}
