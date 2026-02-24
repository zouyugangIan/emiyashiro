use crate::protocol::InputEventKind;
use crate::resources::SaveFileData;
use bevy::prelude::*;
use std::cmp::Ordering;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LeaderboardEntry {
    pub player_id: String,
    pub score: u32,
    pub distance: f32,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Resource, Default)]
pub struct LeaderboardStore {
    entries: HashMap<String, LeaderboardEntry>,
}

impl LeaderboardStore {
    pub fn submit_score(&mut self, player_id: String, score: u32, distance: f32) {
        let now = chrono::Utc::now();
        let should_update = self
            .entries
            .get(&player_id)
            .map(|entry| score >= entry.score || distance >= entry.distance)
            .unwrap_or(true);

        if should_update {
            self.entries.insert(
                player_id.clone(),
                LeaderboardEntry {
                    player_id,
                    score,
                    distance,
                    updated_at: now,
                },
            );
        }
    }

    pub fn top(&self, limit: usize) -> Vec<LeaderboardEntry> {
        let mut entries = self.entries.values().cloned().collect::<Vec<_>>();
        entries.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| {
                    b.distance
                        .partial_cmp(&a.distance)
                        .unwrap_or(Ordering::Equal)
                })
                .then_with(|| a.updated_at.cmp(&b.updated_at))
        });
        entries.truncate(limit);
        entries
    }
}

#[derive(Debug, Clone)]
pub struct ReplayFrame {
    pub tick: u64,
    pub position: Vec3,
    pub velocity: Vec3,
    pub input_event: Option<InputEventKind>,
}

#[derive(Debug, Clone)]
pub struct ReplayData {
    pub replay_id: String,
    pub player_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub frames: Vec<ReplayFrame>,
}

#[derive(Resource, Default)]
pub struct ReplayStore {
    replays: HashMap<String, ReplayData>,
}

impl ReplayStore {
    pub fn save_replay(&mut self, replay: ReplayData) {
        self.replays.insert(replay.replay_id.clone(), replay);
    }

    pub fn get_replay(&self, replay_id: &str) -> Option<&ReplayData> {
        self.replays.get(replay_id)
    }
}

#[derive(Resource, Default)]
pub struct CloudSaveStore {
    saves: HashMap<(String, String), SaveFileData>,
}

impl CloudSaveStore {
    pub fn upload(&mut self, player_id: &str, slot: &str, save_data: SaveFileData) {
        self.saves
            .insert((player_id.to_string(), slot.to_string()), save_data);
    }

    pub fn download(&self, player_id: &str, slot: &str) -> Option<SaveFileData> {
        self.saves
            .get(&(player_id.to_string(), slot.to_string()))
            .cloned()
    }
}

#[derive(Debug, Clone)]
pub struct PublishRunRequest {
    pub player_id: String,
    pub slot: String,
    pub score: u32,
    pub distance: f32,
    pub save_data: SaveFileData,
    pub replay_frames: Vec<ReplayFrame>,
}

pub fn publish_online_run_result(
    leaderboard: &mut LeaderboardStore,
    replay_store: &mut ReplayStore,
    cloud_store: &mut CloudSaveStore,
    request: PublishRunRequest,
) -> String {
    let PublishRunRequest {
        player_id,
        slot,
        score,
        distance,
        save_data,
        replay_frames,
    } = request;

    leaderboard.submit_score(player_id.clone(), score, distance);

    let replay_id = format!(
        "replay-{}-{}",
        player_id,
        chrono::Utc::now().timestamp_millis()
    );
    replay_store.save_replay(ReplayData {
        replay_id: replay_id.clone(),
        player_id: player_id.clone(),
        created_at: chrono::Utc::now(),
        frames: replay_frames,
    });

    cloud_store.upload(&player_id, &slot, save_data);
    replay_id
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_save_data() -> SaveFileData {
        let metadata = crate::resources::SaveFileMetadata {
            name: "cloud-slot".to_string(),
            score: 100,
            distance: 321.0,
            play_time: 12.0,
            save_timestamp: chrono::Utc::now(),
            file_path: "unused.json".to_string(),
            selected_character: crate::states::CharacterType::Shirou1,
        };
        SaveFileData::new(metadata, crate::resources::CompleteGameState::default())
    }

    #[test]
    fn leaderboard_orders_by_score_then_distance() {
        let mut store = LeaderboardStore::default();
        store.submit_score("p1".to_string(), 100, 300.0);
        store.submit_score("p2".to_string(), 120, 200.0);
        store.submit_score("p3".to_string(), 120, 260.0);

        let top = store.top(3);
        assert_eq!(top[0].player_id, "p3");
        assert_eq!(top[1].player_id, "p2");
        assert_eq!(top[2].player_id, "p1");
    }

    #[test]
    fn cloud_save_supports_cross_device_download() {
        let mut cloud = CloudSaveStore::default();
        let save_data = make_save_data();
        let checksum = save_data.checksum.clone();

        cloud.upload("player-1", "slot-a", save_data);
        let downloaded = cloud
            .download("player-1", "slot-a")
            .expect("save should be available on another device");

        assert_eq!(downloaded.checksum, checksum);
        assert!(downloaded.verify_checksum());
    }

    #[test]
    fn online_ecosystem_minimal_loop_runs_end_to_end() {
        let mut leaderboard = LeaderboardStore::default();
        let mut replay_store = ReplayStore::default();
        let mut cloud_store = CloudSaveStore::default();
        let save_data = make_save_data();
        let replay_frames = vec![
            ReplayFrame {
                tick: 1,
                position: Vec3::new(1.0, 0.0, 0.0),
                velocity: Vec3::new(10.0, 0.0, 0.0),
                input_event: None,
            },
            ReplayFrame {
                tick: 2,
                position: Vec3::new(2.0, 0.0, 0.0),
                velocity: Vec3::new(10.0, 2.0, 0.0),
                input_event: Some(InputEventKind::Jump),
            },
        ];

        let replay_id = publish_online_run_result(
            &mut leaderboard,
            &mut replay_store,
            &mut cloud_store,
            PublishRunRequest {
                player_id: "player-42".to_string(),
                slot: "slot-1".to_string(),
                score: 999,
                distance: 456.0,
                save_data: save_data.clone(),
                replay_frames: replay_frames.clone(),
            },
        );

        let top = leaderboard.top(1);
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].player_id, "player-42");
        assert_eq!(top[0].score, 999);

        let replay = replay_store
            .get_replay(&replay_id)
            .expect("replay must be stored");
        assert_eq!(replay.player_id, "player-42");
        assert_eq!(replay.frames.len(), replay_frames.len());

        let downloaded = cloud_store
            .download("player-42", "slot-1")
            .expect("cloud save must be downloadable");
        assert_eq!(downloaded.checksum, save_data.checksum);
    }
}
