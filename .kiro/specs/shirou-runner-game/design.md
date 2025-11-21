# Design Document: G-Engine & Shirou Runner

## Overview

**G-Engine** 是一个基于 Bevy 0.17 构建的云原生游戏引擎，专为 **Shirou Runner** 打造。它采用 **Client-Server** 架构，支持 WebGPU 渲染和 WebSocket 实时通信。

## Architecture

### System Architecture

```mermaid
graph TD
    subgraph Client [Client (WebGPU/WASM)]
        Input[Input System] -->|Action Packet| WS_C[WebSocket Client]
        WS_C -->|World Snapshot| Render[Render System]
        Render -->|Draw| Screen
        Predict[Prediction System] --> Render
    end

    subgraph Server [Server (Native/Linux)]
        WS_S[WebSocket Server] -->|Action| Logic[Game Logic (ECS)]
        Logic -->|State Update| Redis[(Redis Cache)]
        Logic -->|Save Event| Rabbit[RabbitMQ]
        
        subgraph Workers
            SaveWorker[Save Worker]
            AIWorker[AI Worker]
        end
        
        Rabbit --> SaveWorker
        Rabbit --> AIWorker
        SaveWorker --> DB[(Postgres)]
    end
```

### Core Modules

1.  **ShirouClient**: 负责渲染、音频和用户输入。不包含任何游戏逻辑权威校验。
2.  **ShirouServer**: 权威服务器。运行物理模拟、验证输入、管理数据库连接。
3.  **ShirouShared**: 共享代码库。包含协议定义 (`Packet`)、基础组件 (`Velocity`, `Player`) 和资源路径。

## Data Models

### Network Protocol (WebSocket)

```rust
enum GamePacket {
    // Client -> Server
    ClientLogin { token: String },
    ClientInput { action: PlayerAction, timestamp: f64 },
    
    // Server -> Client
    ServerSnapshot { 
        tick: u64, 
        players: Vec<PlayerState> 
    },
}

enum PlayerAction {
    MoveLeft,
    MoveRight,
    Jump,
    Stop,
}
```

### Infrastructure Data

*   **Redis Key Schema**:
    *   `player:{id}:pos` -> `x,y,vx,vy` (Hash)
    *   `map:{id}:state` -> JSON (Map Entities)
*   **RabbitMQ Queues**:
    *   `q_save_game`: 存档任务
    *   `q_ai_inference`: AI 推理请求

## Components (ECS)

### Shared Components
*   `NetworkId(u64)`: 网络同步的唯一标识。
*   `Transform`, `Velocity`: 物理状态。

### Client-Only Components
*   `InterpolationBuffer`: 用于平滑远程玩家移动的缓冲区。
*   `LocalPlayer`: 标记当前客户端控制的角色。

### Server-Only Components
*   `PlayerConnection`: 保存 WebSocket 连接句柄。
*   `DbHandle`: 数据库连接引用。

## Technical Considerations

### WebGPU Compatibility
*   **限制**: 浏览器环境无法使用 `std::net::TcpStream` 或 `sqlx`。
*   **解决方案**: 
    *   网络层使用 `gloo-net` (WASM) 或 `tokio-tungstenite` (Native) 的条件编译封装。
    *   数据库层完全隔离在 Server 端。

### Latency Handling
*   **Client-Side Prediction**: 本地玩家立即响应按键，不等待服务器确认。
*   **Reconciliation**: 如果服务器返回的位置与本地预测误差超过阈值 (e.g. 10px)，强制修正本地位置。