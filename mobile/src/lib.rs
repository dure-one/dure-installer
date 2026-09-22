#![allow(clippy::float_cmp)]
#![allow(clippy::manual_range_contains)]
#![recursion_limit = "2048"]

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[cfg(not(target_arch = "wasm32"))]
use directories::ProjectDirs;

// Force linkage of libsqlite3-hotbundle for encryption support
#[cfg(not(target_arch = "wasm32"))]
extern crate libsqlite3_hotbundle;

// Core modules
pub mod api;
pub mod attestation;
pub mod calc;
pub mod i18n;
pub mod logging;
pub mod site;
pub mod storage;

// Android-specific modules
#[cfg(target_os = "android")]
pub mod android;

#[cfg(feature = "gui")]
pub mod ui_dlg;
#[cfg(feature = "gui")]
pub mod ui_components;

// Desktop-only modules (minimal implementations for CLI)
pub mod config;
#[cfg(not(target_arch = "wasm32"))]
pub mod config_migration;
// #[cfg(not(target_arch = "wasm32"))]
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
#[cfg(not(target_arch = "wasm32"))]
pub mod cli;
#[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
pub mod install;
#[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
pub mod install_stt;
#[cfg(all(
    feature = "tray-icon",
    not(any(target_os = "android", target_arch = "wasm32")),
    not(target_os = "openbsd")
))]
pub mod tray;
// HTTP server for OAuth callbacks (platform-specific: darkhttpd/winhttpd)
// TODO: Re-enable when HTTP server implementation is available
// #[cfg(not(target_arch = "wasm32"))]
// pub mod http_server;
// #[cfg(not(target_arch = "wasm32"))]
// pub mod attestation;
// #[cfg(not(target_arch = "wasm32"))]
// pub mod validation;
// #[cfg(not(target_arch = "wasm32"))]
// pub mod sync;
// #[cfg(not(target_arch = "wasm32"))]
// pub mod mcp;
// #[cfg(not(target_arch = "wasm32"))]
// pub mod output;
#[cfg(not(target_arch = "wasm32"))]
pub mod log_capture;

// Platform-specific entry points
#[cfg(all(target_os = "android", feature = "gui"))]
pub mod main_android;
#[cfg(target_arch = "wasm32")]
pub mod main_wasm;

/// Trait for platform-specific screen size detection
pub trait ScreenSizeProvider: Send + Sync {
    fn get_screen_size(&self) -> std::io::Result<(i32, i32)>;
}

/// Get the application config directory (~/.config/dure_installer)
///
/// Note: In multi-profile mode, this returns the base directory.
/// For profile-specific config, use ProfileContext::config_file
#[cfg(not(target_arch = "wasm32"))]
pub fn get_app_config_dir() -> Result<PathBuf> {
    #[cfg(not(target_os = "android"))]
    {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .context("Failed to get home directory")?;
        let config_dir = PathBuf::from(home).join(".config").join("dure_installer");
        Ok(config_dir)
    }

    #[cfg(target_os = "android")]
    {
        Ok(PathBuf::from("/data/data/app.dure.installer/files"))
    }
}

/// Get the profiles base directory (~/.config/dure_installer)
#[cfg(not(target_arch = "wasm32"))]
pub fn get_profiles_base_dir() -> Result<PathBuf> {
    // Test override for unit tests
    if let Ok(test_dir) = std::env::var("DURE_TEST_PROFILES_DIR") {
        return Ok(PathBuf::from(test_dir));
    }

    #[cfg(not(target_os = "android"))]
    {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .context("Failed to get home directory")?;
        let profiles_dir = PathBuf::from(home).join(".config").join("dure_installer");
        Ok(profiles_dir)
    }

    #[cfg(target_os = "android")]
    {
        Ok(PathBuf::from("/data/data/app.dure.installer/profiles"))
    }
}

/// Get config path for the current profile
///
/// Returns the config.yml path for the given profile, or an error if no profile is provided.
/// This enforces that config operations only happen within a profile context.
#[cfg(not(target_arch = "wasm32"))]
pub fn get_profile_config_path(profile: Option<&calc::profile::ProfileContext>) -> Result<PathBuf> {
    match profile {
        Some(ctx) => Ok(ctx.config_file.clone()),
        None => anyhow::bail!("No active profile - config operations require a profile to be selected"),
    }
}

/// Get the application cache directory (~/.cache/dure-installer)
#[cfg(not(target_arch = "wasm32"))]
pub fn get_app_cache_dir() -> Result<PathBuf> {
    #[cfg(not(target_os = "android"))]
    {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .context("Failed to get home directory")?;
        let cache_dir = PathBuf::from(home).join(".cache").join("dure-installer");
        Ok(cache_dir)
    }

    #[cfg(target_os = "android")]
    {
        Ok(PathBuf::from("/data/data/app.dure.installer/cache"))
    }
}

/// Application directory paths configuration
#[derive(Debug, Clone)]
pub struct Config {
    pub config_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub tmp_dir: PathBuf,
    pub data_dir: PathBuf,
    #[cfg(not(target_arch = "wasm32"))]
    pub app_config: config::AppConfig,
}

impl Config {
    /// Create a new Config instance
    ///
    /// # Arguments
    /// * `profile_dir` - Optional profile directory path for per-profile config
    ///   If None, uses global config directory (~/.config/dure_installer)
    ///   If Some, loads config from profile directory ({profile}/config.yml)
    pub fn new(profile_dir: Option<PathBuf>) -> Result<Self> {
        #[cfg(target_os = "android")]
        {
            // Android-specific paths
            let base_dir = PathBuf::from("/data/data/app.dure.installer");
            let config_dir = profile_dir.unwrap_or_else(|| base_dir.join("files"));
            let cache_dir = base_dir.join("cache");

            dure_info!(
                "Android config paths - config_dir: {:?}, cache_dir: {:?}",
                config_dir,
                cache_dir
            );

            let tmp_dir = cache_dir.join("tmp");
            let data_dir = cache_dir.join("data");

            // Create directories if they don't exist
            for dir in [&config_dir, &cache_dir, &tmp_dir, &data_dir] {
                match fs::create_dir_all(dir) {
                    Ok(()) => dure_info!("Successfully created directory: {:?}", dir),
                    Err(e) => dure_error!("Failed to create directory: {:?} - Error: {}", dir, e),
                }
            }

            // Load configuration
            let config_file = config_dir.join("config.yml");
            let app_config = config::AppConfig::load_or_default(&config_file);

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
            dure_info!("WASM config - no filesystem access");

            Ok(Config {
                config_dir: PathBuf::new(),
                cache_dir: PathBuf::new(),
                tmp_dir: PathBuf::new(),
                data_dir: PathBuf::new(),
            })
        }

        #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
        {
            // Desktop platforms (Linux, Windows, macOS)
            let config_dir = if let Some(profile_path) = profile_dir {
                // Use profile-specific config directory
                profile_path
            } else {
                // Use global config directory
                get_app_config_dir()?
            };

            let cache_dir = get_app_cache_dir()?;

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
                    dure_info!("Created config file: {:?}", config_file);
                }
            }

            // Load configuration
            let app_config = config::AppConfig::load_or_default(&config_file);
            dure_info!("Loaded config from: {:?}", config_file);

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
