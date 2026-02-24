# 2026 引擎升级状态总览

> 最后更新：2026-02-24  
> 作用：作为“升级完成状态 + 指标快照 + 文档索引”的单一事实来源（SSOT）

## 1) 文档收口结果

为避免文档分裂，已将“已完成任务板、来源清单、指标报告”归并到本文件并移除冗余文件。

当前仅保留 3 份活文档：

- `docs/2026-upgrade-status.md`（本文件，状态 SSOT）
- `docs/bevy-upgrade-regression-checklist.md`（升级回归入口）
- `docs/ops-runbook.md`（运行与发布入口）

本次归并并删除：

- `docs/2026-architecture-upgrade-tasks.md`
- `docs/2026-best-practice-sources.md`
- `docs/2026-architecture-metrics-report.md`
- `docs/documentation-completeness-audit-2026-02-23.md`
- `docs/2026-bevy-upgrade-assessment-zh.md`
- `IMPLEMENTATION-SUMMARY.md`

## 2) 质量门禁状态（2026-02-24 实测）

以下命令在当前基线通过：

- `cargo fmt --check`
- `cargo check`
- `cargo check --all-features --future-incompat-report`
- `cargo clippy --all-features --all-targets -- -D warnings`
- `cargo test --lib --all-features`

结果：`119 passed, 0 failed`，且 `future-incompat` 警告为 `0`。

## 3) Jobs 完成矩阵

| Job                           | 状态 | 验收结论                                                   |
| ----------------------------- | ---- | ---------------------------------------------------------- |
| T-001 客户端预测 + 服务器校正 | 完成 | deadzone/smooth/snap 校正链路与指标已落地                  |
| T-002 断线恢复会话            | 完成 | `ResumeSession` + 身份映射恢复 + 生命周期顺序校验已落地    |
| T-003 输入协议分层            | 完成 | `InputState`（状态流）+ `InputEvent`（事件流）已替换旧链路 |
| T-004 快照差量同步            | 完成 | `WorldSnapshotDelta` 已接入，客户端支持更新/移除           |
| T-005 存档 SSOT               | 完成 | 仅 `SaveFileData v2`，旧 schema 直接拒绝                   |
| T-006 Redis 后台写队列        | 完成 | 主循环无直接 I/O，失败重试可配置且可观测                   |
| T-007 性能预算守护            | 完成 | 1080p 场景预算基线已给出（低/中/高）                       |
| T-008 在线生态最小闭环        | 完成 | 排行榜/回放/云存档闭环与跨设备验证通过                     |

## 4) 关键指标快照（2026-02-24）

指标由 `cargo run --release --bin architecture_metrics` 生成（工具：`src/bin/architecture_metrics.rs`）。

### 4.1 T-001 体验指标

- 首次纠偏延迟 p50：`30.82 ms`
- 首次纠偏延迟 p95：`36.46 ms`
- 快照抖动（stddev）：`3.28 ms`
- 纠偏触发率：`39.17%`
- Snap 触发率：`1.67%`

### 4.2 T-003 输入协议带宽

- 10s@60Hz 包数：`612 -> 112`（`-81.70%`）
- 10s@60Hz payload：`7248 -> 1744 bytes`（`-75.94%`）

### 4.3 T-004 快照同步带宽

- 10s@60Hz payload：`3388800 -> 435520 bytes`（`-87.15%`）

### 4.4 T-007 场景预算基线（Headless ECS）

- Low（1000 装饰）：`0.0306ms avg / 0.0344ms p95`
- Medium（5000 装饰）：`0.0353ms avg / 0.0420ms p95`
- High（10000 装饰）：`0.0432ms avg / 0.0468ms p95`

## 5) 2026 最佳实践来源（归并）

- Cargo `check` / `test` / future incompat report：
  - https://doc.rust-lang.org/cargo/commands/cargo-check.html
  - https://doc.rust-lang.org/cargo/commands/cargo-test.html
  - https://doc.rust-lang.org/cargo/reference/future-incompat-report.html
- Clippy：
  - https://doc.rust-lang.org/clippy/
- GitHub Actions：
  - https://docs.github.com/en/actions/writing-workflows/choosing-what-your-workflow-does/control-the-concurrency-of-workflows-and-jobs
  - https://docs.github.com/en/actions/using-workflows/events-that-trigger-workflows#workflow_dispatch
- Rust cache action：
  - https://github.com/Swatinem/rust-cache
- Bevy 迁移与 API：
  - https://bevy.org/learn/migration-guides/0-17-to-0-18/
  - https://docs.rs/bevy/latest/bevy/app/struct.FixedUpdate.html
  - https://docs.rs/bevy/latest/bevy/prelude/struct.App.html#method.add_plugins
- Tokio mpsc：
  - https://docs.rs/tokio/latest/tokio/sync/mpsc/
