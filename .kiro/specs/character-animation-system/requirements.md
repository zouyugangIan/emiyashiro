# Requirements Document

## Introduction

角色動畫系統負責管理遊戲中玩家角色（1P 和 2P）的精靈動畫資源。系統需要支持從 `assets/images/characters/` 目錄中自動加載對應角色的動畫幀序列，並提供統一的接口供遊戲邏輯調用。

## Glossary

- **Animation System**: 角色動畫系統，負責管理和播放角色的動畫幀序列
- **Animation Frame**: 動畫幀，單個靜態圖片，多個幀組成完整動畫
- **1P (Player 1)**: 第一位玩家，使用 Shirou 角色
- **2P (Player 2)**: 第二位玩家，使用 Sakura 角色
- **Texture Path**: 紋理路徑，指向資源文件的相對路徑字符串
- **Character Identifier**: 角色標識符，用於區分不同角色（如 "shirou", "sakura"）

## Requirements

### Requirement 1: 角色動畫資源定義

**User Story:** 作為開發者，我希望系統能夠明確定義每個角色的動畫幀資源路徑，以便在遊戲中正確加載和顯示角色動畫。

#### Acceptance Criteria

1. WHEN 系統初始化時 THEN the Animation System SHALL 為 Shirou 角色定義所有可用的動畫幀路徑
2. WHEN 系統初始化時 THEN the Animation System SHALL 為 Sakura 角色定義所有可用的動畫幀路徑
3. WHEN 查詢角色動畫幀時 THEN the Animation System SHALL 返回該角色的完整動畫幀路徑列表
4. WHEN 動畫幀路徑被定義時 THEN the Animation System SHALL 確保路徑格式為 "images/characters/{character_name}_idle{frame_number}.{extension}"

### Requirement 2: 玩家角色映射

**User Story:** 作為開發者，我希望系統能夠將玩家編號（1P/2P）映射到對應的角色資源，以便在多人遊戲中正確顯示不同玩家的角色。

#### Acceptance Criteria

1. WHEN 創建 1P 玩家實體時 THEN the Animation System SHALL 自動分配 Shirou 角色的動畫資源
2. WHEN 創建 2P 玩家實體時 THEN the Animation System SHALL 自動分配 Sakura 角色的動畫資源
3. WHEN 查詢玩家的紋理路徑時 THEN the Animation System SHALL 根據玩家編號返回對應角色的當前動畫幀路徑

### Requirement 3: 動畫幀訪問接口

**User Story:** 作為開發者，我希望通過統一的接口訪問角色的動畫幀，以便在渲染系統中使用。

#### Acceptance Criteria

1. WHEN 調用 get_texture_path() 方法時 THEN the Animation System SHALL 返回當前動畫幀的完整路徑字符串
2. WHEN 指定動畫幀索引時 THEN the Animation System SHALL 返回該索引對應的動畫幀路徑
3. WHEN 請求的動畫幀索引超出範圍時 THEN the Animation System SHALL 返回默認的第一幀路徑或錯誤信息

### Requirement 4: 動畫幀序列完整性

**User Story:** 作為開發者，我希望系統能夠驗證動畫幀序列的完整性，確保所有定義的資源文件都存在。

#### Acceptance Criteria

1. WHEN 系統加載動畫資源時 THEN the Animation System SHALL 驗證所有定義的動畫幀路徑對應的文件存在於 assets 目錄中
2. WHEN 發現缺失的動畫幀文件時 THEN the Animation System SHALL 記錄警告信息並使用占位符紋理
3. WHEN 所有動畫幀文件都存在時 THEN the Animation System SHALL 成功初始化並準備好播放動畫
