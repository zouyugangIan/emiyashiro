# G-Engine 实现总结（Phase 3-5，归档）

> 文档类型：历史归档（用于回顾实现，不作为实时功能承诺）  
> 最后对齐日期：2026-02-23  
> 对齐方式：源码静态核对（`src/bin/server.rs`、`src/systems/*`、`src/plugins/*`）

## 归档范围

本文件总结 Phase 3-5 的落地结果：

- Phase 3: Infrastructure Integration
- Phase 4: Gameplay Adaptation
- Phase 5: AI Preparation

## 已落地能力（与当前代码一致）

### Phase 3: Infrastructure Integration

#### 1) Redis 集成

- 文件：`src/systems/sync_redis.rs`
- 系统：`sync_transform_to_redis`
- 行为：将 `Transform + Velocity` 写入 `player:{id}:pos`
- 频率：节流写入（约每 `100ms`，非每帧）
- 接线：`src/bin/server.rs` 的 `Update` 系统中启用

#### 2) RabbitMQ + PostgreSQL 存档链路

- 文件：`src/systems/save_worker.rs`
- 能力：
  - `run_save_worker` 消费 `q_save_game`
  - 解析 `SaveGameTask` 并写入 PostgreSQL
  - `publish_save_task` 发布存档消息
- 接线：`src/bin/server.rs` 启动时 `tokio::spawn` Save Worker
- 基础设施：`docker-compose.yml` 提供 Redis/RabbitMQ/PostgreSQL

### Phase 4: Gameplay Adaptation

#### 1) 输入上报链路

- 输入采样与发送位于：`src/systems/input.rs` 的 `update_game_input`
- 协议类型：`PlayerAction::{Move, Jump, Attack, Ping}`
- 客户端网络资源：`src/systems/network.rs` 的 `NetworkResource`
- 接线：通过客户端插件（`src/plugins/netcode.rs` + `src/plugins/gameplay.rs`）

#### 2) 状态同步与插值

- 服务端广播：`src/bin/server.rs` 中 `broadcast_snapshot_system`
- 客户端消费：`src/systems/network.rs` 中 `handle_network_events`
- 插值渲染：`src/systems/network.rs` 中 `interpolate_positions`
- 插值窗口：`100ms`

### Phase 5: AI Preparation

- 文件：
  - `src/systems/ai.rs`
  - `src/components/ai.rs`
- 能力：
  - `Controller` 抽象输入来源
  - `BotController` 巡逻/跳跃逻辑
  - `bot_control_system` 每帧驱动 Bot 输入
- 服务端默认 Bot：`NetworkId = 9999`

## 当前数据流（简化）

```text
Client Input -> WebSocket -> Server ECS -> WorldSnapshot -> Client Interpolation
                                       -> Redis (player:{id}:pos)
                                       -> RabbitMQ(q_save_game) -> Save Worker -> PostgreSQL
```

## 与旧版描述的修订点

- “`send_player_input` 位于 `network.rs`”已修订为：
  - 当前输入发送逻辑在 `src/systems/input.rs` 的 `update_game_input`。
- “Redis 每帧同步”已修订为：
  - 当前为节流批量同步（约每 `100ms`）。
- “`client.rs` 直接挂载所有网络系统”已修订为：
  - 现为插件化挂载（`EmiyaShiroClientPlugin`）。

## 验证状态说明

由于当前执行环境无法访问 `crates.io`，无法完成联网依赖拉取，因此以下命令未能在本次审计中复跑成功：

- `cargo check`
- `cargo check --all-features`
- `cargo test --lib`

建议在可联网 CI 或开发机上执行上述命令作为发布前门禁。

## 后续工作（与代码状态一致）

- [ ] 客户端预测与服务器校正
- [ ] 差量快照/压缩同步
- [ ] 断线重连与恢复
- [ ] 更完整的 AI 行为模式

## 归档结论

Phase 3-5 的“基础能力”已落地，但项目尚未达到“功能完全完成”的状态。该结论与 `docs/2026-bevy-upgrade-assessment-zh.md` 的总体方向一致。
