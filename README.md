# Fate/stay night Heaven's Feel - EmiyaShiro Runner

基于 Rust + Bevy 的 2D 横版动作跑酷原型，主角为卫宫士郎（1P）与樱（2P）。

## 当前技术基线

- 引擎: `Bevy 0.17`
- 语言: `Rust (edition 2024)`
- 架构: ECS + 状态机 + 领域插件化

## 运行方式

```bash
# 客户端
cargo run --bin client

# 服务器（需 server feature）
cargo run --bin server --features server
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

## CI 门禁

项目包含 GitHub Actions 工作流：`.github/workflows/rust-ci.yml`

- `cargo fmt --check`
- `cargo clippy --lib --all-features`
- `cargo test --lib`
