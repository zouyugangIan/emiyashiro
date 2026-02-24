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
    /// Delta snapshot update (changed/removed entities only)
    WorldSnapshotDelta {
        tick: u64,
        changed_players: Vec<PlayerState>,
        removed_player_ids: Vec<u64>,
    },
    /// Broadcast a chat or system message
    Message(String),
    /// Pong response to Ping
    Pong(u64),
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum InputEventKind {
    Jump,
    Attack,
}

/// Player input sent from Client to Server
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum PlayerAction {
    /// Client heartbeat/ping
    Ping(u64),
    /// Attempt to resume previous network identity after reconnect
    ResumeSession { previous_id: u64 },
    /// Continuous state-stream input (throttled/delta sent by client)
    InputState { sequence: u32, x: f32, y: f32 },
    /// Instant event input (edge-triggered)
    InputEvent { sequence: u32, kind: InputEventKind },
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
