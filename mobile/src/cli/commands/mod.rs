// Minimal CLI commands for DNS functionality

pub mod audit;
pub mod crypt;
pub mod dns;
pub mod keyring;
pub mod ns;
pub mod platform;
#[cfg(feature = "gui")]
pub mod platform_vm; // ViewModel-based async platform VM commands
pub mod site;
pub mod ssh;
