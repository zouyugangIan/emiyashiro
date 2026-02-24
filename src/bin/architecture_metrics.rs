use bevy::prelude::*;
use serde::Serialize;
use std::time::Instant;

use s_emiyashiro::protocol::{GamePacket, InputEventKind, PlayerAction, PlayerState};
use s_emiyashiro::systems::network::ClientPredictionConfig;
use s_emiyashiro::systems::scene_decoration::{
    DecorationLayer, SceneDecoration, dynamic_lighting, move_scene_decorations,
};

#[derive(Debug)]
struct InputBandwidthReport {
    legacy_packets: u64,
    modern_packets: u64,
    legacy_bytes: u64,
    modern_bytes: u64,
    reduction_pct: f64,
}

#[derive(Debug)]
struct SnapshotBandwidthReport {
    legacy_packets: u64,
    modern_packets: u64,
    legacy_bytes: u64,
    modern_bytes: u64,
    reduction_pct: f64,
}

#[derive(Debug)]
struct CorrectionReport {
    latency_p50_ms: f64,
    latency_p95_ms: f64,
    jitter_ms: f64,
    correction_frequency_pct: f64,
    snap_frequency_pct: f64,
}

#[derive(Debug)]
struct ScenePerfProfile {
    label: &'static str,
    entities: usize,
    avg_frame_ms: f64,
    p95_frame_ms: f64,
    estimated_fps: f64,
}

#[derive(Debug, Clone, Serialize)]
struct LegacyWorldSnapshot {
    tick: u64,
    players: Vec<PlayerState>,
}

#[derive(Debug, Clone, Serialize)]
enum LegacyPlayerAction {
    Move { x: f32, y: f32 },
    Jump,
    Attack,
}

fn serialize_len<T: Serialize>(value: &T) -> usize {
    bincode::serialize(value)
        .map(|bytes| bytes.len())
        .unwrap_or(0)
}

fn percentile(samples: &[f64], percentile: f64) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    let mut sorted = samples.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));
    let max_index = sorted.len() - 1;
    let rank = (percentile.clamp(0.0, 1.0) * max_index as f64).round() as usize;
    sorted[rank]
}

fn mean(samples: &[f64]) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    samples.iter().sum::<f64>() / samples.len() as f64
}

fn stddev(samples: &[f64]) -> f64 {
    if samples.len() < 2 {
        return 0.0;
    }
    let avg = mean(samples);
    let variance = samples
        .iter()
        .map(|value| {
            let delta = value - avg;
            delta * delta
        })
        .sum::<f64>()
        / samples.len() as f64;
    variance.sqrt()
}

fn movement_axis_for_tick(tick: u32) -> f32 {
    if tick < 180 {
        1.0
    } else if tick < 360 {
        0.0
    } else {
        -1.0
    }
}

