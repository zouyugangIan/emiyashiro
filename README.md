# Fate/stay night Heaven's Feel - EmiyaShiro Runner

基于 Rust + Bevy 的 2D 横版动作跑酷原型，主角为卫宫士郎（1P）与樱（2P）。

> 文档状态：2026-07-12 已对齐 Bevy 0.19 与 Rust 2024 基线。

## 当前技术基线

- 引擎: `Bevy 0.19`（按需启用 `2d/ui/audio/jpeg`，不携带 3D 默认特性）
- 语言: `Rust stable`（edition 2024，MSRV `1.95`）
- 架构: ECS + 状态机 + 领域插件化（`core/gameplay/netcode/persistence/presentation/ui/server`）

## 快速启动

```bash
# 客户端（默认 feature 包含 client）
cargo run --bin client --features client

# 服务端（联机与基础设施功能）
cargo run --bin server --features server
```

## 服务端基础设施（可选）

```bash
docker-compose up -d

export REDIS_URL="redis://127.0.0.1:6379/"
export RABBITMQ_URL="amqp://guest:guest@127.0.0.1:5672/%2f"
export DATABASE_URL="postgresql://username:password@localhost/shirou_runner"
```

## 主要按键

- `A/D` 或 `←/→`: 移动
- `W` 或 `Space`: 跳跃
- `S` 或 `↓`: 蹲下
- `J/Z/L`: 轻攻击；地面使用轻攻击表，空中使用空连表，蹲下使用机动表
- `K`: 重攻击；蹲下 `K` 使用奥义表
- `X`: 忍术投射；`Shift+X` 使用影分身语义，蹲下 `X` 使用武器投影表
- `Y/U/I/O/P`: 直接指定当前动作上下文的第 1-5 行；站立为轻攻击，空中为空连，蹲下为机动
- `Shift+Y/U/I/O/P`: 直接指定重攻击表第 1-5 行
- `Shift+V` 或 `Ctrl+V`: 绯红圣骸布/Overedge 本体升级，持续 10 秒并消耗 5 HP
- 绯红圣骸布期间：所有攻击键锁定为本体 Overedge 动作，`J/Z/L/X/Y/U/I/O/P` 走轻攻击连段，`K` 与 `Shift+Y/U/I/O/P` 走重攻击；未开启时默认使用 reference 文件夹下的新动作表，不再生成屏幕中央的参考预览角色
- `Esc`: 暂停
- `R`: GameOver 后复活
- `M`: GameOver 返回主菜单

## 目录结构（核心）

```text
src/
  bin/
    client.rs
    server.rs
  plugins/
    core.rs
    gameplay.rs
    netcode.rs
    persistence.rs
    presentation.rs
    server.rs
    ui.rs
  systems/
  components/
  resources.rs
  states.rs
```

## 文档索引

- `docs/animation-system.md`: HF 士郎精灵图集主链、RON 配置和樱的逐帧图片动画
- `docs/G-ENGINE-SETUP.md`: 联机与基础设施部署指南
- `docs/SCENE_ENHANCEMENT.md`: 场景视差与装饰系统说明
- `docs/2026-upgrade-status.md`: 2026 升级状态总览（文档完成状态 SSOT）
- `docs/ubuntu-dev-workflow.md`: Ubuntu 开发期构建与调试基线
- `docs/hf-shirou-attack-module-design.md`: HF 士郎攻击模组与资源对应
- `docs/sky-city-level.md`: LDtk 浮空都市长关卡、实体约定与再生成流程

## CI 门禁

项目包含 GitHub Actions 工作流 `/.github/workflows/rust-ci.yml`：

- `cargo fmt --check`
- `cargo check`
- `cargo check --all-features --future-incompat-report`
- `cargo clippy --lib --all-features -- -D warnings`
- `cargo test --lib --all-features`

## 架构升级状态（2026）

当前技术基线、已接入的运行链和验证边界统一记录在
`docs/2026-upgrade-status.md`。仅在测试中运行的原型不再标记为已完成能力。
