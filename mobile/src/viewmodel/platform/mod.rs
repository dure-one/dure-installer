//! Platform actor for GCP operations

mod actor;
mod commands;
mod drawer_repository;
mod events;

#[cfg(test)]
mod tests;

pub use actor::PlatformActor;
pub use commands::{DeleteOptions, PlatformCommand};
pub use drawer_repository::DrawerRepository;
pub use events::{PlatformEvent, VmInfo, VmStatus, FirewallStatus, SshStatus};
