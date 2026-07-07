// internet <-- api --> <-- calc --> ui

pub mod ehttp_cache;

#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
pub mod desktop;

#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]

#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
pub mod gcp;

// Nameserver API modules
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
pub mod ns_cloudflare;

#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
pub mod ns_duckdns;

#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
pub mod ns_gcp;

#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
pub mod ns_porkbun;
