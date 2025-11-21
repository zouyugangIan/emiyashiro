use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Network packet sent from Server to Client
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum GamePacket {
    /// Initial handshake or state sync
    Welcome { id: u64, message: String },
    /// World state update (snapshot)
    WorldSnapshot {
        tick: u64,
        players: Vec<PlayerState>,
    },
    /// Broadcast a chat or system message
    Message(String),
    /// Pong response to Ping
    Pong(u64),
}

/// Player input sent from Client to Server
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PlayerAction {
    /// Client heartbeat/ping
    Ping(u64),
    /// Movement input
    Move { x: f32, y: f32 },
    /// Jump action
    Jump,
    /// Attack action
    Attack,
}

/// Serializable player state for snapshots
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerState {
    pub id: u64,
    pub position: Vec3,
    pub velocity: Vec3,
    pub facing_right: bool,
    pub animation_state: String,
}
