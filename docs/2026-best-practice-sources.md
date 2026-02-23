# 2026 Best Practice Sources

本文件记录本次“文档与 CI jobs 收口”采用的官方参考来源，以及对应落地动作。

## 1) Cargo 命令与质量门禁

- Cargo `check` 命令文档  
  https://doc.rust-lang.org/cargo/commands/cargo-check.html
- Cargo `test` 命令文档  
  https://doc.rust-lang.org/cargo/commands/cargo-test.html
- Cargo future incompat report 机制  
  https://doc.rust-lang.org/cargo/reference/future-incompat-report.html

对应落地：

- CI 增加 `cargo check` 与 `cargo check --all-features --future-incompat-report`
- CI 测试升级为 `cargo test --lib --all-features`
- 本地执行并记录上述 jobs 结果

## 2) Clippy 严格模式

- Clippy 文档（命令行与配置）  
  https://doc.rust-lang.org/clippy/

对应落地：

- CI 使用 `cargo clippy --lib --all-features -- -D warnings`
- 修正 MSRV 声明与代码 lint 一致性（`rust-version = "1.87"`）

## 3) GitHub Actions 运行策略

- 控制 workflow/job 并发（concurrency）  
  https://docs.github.com/en/actions/writing-workflows/choosing-what-your-workflow-does/control-the-concurrency-of-workflows-and-jobs
- 手动触发 workflow（workflow_dispatch）  
  https://docs.github.com/en/actions/using-workflows/events-that-trigger-workflows#workflow_dispatch

对应落地：

- `.github/workflows/rust-ci.yml` 新增 `concurrency`
- `.github/workflows/rust-ci.yml` 新增 `workflow_dispatch`
- 设定 job 超时与只读权限

## 4) Rust 缓存 Action

- `Swatinem/rust-cache` 官方仓库文档  
  https://github.com/Swatinem/rust-cache

对应落地：

- 保持并沿用 `rust-cache`，与新增 jobs 协同

## 5) Bevy 升级治理参考

- Bevy 0.17 -> 0.18 Migration Guide  
  https://bevy.org/learn/migration-guides/0-17-to-0-18/
- Bevy 仓库 README（包含迁移指南入口）  
  https://github.com/bevyengine/bevy

对应落地：

- 文档基线统一为 Bevy `0.18`
- 与 `docs/bevy-upgrade-regression-checklist.md` 形成可执行回归闭环
