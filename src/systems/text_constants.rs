//! 统一的英文文本常量系统
//! 包含所有 UI 界面的英文文本，确保一致性和易于维护

/// 主菜单文本常量
pub struct MainMenuText;

impl MainMenuText {
    pub const TITLE: &'static str = "Fate/stay night Heaven's Feel - Shirou Runner";
    pub const START_GAME: &'static str = "Start Game";
    pub const LOAD_GAME: &'static str = "Load Game";
    pub const SETTINGS: &'static str = "Settings";
    pub const EXIT: &'static str = "Exit";
    pub const CHARACTER_SELECT: &'static str = "Select Character";
    pub const SHIROU1: &'static str = "Shirou (Default)";
    pub const SHIROU2: &'static str = "Shirou (Alternative)";
}

/// 暂停菜单文本常量
pub struct PauseMenuText;

impl PauseMenuText {
    pub const TITLE: &'static str = "Game Paused";
    pub const RESUME_GAME: &'static str = "Resume Game";
    pub const SAVE_GAME: &'static str = "Save Game";
    pub const LOAD_GAME: &'static str = "Load Game";
    pub const MAIN_MENU: &'static str = "Main Menu";
    pub const ESC_RESUME: &'static str = "ESC: Resume";
    pub const Q_MAIN_MENU: &'static str = "Q: Main Menu";
    pub const CONTROLS_HINT: &'static str = "WASD/Arrow Keys: Move | ESC: Pause";
}

/// 存档/读档系统文本常量
pub struct SaveLoadText;

impl SaveLoadText {
    // 对话框标题
    pub const SAVE_DIALOG_TITLE: &'static str = "Save Game";
    pub const LOAD_DIALOG_TITLE: &'static str = "Load & Manage Saves";
    pub const RENAME_DIALOG_TITLE: &'static str = "Rename Save";

    // 按钮标签
    pub const SAVE_BUTTON: &'static str = "Save";
    pub const LOAD_BUTTON: &'static str = "Load";
    pub const CANCEL_BUTTON: &'static str = "Cancel";
    pub const CONFIRM_BUTTON: &'static str = "Confirm";
    pub const BACK_BUTTON: &'static str = "Back";
    pub const REFRESH_BUTTON: &'static str = "Refresh";
    pub const RENAME_BUTTON: &'static str = "Rename";
    pub const DELETE_BUTTON: &'static str = "Delete";

    // 表格列标题
    pub const COL_NAME: &'static str = "Name";
    pub const COL_PLAYERS: &'static str = "Players";
    pub const COL_SCORE: &'static str = "Score";
    pub const COL_DISTANCE: &'static str = "Distance";
    pub const COL_TIME: &'static str = "Time";
    pub const COL_DATE: &'static str = "Date";
    pub const COL_ACTIONS: &'static str = "Actions";

    // 输入提示
    pub const ENTER_SAVE_NAME: &'static str = "Enter save name:";
    pub const ENTER_NEW_NAME: &'static str = "Enter new name:";
    pub const NAME_PLACEHOLDER: &'static str = "Enter name...";
    pub const INPUT_HINT: &'static str =
        "Use A-Z, 0-9, space, and hyphen. Max 25 characters. Existing names overwrite.";

    // 状态消息
    pub const NO_SAVES_FOUND: &'static str = "No save files found";
    pub const SAVE_SUCCESS: &'static str = "Game saved successfully";
    pub const LOAD_SUCCESS: &'static str = "Game loaded successfully";
    pub const RENAME_SUCCESS: &'static str = "Save renamed successfully";
    pub const DELETE_SUCCESS: &'static str = "Save deleted successfully";
    pub const SCANNING_SAVES: &'static str = "Scanning save files...";

    // 错误消息
    pub const SAVE_ERROR: &'static str = "Failed to save game";
    pub const LOAD_ERROR: &'static str = "Failed to load game";
    pub const RENAME_ERROR: &'static str = "Failed to rename save";
    pub const DELETE_ERROR: &'static str = "Failed to delete save";
    pub const NAME_EXISTS_ERROR: &'static str = "Save name already exists";
    pub const INVALID_NAME_ERROR: &'static str = "Invalid save name";
    pub const FILE_NOT_FOUND_ERROR: &'static str = "Save file not found";
    pub const PERMISSION_DENIED_ERROR: &'static str = "Permission denied";
    pub const CORRUPTED_FILE_ERROR: &'static str = "Save file is corrupted";

    // 操作指引
    pub const CLICK_TO_LOAD: &'static str = "Click on a save file to load it";
    pub const MANAGE_SAVES_HINT: &'static str = "Use the action buttons to rename or delete saves";
    pub const CONFIRM_DELETE: &'static str = "Are you sure you want to delete this save?";

    // 默认名称
    pub const DEFAULT_SAVE_NAME: &'static str = "DefaultSave";

    // 玩家数量指示器
    pub const PLAYER_1P: &'static str = "1P";
    pub const PLAYER_2P: &'static str = "2P";
}

/// 游戏内HUD文本常量
pub struct GameHUDText;

impl GameHUDText {
    pub const SCORE_LABEL: &'static str = "Score: ";
    pub const DISTANCE_LABEL: &'static str = "Distance: ";
    pub const TIME_LABEL: &'static str = "Time: ";
    pub const JUMPS_LABEL: &'static str = "Jumps: ";
    pub const METERS_UNIT: &'static str = "m";
    pub const SECONDS_UNIT: &'static str = "s";
}
