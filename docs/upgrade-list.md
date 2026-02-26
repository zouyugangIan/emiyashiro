先给你一个客观判断：你“Rust 基础薄弱”这个自我评价有点偏低了。  
从项目看，你已经有比较完整的工程化能力了。

我刚在你仓库里做了快速体检，结果是：

- `cargo check --all-features` 通过
- `cargo clippy --all-features --all-targets -- -D warnings` 通过
- `cargo test --lib --all-features` 通过，`123 passed`
- `cargo run --release --bin architecture_metrics` 在 2026-02-26 跑通，网络与场景指标都不错

你现在最缺的不是“能不能写 Rust”，而是“如何把大项目持续演进得更稳”。这正好也是 Rust 进阶核心。

**最优先改造路线（同时提升项目 + Rust 功力）**

1. `P0` 拆 `UI` 巨文件，建立模块边界。  
   目标：把 `src/systems/ui.rs:641`、`src/systems/ui.rs:1081`、`src/systems/ui.rs:1663`、`src/systems/ui.rs:1992` 这些超长函数拆到 `hud/pause/save/load/rename` 子模块。  
   Rust 提升点：模块系统、可见性控制、函数签名设计、减少可变借用范围。  
   项目收益：可维护性和迭代速度会显著提升。  
   额外细节：`src/systems/ui.rs:633` 一带注释存在乱码，建议统一 UTF-8 并清理。

2. `P0` 重构网络快照处理，消除重复逻辑。  
   目标：`src/systems/network.rs:571`（全量快照）和 `src/systems/network.rs:694`（增量快照）有大量重复分支，抽成共享 helper。  
   Rust 提升点：`match` 分解、借用与查询参数组织、结构化状态更新。  
   项目收益：减少隐性分叉 bug，未来加新同步字段会更安全。

3. `P0` 修并发热点：缩短 `Mutex` 持锁时间。  
   目标：`src/systems/network.rs:542` 现在拿锁后直接在锁内循环处理包。建议先把队列 drain 到本地 `Vec`，解锁后再处理。  
   Rust 提升点：并发模型设计、锁粒度控制、数据所有权转移。  
   项目收益：网络线程与主线程争锁概率下降，帧稳定性更好。

4. `P1` 清理“未落地抽象”，让架构更真实。  
   目标：`src/systems/interfaces.rs:51` 的 `GameSystem` / `StateSystem` 当前没有实现方；`src/systems/system_sets.rs:42` 的 `GameStateSystemSet` 仅定义未使用；`SystemPerformanceMonitor` 也是孤立定义。  
   Rust 提升点：抽象何时该做、何时该删（这是高级能力，不是基础能力）。  
   项目收益：代码认知负担下降，团队协作更清晰。

5. `P1` 统一运行时错误处理，去掉潜在 panic。  
   目标：`src/database/operations.rs:144`、`src/database/operations.rs:179` 的 `unwrap`，以及 `src/systems/pause_save.rs:394` 的 `unwrap` 改为显式错误传播。  
   Rust 提升点：`Result` 设计、错误枚举（推荐 `thiserror`）、上下文信息传递。  
   项目收益：线上健壮性提升，调试成本降低。

6. `P1` 输入系统做减法，避免“伪复杂”。  
   目标：`src/systems/input.rs:414` 的 `filter_input` 与 `src/systems/input.rs:61` 的 `input_filter` 当前没有进入主流程，可选择删掉或真正接入。  
   Rust 提升点：YAGNI、最小可用设计、状态机简化。  
   项目收益：输入链路可读性更高，减少未来回归。

**你可以照这个 6 周节奏练（非常实战）**

1. 第 1-2 周：只做 UI 拆分，不改行为，确保测试全绿。
2. 第 3 周：网络快照逻辑去重 + 锁粒度优化。
3. 第 4 周：错误处理系统化（去 `unwrap`，引入错误类型）。
4. 第 5 周：删掉未使用抽象或把它们真正落地。
5. 第 6 周：补一批“重构保护型测试”（不是新增功能测试）。

**每次改造的硬性验收标准**

