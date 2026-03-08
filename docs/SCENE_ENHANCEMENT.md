# 场景增强系统

> 文件性质：功能说明（非版本发布说明）  
> 最后与代码对齐：2026-02-23（对齐文件：`src/systems/scene_decoration.rs`、`src/plugins/presentation.rs`、`src/plugins/ui.rs`）

## 🎨 问题

2D 游戏场景过于单调，只有：

- 灰色地面
- 少量云彩
- 单一背景色

## ✨ 解决方案

创建了一个全新的**场景装饰系统**，添加多层视觉元素！

## 🌟 新增功能

### 1. 多层视差背景 (Parallax Background)

**效果**: 不同层级以不同速度移动，创造深度感

**层级结构**:

```
远景层 (z = -10.0) - 20% 速度
  └─ 使用封面图片作为远景山脉/建筑

中景层 (z = -7.0) - 50% 速度
  └─ 云彩（半透明）

近景层 (z = -3.0) - 80% 速度
  └─ 云彩（不透明）

地面层 (z = 0.5) - 100% 速度
  └─ 草、石头、小树等装饰物
```

### 2. 增强云彩系统

**改进**:

- ✅ 更频繁生成（每 3 秒一朵）
- ✅ 随机大小（0.6 - 1.2 倍）
- ✅ 随机透明度（0.5 - 1.0）
- ✅ 多层级（近景和中景）
- ✅ 不同速度（视差效果）

### 3. 地面装饰物

**类型**:

- 🌱 **草**: 绿色，20x30 像素
- 🪨 **石头**: 灰色，15x15 像素
- 🌳 **小树**: 深绿色，10x40 像素

**生成**: 每 2 秒随机生成一个

### 4. 动态光照效果

**效果**: 远景背景会随时间缓慢变亮变暗，模拟日光变化

**周期**: 约 60 秒一个完整循环

## 📁 文件结构

```
src/systems/scene_decoration.rs
├── SceneDecoration 组件
├── DecorationLayer 枚举
├── setup_parallax_background() - 设置多层背景
├── spawn_enhanced_clouds() - 生成增强云彩
├── spawn_ground_decorations() - 生成地面装饰
├── move_scene_decorations() - 视差移动
├── cleanup_offscreen_decorations() - 清理离屏物体
├── loop_far_background() - 远景循环
└── dynamic_lighting() - 动态光照
```

## 🎮 系统注册

### OnEnter(GameState::Playing)

```rust
setup_parallax_background  // 初始化多层背景
```

### Update (Playing 状态)

```rust
spawn_enhanced_clouds           // 生成云彩
spawn_ground_decorations        // 生成地面装饰
move_scene_decorations          // 视差移动
cleanup_offscreen_decorations   // 清理
loop_far_background             // 远景循环
dynamic_lighting                // 动态光照
```

## 🎯 视觉效果

### 视差效果示例

```
玩家移动 100 像素时：

远景层移动: 20 像素  (慢)  ←────
中景层移动: 50 像素        ←──────
近景层移动: 80 像素        ←────────
地面层移动: 100 像素 (快)  ←──────────
```

### 深度感

```
远景 (模糊、慢、半透明)
  ↓
中景 (清晰、中速)
  ↓
近景 (清晰、快速)
  ↓
地面 (最清晰、最快)
  ↓
玩家
```

## 🎨 使用的资源

### 远景背景

- `images/ui/cover10.jpg`
- `images/ui/cover11.jpg`
- `images/ui/cover12.jpg`

### 云彩

- `images/cloud/cloud01.png`
- `images/cloud/cloud02.png`

### 地面装饰

- 程序化生成（彩色矩形）

## 🔧 可调参数

### 速度

```rust
const BASE_SPEED: f32 = 50.0;  // 基础移动速度

FarBackground:   20% (10 px/s)
MidBackground:   50% (25 px/s)
NearBackground:  80% (40 px/s)
Ground:         100% (50 px/s)
```

### 生成频率

```rust
云彩: 每 3 秒
地面装饰: 每 2 秒
```

### 大小范围

```rust
云彩: 0.6 - 1.2 倍
地面装饰: 固定大小
```

## 📊 性能考虑

### 实体管理

- ✅ 自动清理离屏物体
- ✅ 远景背景循环使用（不清理）
- ✅ 限制生成频率

### 预期实体数量

```
远景背景: 3 个（固定）
云彩: 约 10-15 个
地面装饰: 约 5-10 个
总计: 约 20-30 个装饰实体
```

## 🎮 游戏体验提升

### 之前

- ❌ 单调的灰色背景
- ❌ 缺乏深度感
- ❌ 视觉疲劳

### 之后

- ✅ 丰富的多层背景
- ✅ 明显的深度感
- ✅ 动态变化的场景
- ✅ 更有沉浸感

## 🚀 未来扩展建议

1. **添加更多装饰类型**
   - 鸟类飞行
   - 蝴蝶
   - 落叶效果

2. **天气系统**
   - 雨天效果
   - 雪花
   - 雾气

3. **时间系统**
   - 白天/黑夜循环
   - 日出/日落效果

4. **互动元素**
   - 可破坏的装饰物
   - 收集品

5. **使用真实图片**
   - 替换程序化地面装饰为真实图片
   - 添加更多背景图片变化

## ✅ 验收清单（2026）

- [x] `setup_parallax_background` 在 `OnEnter(GameState::Playing)` 注册
- [x] 云彩（3 秒）与地面装饰（2 秒）生成频率符合实作
- [x] 远景循环与离屏清理策略存在且已分层处理
- [x] `dynamic_lighting` 仅作用于 FarBackground 层
- [x] 返回 `GameState::Menu` 时会清理场景装饰实体
- [ ] 效能压测基准（不同机型 FPS 影响）尚未文档化

## 📝 总结

场景增强系统通过多层视差背景、动态云彩和地面装饰，将单调的 2D 场景转变为丰富、有深度的游戏世界！

现在你的游戏场景充满生机和动感！🎮✨
