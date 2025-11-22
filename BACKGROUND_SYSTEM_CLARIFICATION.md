# 背景系統說明

## 🌥️ 遊戲背景系統架構

你的遊戲有**三個不同的背景系統**，各自用於不同的場景：

### 1. 程序化雲彩系統（遊戲中）
**位置**: `src/systems/background.rs`  
**使用場景**: `GameState::Playing`（遊戲進行中）

**特點**:
- ✅ 程序化生成白色雲朵
- ✅ 每 5 秒生成一朵新雲
- ✅ 雲朵從右向左移動（速度：50.0）
- ✅ 雲朵由 3 個白色圓圈組成
- ✅ 自動清理離開屏幕的雲朵

**系統註冊**:
```rust
.add_systems(
    Update,
    (
        spawn_clouds_system,
        move_clouds_system,
        despawn_offscreen_clouds_system,
    )
        .run_if(in_state(GameState::Playing)),
)
```

**組件**:
```rust
#[derive(Component)]
pub struct Cloud;
```

---

### 2. 封面圖片輪播系統（主菜單）
**位置**: `src/systems/menu.rs`  
**使用場景**: `GameState::Menu`（主菜單）

**特點**:
- ✅ 使用 `UI_COVER_IMAGES` 中的角色封面圖片
- ✅ 兩張圖片交替淡入淡出
- ✅ 使用 `CoverImage1` 和 `CoverImage2` 組件
- ✅ 使用 `CoverFadeState` 控制透明度動畫

**圖片來源**:
```rust
pub const UI_COVER_IMAGES: &[&str] = &[
    IMAGE_UI_COVER00, IMAGE_UI_COVER01, IMAGE_UI_COVER02, ...
];
```

**動畫系統**:
```rust
pub fn cover_fade_animation(
    mut query: Query<(&mut BackgroundColor, &mut CoverFadeState, ...)>,
    time: Res<Time>,
) { ... }
```

---

### 3. 圖片背景系統（備用，未使用）
**位置**: `src/systems/frame_animation.rs`  
**函數**: `setup_animated_background`  
**狀態**: ⚠️ 目前未使用，已標記為 `#[allow(dead_code)]`

**說明**:
- 這是一個備用系統，可以用封面圖片創建動畫背景
- 目前遊戲使用程序化雲彩系統，不使用此函數
- 如果需要切換到圖片背景，可以在 `client.rs` 中註冊此系統

---

## 🎮 角色動畫系統（獨立）

**位置**: `src/systems/frame_animation.rs`  
**用途**: 玩家控制的精靈動畫（Shirou 和 Sakura）

**特點**:
- ✅ 為 1P (Shirou) 和 2P (Sakura) 提供動畫
- ✅ 支持 4 種動畫狀態：待機、跑步、跳躍、蹲下
- ✅ 使用乒乓循環和重複幀技術
- ✅ 根據玩家速度自動切換動畫

**重要**: 角色動畫**不用於背景**，只用於玩家精靈！

---

## 📊 系統對比

| 系統 | 使用場景 | 內容 | 動畫方式 |
|------|----------|------|----------|
| 程序化雲彩 | 遊戲中 | 白色雲朵 | 程序化生成 + 移動 |
| 封面輪播 | 主菜單 | 角色封面圖片 | 淡入淡出 |
| 角色動畫 | 遊戲中 | 玩家精靈 | 幀動畫 |
| 圖片背景（備用） | 未使用 | 封面圖片 | 幀動畫 |

---

## 🔧 如何切換背景系統

### 選項 A: 保持雲彩系統（當前）
無需任何更改，雲彩系統已經正常運行。

### 選項 B: 切換到圖片背景
1. 在 `client.rs` 中註冊 `setup_animated_background`：
```rust
.add_systems(
    OnEnter(GameState::Playing),
    (
        game::setup_game,
        systems::frame_animation::setup_animated_background, // 添加這行
    ),
)
```

2. 移除或註釋掉雲彩系統：
```rust
// .add_systems(
//     Update,
//     (
//         spawn_clouds_system,
//         move_clouds_system,
//         despawn_offscreen_clouds_system,
//     )
//         .run_if(in_state(GameState::Playing)),
// )
```

### 選項 C: 同時使用兩者
可以同時運行雲彩和圖片背景，只需調整 Z 軸順序：
- 圖片背景：`z = -10.0`
- 雲彩：`z = -5.0`（需要修改 `background.rs`）

---

## ✅ 當前狀態總結

- ✅ **雲彩系統**：正常運行，用於遊戲背景
- ✅ **封面輪播**：正常運行，用於主菜單背景
- ✅ **角色動畫**：正常運行，用於玩家精靈
- ⚠️ **圖片背景**：備用狀態，未使用

你的遊戲背景系統設計合理，各司其職！🎮✨
