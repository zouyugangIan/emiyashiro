# Zed Ubuntu Debug Guide (2026)

本文件是仓库内的 Zed 调试基线，目标是提供稳定、可复现、跨任务一致的 Ubuntu 调试体验。

## 设计原则

1. Linux 与 Windows 产物完全隔离，避免 `.exe` / ELF 混用。
2. 快速迭代与深度定位分离：
   1. `Fast` 追求构建速度。
   2. `Deep` 追求可调试性（更完整符号和更少优化干扰）。
3. 先预检查，再构建，再调试。
4. 服务端调试必须与线上行为一致（明确 feature 组合）。

## 调试入口

配置文件：`.zed/debug.json`

### Launch（推荐日常使用）

1. `Debug client (Linux Fast)`
2. `Debug server (Linux Fast)`
3. `Debug architecture_metrics (Linux Fast)`
4. `Debug client (Linux Deep)`
5. `Debug server (Linux Deep)`
6. `Debug architecture_metrics (Linux Deep)`
7. `Debug lib tests (Linux Deep)`

### Attach（用于不重启场景）

1. `Attach client (Linux Fast, waitFor)`
2. `Attach server (Linux Fast, waitFor)`
3. `Attach client (Linux Deep, waitFor)`
4. `Attach server (Linux Deep, waitFor)`

## 脚本工作流

### 预检查 + 构建

`scripts/debug/build_bin.sh <bin> <mode>`

1. 对 `server`：
   1. 检查 `8080` 端口占用。
   2. 检查 `5432/6379/5672` 可达性（告警不阻断）。
   3. 构建时强制 `--no-default-features --features server`。
2. 对 `client`：
   1. 检查 `127.0.0.1:8080` 可达性（告警不阻断）。
3. 对所有 bin：
   1. 校验最终可执行文件存在且可运行。

### 测试调试入口

`scripts/debug/build_test_runner.sh <mode>`

1. 执行 `cargo test --lib --no-run`。
2. 从 `cargo --message-format=json` 解析测试可执行文件。
3. 创建稳定链接：
   1. `target-linux/debug/emiyashiro-lib-tests`
   2. `target-linux-devdebug/dev-debug/emiyashiro-lib-tests`

### 直接运行（便于 Attach）

`scripts/debug/run_bin.sh <bin> <mode> [-- <args...>]`

1. 先调用 `build_bin.sh`。
2. 自动注入默认调试环境变量：
   1. `RUST_BACKTRACE=full`
   2. `RUST_LIB_BACKTRACE=1`
   3. 默认 `RUST_LOG`（可被外部覆盖）
3. 启动对应产物并保持前台运行，适合 `waitFor` attach。

## 产物目录约定

1. `Fast`：`target-linux/debug/`
2. `Deep`：`target-linux-devdebug/dev-debug/`

## 常见问题

### 1) 看到连接失败日志

`client` 连接 `127.0.0.1:8080` 失败时会报 `Connection refused`。  
这是运行时状态，不是调试配置问题。先启动 `server` 再启动 `client`。

### 2) 断点命中但变量不稳定

切换到 `Deep` 配置重试。`Deep` 使用 `profile.dev-debug`，优化级别更低，变量可见性更高。

### 3) Attach 一直在等待

1. 确认目标进程已经运行（建议用 `run_bin.sh` 启动）。
2. 确认 attach 入口和目标二进制模式一致（Fast 对 Fast，Deep 对 Deep）。
