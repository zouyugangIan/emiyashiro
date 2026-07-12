pub mod asset_paths;
pub mod components;
#[cfg(feature = "server")]
pub mod database;
pub mod events;
pub mod plugins;
pub mod protocol;
pub mod resources;
pub mod states;
pub mod systems;

#[cfg(test)]
mod tests;

// Re-export common types if needed
pub use components::*;
pub use events::*;
pub use resources::*;
pub use states::*;

#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        {
            if cfg!(debug_assertions) {
            println!($($arg)*);
            }
        }
    };
}