fn simulate_input_bandwidth(total_ticks: u32) -> InputBandwidthReport {
    let mut legacy_packets = 0u64;
    let mut modern_packets = 0u64;
    let mut legacy_bytes = 0u64;
    let mut modern_bytes = 0u64;

    let mut sequence = 1u32;
    let mut last_sent_x = 0.0f32;
    let mut last_state_tick: i32 = i32::MIN / 2;
    let state_interval_ticks = 6; // 60Hz * 0.1s

    for tick in 0..total_ticks {
        let x = movement_axis_for_tick(tick);
        let y = 0.0;
        let jump_trigger = tick % 120 == 0;
        let attack_trigger = tick % 90 == 45;

        let legacy_move = LegacyPlayerAction::Move { x, y };
        legacy_bytes += serialize_len(&legacy_move) as u64;
        legacy_packets += 1;

        if jump_trigger {
            let legacy_jump = LegacyPlayerAction::Jump;
            legacy_bytes += serialize_len(&legacy_jump) as u64;
            legacy_packets += 1;
        }
        if attack_trigger {
            let legacy_attack = LegacyPlayerAction::Attack;
            legacy_bytes += serialize_len(&legacy_attack) as u64;
            legacy_packets += 1;
        }

        let state_changed = (x - last_sent_x).abs() > f32::EPSILON;
        let throttle_expired = (tick as i32 - last_state_tick) >= state_interval_ticks;
        if state_changed || throttle_expired {
            let modern_state = PlayerAction::InputState { sequence, x, y };
            modern_bytes += serialize_len(&modern_state) as u64;
            modern_packets += 1;
            sequence = sequence.wrapping_add(1);
            last_sent_x = x;
            last_state_tick = tick as i32;
        }

        if jump_trigger {
            let modern_event = PlayerAction::InputEvent {
                sequence,
                kind: InputEventKind::Jump,
            };
            modern_bytes += serialize_len(&modern_event) as u64;
            modern_packets += 1;
            sequence = sequence.wrapping_add(1);
        }
        if attack_trigger {
            let modern_event = PlayerAction::InputEvent {
                sequence,
                kind: InputEventKind::Attack,
            };
            modern_bytes += serialize_len(&modern_event) as u64;
            modern_packets += 1;
            sequence = sequence.wrapping_add(1);
        }
    }

    let reduction_pct = if legacy_bytes > 0 {
        ((legacy_bytes as f64 - modern_bytes as f64) / legacy_bytes as f64) * 100.0
    } else {
        0.0
    };

    InputBandwidthReport {
        legacy_packets,
        modern_packets,
        legacy_bytes,
        modern_bytes,
        reduction_pct,
    }
}

fn build_player_state(id: u64, x: f32) -> PlayerState {
    PlayerState {
        id,
        position: Vec3::new(x, 0.0, 0.0),
        velocity: Vec3::new(30.0, 0.0, 0.0),
        facing_right: true,
        animation_state: "Run".to_string(),
    }
}

fn simulate_snapshot_bandwidth(
    total_ticks: u64,
    total_players: usize,
    changed_players_per_tick: usize,
) -> SnapshotBandwidthReport {
    let mut players: Vec<PlayerState> = (0..total_players)
        .map(|index| build_player_state(index as u64 + 1, index as f32 * 2.0))
        .collect();

    let mut legacy_packets = 0u64;
    let mut modern_packets = 0u64;
    let mut legacy_bytes = 0u64;
    let mut modern_bytes = 0u64;

    for tick in 1..=total_ticks {
        for index in 0..changed_players_per_tick.min(players.len()) {
            let phase = (tick as f32 * 0.15) + index as f32 * 0.25;
            players[index].position.x += 0.75 + phase.sin() * 0.1;
            players[index].velocity.x = 20.0 + phase.cos() * 5.0;
            players[index].facing_right = players[index].velocity.x >= 0.0;
            players[index].animation_state = if players[index].velocity.x.abs() > 0.5 {
                "Run".to_string()
            } else {
                "Idle".to_string()
            };
        }

        let legacy_packet = LegacyWorldSnapshot {
            tick,
            players: players.clone(),
        };
        legacy_packets += 1;
        legacy_bytes += serialize_len(&legacy_packet) as u64;

        let send_full = tick == 1 || (tick - 1) % 30 == 0;
        if send_full {
            let modern_packet = GamePacket::WorldSnapshot {
                tick,
                players: players.clone(),
            };
            modern_packets += 1;
            modern_bytes += serialize_len(&modern_packet) as u64;
            continue;
        }

        let changed_players: Vec<PlayerState> = players
            .iter()
            .take(changed_players_per_tick.min(players.len()))
            .cloned()
            .collect();
        let modern_packet = GamePacket::WorldSnapshotDelta {
            tick,
            changed_players,
            removed_player_ids: Vec::new(),
        };
        modern_packets += 1;
        modern_bytes += serialize_len(&modern_packet) as u64;
    }

    let reduction_pct = if legacy_bytes > 0 {
        ((legacy_bytes as f64 - modern_bytes as f64) / legacy_bytes as f64) * 100.0
    } else {
        0.0
    };

    SnapshotBandwidthReport {
        legacy_packets,
        modern_packets,
        legacy_bytes,
        modern_bytes,
        reduction_pct,
    }
}

