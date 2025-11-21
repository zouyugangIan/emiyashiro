use lapin::{options::*, types::FieldTable, Connection, ConnectionProperties};
use sqlx::PgPool;
use std::env;
use futures_lite::stream::StreamExt;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SaveRequest {
    pub player_id: uuid::Uuid,
    pub save_name: String,
    pub game_data: serde_json::Value,
}

pub async fn run_save_worker(pool: PgPool) {
    let rabbitmq_addr = env::var("RABBITMQ_URL").unwrap_or_else(|_| "amqp://guest:guest@127.0.0.1:5672/%2f".into());
    
    println!("Connecting to RabbitMQ at {}", rabbitmq_addr);
    
    let conn = match Connection::connect(&rabbitmq_addr, ConnectionProperties::default()).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to connect to RabbitMQ: {}", e);
            return;
        }
    };

    let channel = match conn.create_channel().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to create RabbitMQ channel: {}", e);
            return;
        }
    };

    let _queue = channel
        .queue_declare(
            "save_game_queue",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("Failed to declare queue");

    let mut consumer = channel
        .basic_consume(
            "save_game_queue",
            "save_worker",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("Failed to create consumer");

    println!("Save Worker started, waiting for messages...");

    while let Some(delivery) = consumer.next().await {
        if let Ok(delivery) = delivery {
            let data = delivery.data.clone();
            if let Ok(save_req) = serde_json::from_slice::<SaveRequest>(&data) {
                println!("Processing save for player: {}", save_req.player_id);
                
                // Write to Postgres
                let result = sqlx::query(
                    r#"
                    INSERT INTO save_games (player_id, save_name, game_data)
                    VALUES ($1, $2, $3)
                    "#
                )
                .bind(save_req.player_id)
                .bind(&save_req.save_name)
                .bind(&save_req.game_data)
                .execute(&pool)
                .await;

                match result {
                    Ok(_) => {
                        println!("Save successful for {}", save_req.player_id);
                        let _ = delivery.ack(BasicAckOptions::default()).await;
                    }
                    Err(e) => {
                        eprintln!("Failed to save to DB: {}", e);
                        // Optionally nack or retry
                        let _ = delivery.nack(BasicNackOptions::default()).await;
                    }
                }
            } else {
                eprintln!("Failed to deserialize save request");
                let _ = delivery.nack(BasicNackOptions { requeue: false, ..Default::default() }).await;
            }
        }
    }
}
