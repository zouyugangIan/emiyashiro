# Fate/stay night Heaven's Feel - EmiyaShiro Runner

基于 Rust + Bevy 的 2D 横版动作跑酷原型，主角为卫宫士郎（1P）与樱（2P）。

> 文档状态：2026-02-23 已按仓库代码静态对齐。  
> 说明：当前环境无法访问 crates.io，未完成联网编译复验。

## 当前技术基线

- 引擎: `Bevy 0.18`
- 语言: `Rust (edition 2024)`
- 架构: ECS + 状态机 + 领域插件化（`core/gameplay/netcode/persistence/presentation/ui`）

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
- `J`: 攻击/投影
- `K`: 圣骸布开关
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
    ui.rs
  systems/
  components/
  resources.rs
  states.rs
```

## 文档索引

- `G-ENGINE-SETUP.md`: 联机与基础设施部署指南
- `IMPLEMENTATION-SUMMARY.md`: Phase 3-5 历史实现总结（已标记归档）
- `SCENE_ENHANCEMENT.md`: 场景视差与装饰系统说明
- `docs/2026-bevy-upgrade-assessment-zh.md`: 2026 全面评估与升级蓝图
- `docs/bevy-upgrade-regression-checklist.md`: 引擎升级回归清单
- `docs/documentation-completeness-audit-2026-02-23.md`: 本次文档完成度审计

## CI 门禁

项目包含 GitHub Actions 工作流 `/.github/workflows/rust-ci.yml`：

- `cargo fmt --check`
- `cargo clippy --lib --all-features`
- `cargo test --lib`

## 已知待完善项（2026）

- 客户端预测与服务器校正
- 断线重连与恢复
- WorldSnapshot 差量同步
