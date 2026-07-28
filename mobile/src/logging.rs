//! Dure logging infrastructure with automatic module prefixes
//!
//! Provides custom logging macros that automatically inject module context
//! and format logs consistently across all platforms.

/// Transform Rust module path into clean component name
///
/// Examples:
/// - "dure::api::gcp::oauth" -> "GCP::OAuth"
/// - "dure::calc::db" -> "DB"
/// - "dure::ui_tabs::ssh" -> "UI::SSH"
/// - "dure::viewmodel::platform::actor" -> "Platform::Actor"
pub fn module_name(path: &str) -> String {
    // Strip "dure::" prefix if present
    let path = path.strip_prefix("dure::").unwrap_or(path);

    // Handle special cases first
    if path.starts_with("api::gcp::") {
        // api::gcp::oauth -> GCP::OAuth
        let component = path.strip_prefix("api::gcp::").unwrap();
        return format!("GCP::{}", capitalize_component(component));
    }

    if path.starts_with("ui_tabs::") {
        // ui_tabs::ssh -> UI::SSH
        let component = path.strip_prefix("ui_tabs::").unwrap();
        return format!("UI::{}", component.to_uppercase());
    }

    if path.starts_with("viewmodel::") {
        // viewmodel::platform::actor -> Platform::Actor
        let rest = path.strip_prefix("viewmodel::").unwrap();
        let parts: Vec<&str> = rest.split("::").collect();
        if parts.len() >= 2 {
            return format!("{}::{}", capitalize_component(parts[0]), capitalize_component(parts[1]));
        }
        return capitalize_component(rest);
    }

    if path.starts_with("wss::") {
        // wss::server -> WSS::Server
        let component = path.strip_prefix("wss::").unwrap();
        return format!("WSS::{}", capitalize_component(component));
    }

    // Handle calc:: (business logic)
    if path.starts_with("calc::") {
        // calc::db -> DB
        let component = path.strip_prefix("calc::").unwrap();
        return component.to_uppercase();
    }

    // Handle api:: (non-GCP)
    if path.starts_with("api::") {
        // api::ehttp_cache -> API::EhttpCache
        let component = path.strip_prefix("api::").unwrap();
        return format!("API::{}", capitalize_component(component));
    }

    // Handle android::
    if path.starts_with("android::") {
        return "Android".to_string();
    }

    // Handle attestation::
    if path.starts_with("attestation::") {
        return "Attestation".to_string();
    }

    // Handle site::
    if path.starts_with("site::") {
        return "Site".to_string();
    }

    // Fallback: use last 2 components or capitalize single component
    let parts: Vec<&str> = path.split("::").collect();
    if parts.len() >= 2 {
        format!("{}::{}",
                capitalize_component(parts[parts.len() - 2]),
                capitalize_component(parts[parts.len() - 1]))
    } else if parts.len() == 1 {
        capitalize_component(parts[0])
    } else {
        "Unknown".to_string()
    }
}

/// Capitalize a component name (handle acronyms and snake_case)
fn capitalize_component(s: &str) -> String {
    // Handle known acronyms
    match s {
        "gcp" => "GCP".to_string(),
        "wss" => "WSS".to_string(),
        "ssh" => "SSH".to_string(),
        "db" => "DB".to_string(),
        "dns" => "DNS".to_string(),
        "api" => "API".to_string(),
        "oauth" => "OAuth".to_string(),
        "vm" => "VM".to_string(),
        "ui" => "UI".to_string(),
        _ => {
            // Handle snake_case: ehttp_cache -> EhttpCache
            s.split('_')
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().chain(chars).collect(),
                    }
                })
                .collect::<Vec<_>>()
                .join("")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_name_gcp() {
        assert_eq!(module_name("dure::api::gcp::oauth"), "GCP::OAuth");
        assert_eq!(module_name("dure::api::gcp::compute"), "GCP::Compute");
    }

    #[test]
    fn test_module_name_calc() {
        assert_eq!(module_name("dure::calc::db"), "DB");
        assert_eq!(module_name("dure::calc::ssh"), "SSH");
    }

    #[test]
    fn test_module_name_ui() {
        assert_eq!(module_name("dure::ui_tabs::ssh"), "UI::SSH");
        assert_eq!(module_name("dure::ui_tabs::platform"), "UI::PLATFORM");
    }

    #[test]
    fn test_module_name_viewmodel() {
        assert_eq!(module_name("dure::viewmodel::platform::actor"), "Platform::Actor");
    }

    #[test]
    fn test_module_name_wss() {
        assert_eq!(module_name("dure::wss::server"), "WSS::Server");
        assert_eq!(module_name("dure::wss::client"), "WSS::Client");
    }

    #[test]
    fn test_module_name_android() {
        assert_eq!(module_name("dure::android::activity"), "Android");
    }
}

// Non-WASM platforms: use standard log crate
#[cfg(not(target_family = "wasm"))]
#[macro_export]
macro_rules! dure_info {
    ($($arg:tt)*) => {
        log::info!("[{}]\t{}", $crate::logging::module_name(module_path!()), format!($($arg)*))
    };
}

#[cfg(not(target_family = "wasm"))]
#[macro_export]
macro_rules! dure_debug {
    ($($arg:tt)*) => {
        log::debug!("[{}]\t{}", $crate::logging::module_name(module_path!()), format!($($arg)*))
    };
}

#[cfg(not(target_family = "wasm"))]
#[macro_export]
macro_rules! dure_warn {
    ($($arg:tt)*) => {
        log::warn!("[{}]\t{}", $crate::logging::module_name(module_path!()), format!($($arg)*))
    };
}

#[cfg(not(target_family = "wasm"))]
#[macro_export]
macro_rules! dure_error {
    ($($arg:tt)*) => {
        log::error!("[{}]\t{}", $crate::logging::module_name(module_path!()), format!($($arg)*))
    };
}

// WASM platform: use web_sys::console
#[cfg(target_family = "wasm")]
#[macro_export]
macro_rules! dure_info {
    ($($arg:tt)*) => {
        web_sys::console::log_1(
            &format!("[{}]\t{}", $crate::logging::module_name(module_path!()), format!($($arg)*)).into()
        )
    };
}

#[cfg(target_family = "wasm")]
#[macro_export]
macro_rules! dure_debug {
    ($($arg:tt)*) => {
        // In WASM, debug logs still go to console (user controls via browser devtools)
        web_sys::console::debug_1(
            &format!("[{}]\t{}", $crate::logging::module_name(module_path!()), format!($($arg)*)).into()
        )
    };
}

#[cfg(target_family = "wasm")]
#[macro_export]
macro_rules! dure_warn {
    ($($arg:tt)*) => {
        web_sys::console::warn_1(
            &format!("[{}]\t{}", $crate::logging::module_name(module_path!()), format!($($arg)*)).into()
        )
    };
}

#[cfg(target_family = "wasm")]
#[macro_export]
macro_rules! dure_error {
    ($($arg:tt)*) => {
        web_sys::console::error_1(
            &format!("[{}]\t{}", $crate::logging::module_name(module_path!()), format!($($arg)*)).into()
        )
    };
}
