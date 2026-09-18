//! Platform actor for GCP operations

mod actor;
mod commands;
mod drawer_actor;
mod drawer_repository;
mod drawer_types;
mod events;

#[cfg(test)]
mod tests;

pub use actor::PlatformActor;
pub use commands::{DeleteOptions, PlatformCommand};
pub use drawer_actor::DrawerActor;
pub use drawer_repository::DrawerRepository;
pub use drawer_types::{DrawerCommand, DrawerEvent, DrawerState, DrawerTab};
pub use events::{PlatformEvent, VmInfo, VmStatus, FirewallStatus, SshStatus};
