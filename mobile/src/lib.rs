#![allow(clippy::float_cmp)]
#![allow(clippy::manual_range_contains)]
#![recursion_limit = "2048"]

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
use directories::ProjectDirs;

// Core modules
pub mod api;
pub mod attestation;
pub mod calc;
pub mod i18n;
pub mod site;
pub mod storage;
#[cfg(feature = "gui")]
pub mod ui_dlg;

// Desktop-only modules (minimal implementations for CLI)
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
pub mod config;
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
pub mod config_migration;
// #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
// pub mod error;

/// No-op macro replacing egui's demo github file link widget.
#[macro_export]
macro_rules! egui_github_link_file {
    () => {
        egui::Label::new("")
    };
}

// Export modules for external use (GUI-only)
#[cfg(feature = "gui")]
pub use dure::DureApp as GuiApp;
#[cfg(feature = "gui")]
pub mod dure;
#[cfg(feature = "gui")]
pub mod dure_stt;
#[cfg(feature = "gui")]
pub mod ui_tabs;
#[cfg(feature = "gui")]
pub mod viewmodel;

// Desktop-only modules
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
pub mod cli;
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
pub mod install;
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
pub mod install_stt;
#[cfg(all(
    feature = "gui",
    not(any(target_os = "android", target_arch = "wasm32")),
    not(target_os = "openbsd")
))]
pub mod tray;
// WSS server/client (HTTPS + WebSocket Secure)
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
pub mod wss;
// HTTP server for OAuth callbacks (platform-specific: darkhttpd/winhttpd)
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
pub mod http_server;
// #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
// pub mod attestation;
// #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
// pub mod validation;
// #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
// pub mod sync;
// #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
// pub mod mcp;
// #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
// pub mod output;
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
pub mod log_capture;

// Platform-specific entry points
#[cfg(target_os = "android")]
pub mod main_android;
#[cfg(target_arch = "wasm32")]
pub mod main_wasm;

/// Trait for platform-specific screen size detection
pub trait ScreenSizeProvider: Send + Sync {
    fn get_screen_size(&self) -> std::io::Result<(i32, i32)>;
}

/// Application directory paths configuration
#[derive(Debug, Clone)]
pub struct Config {
    pub config_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub tmp_dir: PathBuf,
    pub data_dir: PathBuf,
    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    pub app_config: config::AppConfig,
}

impl Config {
    pub fn new() -> Result<Self> {
        #[cfg(target_os = "android")]
        {
            // Android-specific paths
            let config_dir = PathBuf::from("/data/data/pe.nikescar.dure/files");
            let cache_dir = PathBuf::from("/data/data/pe.nikescar.dure/cache");

            log::info!(
                "Android config paths - config_dir: {:?}, cache_dir: {:?}",
                config_dir,
                cache_dir
            );

            let tmp_dir = cache_dir.join("tmp");
            let data_dir = cache_dir.join("data");

            // Create directories if they don't exist
            for dir in [&config_dir, &cache_dir, &tmp_dir, &data_dir] {
                match fs::create_dir_all(dir) {
                    Ok(()) => log::info!("Successfully created directory: {:?}", dir),
                    Err(e) => log::error!("Failed to create directory: {:?} - Error: {}", dir, e),
                }
            }

            let app_config = config::AppConfig::default();

            Ok(Config {
                config_dir: config_dir.clone(),
                cache_dir,
                tmp_dir,
                data_dir,
                app_config,
            })
        }

        #[cfg(target_arch = "wasm32")]
        {
            // WASM: No filesystem, return empty paths
            log::info!("WASM config - no filesystem access");

            Ok(Config {
                config_dir: PathBuf::new(),
                cache_dir: PathBuf::new(),
                tmp_dir: PathBuf::new(),
                data_dir: PathBuf::new(),
            })
        }

        #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
        {
            // Desktop platforms (Linux, Windows, macOS)
            let proj_dirs = ProjectDirs::from("pe", "nikescar", "dure")
                .context("Failed to get project directories")?;

            let config_dir = proj_dirs.config_dir().to_path_buf();
            let cache_dir = proj_dirs.cache_dir().to_path_buf();

            let tmp_dir = cache_dir.join("tmp");
            let data_dir = cache_dir.join("data");

            // Create directories if they don't exist
            fs::create_dir_all(&config_dir)?;
            fs::create_dir_all(&cache_dir)?;
            fs::create_dir_all(&tmp_dir)?;
            fs::create_dir_all(&data_dir)?;

            // Copy config.example.yml if config.yml doesn't exist
            let config_file = config_dir.join("config.yml");
            if !config_file.exists() {
                // Try to find config.example.yml in current directory or parent
                let example_config = PathBuf::from("mobile/config.example.yml");
                if example_config.exists() {
                    fs::copy(&example_config, &config_file)?;
                    log::info!("Created config file: {:?}", config_file);
                }
            }

            // Load configuration
            let app_config = config::AppConfig::load_or_default(&config_file);

            Ok(Config {
                config_dir: config_dir.clone(),
                cache_dir,
                tmp_dir,
                data_dir,
                app_config,
            })
        }
    }
}

/// Log level enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LogLevel {
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub show_logs: bool,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default = "default_theme_mode")]
    pub theme_mode: String,
    #[serde(default = "default_display_size")]
    pub display_size: String,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default)]
    pub font_path: String,
    #[serde(default)]
    pub override_text_style: String,
    #[serde(default = "default_theme_name")]
    pub theme_name: String,
    #[serde(default)]
    pub autoupdate: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            show_logs: false,
            log_level: default_log_level(),
            theme_mode: default_theme_mode(),
            display_size: default_display_size(),
            language: default_language(),
            font_path: String::new(),
            override_text_style: String::new(),
            theme_name: default_theme_name(),
            autoupdate: false,
        }
    }
}

fn default_language() -> String {
    "Auto".to_string()
}

fn default_log_level() -> String {
    "Error".to_string()
}

fn default_theme_mode() -> String {
    "Auto".to_string()
}

fn default_display_size() -> String {
    "Desktop (1024x768)".to_string()
}

fn default_theme_name() -> String {
    "default".to_string()
}
