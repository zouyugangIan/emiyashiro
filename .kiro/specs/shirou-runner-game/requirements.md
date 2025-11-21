# Requirements Document

## Introduction

Shirou Runner 是一个基于 **G-Engine** (基于 Bevy 深度定制的云原生游戏引擎) 开发的 2D 横版跑酷游戏。游戏支持 WebGPU 跨平台运行（浏览器/桌面），并具备实时多人联机功能。

## Requirements

### Requirement 1: 核心玩法 (Core Gameplay)

**User Story:** 作为玩家，我希望控制卫宫士郎进行跑酷，体验流畅的动作。

#### Acceptance Criteria
1.  **移动控制**: 支持 A/D (移动) 和 W/Space (跳跃)。
2.  **物理反馈**: 具备重力、碰撞检测和跳跃惯性。
3.  **摄像机**: 智能跟随角色，保持视野清晰。

### Requirement 2: 多人联机 (Multiplayer)

**User Story:** 作为玩家，我希望能看到其他玩家在同一个世界中奔跑。

#### Acceptance Criteria
1.  **实时同步**: 系统 SHALL 通过 WebSocket 以至少 20Hz 的频率同步其他玩家的位置。
2.  **加入/退出**: 玩家进入游戏时，系统 SHALL 自动连接服务器并同步当前世界状态。
3.  **平滑插值**: 当网络波动时，系统 SHALL 使用插值算法平滑其他玩家的移动轨迹。
4.  **延迟补偿**: 客户端 SHALL 实施预测算法，确保本地操作无延迟感。

### Requirement 3: 跨平台引擎 (G-Engine / WebGPU)

**User Story:** 作为玩家，我希望能在浏览器中直接打开游戏，无需下载安装包。

#### Acceptance Criteria
1.  **Web 运行**: 客户端 SHALL 能够编译为 WASM 并通过 WebGPU 在现代浏览器中运行。
2.  **资源加载**: 游戏资源 SHALL 支持通过 HTTP 远程加载。
3.  **输入适配**: 引擎 SHALL 自动适配桌面键盘和触摸屏输入（预留）。

### Requirement 4: 云端架构 (Cloud Infrastructure)

**User Story:** 作为开发者，我希望游戏后端具备高可用性和持久化能力。

#### Acceptance Criteria
1.  **热数据缓存**: 玩家实时位置和状态 SHALL 存储于 Redis 中，以支持快速读写。
2.  **数据持久化**: 玩家存档（分数、解锁物品） SHALL 异步写入 Postgres 数据库。
3.  **任务解耦**: 耗时的 I/O 操作（如保存存档、AI 计算） SHALL 通过 RabbitMQ 消息队列异步处理。

### Requirement 5: AI 扩展性 (AI Extensibility)

**User Story:** 作为开发者，我希望预留 AI 接口，以便未来接入智能代理。

#### Acceptance Criteria
1.  **输入抽象**: 系统 SHALL 将“输入源”抽象为通用接口，允许键盘、网络或 AI 脚本控制角色。
2.  **状态观测**: 系统 SHALL 提供序列化的“世界快照”接口，供 AI 模型读取环境信息。