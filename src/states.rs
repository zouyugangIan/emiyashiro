use bevy::prelude::*;

/// 游戏状态枚举
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Menu,        // 主菜单/封面
    Playing,     // 游戏中
    Paused,      // 暂停
    SaveDialog,  // 存档对话框
    LoadTable,   // 加载表格
}

/// 角色选择状态
#[derive(Resource, Default)]
pub struct CharacterSelection {
    pub selected_character: CharacterType,
}

/// 角色类型
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum CharacterType {
    #[default]
    Shirou1,   // 士郎角色1
    Shirou2,   // 士郎角色2
}

impl CharacterType {
    pub fn get_texture_path(&self) -> &'static str {
        match self {
            CharacterType::Shirou1 => "images/characters/shirou_idle1.jpg",
            CharacterType::Shirou2 => "images/characters/shirou_idle2.jpg",
        }
    }
}