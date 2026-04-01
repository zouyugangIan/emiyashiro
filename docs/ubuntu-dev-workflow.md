# Ubuntu Dev Workflow

本文件定义仓库在 Ubuntu 上的单一开发期构建与调试基线。

## 目标

1. 只使用 Cargo 标准产物目录 `target/`。
2. 只保留一个开发 profile：标准 `dev`。
3. 让命令行、VS Code 和 Cargo 的默认行为保持一致，避免额外 target 目录和缓存分叉。

## 产物目录

开发期产物统一输出到 `target/debug/`：

1. `target/debug/client`
2. `target/debug/server`
3. `target/debug/architecture_metrics`

发布产物使用 Cargo 默认目录 `target/release/`。

## 开发 profile

项目使用单一的 `[profile.dev]`：

1. 保留完整调试符号，便于 LLDB 观察变量和回溯。
2. 保留 Bevy 常见的依赖优化覆盖，避免运行时过慢。
3. 不再维护单独的 `dev-debug` profile，也不再拆分 `target-linux*` 目录。

## 推荐命令

```bash
# client
cargo run --bin client

# server
`cargo run --bin server --no-default-features --features server
`
# metrics
cargo run --bin architecture_metrics

# 基础检查
cargo check
cargo check --all-features --future-incompat-report
```

## VS Code 调试

调试配置文件：`.vscode/launch.json`

当前保留三条 Ubuntu 开发入口：

1. `Ubuntu Dev: client`
2. `Ubuntu Dev: server`
3. `Ubuntu Dev: architecture_metrics`

约定：

1. 全部使用 CodeLLDB 的 Cargo 集成直接构建并启动。
2. 统一注入 `RUST_BACKTRACE=full` 和 `RUST_LIB_BACKTRACE=1`。
3. 全部复用默认 `target/debug/`，不再区分 Fast / Deep。

## 常见问题

### 1) client 连接失败

`client` 连接 `127.0.0.1:8080` 失败时会报 `Connection refused`。  
这是运行时状态，不是调试配置问题。先启动 `server` 再启动 `client`。

### 2) server 为什么不能直接走默认 feature

`server` 需要显式关闭默认 feature 并启用 `server`，否则会把客户端默认 feature 一起带上，偏离服务端运行形态。

### 3) 为什么不再保留多个 target 目录

Cargo、IDE 插件和 `cargo metadata` 默认都以 `target/` 为基准。开发期只维护一套目录，缓存命中更稳定，问题定位也更直接。
