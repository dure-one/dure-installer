// internet <-- api --> <-- calc --> ui

pub mod ehttp_cache;

// Desktop-only (X11 desktop environment detection)
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
pub mod desktop;

// Platform-independent HTTP APIs
pub mod gcp;
pub mod ns_cloudflare;
pub mod ns_duckdns;
pub mod ns_gcp;
pub mod ns_porkbun;
