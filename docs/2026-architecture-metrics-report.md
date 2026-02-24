# 2026 Architecture Metrics Report

> Generated at: 2026-02-24 16:37:48 UTC
> Runtime: linux / x86_64 / profile=release
> Command: `cargo run --release --bin architecture_metrics > docs/2026-architecture-metrics-report.md`

## T-001 Client Prediction + Server Reconciliation

| Metric | Value |
| --- | ---: |
| First correction latency p50 | 30.82 ms |
| First correction latency p95 | 36.46 ms |
| Snapshot jitter (stddev) | 3.28 ms |
| Correction frequency | 39.17% |
| Snap correction frequency | 1.67% |

## T-003 Input Protocol (State Stream + Event Stream)

| Metric | Legacy | 2026 Protocol | Improvement |
| --- | ---: | ---: | ---: |
| Packets over 10s @60Hz | 612 | 112 | 81.70% |
| Payload bytes over 10s @60Hz | 7248 | 1744 | 75.94% |

## T-004 Snapshot Delta

| Metric | Legacy (full each tick) | 2026 delta/full mix | Improvement |
| --- | ---: | ---: | ---: |
| Packets over 10s @60Hz | 600 | 600 | 0.00% |
| Payload bytes over 10s @60Hz | 3388800 | 435520 | 87.15% |

## T-007 1080p Scene Budget Baseline (Headless ECS)

| Profile | Decoration entities | Avg frame | p95 frame | Estimated FPS |
| --- | ---: | ---: | ---: | ---: |
| Low | 1000 | 0.0288 ms | 0.0311 ms | 34740.9 |
| Medium | 5000 | 0.0383 ms | 0.0405 ms | 26116.1 |
| High | 10000 | 0.0398 ms | 0.0447 ms | 25107.5 |

## Notes

- This report is generated from deterministic synthetic workloads in repository code.
- Delta snapshot sizing uses 128 actors with 12 changed actors per tick.
- Scene baseline is headless ECS CPU cost, suitable for trend guardrails in CI.
