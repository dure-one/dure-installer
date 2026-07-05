//! Platform actor for GCP operations

mod commands;
mod events;
mod actor;

#[cfg(test)]
mod tests;

pub use commands::PlatformCommand;
pub use events::{PlatformEvent, VmInfo};
pub use actor::PlatformActor;
