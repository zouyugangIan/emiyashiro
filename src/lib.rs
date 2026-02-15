pub mod asset_paths;
pub mod components;
#[cfg(feature = "server")]
pub mod database;
#[cfg(not(feature = "server"))]
pub mod database {
    // Mock database module for client
    #[derive(Clone)]
    pub struct Database;
}
pub mod events;
pub mod protocol;
pub mod resources;
pub mod states;
pub mod systems;
pub mod tools;

#[cfg(test)]
mod tests;

// Re-export common types if needed
pub use components::*;
pub use events::*;
pub use resources::*;
pub use states::*;
