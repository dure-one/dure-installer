// controller for api, db, android  <-- calc --> ui.

pub mod acme; // DNS provider utilities only (no CLI)
pub mod audit;
pub mod crypt;
pub mod db;
pub mod dns;
#[cfg(not(target_arch = "wasm32"))]
pub mod docker;
#[cfg(not(target_arch = "wasm32"))]
pub mod ansible;
pub mod dure_wss;
pub mod gcp;
pub mod hosting_gcp;
pub mod keyring;
pub mod ns;
pub mod platform;
pub mod platform_gcp;
pub mod profile;
pub mod session;
pub mod site;
pub mod ssh;
