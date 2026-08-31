// controller for api, db, android  <-- calc --> ui.

#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
pub mod acme; // DNS provider utilities only (no CLI)
pub mod audit;
pub mod crypt;
pub mod db;
pub mod dns;
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
pub mod docker;
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
pub mod ansible;
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
pub mod dure_wss;
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
pub mod gcp;
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
pub mod hosting_gcp;
pub mod keyring;
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
pub mod ns;
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
pub mod platform;
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
pub mod platform_gcp;
pub mod session;
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
pub mod site;
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
pub mod ssh;
