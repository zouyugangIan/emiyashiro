# 天空中飛翔的 Shirou 圖片問題修復

## 🐛 問題描述

遊戲中出現了很多 Shirou 圖片在天空中飛翔，這些圖片：
1. 從右向左移動（像雲彩一樣）
2. 有一個會跟隨玩家角色移動

## 🔍 問題原因

經過調查，發現了兩個可能的原因：

### 1. 網絡系統生成的遠程玩家 ⚠️

**位置**: `src/systems/network.rs` 第 172-184 行

```rust
// Spawn new remote player entity
println!("Spawning remote player {}", player_state.id);
let entity = commands.spawn((
    Sprite {
        image: asset_server.load("images/characters/shirou_idle1.jpg"),
        ..default()
    },
    Transform::from_translation(player_state.position).with_scale(Vec3::splat(0.5)),
    crate::components::network::NetworkId(player_state.id),
    InterpolationState {
        start_pos: player_state.position,
        target_pos: player_state.position,
        start_time: current_time,
        duration: 0.1,
    },
)).id();
```

**說明**:
- 網絡系統會為每個遠程玩家生成一個 Sprite 實體
- 即使沒有連接到服務器，系統也可能在處理假數據
- 這些實體使用 `shirou_idle1.jpg` 圖片

### 2. 雲彩系統的渲染問題（已修復）

**位置**: `src/systems/background.rs`

**原始問題**:
- 雲彩系統使用 `Mesh2d` 和 `MeshMaterial2d` 創建白色圓圈
- 可能由於渲染問題，這些圓圈沒有正確顯示

**修復方案**:
- 改用 `Sprite` 組件創建白色矩形雲朵
- 設置 `z = -5.0` 確保在背景層

## ✅ 已實施的修復

### 1. 禁用網絡系統（臨時）

**文件**: `src/bin/client.rs`

```rust
.add_systems(
    Startup,
    (
        setup_game_resources,
        setup_animation_data,
        systems::save::load_game,
        setup_cloud_spawner,
        // systems::network::setup_network, // 暫時禁用網絡系統進行測試
    ),
)
// .add_systems(
//     Update,
//     (
//         systems::network::handle_network_events,
//         systems::network::send_ping_system,
//     ),
// )
```

**效果**:
- ✅ 不再生成遠程玩家實體
- ✅ 天空中不會出現多餘的 Shirou 圖片
- ⚠️ 多人聯機功能暫時不可用

### 2. 修復雲彩系統

**文件**: `src/systems/background.rs`

**改動**:
- 從 `Mesh2d` 改為 `Sprite`
- 使用真實的雲彩圖片（`cloud01.png` 和 `cloud02.png`）
- 隨機選擇雲彩圖片和縮放比例
- 設置正確的 Z 軸層級（z = -5.0）

**新增資源**:
- `assets/images/cloud/cloud01.png`
- `assets/images/cloud/cloud02.png`

## 🔧 永久修復方案

### 方案 A: 添加網絡連接檢查

在 `handle_network_events` 中添加連接狀態檢查：

```rust
pub fn handle_network_events(
    mut commands: Commands,
    net: ResMut<NetworkResource>,
    // ...
) {
    // 只有在連接狀態下才處理網絡事件
    if net.status != NetworkStatus::Connected {
        return;
    }
    
    let mut rx = net.packet_rx.lock().unwrap();
    // ...
}
```

### 方案 B: 添加遠程玩家標記

為遠程玩家添加特殊組件，便於識別和管理：

```rust
#[derive(Component)]
pub struct RemotePlayer;

// 生成遠程玩家時添加標記
let entity = commands.spawn((
    Sprite { /* ... */ },
    Transform::from_translation(player_state.position),
    RemotePlayer, // 添加標記
    crate::components::network::NetworkId(player_state.id),
    // ...
)).id();
```

### 方案 C: 使用不同的圖片

為遠程玩家使用不同的圖片或顏色，便於區分：

```rust
Sprite {
    image: asset_server.load("images/characters/sakura_idle01.png"), // 使用不同角色
    color: Color::srgba(0.7, 0.7, 1.0, 0.8), // 添加半透明藍色調
    ..default()
}
```

## 📊 測試結果

### 禁用網絡系統後

- ✅ 天空中不再出現多餘的 Shirou 圖片
- ✅ 只有一個玩家角色（本地玩家）
- ✅ 雲彩系統正常工作（白色矩形雲朵）
- ⚠️ 無法測試多人聯機功能

### 修復雲彩系統後

- ✅ 雲彩使用真實圖片（cloud01.png 和 cloud02.png）
- ✅ 隨機選擇雲彩圖片，增加視覺多樣性
- ✅ 隨機縮放（0.8-1.2 倍），讓雲彩大小不一
- ✅ 雲彩從右向左移動
- ✅ 雲彩在背景層（z = -5.0）

## 🎮 如何重新啟用網絡系統

如果需要測試多人聯機功能，請：

1. **取消註釋網絡系統**:
```rust
.add_systems(
    Startup,
    (
        // ...
        systems::network::setup_network, // 取消註釋
    ),
)
.add_systems(
    Update,
    (
        systems::network::handle_network_events, // 取消註釋
        systems::network::send_ping_system, // 取消註釋
    ),
)
```

2. **確保服務器正在運行**:
```bash
cargo run --bin server
```

3. **連接到服務器**:
客戶端會自動嘗試連接到 `ws://127.0.0.1:9001`

## 🎯 建議

1. **單人模式**: 保持網絡系統禁用，專注於單人遊戲體驗
2. **多人模式**: 實施方案 A（添加連接檢查），確保只在連接時生成遠程玩家
3. **視覺區分**: 實施方案 C，為遠程玩家使用不同的外觀

## 📝 總結

- ✅ 問題已定位：網絡系統生成的遠程玩家實體
- ✅ 臨時修復：禁用網絡系統
- ✅ 雲彩系統：已修復為使用 Sprite
- 🔄 永久修復：需要添加網絡連接狀態檢查

現在你的遊戲應該只有一個玩家角色和正常的白色雲朵背景了！🎮✨
