use crate::resources::CompleteGameState;
use bevy::prelude::*;

/// Event to request saving the game state to a file with the given name.
#[derive(Message, Clone)]
pub struct StartSaveGame {
    pub save_name: String,
    pub state: CompleteGameState,
}

/// Event to request loading the game state from the given file path.
#[derive(Message, Clone)]
pub struct StartLoadGame {
    pub file_path: String,
}
