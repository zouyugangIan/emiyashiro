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
) {
    let Some(redis) = redis else {
        return; // Redis not available, skip
    };

    for (transform, net_id, velocity) in query.iter() {
        let pos = transform.translation;
        let vel = velocity.map(|v| (v.x, v.y)).unwrap_or((0.0, 0.0));

        let key = format!("player:{}:pos", net_id.0);
        let value = format!("{},{},{},{}", pos.x, pos.y, vel.0, vel.1);

        if let Err(e) = redis.set_key(&key, &value) {
            eprintln!("Failed to sync player {} to Redis: {}", net_id.0, e);
        }
    }
}
