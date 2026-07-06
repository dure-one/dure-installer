//! Platform actor for GCP operations

mod actor;
mod commands;
mod events;

#[cfg(test)]
mod tests;

pub use actor::PlatformActor;
pub use commands::PlatformCommand;
pub use events::{PlatformEvent, VmInfo};
