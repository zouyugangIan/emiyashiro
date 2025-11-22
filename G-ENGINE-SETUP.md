# G-Engine 设置指南

## 概述

G-Engine 是一个基于 Bevy 0.17 构建的云原生游戏引擎，支持 WebGPU 渲染和实时多人联机。

## 架构组件

### 1. Redis (热数据缓存)
- 存储玩家实时位置和状态
- Key Schema: `player:{id}:pos` -> `x,y,vx,vy`

### 2. RabbitMQ (消息队列)
- 队列: `q_save_game` - 存档任务
- 队列: `q_ai_inference` - AI 推理请求

### 3. PostgreSQL (数据持久化)
- 玩家存档、分数、解锁物品

## 快速启动

### 1. 启动基础设施 (Docker)

```bash
docker-compose up -d
```

这将启动:
- Redis (端口 6379)
- RabbitMQ (端口 5672, 管理界面 15672)
- PostgreSQL (端口 5432)

### 2. 设置环境变量

```bash
# Redis
export REDIS_URL="redis://127.0.0.1:6379/"

# RabbitMQ
export RABBITMQ_URL="amqp://guest:guest@127.0.0.1:5672/%2f"

# PostgreSQL
export DATABASE_URL="postgresql://username:password@localhost/shirou_runner"
```

### 3. 编译和运行

#### 服务端 (Native)

```bash
cargo build --bin server --features server
cargo run --bin server --features server
```

服务端功能:
- WebSocket 服务器 (端口 8080)
- 游戏逻辑权威校验
- Redis 状态同步 (每帧)
- RabbitMQ 存档消费者
- AI Bot 控制

#### 客户端 (Native)

```bash
cargo build --bin client --features client
cargo run --bin client --features client
```

客户端功能:
- WebGPU 渲染
- WebSocket 连接到服务器
- 客户端预测
- 插值渲染

#### 客户端 (WASM)

```bash
# 安装 wasm 工具
cargo install wasm-bindgen-cli
rustup target add wasm32-unknown-unknown

# 编译
cargo build --bin client --target wasm32-unknown-unknown --features client

# 运行 (需要 HTTP 服务器)
# 使用 basic-http-server 或其他工具
```

## 功能特性

### Phase 3: Infrastructure Integration ✅
- ✅ Redis 集成 - 每帧同步 ECS Transform
- ✅ RabbitMQ & Postgres - 异步存档消费者

### Phase 4: Gameplay Adaptation ✅
- ✅ 输入重构 - KeyboardInput → PlayerAction
- ✅ 状态同步 - WorldSnapshot + 插值渲染

### Phase 5: AI Preparation ✅
- ✅ Controller trait - 输入源抽象
- ✅ BotController - 自动巡逻

## 网络协议

### Client → Server
```rust
PlayerAction::Move { x: f32, y: f32 }
PlayerAction::Jump
PlayerAction::Ping(u64)
```

### Server → Client
```rust
GamePacket::Welcome { id: u64, message: String }
GamePacket::WorldSnapshot { tick: u64, players: Vec<PlayerState> }
GamePacket::Pong(u64)
```

## 测试

### 测试多人联机
1. 启动服务端
2. 启动多个客户端实例
3. 观察玩家同步和插值效果

### 测试 AI Bot
服务端会自动生成一个 Bot (ID: 9999)，在 x=100 附近巡逻

### 测试 Redis 同步
```bash
# 连接到 Redis
redis-cli

# 查看玩家位置
GET player:1:pos
GET player:9999:pos
```

### 测试 RabbitMQ
访问管理界面: http://localhost:15672
- 用户名: guest
- 密码: guest

查看 `q_save_game` 队列的消息

## 故障排除

### Redis 连接失败
```bash
# 检查 Redis 是否运行
docker ps | grep redis

# 测试连接
redis-cli ping
```

### RabbitMQ 连接失败
```bash
# 检查 RabbitMQ 是否运行
docker ps | grep rabbitmq

# 查看日志
docker logs <rabbitmq-container-id>
```

### PostgreSQL 连接失败
```bash
# 检查 PostgreSQL 是否运行
docker ps | grep postgres

# 测试连接
psql -h localhost -U username -d shirou_runner
```

## 性能优化

### 服务端
- 调整 Redis 同步频率 (当前: 每帧)
- 调整 WorldSnapshot 广播频率 (当前: 每帧)
- 配置 RabbitMQ 预取数量

### 客户端
- 调整插值持续时间 (当前: 100ms)
- 启用客户端预测
- 优化渲染管线

## 下一步

- [ ] 实现客户端预测和服务器校正
- [ ] 添加更多 AI 行为模式
- [ ] 实现地图状态同步
- [ ] 添加聊天系统
- [ ] 实现观战模式