fn compute_correction_report(config: &ClientPredictionConfig) -> CorrectionReport {
    let mut latencies_ms = Vec::new();
    let mut error_samples = Vec::new();

    for index in 0..120 {
        let rtt_ms = 28.0 + ((index as f64 * 0.37).sin() + (index as f64 * 0.11).cos()) * 6.5;
        let rtt_ms = rtt_ms.max(12.0);
        latencies_ms.push(rtt_ms * 0.5 + 1000.0 / 60.0);

        let base_error = ((index as f32 * 0.13).sin().abs()) * 7.0;
        let burst_error = if index % 90 == 0 {
            180.0
        } else if index % 24 == 0 {
            32.0
        } else {
            0.0
        };
        error_samples.push(base_error + burst_error);
    }

    let mut correction_frames = 0usize;
    let mut snap_frames = 0usize;
    for error in &error_samples {
        if *error > config.correction_deadzone {
            correction_frames += 1;
        }
        if *error >= config.snap_threshold {
            snap_frames += 1;
        }
    }

    let total_frames = error_samples.len().max(1) as f64;
    CorrectionReport {
        latency_p50_ms: percentile(&latencies_ms, 0.5),
        latency_p95_ms: percentile(&latencies_ms, 0.95),
        jitter_ms: stddev(&latencies_ms),
        correction_frequency_pct: (correction_frames as f64 / total_frames) * 100.0,
        snap_frequency_pct: (snap_frames as f64 / total_frames) * 100.0,
    }
}

fn run_scene_profile(label: &'static str, entities: usize) -> ScenePerfProfile {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Update, (move_scene_decorations, dynamic_lighting));

    for index in 0..entities {
        let layer = match index % 3 {
            0 => DecorationLayer::FarBackground,
            1 => DecorationLayer::MidBackground,
            _ => DecorationLayer::NearBackground,
        };
        let x = (index % 240) as f32 * 8.0;
        let y = (index / 240) as f32 * 6.0;
        app.world_mut().spawn((
            Sprite {
                custom_size: Some(Vec2::new(24.0, 24.0)),
                color: Color::srgba(1.0, 1.0, 1.0, 0.7),
                ..default()
            },
            Transform::from_xyz(x, y, layer.z_index()),
            SceneDecoration {
                layer,
                speed_multiplier: layer.speed_multiplier(),
            },
        ));
    }

    for _ in 0..20 {
        app.update();
    }

    let updates_per_batch = 20usize;
    let mut samples_ms = Vec::with_capacity(80);
    for _ in 0..80 {
        let start = Instant::now();
        for _ in 0..updates_per_batch {
            app.update();
        }
        let elapsed_ms = (start.elapsed().as_secs_f64() * 1000.0) / updates_per_batch as f64;
        samples_ms.push(elapsed_ms);
    }

    let avg_frame_ms = mean(&samples_ms);
    let p95_frame_ms = percentile(&samples_ms, 0.95);
    let estimated_fps = if avg_frame_ms > 0.0 {
        1000.0 / avg_frame_ms
    } else {
        0.0
    };

    ScenePerfProfile {
        label,
        entities,
        avg_frame_ms,
        p95_frame_ms,
        estimated_fps,
    }
}

