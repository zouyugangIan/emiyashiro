# 資源路徑修復總結

## 🐛 問題

遊戲啟動時出現多個資源加載錯誤：

```
ERROR bevy_asset::server: Path not found: F:\projects\emiyashiro\assets\images/ui/cove01.png
ERROR bevy_asset::server: Path not found: F:\projects\emiyashiro\assets\images/ui/cover18.jpg
ERROR bevy_asset::server: Path not found: F:\projects\emiyashiro\assets\images/characters/sakura_idle_13.png
ERROR bevy_asset::server: Path not found: F:\projects\emiyashiro\assets\images/characters/sakura_idle09.png
ERROR bevy_asset::server: Path not found: F:\projects\emiyashiro\assets\images/characters/shirou_idle12.png
ERROR bevy_asset::server: Path not found: F:\projects\emiyashiro\assets\images/characters/shirou_idle13.png
ERROR bevy_asset::server: Path not found: F:\projects\emiyashiro\assets\images/characters/shirou_idle14.png
ERROR bevy_asset::server: Path not found: F:\projects\emiyashiro\assets\images/characters/sakura_idle1.jpg
```

## ✅ 修復內容

### 1. UI 封面圖片路徑

**修復**:
- `cove01.png` → `cover01.png` (拼寫錯誤)
- `cover18.jpg` → `cover18.png` (副檔名錯誤)
- 移除不存在的 `IMAGE_UI_COVER00`

### 2. Shirou 角色圖片路徑

**修復**:
- 移除不存在的 `shirou_idle12.png`、`shirou_idle13.png`、`shirou_idle14.png`
- 添加存在的 `shirou_idle9.png`
- 更新動畫幀數組使用實際存在的文件

**新的動畫幀配置**:
- 待機動畫：idle1, idle2, idle3 (乒乓循環)
- 跑步動畫：idle4, idle5, idle6, idle7 (重複循環)
- 跳躍動畫：idle8, idle9, idle10
- 蹲下動畫：idle10, idle11

### 3. Sakura 角色圖片路徑

**修復**:
- 移除不存在的 `sakura_idle1.jpg`
- 移除不存在的 `sakura_idle_13.png`
- `sakura_idle09.png` → `sakura_idle9.jpg` (實際文件名)
- 添加存在的 `sakura_idle15.png`、`sakura_idle16.jpg`

**新的動畫幀配置**:
- 待機動畫：idle01-04 (乒乓循環)
- 跑步動畫：idle05-08 (重複循環)
- 跳躍動畫：idle09-11 (實際是 idle9.jpg, idle10.png, idle11.png)
- 蹲下動畫：idle13, idle14

### 4. 其他角色圖片

**修復**:
- `teacher_idle.jpg` → `teacher_idle02.jpg` (使用實際存在的文件)

## 📊 修復後的文件結構

### Shirou 動畫幀
```
待機: shirou_idle1.jpg, shirou_idle2.jpg, shirou_idle3.jpg
跑步: shirou_idle4.png, shirou_idle5.png, shirou_idle6.png, shirou_idle7.png
跳躍: shirou_idle8.png, shirou_idle9.png, shirou_idle10.png
蹲下: shirou_idle10.png, shirou_idle11.png
```

### Sakura 動畫幀
```
待機: sakura_idle01.png, sakura_idle02.png, sakura_idle03.png, sakura_idle04.png
跑步: sakura_idle05.png, sakura_idle06.png, sakura_idle07.png, sakura_idle08.png
跳躍: sakura_idle9.jpg, sakura_idle10.png, sakura_idle11.png
蹲下: sakura_idle13.jpg, sakura_idle14.png
```

### UI 封面圖片
```
cover01.png - cover18.png (共 18 張)
```

## 🎯 測試結果

- ✅ 編譯成功，無錯誤
- ✅ 所有動畫測試通過 (10/10)
- ✅ 資源路徑全部正確
- ✅ 不再出現 "Path not found" 錯誤

## 📝 注意事項

1. **文件命名不一致**: 
   - Shirou 使用 `idle1`, `idle2` 格式
   - Sakura 使用 `idle01`, `idle02` 格式
   - 部分文件使用 `.jpg`，部分使用 `.png`

2. **缺失的幀**:
   - Shirou 缺少 `idle12`, `idle13`, `idle14`
   - Sakura 缺少 `idle12`

3. **建議**:
   - 統一文件命名格式
   - 統一使用 `.png` 格式（支持透明度）
   - 補充缺失的動畫幀

## 🎮 現在可以正常運行

所有資源路徑已修復，遊戲可以正常加載所有圖片資源！
