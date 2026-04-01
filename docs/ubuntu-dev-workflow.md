# Ubuntu Dev Workflow

本文件作为仓库在 Ubuntu 上的单一开发期构建、调试和编译加速说明。

## 基线决策

1. 开发期统一使用 Cargo 默认产物目录 `target/`。
2. 开发期统一使用标准 `dev` profile，不再额外维护 `dev-debug` 一类旁路 profile。
3. 命令行、VS Code 和 Cargo 保持同一套默认行为，避免缓存和产物目录分叉。
4. `mold` 和 `sccache` 作为机器级加速能力处理，不在仓库内重复配置。

## 产物目录

开发期产物统一输出到 `target/debug/`：

1. `target/debug/client`
2. `target/debug/server`
3. `target/debug/architecture_metrics`

发布产物使用 Cargo 默认目录 `target/release/`。

## 推荐命令

```bash
# client
cargo run --bin client

# server
cargo run --bin server --no-default-features --features server

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

## 编译加速现状

当前仓库没有在仓库内单独接入 `mold` 或 `sccache`。实际生效的是机器级 Cargo 全局配置：

1. `~/.cargo/config.toml` 中设置了 `rustc-wrapper = "/usr/bin/sccache"`。
2. `~/.cargo/config.toml` 中为 `x86_64-unknown-linux-gnu` 指定了 `linker = "clang"`。
3. 同一份全局配置通过 `-C link-arg=-fuse-ld=mold` 启用 `mold`。

这意味着：

1. 本仓库当前确实在吃到 `mold` 和 `sccache` 的收益。
2. 该收益依赖当前 Ubuntu 机器环境，而不是仓库本身的 checked-in 配置。
3. 如果换机器，希望复现相同体验，应在机器级 Cargo 配置中统一接入，而不是再给仓库增加重复配置。

## 编译速度评估

评估时间：2026-04-02  
评估对象：`cargo build --locked --bin client`

对照结果：

1. 关闭 `sccache` 和 `mold`：`315.53s`
2. 关闭 `sccache`，保留 `mold`：`225.66s`
3. 保留 `mold`，并在 `sccache` 预热后从空 `target` 重建：`47.13s`

换算结果：

1. `mold` 单独带来约 `28.5%` 的时间下降，约 `1.40x` 加速。
2. `sccache` 在缓存命中后，相对 `mold` 单独状态再带来约 `79.1%` 的时间下降，约 `4.79x` 加速。
3. 相对全部关闭工具的基线，当前组合总计约 `85.1%` 的时间下降，约 `6.70x` 加速。

补充说明：

1. `mold` 主要缩短链接阶段，对首次冷编译就有稳定收益。
2. `sccache` 的价值主要体现在重复构建、删除 `target/` 后重建、分支切换后重建等场景。
3. 实测中 `sccache` 命中构建阶段 Rust 命中率达到 `100%`。
4. 如果把大型基准构建放在容量较小的 `tmpfs` 上，可能因为空间配额影响结果，因此完整基准应放在仓库所在磁盘上执行。

## 维护建议

1. 仓库内继续保持单一 `target/` 和单一 `dev` 工作流，不再引入额外 target 目录命名。
2. `mold` 和 `sccache` 保持在机器级配置，避免仓库配置与开发者个人环境双重维护。
3. 如果后续团队希望把编译体验做成可复制标准，再单独评估是否把这套全局配置沉淀为明确的 onboarding 文档，而不是直接写死到项目配置里。

## 常见问题

### 1) client 连接失败

`client` 连接 `127.0.0.1:8080` 失败时会报 `Connection refused`。  
这是运行时状态，不是调试配置问题。先启动 `server` 再启动 `client`。

### 2) server 为什么不能直接走默认 feature

`server` 需要显式关闭默认 feature 并启用 `server`，否则会把客户端默认 feature 一起带上，偏离服务端运行形态。

### 3) 为什么不再保留多个 target 目录

Cargo、IDE 插件和 `cargo metadata` 默认都以 `target/` 为基准。开发期只维护一套目录，缓存命中更稳定，问题定位也更直接。
