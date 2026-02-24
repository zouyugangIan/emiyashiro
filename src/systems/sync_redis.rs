use crate::components::network::NetworkId;
use crate::database::redis::RedisManager;
use bevy::prelude::*;

/// 每一帧同步 ECS Transform 到 Redis
/// 按照设计文档中的 Redis Key Schema: `player:{id}:pos` -> `x,y,vx,vy`
pub fn sync_transform_to_redis(
    redis: Option<Res<RedisManager>>,
    query: Query<(
        &Transform,
        &NetworkId,
        Option<&crate::components::physics::Velocity>,
    )>,
    time: Res<Time>,
    mut flush_timer: Local<f32>,
    mut error_log_cooldown: Local<f32>,
    mut metrics_log_timer: Local<f32>,
) {
    let Some(redis) = redis else {
        return; // Redis not available, skip
    };

    *flush_timer += time.delta_secs();
    *error_log_cooldown = (*error_log_cooldown - time.delta_secs()).max(0.0);
    *metrics_log_timer += time.delta_secs();
    if *flush_timer < 0.1 {
        if *metrics_log_timer >= 30.0 {
            let metrics = redis.metrics_snapshot();
            info!(
                "Redis write queue metrics: queued={}, processed={}, dropped={}, failed={}, retries={}, pending={}",
                metrics.queued_batches,
                metrics.processed_batches,
                metrics.dropped_batches,
                metrics.failed_batches,
                metrics.retry_attempts,
                metrics.estimated_pending_batches
            );
            *metrics_log_timer = 0.0;
        }
        return;
    }
    *flush_timer = 0.0;

    let mut entries = Vec::new();
    for (transform, net_id, velocity) in query.iter() {
        let pos = transform.translation;
        let vel = velocity.map(|v| (v.x, v.y)).unwrap_or((0.0, 0.0));

        let key = format!("player:{}:pos", net_id.0);
        let value = format!("{},{},{},{}", pos.x, pos.y, vel.0, vel.1);
        entries.push((key, value));
    }

    if let Err(error) = redis.set_many(&entries)
        && *error_log_cooldown <= 0.0
    {
        warn!("Failed to batch sync player transforms to Redis: {}", error);
        *error_log_cooldown = 5.0;
    }

    if *metrics_log_timer >= 30.0 {
        let metrics = redis.metrics_snapshot();
        info!(
            "Redis write queue metrics: queued={}, processed={}, dropped={}, failed={}, retries={}, pending={}",
            metrics.queued_batches,
            metrics.processed_batches,
            metrics.dropped_batches,
            metrics.failed_batches,
            metrics.retry_attempts,
            metrics.estimated_pending_batches
        );
        *metrics_log_timer = 0.0;
    }
}
