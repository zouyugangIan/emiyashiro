/// English text constants for the save/load system
pub struct SaveLoadText;

impl SaveLoadText {
    // Dialog titles
    pub const SAVE_DIALOG_TITLE: &'static str = "Save Game";
    pub const LOAD_DIALOG_TITLE: &'static str = "Load & Manage Saves";
    pub const RENAME_DIALOG_TITLE: &'static str = "Rename Save";
    
    // Button labels
    pub const SAVE_BUTTON: &'static str = "Save";
    pub const CANCEL_BUTTON: &'static str = "Cancel";
    pub const CONFIRM_BUTTON: &'static str = "Confirm";
    pub const BACK_BUTTON: &'static str = "Back";
    pub const REFRESH_BUTTON: &'static str = "Refresh";
    pub const RENAME_BUTTON: &'static str = "Rename";
    pub const DELETE_BUTTON: &'static str = "Delete";
    
    // Column headers
    pub const COL_NAME: &'static str = "Name";
    pub const COL_SCORE: &'static str = "Score";
    pub const COL_DISTANCE: &'static str = "Distance";
    pub const COL_TIME: &'static str = "Time";
    pub const COL_DATE: &'static str = "Date";
    pub const COL_ACTIONS: &'static str = "Actions";
    
    // Input prompts
    pub const ENTER_SAVE_NAME: &'static str = "Enter save name:";
    pub const ENTER_NEW_NAME: &'static str = "Enter new name:";
    pub const NAME_PLACEHOLDER: &'static str = "Enter name...";
    
    // Messages
    pub const NO_SAVES_FOUND: &'static str = "No save files found";
    pub const SAVE_SUCCESS: &'static str = "Game saved successfully";
    pub const LOAD_SUCCESS: &'static str = "Game loaded successfully";
    pub const RENAME_SUCCESS: &'static str = "Save renamed successfully";
    pub const DELETE_SUCCESS: &'static str = "Save deleted successfully";
    
    // Error messages
    pub const SAVE_ERROR: &'static str = "Failed to save game";
    pub const LOAD_ERROR: &'static str = "Failed to load game";
    pub const RENAME_ERROR: &'static str = "Failed to rename save";
    pub const DELETE_ERROR: &'static str = "Failed to delete save";
    pub const NAME_EXISTS_ERROR: &'static str = "Save name already exists";
    pub const INVALID_NAME_ERROR: &'static str = "Invalid save name";
    
    // Instructions
    pub const CLICK_TO_LOAD: &'static str = "Click on a save file to load it";
    pub const MANAGE_SAVES_HINT: &'static str = "Use the action buttons to rename or delete saves";
    
    // Default names
    pub const DEFAULT_SAVE_NAME: &'static str = "DefaultSave";
}