fn print_report(
    correction: &CorrectionReport,
    input_bandwidth: &InputBandwidthReport,
    snapshot_bandwidth: &SnapshotBandwidthReport,
    scene_profiles: &[ScenePerfProfile],
) {
    let profile = if cfg!(debug_assertions) {
        "dev"
    } else {
        "release"
    };

    println!("# 2026 Architecture Metrics Report");
    println!();
    println!(
        "> Generated at: {}",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    );
    println!(
        "> Runtime: {} / {} / profile={}",
        std::env::consts::OS,
        std::env::consts::ARCH,
        profile
    );
    println!(
        "> Command: `cargo run --release --bin architecture_metrics > docs/2026-architecture-metrics-report.md`"
    );
    println!();

    println!("## T-001 Client Prediction + Server Reconciliation");
    println!();
    println!("| Metric | Value |");
    println!("| --- | ---: |");
    println!(
        "| First correction latency p50 | {:.2} ms |",
        correction.latency_p50_ms
    );
    println!(
        "| First correction latency p95 | {:.2} ms |",
        correction.latency_p95_ms
    );
    println!(
        "| Snapshot jitter (stddev) | {:.2} ms |",
        correction.jitter_ms
    );
    println!(
        "| Correction frequency | {:.2}% |",
        correction.correction_frequency_pct
    );
    println!(
        "| Snap correction frequency | {:.2}% |",
        correction.snap_frequency_pct
    );
    println!();

    println!("## T-003 Input Protocol (State Stream + Event Stream)");
    println!();
    println!("| Metric | Legacy | 2026 Protocol | Improvement |");
    println!("| --- | ---: | ---: | ---: |");
    println!(
        "| Packets over 10s @60Hz | {} | {} | {:.2}% |",
        input_bandwidth.legacy_packets,
        input_bandwidth.modern_packets,
        ((input_bandwidth.legacy_packets as f64 - input_bandwidth.modern_packets as f64)
            / input_bandwidth.legacy_packets as f64)
            * 100.0
    );
    println!(
        "| Payload bytes over 10s @60Hz | {} | {} | {:.2}% |",
        input_bandwidth.legacy_bytes, input_bandwidth.modern_bytes, input_bandwidth.reduction_pct
    );
    println!();

    println!("## T-004 Snapshot Delta");
    println!();
    println!("| Metric | Legacy (full each tick) | 2026 delta/full mix | Improvement |");
    println!("| --- | ---: | ---: | ---: |");
    println!(
        "| Packets over 10s @60Hz | {} | {} | {:.2}% |",
        snapshot_bandwidth.legacy_packets,
        snapshot_bandwidth.modern_packets,
        ((snapshot_bandwidth.legacy_packets as f64 - snapshot_bandwidth.modern_packets as f64)
            / snapshot_bandwidth.legacy_packets as f64)
            * 100.0
    );
    println!(
        "| Payload bytes over 10s @60Hz | {} | {} | {:.2}% |",
        snapshot_bandwidth.legacy_bytes,
        snapshot_bandwidth.modern_bytes,
        snapshot_bandwidth.reduction_pct
    );
    println!();

    println!("## T-007 1080p Scene Budget Baseline (Headless ECS)");
    println!();
    println!("| Profile | Decoration entities | Avg frame | p95 frame | Estimated FPS |");
    println!("| --- | ---: | ---: | ---: | ---: |");
    for profile in scene_profiles {
        println!(
            "| {} | {} | {:.4} ms | {:.4} ms | {:.1} |",
            profile.label,
            profile.entities,
            profile.avg_frame_ms,
            profile.p95_frame_ms,
            profile.estimated_fps
        );
    }
    println!();
    println!("## Notes");
    println!();
    println!(
        "- This report is generated from deterministic synthetic workloads in repository code."
    );
    println!("- Delta snapshot sizing uses 128 actors with 12 changed actors per tick.");
    println!("- Scene baseline is headless ECS CPU cost, suitable for trend guardrails in CI.");
}

fn main() {
    let correction = compute_correction_report(&ClientPredictionConfig::default());
    let input_bandwidth = simulate_input_bandwidth(600);
    let snapshot_bandwidth = simulate_snapshot_bandwidth(600, 128, 12);
    let scene_profiles = vec![
        run_scene_profile("Low", 1000),
        run_scene_profile("Medium", 5000),
        run_scene_profile("High", 10000),
    ];

    print_report(
        &correction,
        &input_bandwidth,
        &snapshot_bandwidth,
        &scene_profiles,
    );
}
