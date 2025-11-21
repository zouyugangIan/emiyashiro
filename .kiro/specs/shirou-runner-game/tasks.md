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
- [ ] **3.1 Redis 集成**
    - [ ] 实现 `RedisPlugin`
    - [ ] 每一帧同步 ECS `Transform` 到 Redis
- [ ] **3.2 RabbitMQ & Postgres**
    - [ ] 搭建 Docker 环境 (docker-compose.yml)
    - [ ] 实现异步存档消费者 (Save Worker)

## Phase 4: Gameplay Adaptation
- [ ] **4.1 输入重构**
    - [ ] 将 `KeyboardInput` 转换为 `PlayerAction`
    - [ ] 客户端发送 `PlayerAction`
    - [ ] 服务端接收并应用物理力
- [ ] **4.2 状态同步**
    - [ ] 服务端下发 `WorldSnapshot`
    - [ ] 客户端实现插值渲染 (Interpolation)

## Phase 5: AI Preparation
- [ ] **5.1 AI 接口**
    - [ ] 定义 `Controller` trait
    - [ ] 实现简单的 `BotController` (自动巡逻)