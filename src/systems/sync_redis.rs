use bevy::prelude::*;
use crate::database::redis::RedisManager;
use crate::components::network::NetworkId;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct RedisTransform {
    x: f32,
    y: f32,
    z: f32,
}

pub fn sync_transform_to_redis(
    redis_manager: Res<RedisManager>,
    query: Query<(&NetworkId, &Transform)>,
) {
    for (net_id, transform) in query.iter() {
        let redis_transform = RedisTransform {
            x: transform.translation.x,
            y: transform.translation.y,
            z: transform.translation.z,
        };

        if let Ok(json) = serde_json::to_string(&redis_transform) {
            let key = format!("player:{}:transform", net_id.0);
            // We ignore errors here to avoid spamming logs if redis is down for a moment
            let _ = redis_manager.set_key(&key, &json);
        }
    }
}
