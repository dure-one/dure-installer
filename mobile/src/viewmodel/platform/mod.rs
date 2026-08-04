//! Platform actor for GCP operations

mod actor;
mod commands;
mod events;

#[cfg(test)]
mod tests;

pub use actor::PlatformActor;
pub use commands::{DeleteOptions, PlatformCommand};
pub use events::{PlatformEvent, VmInfo, VmStatus, FirewallStatus, SshStatus};
