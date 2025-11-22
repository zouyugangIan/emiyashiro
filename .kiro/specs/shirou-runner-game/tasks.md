# Implementation Tasks: G-Engine

## Phase 1: Project Restructuring (G-Engine Core)

- [x] **1.1 拆分项目结构** ✅ 已完成
  - [x] 创建 `bin/client.rs` (WebGPU 客户端)
  - [x] 创建 `bin/server.rs` (Native 服务端)
  - [x] 提取共享库 `lib.rs` (组件、协议)
- [x] **1.2 依赖管理** ✅ 已完成
  - [x] 配置 `Cargo.toml` features
  - [x] 将 `sqlx`, `tokio` (full) 移至 server-only
  - [x] 引入 `serde`, `bincode` 用于协议序列化

## Phase 2: Networking Layer (WebSocket)

- [x] **2.1 协议定义** ✅ 已完成
  - [x] 定义 `GamePacket` 枚举
  - [x] 定义 `PlayerAction` 输入抽象
- [x] **2.2 服务端网络实现** ✅ 已完成
  - [x] 集成 `tokio-tungstenite`
  - [x] 实现多客户端连接管理
  - [x] 实现广播机制 (Broadcast Loop)
- [x] **2.3 客户端网络实现** ✅ 已完成
  - [x] 集成 `gloo-net` (WASM) / `tungstenite` (Native)
  - [x] 实现自动重连机制

## Phase 3: Infrastructure Integration

- [x] **3.1 Redis 集成** ✅ 已完成
  - [x] 实现 `RedisPlugin`
  - [x] 每一帧同步 ECS `Transform` 到 Redis
- [x] **3.2 RabbitMQ & Postgres** ✅ 已完成
  - [x] 搭建 Docker 环境 (docker-compose.yml)
  - [x] 实现异步存档消费者 (Save Worker)

## Phase 4: Gameplay Adaptation

- [x] **4.1 输入重构** ✅ 已完成
  - [x] 将 `KeyboardInput` 转换为 `PlayerAction`
  - [x] 客户端发送 `PlayerAction`
  - [x] 服务端接收并应用物理力
- [x] **4.2 状态同步** ✅ 已完成
  - [x] 服务端下发 `WorldSnapshot`
  - [x] 客户端实现插值渲染 (Interpolation)

## Phase 5: AI Preparation

- [x] **5.1 AI 接口** ✅ 已完成
  - [x] 定义 `Controller` trait
  - [x] 实现简单的 `BotController` (自动巡逻)

## 总结

所有 Phase 3-5 的任务已全部完成！

### 新增功能

1. **Redis 集成** - 实时状态同步到 Redis
2. **RabbitMQ & Postgres** - 异步存档处理
3. **输入重构** - 客户端输入转换为网络协议
4. **状态同步** - 服务器广播 + 客户端插值
5. **AI 接口** - Bot 自动控制系统


### 文件清单

- `src/systems/sync_redis.rs` - Redis 同步系统
- `src/systems/save_worker.rs` - RabbitMQ 存档消费者
- `src/systems/ai.rs` - AI 控制系统
- `src/components/ai.rs` - BotController 组件
- `G-ENGINE-SETUP.md` - 设置指南
- `IMPLEMENTATION-SUMMARY.md` - 实现总结

### 运行指南

1. 启动基础设施: `docker-compose up -d`
2. 启动服务端: `cargo run --bin server --features server`
3. 启动客户端: `cargo run --bin client --features client`
