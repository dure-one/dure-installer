// Minimal CLI commands for DNS and ACME functionality

pub mod acme;
pub mod audit;
pub mod crypt;
pub mod dns;
pub mod hosting;
pub mod keyring;
pub mod nft;
pub mod ns;
pub mod platform;
#[cfg(feature = "gui")]
pub mod platform_vm; // ViewModel-based async platform VM commands
pub mod site;
pub mod ssh;
pub mod webhook;
pub mod wss;
