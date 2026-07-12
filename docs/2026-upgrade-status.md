# 2026 引擎升级状态总览

> 最后更新：2026-07-12
> 作用：技术基线、架构状态与验证记录的单一事实来源。

## 当前技术基线

- Bevy `0.19`，仅启用项目需要的 `2d/ui/audio/jpeg` 特性集。
- Rust stable，edition 2024，Cargo 声明 MSRV `1.95`。
- Tokio `1.52`、SQLx `0.9`、UUID `1.23`、RON `0.12.2`。
- 工具链由 `rust-toolchain.toml` 统一，依赖解析由 `Cargo.lock` 锁定。

## 2026-07-12 复核结论

- 已完成 Bevy 0.18 → 0.19 API 迁移：Parley 文本字体/字号、系统条件和状态转换均使用 0.19 接口。
- 删除未注册或与实际主链重复的动画、背景、输入历史、着地判定、音频、程序化占位素材和伪服务。
- 删除仅在测试里流转的内存排行榜/回放/云存档原型，避免将未接入能力误标为线上闭环。
- 士郎使用精灵图集主链；樱因源素材不是等尺寸网格，正式使用逐张图片序列主链。
- 文件存档始终使用同目录原子替换，不再在异常时降级为直接覆盖。
- PostgreSQL 建表已转为 SQLx 版本化迁移，并补齐存档 upsert 依赖的唯一约束。
- RabbitMQ 存档队列和消息已持久化，消费者异常会返回重连循环，不再 `expect` 崩溃。

## 验证边界

本次按需求不执行测试用例。2026-07-12 本地实际通过：

- `cargo fmt --all -- --check`
- `cargo check`
- `cargo check --all-features --future-incompat-report`（`0` 个 future-incompatible 依赖警告）
- `cargo clippy --lib --all-features -- -D warnings`

2026-02-24 的 `119 passed` 和架构指标仅是 Bevy 0.18 时期的历史快照，不代表当前基线。指标可用 `cargo run --release --bin architecture_metrics` 重新生成。

## 架构与文档入口

- `docs/animation-system.md`：精灵图集与图片序列动画主链。
- `docs/G-ENGINE-SETUP.md`：联机、PostgreSQL、Redis 和 RabbitMQ 环境。
- `docs/SCENE_ENHANCEMENT.md`：视差背景与场景装饰。
- `docs/ubuntu-dev-workflow.md`：开发与构建流程。
- `docs/hf-shirou-attack-module-design.md`：HF 士郎攻击模组和动作资源映射。

## 官方基线来源

- Bevy 0.19 发布与快速入门：<https://bevy.org/news/>、<https://bevy.org/learn/quick-start/getting-started/>
- Bevy 0.18 → 0.19 迁移指南：<https://bevy.org/learn/migration-guides/0-18-to-0-19/>
- Rust 2024 Cargo resolver：<https://doc.rust-lang.org/stable/edition-guide/rust-2024/cargo-resolver.html>
- Cargo 检查与 future incompat report：<https://doc.rust-lang.org/cargo/commands/cargo-check.html>、<https://doc.rust-lang.org/cargo/reference/future-incompat-report.html>
- Clippy：<https://doc.rust-lang.org/clippy/>
