//! Desktop integration utilities
//!
//! This module provides platform-specific desktop integrations.
//! - Desktop: Native file browser, system tray support
//! - Android: Mobile-specific UI integrations
//! - WASM: Browser-based integrations

#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
use std::process::Command;

/// Detect the desktop environment (Linux only)
#[cfg(target_os = "linux")]
fn get_desktop_environment() -> String {
    // Check common environment variables
    if let Ok(de) = std::env::var("XDG_CURRENT_DESKTOP") {
        return de.to_lowercase();
    }
    if let Ok(de) = std::env::var("DESKTOP_SESSION") {
        return de.to_lowercase();
    }

    // Try to detect from running processes
    let processes = [
        "gnome-shell",
        "plasma",
        "xfce4-session",
        "mate-session",
        "cinnamon",
    ];
    for process in &processes {
        if let Ok(output) = Command::new("pgrep").arg("-x").arg(process).output() {
            if output.status.success() && !output.stdout.is_empty() {
                return process
                    .trim_end_matches("-session")
                    .trim_end_matches("-shell")
                    .to_string();
            }
        }
    }

    "unknown".to_string()
}

/// Get the user who owns the X session
#[cfg(target_os = "linux")]
fn get_x_session_user() -> Option<String> {
    use std::os::unix::fs::MetadataExt;

    // Get current display from DISPLAY env var (e.g., ":0", ":1")
    let display = std::env::var("DISPLAY").unwrap_or_else(|_| ":0".to_string());
    let display_num = display
        .trim_start_matches(':')
        .split('.')
        .next()
        .unwrap_or("0");

    // Check X11 socket owner
    let x11_socket = format!("/tmp/.X11-unix/X{}", display_num);
    if let Ok(metadata) = std::fs::metadata(&x11_socket) {
        let uid = metadata.uid();

        // Get username from UID
        if let Ok(output) = Command::new("id").arg("-un").arg(uid.to_string()).output() {
            if output.status.success() {
                return Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
            }
        }
    }

    // Fallback: check XAUTHORITY file owner
    if let Ok(xauth) = std::env::var("XAUTHORITY") {
        if let Ok(metadata) = std::fs::metadata(&xauth) {
            let uid = metadata.uid();
            if let Ok(output) = Command::new("id").arg("-un").arg(uid.to_string()).output() {
                if output.status.success() {
                    return Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
                }
            }
        }
    }

    None
}

/// Check if desktop user and runtime user are different (Linux with X11)
/// Returns (current_user, desktop_user, is_different)
#[cfg(target_os = "linux")]
pub fn check_user_mismatch() -> (String, String, bool) {
    let current_user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string());

    let desktop_user = get_x_session_user().unwrap_or_else(|| "unknown".to_string());

    let is_different =
        current_user != desktop_user && desktop_user != "unknown" && current_user != "unknown";

    (current_user, desktop_user, is_different)
}

/// Check if desktop user and runtime user are different (Windows/macOS stub)
#[cfg(any(target_os = "windows", target_os = "macos"))]
pub fn check_user_mismatch() -> (String, String, bool) {
    let current_user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string());
    (current_user.clone(), current_user, false)
}

/// Check if desktop user and runtime user are different (Android/WASM stub)
#[cfg(any(target_os = "android", target_arch = "wasm32"))]
pub fn check_user_mismatch() -> (String, String, bool) {
    ("n/a".to_string(), "n/a".to_string(), false)
}

/// Check if desktop user and runtime user are different (OpenBSD)
#[cfg(target_os = "openbsd")]
pub fn check_user_mismatch() -> (String, String, bool) {
    let current_user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string());
    (current_user.clone(), current_user, false)
}
