# G-Engine 设置指南

> 文档状态：2026-02-23 已按当前代码对齐。  
> 作用：说明联机与基础设施能力的“已实现范围 + 启动方式 + 验证方法”。

## 概述

G-Engine 基于 `Bevy 0.18`，当前提供：

- Native 客户端 + Native 权威服务端
- WebSocket 实时同步（`PlayerAction` / `WorldSnapshot`）
- Redis 热数据同步
- RabbitMQ 存档任务队列 + PostgreSQL 持久化
- 服务端 Bot（ID: `9999`）

## 架构组件

### Redis（热数据缓存）

- 用途：缓存玩家位置/速度
- Key Schema: `player:{id}:pos` -> `x,y,vx,vy`
- 代码位置：`src/systems/sync_redis.rs`
- 当前节流：约每 `100ms` 一次批量写入（非每帧）

### RabbitMQ（消息队列）

- 已实现队列：`q_save_game`（存档任务）
- 代码位置：`src/systems/save_worker.rs`
- 当前未实现：`q_ai_inference`（保留为未来扩展）

### PostgreSQL（数据持久化）

- 用途：存档与游戏数据
- 代码位置：`src/database/mod.rs`、`src/systems/save_worker.rs`

## 快速启动

### 1. 启动基础设施（Docker）

```bash
docker-compose up -d
```

将启动：

- Redis（`6379`）
- RabbitMQ（`5672`，管理界面 `15672`）
- PostgreSQL（`5432`）

### 2. 设置环境变量

```bash
export REDIS_URL="redis://127.0.0.1:6379/"
export RABBITMQ_URL="amqp://guest:guest@127.0.0.1:5672/%2f"
export DATABASE_URL="postgresql://username:password@localhost/shirou_runner"
```

### 3. 启动服务端

```bash
cargo run --bin server --features server
```

服务端当前行为：

- WebSocket 监听 `127.0.0.1:8080`
- 60Hz 主循环广播 `WorldSnapshot`
- Redis 节流批量同步
- Save Worker 消费 `q_save_game`
- 启动时生成 1 个 Bot（`NetworkId = 9999`）

### 4. 启动客户端（Native）

```bash
cargo run --bin client --features client
```

客户端当前行为：

- WebGPU 渲染
- 连接 `ws://127.0.0.1:8080`
- 接收 `WorldSnapshot` 并执行插值渲染（100ms）

### 5. 客户端（WASM）构建

```bash
cargo install wasm-bindgen-cli
rustup target add wasm32-unknown-unknown
cargo build --bin client --target wasm32-unknown-unknown --features client
```

说明：WASM 路径可编译目标已配置，运行时链路仍需按部署环境单独验证。

## 功能状态（2026-02-23）

### 已完成

- Redis 状态同步（节流 + 批量队列）
- RabbitMQ/Postgres 存档链路（`q_save_game`）
- 基础输入上报与快照同步
- Bot 控制基础行为

### 计划中

- 客户端预测与服务器校正
- 断线重连与会话恢复
- 差量快照/压缩同步
- AI 推理任务队列（`q_ai_inference`）

## 网络协议

### Client -> Server

```rust
PlayerAction::Move { x: f32, y: f32 }
PlayerAction::Jump
PlayerAction::Attack
PlayerAction::Ping(u64)
```

### Server -> Client

```rust
GamePacket::Welcome { id: u64, message: String }
GamePacket::WorldSnapshot { tick: u64, players: Vec<PlayerState> }
GamePacket::Pong(u64)
```

## 验证清单

### 联机基本验证

1. 启动服务端。
2. 启动两个客户端。
3. 确认能收到 `Welcome` 与 `WorldSnapshot`。
4. 观察远端实体插值是否平滑。

### Redis 验证

```bash
redis-cli
GET player:1:pos
GET player:9999:pos
```

### RabbitMQ 验证

- 打开 `http://localhost:15672`
- 用户名：`guest`
- 密码：`guest`
- 检查 `q_save_game` 队列

## 故障排除

### Redis 连接失败

```bash
docker ps | rg redis
redis-cli ping
```

### RabbitMQ 连接失败

```bash
docker ps | rg rabbitmq
docker logs <rabbitmq-container-id>
```

### PostgreSQL 连接失败

```bash
docker ps | rg postgres
psql -h localhost -U username -d shirou_runner
```

## 性能参数（当前默认）

- 服务端更新：`60 Hz`
- WorldSnapshot 广播：`60 Hz`
- Redis 同步节流：`100ms`（约 `10 Hz`）
- 客户端插值窗口：`100ms`
