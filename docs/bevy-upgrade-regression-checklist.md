# Bevy 升级回归清单（0.17 -> 0.18+）

用于每次引擎升级分支（如 `upgrade/bevy-0.18`）的标准回归流程，确保“先可编译，再行为一致”。

> 最近一次门禁复验：2026-02-24

## 1. 编译与静态检查

- [x] `cargo fmt --check`
- [x] `cargo check`
- [x] `cargo check --all-features`
- [x] `cargo clippy --lib --all-features`
- [x] `cargo test --lib`

## 2. 核心状态机回归

- [x] `Menu -> Playing -> Paused -> Playing` 正常
- [x] `Playing -> GameOver -> Reviving -> Playing` 正常
- [x] `SaveDialog/LoadTable/RenameDialog` 进出状态正常

## 3. 玩法行为回归

- [x] 玩家移动、跳跃、蹲下在 `FixedUpdate` 下行为稳定
- [x] 投射物命中敌人扣血与死亡清理正常
- [x] 玩家接触敌人受击，生命值与 HUD 同步
- [x] 坠落死亡触发 `GameOver`，`R` 可复活
- [x] 圣骸布扣血与伤害事件管线正常

## 4. 联机回归

- [x] 服务端发送 `Welcome` 包
- [x] 客户端握手后拿到 `MyNetworkId`
- [x] `NetworkStatus` 按连接事件变化（`Connecting/Connected/Disconnected`）
- [x] 重连后 `ResumeSession` 可恢复实体映射且不产生重复实体
- [x] 输入发送单通道（`update_game_input`）无重复上报
- [x] 输入协议分层（`InputState` + `InputEvent`）可通过自动化回归
- [x] 快照插值无明显抖动
- [x] `WorldSnapshotDelta` 更新/移除链路可通过自动化回归

## 5. 存档回归

- [x] 统一写入 `SaveFileData v2`
- [x] 仅接受 `SaveFileData v2`（旧 schema 直接拒绝）
- [x] 校验和不匹配时硬失败（无兼容模式）
- [x] v2 schema 回归测试覆盖保存/加载/损坏校验

## 6. 性能与可运维

- [x] 进度 UI 为增量更新（非每帧销毁重建）
- [x] Redis 同步为批量 + 节流 + 后台写队列
- [x] release 构建无高频 debug 输出噪音

## 7. 人工验收建议（5 分钟）

- [x] 启动客户端，主菜单可交互（自动化状态机测试覆盖）
- [x] 进入游戏后 30 秒内完成移动、攻击、受击、暂停、恢复、死亡、复活（自动化玩法回归覆盖）
- [x] 执行一次保存并从加载界面恢复（自动化存档回归覆盖）
- [x] （如启服务端）连接后确认远端实体同步（网络握手/快照/插值自动化回归覆盖）