- 保持 `cargo clippy -- -D warnings` 通过
- 保持 `cargo test --lib --all-features` 全通过
- 每次 PR 聚焦一个主题，控制在 200-400 行有效改动
- 功能不变时，优先“重构 + 测试补强”，不要顺手加需求

先回答你最核心的两个判断。

**1) 为什么现在没用 `wgpu 28`**
截至 **2026-02-26**，你项目里是 `bevy 0.18.0`（`Cargo.toml`），依赖树显示 `bevy_render` 锁定的是 `wgpu 27.0.1`。  
我本地确认过：

- `cargo search wgpu` 显示最新是 `28.0.0`
- `cargo search bevy` / `cargo info bevy` 显示 crates.io 上最新稳定是 `0.18.0`
- `cargo tree -i wgpu` 显示你的 `wgpu` 来自 `bevy_render`

所以不是“你不用新版本”，而是“当前 Bevy 稳定版生态链还在 27”。  
强行在这个项目里 patch 到 28，通常会引发 Bevy 渲染层 API 兼容问题，不建议。

**结论**：要上 `wgpu 28`，优先路径是“等/跟进支持 28 的 Bevy 版本”，而不是在 `bevy 0.18` 上硬改依赖。

---

**2) 既然用 wgpu，要不要写 WGSL 提升性能**
要写，但要“按瓶颈写”，不是一上来全自定义。

你现在是 2D + 联机，常见瓶颈通常先在：

- CPU/ECS（系统查询、状态流转、锁竞争）
- 网络序列化/带宽
- Web 端 wasm 包大小和 JS<->wasm 开销

WGSL 真正高收益的场景是：

- 大量粒子/特效（可上 GPU 计算或更高效 fragment）
- 自定义后处理链（Bloom/CRT/色调映射等）
- 特定批处理管线（单材质大批量 Sprite）

不是“写了 WGSL 就一定更快”。  
你先把 CPU/网络基础打穿，再上 WGSL，会更稳。

---

**你这个仓库的现状，已经具备 Web+Desktop 复用基础**

你已经有跨平台网络连接分支：

- 原生路径：`src/systems/network.rs:132`
- wasm 路径：`src/systems/network.rs:223`

这说明方向是对的。下一步是把“复用边界”切干净。

---

**我建议的目标架构（Web 优先，桌面是增量）**

1. `core_sim`：纯游戏逻辑，尽量不依赖 Bevy 渲染/UI。
2. `net_protocol`：协议与序列化（你现在在 `src/protocol.rs`，可独立成 crate）。
3. `net_transport`：定义 trait，分别实现 tokio(ws) 和 gloo(ws)。
4. `client_app`：Bevy 渲染、输入、音频，只做适配层。
5. `server_app`：权威服固定 tick（你已有 `src/bin/server.rs` + `src/plugins/server.rs` 基础）。

这样 Desktop/Web 共用 80%-90% 代码，平台差异只留在 transport + shell。

---

**想“榨干 wgpu 和 Rust 性能”，优先级应该是这个顺序**

1. 先做数据和并发优化。
2. 再做渲染批处理和资源布局。
3. 最后用 WGSL 打穿特定热点。

对应你项目里最该先改的点：

1. `src/systems/network.rs`：缩短锁持有时间（先 drain 队列再处理），减少主线程抖动。
2. `src/protocol.rs`：`animation_state: String` 改为紧凑枚举（`u8`），减少分配和带宽。
3. `Cargo.toml`：拆 `client` feature 为 `client-native` / `client-web`，避免 wasm 包引入不必要依赖。
4. `src/systems/ui.rs`：超大文件拆模块，降低每帧 UI 逻辑维护成本和回归风险。
5. Release profile 单独调优（LTO、codegen-units、panic 策略）以提升实际吞吐。

---

**关于你问的“3个系统都能用桌面端”**
如果你的“3个系统”是 Windows/Linux/Web，建议反过来定义：  
把 **Web 当最低公共平台**（最严格约束），Desktop 只是在其上加能力。这样长期复用率更高，联机一致性也更好。

---

如果你愿意，我下一步可以直接给你落地第一刀：  
把 `Cargo.toml` 的 feature 拆分成 `client-native/client-web/server`，并改最小编译矩阵（不改游戏行为），这样你马上就能验证 Web/桌面复用链和包体差异。
