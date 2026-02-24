# 2026 引擎升级状态总览

> 最后更新：2026-02-24  
> 作用：作为“文档完成状态 + 架构升级现状”的单一事实来源（SSOT）

## 1) 本次归并说明

本文件归并了以下已完成且阶段性文档的核心结论：

- `docs/documentation-completeness-audit-2026-02-23.md`
- `IMPLEMENTATION-SUMMARY.md`
- `docs/2026-bevy-upgrade-assessment-zh.md`

上述文档已完成使命，已从主文档集合中移除，避免状态分裂与重复维护。

## 2) 质量门禁状态（2026-02-24 实测）

以下命令均已在当前代码基线执行通过：

- `cargo fmt --check`
- `cargo check`
- `cargo check --all-features --future-incompat-report`
- `cargo clippy --lib --all-features -- -D warnings`
- `cargo clippy --all-features --all-targets -- -D warnings`
- `cargo test --lib --all-features`

测试结果：`109 passed, 0 failed`。

## 3) 架构升级完成项（本轮）

### 3.1 服务端运行时插件化

- 新增 `src/plugins/server.rs`，将服务端 ECS 运行时从 `src/bin/server.rs` 剥离。
- `src/bin/server.rs` 收敛为“网络接入 + App 启动”入口，职责更清晰。

### 3.2 服务端固定步长调度（Deterministic Tick）

- 服务端主逻辑迁入 `FixedUpdate`，并显式配置 `Time::<Fixed>::from_hz(60.0)`。
- 玩家物理与快照广播在固定步长上运行，降低帧间抖动与时序漂移。

### 3.3 客户端网络稳态增强

- 网络配置资源化：`NetworkConfig`（连接地址、重连间隔、心跳间隔）。
- 增加 `auto_reconnect_network`：断连后按冷却窗口自动重连。
- 增加 `send_heartbeat_ping_system`：稳定连接探活（周期 Ping）。

### 3.4 广播链路稳健性提升

- 服务端广播改为“每连接独立 writer task + 发送通道”模型。
- 避免旧实现中同包重复发送与 sink 写入路径耦合。

### 3.5 存档链路零 legacy 化

- 删除 `SaveData` 与 legacy 迁移/兼容代码路径。
- 存档读取严格限定为 `SaveFileData v2`，旧 schema 直接拒绝。
- 校验和策略统一为 `BLAKE3`，校验失败即硬失败（无兼容模式）。
- 文件解码统一为 `Plain JSON + Zstd`，移除 Gzip 兼容路径。

## 4) 与 2026 最佳实践对齐结论

- 插件化与职责分层：`已对齐`（客户端 + 服务端双入口收敛）。
- 固定步长核心逻辑：`已对齐`（服务端 60Hz 固定 tick）。
- 网络生命周期管理：`部分对齐`（已具备握手、状态机、重连、心跳；预测校正与会话恢复待完成）。
- 文档与门禁一致性：`已对齐`（门禁结果已刷新，避免“文档通过但代码退化”）。
- 存档治理（单路径 + 严格校验）：`已对齐`（v2 only，无 legacy 分支）。

## 5) 仍在推进的升级项

当前已进入 T-001 第一阶段实现：

- 客户端位置层已接入“本地预测 + 服务器误差校正”机制（deadzone / smooth / snap）。
- 下一步是补齐量化验证（端到端延迟、校正触发率、抖动曲线）。

剩余工作已统一迁移到：

- `docs/2026-architecture-upgrade-tasks.md`
- `docs/bevy-upgrade-regression-checklist.md`

执行策略：以回归清单为验收口，以 `TASKS` 为排期口，避免文档重复承诺。

## 6) Jobs 完成度（2026-02-24）

- `docs/bevy-upgrade-regression-checklist.md`：已完成（全部勾选）。
- `docs/ops-runbook.md` Release Readiness：已完成（按当前执行环境验收）。
- 存档系统作业：已完成（零 legacy、严格校验、测试覆盖）。
