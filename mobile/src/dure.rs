//! Dure eframe UI (Cross-platform: Desktop, Android, WASM)
//!
//! This module provides the main eframe application UI that works across all platforms.
//! Platform-specific functionality is injected via traits.

use crate::{dure_info, dure_debug, dure_trace, dure_warn, dure_error};
#[cfg(not(target_arch = "wasm32"))]
use crate::api::desktop::check_user_mismatch;
#[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
use crate::install;
use crate::ui_dlg::DlgSettings;
use crate::{Config, Settings};

// ViewModel (MVVM architecture)
use crate::viewmodel::ViewModel;

// Desktop-only imports
use eframe::egui;
use eframe::egui::Color32;
use egui::pos2;
use egui_i18n::tr;
// Theme loading is done in main.rs, not here
use egui_material3::*;
use log::info;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

// Static variables for menu and search toggles
pub static MENU_TOGGLE: AtomicBool = AtomicBool::new(false);
pub static SEARCH_TOGGLE: AtomicBool = AtomicBool::new(false);
static SETTINGS_TOGGLE: AtomicBool = AtomicBool::new(false);
static ABOUT_TOGGLE: AtomicBool = AtomicBool::new(false);

// Static flags for menu actions
static MENU_SHOW_APP: AtomicBool = AtomicBool::new(false);
static MENU_CACHE_DIR: AtomicBool = AtomicBool::new(false);
static MENU_NEXT_MARKET: AtomicBool = AtomicBool::new(false);
static MENU_KEEP_CURRENT: AtomicBool = AtomicBool::new(false);
static MENU_BLACKLIST_CURRENT: AtomicBool = AtomicBool::new(false);
static MENU_RANDOM_FAVORITE: AtomicBool = AtomicBool::new(false);
static MENU_INSTALL: AtomicBool = AtomicBool::new(false);
static MENU_QUIT: AtomicBool = AtomicBool::new(false);

/// Main Dure application state
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct DureApp {
    pub title: String,
    pub title_bar: bool,
    pub collapsible: bool,
    pub resizable: bool,
    pub constrain: bool,
    pub anchored: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub anchor: egui::Align2,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub anchor_offset: egui::Vec2,

    #[cfg_attr(feature = "serde", serde(skip))]
    pub config: Option<Config>,

    // Settings
    pub settings: Settings,

    // Dialog states
    pub dlg_settings: DlgSettings,
    pub dlg_about: crate::ui_dlg::DlgAbout,
    pub dlg_profile_login: crate::ui_dlg::DlgProfileLogin,
    pub dlg_profile_create: crate::ui_dlg::DlgProfileCreate,
    pub dlg_profile_delete: crate::ui_dlg::DlgProfileDelete,

    // Profile state
    #[cfg_attr(feature = "serde", serde(skip))]
    pub current_profile: Option<crate::calc::profile::ProfileContext>,
    pub pending_profile_name: Option<String>,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub current_profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,

    // Installation status (desktop only)
    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
    pub install_status: crate::install_stt::InstallStatus,
    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
    pub install_dialog_open: bool,
    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
    pub install_message: String,
    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
    pub install_in_progress: bool,

    // Update status
    pub update_status: String,
    pub update_available: bool,
    pub update_checking: bool,
    pub update_download_url: String,
    pub update_current_version: String,
    pub update_latest_version: String,

    // User mismatch warning (desktop only)
    #[cfg_attr(feature = "serde", serde(skip))]
    pub user_mismatch_warning: Option<String>,

    // Screen size state
    #[cfg_attr(feature = "serde", serde(skip))]
    pub screen_size_provider: Option<Arc<dyn crate::ScreenSizeProvider + Send + Sync>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub cached_screen_size: Option<(f32, f32)>,
    pub screen_size_failed: bool,
    pub screen_ratio: f32,

    // Rectangle overlay state
    pub square_size_factor: f32,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub square_center: egui::Pos2,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub square_corners: [egui::Pos2; 4],

    // HTTP cache
    #[cfg_attr(feature = "serde", serde(skip))]
    pub ehttp_cache: Option<Arc<crate::api::ehttp_cache::EhttpCache>>,

    // ViewModel (MVVM architecture)
    #[cfg_attr(feature = "serde", serde(skip))]
    pub viewmodel: Option<ViewModel>,

    // Tabs state (infrastructure management only)
    pub active_tab: crate::ui_tabs::Tab,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub previous_tab: Option<crate::ui_tabs::Tab>,
    pub scrolling_selected: usize,
    pub tab_platform: crate::ui_tabs::platform::PlatformTab,
    pub tab_ssh: crate::ui_tabs::ssh::SshTab,
    pub tab_ns: crate::ui_tabs::ns::NsTab,
    pub tab_site: crate::ui_tabs::site::SiteTab,
}

impl Default for DureApp {
    fn default() -> Self {
        // Do NOT load config at startup - wait for profile selection
        let config: Option<Config> = None;
        info!("App starting without config - waiting for profile selection");

        // Check for user mismatch (desktop user vs runtime user) - desktop only
        #[cfg(not(target_arch = "wasm32"))]
        let user_mismatch_warning = {
            let (current_user, desktop_user, is_different) = check_user_mismatch();
            if is_different {
                let warning = format!(
                    "Warning: Running as '{}' but desktop session is owned by '{}'. Desktop integration may not work properly.",
                    current_user, desktop_user
                );
                info!("{}", warning);
                Some(warning)
            } else {
                info!(
                    "User check: running as '{}', desktop user '{}' - OK",
                    current_user, desktop_user
                );
                None
            }
        };

        #[cfg(any(target_os = "android", target_arch = "wasm32"))]
        let user_mismatch_warning = None;

        // Get actual screen size for rectangle calculation
        let (screen_width, screen_height) = Self::get_initial_screen_size();
        let screen_ratio = screen_width / screen_height;

        info!(
            "Detected screen size: {}x{}, ratio: {:.2}",
            screen_width, screen_height, screen_ratio
        );

        // Initialize rectangle based on screen dimensions (scaled down to fit in UI)
        let rect_scale_factor = 0.3; // Start with 30% of screen size
        let rect_width = screen_width * rect_scale_factor;
        let rect_height = screen_height * rect_scale_factor;

        // Center the rectangle initially
        let square_center = pos2(400.0, 300.0); // Will be updated when image is loaded
        let half_width = rect_width / 2.0;
        let half_height = rect_height / 2.0;

        let square_corners = [
            pos2(square_center.x - half_width, square_center.y - half_height), // Top-left
            pos2(square_center.x + half_width, square_center.y - half_height), // Top-right
            pos2(square_center.x + half_width, square_center.y + half_height), // Bottom-right
            pos2(square_center.x - half_width, square_center.y + half_height), // Bottom-left
        ];

        // Initialize ehttp cache before moving config
        let ehttp_cache = {
            let cache_dir = config.as_ref().map(|c| c.cache_dir.join("ehttp"));
            Some(Arc::new(crate::api::ehttp_cache::EhttpCache::new(
                cache_dir,
                7 * 24 * 3600, // 7 days TTL for image responses
            )))
        };

        Self {
            title: "DureApp Window".to_owned(),
            title_bar: false,
            collapsible: false,
            resizable: false,
            constrain: false,
            anchored: true,
            anchor: egui::Align2::CENTER_TOP,
            anchor_offset: egui::Vec2::ZERO,

            config,
            // Settings
            settings: Settings::default(),
            // Dialog states
            dlg_settings: DlgSettings::default(),
            dlg_about: crate::ui_dlg::DlgAbout::default(),
            dlg_profile_login: crate::ui_dlg::DlgProfileLogin::new(),
            dlg_profile_create: crate::ui_dlg::DlgProfileCreate::new(),
            dlg_profile_delete: crate::ui_dlg::DlgProfileDelete::new(),
            // Profile state
            current_profile: None,
            pending_profile_name: None,
            current_profile_kdbx: None,
            // Installation status (desktop only)
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
            install_status: install::check_install(),
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
            install_dialog_open: false,
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
            install_message: String::new(),
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
            install_in_progress: false,
            // Update status
            update_status: String::new(),
            update_available: false,
            update_checking: false,
            update_download_url: String::new(),
            update_current_version: String::new(),
            update_latest_version: String::new(),
            // Screen / overlay state
            user_mismatch_warning,
            screen_size_provider: None,
            cached_screen_size: None,
            screen_size_failed: false,
            screen_ratio,
            square_size_factor: rect_scale_factor,
            square_center,
            square_corners,
            ehttp_cache,
            viewmodel: None,
            // Tabs (infrastructure management only)
            active_tab: crate::ui_tabs::Tab::Platform,
            previous_tab: None,
            scrolling_selected: 0,
            tab_platform: crate::ui_tabs::platform::PlatformTab::default(),
            tab_ssh: crate::ui_tabs::ssh::SshTab::default(),
            tab_ns: crate::ui_tabs::ns::NsTab::default(),
            tab_site: crate::ui_tabs::site::SiteTab::default(),
        }
    }
}

impl eframe::App for DureApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        dure_trace!("🔥 UPDATE CALLED - active_tab: {:?}, scrolling: {}", self.active_tab, self.scrolling_selected);

        // Initialize ViewModel on first update (lazy initialization)
        if self.viewmodel.is_none() {
            info!("🔥 Initializing ViewModel");
            self.viewmodel = Some(crate::viewmodel::ViewModel::new(ctx.clone()));
        }

        // NOTE: Event polling moved to individual tabs to avoid consuming events centrally
        // Each tab polls and processes its own events

        // Apply Material3 theme to egui context
        self.apply_theme(ctx);

        // Check for settings toggle
        if SETTINGS_TOGGLE.swap(false, Ordering::Relaxed) {
            self.dlg_settings.open = true;
        }

        // Check for about toggle
        if ABOUT_TOGGLE.swap(false, Ordering::Relaxed) {
            self.dlg_about.open();
        }

        // Apply top padding on Android to avoid content under status bar
        let mut central_panel = egui::CentralPanel::default();

        #[cfg(target_os = "android")]
        {
            if let Ok(height_str) = std::env::var("ANDROID_STATUS_BAR_HEIGHT") {
                if let Ok(height) = height_str.parse::<f32>() {
                    let mut frame = egui::Frame::default();
                    frame.inner_margin = egui::Margin {
                        top: height,
                        ..Default::default()
                    };
                    central_panel = central_panel.frame(frame);
                }
            }
        }

        central_panel.show(ctx, |ui| {
            self.ui(ui);
        });

        // Show settings dialog
        self.dlg_settings.show(ctx, &mut self.settings);
        // Handle save and theme changes from settings dialog
        if self.dlg_settings.save_clicked {
            self.dlg_settings.save_clicked = false; // Reset flag after processing
            // TODO: save settings to file
            dure_info!("Settings saved");
        }
        if let Some(theme_name) = self.dlg_settings.theme_to_apply.take() {
            // TODO: apply theme
            dure_info!("Applying theme: {}", theme_name);
        }

        // Show about dialog
        self.dlg_about.show(
            ctx,
            self.update_checking,
            self.update_available,
            &self.update_status,
        );
        // Handle check update and perform update from about dialog
        #[cfg(not(target_arch = "wasm32"))]
        if self.dlg_about.do_check_update {
            self.check_for_update();
        }
        if self.dlg_about.do_perform_update {
            self.perform_update();
        }

        // Show install dialog (desktop only)
        #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
        self.show_install_dialog(ctx);

        // Profile dialogs coordination
        self.handle_profile_dialogs(ctx);
    }
}

impl DureApp {
    fn ui(&mut self, ui: &mut egui::Ui) {
        dure_trace!("🔥 UI CALLED - scrolling: {}, active_tab: {:?}", self.scrolling_selected, self.active_tab);

        // Ensure the UI never exceeds window width
        ui.set_max_width(ui.available_width());

        // Display user mismatch warning (persistent)
        if let Some(warning) = &self.user_mismatch_warning {
            ui.separator();
            ui.horizontal(|ui| {
                ui.colored_label(egui::Color32::from_rgb(255, 165, 0), "⚠ "); // Orange warning
                ui.colored_label(egui::Color32::from_rgb(255, 165, 0), warning);
            });
        }

        // Profile selector (top-left)
        ui.horizontal(|ui| {
            ui.label(tr!("profile"));

            // Get list of available profiles
            let profiles = crate::calc::profile::ProfileManager::list_profiles()
                .unwrap_or_else(|e| {
                    dure_warn!("Failed to list profiles: {}", e);
                    Vec::new()
                });

            // Find current profile index in the profiles list
            let mut selected_profile_idx: Option<usize> = self.current_profile
                .as_ref()
                .and_then(|p| profiles.iter().position(|name| name == &p.name));

            let previous_selection = selected_profile_idx;

            // Build select with profile options
            let mut profile_select = egui_material3::select(&mut selected_profile_idx)
                .variant(egui_material3::SelectVariant::Outlined)
                .label(tr!("profile"))
                .placeholder(tr!("none"))
                .compact(true)
                .width(220.0)
                .menu_max_height(300.0);

            // Add profile options
            for (idx, profile_name) in profiles.iter().enumerate() {
                profile_select = profile_select.option(idx, profile_name.clone());
            }

            ui.add(profile_select);

            // Handle profile selection change
            if selected_profile_idx != previous_selection {
                if let Some(idx) = selected_profile_idx {
                    if let Some(profile_name) = profiles.get(idx) {
                        dure_info!("Profile selected: {}", profile_name);
                        self.pending_profile_name = Some(profile_name.clone());
                    }
                }
            }

            // Add "+ Create New Profile" button
            if ui.add(egui_material3::MaterialButton::outlined(tr!("create-new-profile")).small()).clicked() {
                dure_info!("Create new profile clicked");
                self.dlg_profile_create.open();
            }

            // Add "Delete Profile 'name'" button (only when a profile is selected)
            if let Some(ref profile) = self.current_profile {
                let delete_text = format!("{} '{}'", tr!("delete-profile"), profile.name);
                if ui.add(egui_material3::MaterialButton::outlined(delete_text).small()).clicked() {
                    dure_info!("Delete profile clicked: {}", profile.name);
                    self.dlg_profile_delete.open(profile.name.clone());
                }

                // Add "Close Profile" button
                if ui.add(egui_material3::MaterialButton::outlined(tr!("close-profile")).small()).clicked() {
                    dure_info!("Close profile clicked: {}", profile.name);
                    // Clear current profile and keyring
                    self.current_profile = None;
                    self.current_profile_kdbx = None;
                    // Clear and reload to reset UI
                    self.clear_and_reload_profile();
                }
            }
        });

        ui.add_space(10.0);

        // Tabs navigation
        ui.add(
            tabs_primary(&mut self.scrolling_selected)
                .id_salt("scrolling_primary")
                .tab(tr!("tab-platform"))
                .tab(tr!("tab-ssh"))
                .tab(tr!("tab-domains"))
                .tab(tr!("tab-site")),
        );

        // Sync scrolling_selected with active_tab enum
        use crate::ui_tabs::Tab;
        self.active_tab = match self.scrolling_selected {
            0 => Tab::Platform,
            1 => Tab::Ssh,
            2 => Tab::Ns,
            3 => Tab::Site,
            _ => Tab::Platform,
        };

        // Detect tab change and reload config only if file changed
        if self.previous_tab != Some(self.active_tab) {
            match self.active_tab {
                Tab::Platform => {
                    if self.tab_platform.should_reload_config(&self.current_profile) {
                        self.tab_platform.reset_loaded();
                    }
                }
                Tab::Ssh => {
                    if self.tab_ssh.should_reload_config(&self.current_profile) {
                        self.tab_ssh.reset_loaded();
                    }
                }
                Tab::Ns => {
                    if self.tab_ns.should_reload_config(&self.current_profile) {
                        self.tab_ns.reset_loaded();
                    }
                }
                Tab::Site => {} // Site tab doesn't use config loading pattern
            }
            self.previous_tab = Some(self.active_tab);
        }

        ui.add_space(10.0);

        // Render active tab content
        dure_trace!("🔥 RENDERING TAB: {:?}", self.active_tab);
        match self.active_tab {
            Tab::Platform => self.tab_platform.ui(&self.current_profile, &self.current_profile_kdbx, ui, self.viewmodel.as_mut()),
            Tab::Ssh => self.tab_ssh.ui(&self.current_profile, ui, self.viewmodel.as_mut()),
            Tab::Ns => self.tab_ns.ui(&self.current_profile, ui, self.viewmodel.as_mut()),
            Tab::Site => self.tab_site.ui(&self.current_profile, ui),
        }
    }

    fn get_theme(&self) -> MaterialThemeContext {
        if let Ok(theme) = get_global_theme().lock() {
            theme.clone()
        } else {
            MaterialThemeContext::default()
        }
    }

    fn update_theme<F>(&self, update_fn: F)
    where
        F: FnOnce(&mut MaterialThemeContext),
    {
        if let Ok(mut theme) = get_global_theme().lock() {
            update_fn(&mut theme);
        }
    }

    fn load_theme_from_file(
        &self,
        file_path: &PathBuf,
    ) -> Result<MaterialThemeFile, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(file_path)?;
        let theme: MaterialThemeFile = serde_json::from_str(&content)?;
        Ok(theme)
    }

    fn apply_theme(&self, ctx: &egui::Context) {
        // Use the comprehensive Material Design 3 theme implementation
        // from egui_material3::theme which correctly maps all Material 3
        // color roles to egui visuals, including proper selection colors
        // (inverse_surface/inverse_primary for contrast with on_surface text)
        egui_material3::theme::apply_theme(ctx, None::<fn() -> ThemeMode>);
    }
}

/// Open a directory in the file manager (Desktop only)
#[cfg(not(target_arch = "wasm32"))]
fn open_directory(path: &std::path::Path) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

impl DureApp {
    fn get_screen_size_internal(&self) -> (i32, i32) {
        if let Some(ref provider) = self.screen_size_provider {
            provider.get_screen_size().unwrap_or((1080, 1920))
        } else {
            // Fallback based on platform
            #[cfg(target_os = "android")]
            {
                (1080, 1920) // Default Android resolution
            }
            #[cfg(not(target_os = "android"))]
            {
                (1920, 1080) // Fallback for non-Android
            }
        }
    }

    fn update_square_corners(&mut self) {
        // Get actual screen dimensions for rectangle calculation
        let (screen_width, screen_height) = self.get_actual_screen_size();
        info!(
            "Updating square corners with screen: {}x{}, center: {:?}, factor: {}",
            screen_width, screen_height, self.square_center, self.square_size_factor
        );

        // Ensure screen ratio is valid
        if self.screen_ratio <= 0.0 {
            self.screen_ratio = screen_width / screen_height; // Use actual screen ratio
        }

        // Ensure size factor is valid
        if self.square_size_factor <= 0.0 {
            self.square_size_factor = 0.3; // Default to 30% of screen size
        }

        // Create a rectangle that matches screen aspect ratio and actual dimensions
        // Size factor now represents the percentage of screen size to use
        let rect_width = screen_width * self.square_size_factor;
        let _rect_height = screen_height * self.square_size_factor;

        // Ensure rectangle maintains screen aspect ratio
        let corrected_height = rect_width / self.screen_ratio;
        let final_width = rect_width;
        let final_height = corrected_height;

        let half_width = final_width / 2.0;
        let half_height = final_height / 2.0;

        self.square_corners = [
            pos2(
                self.square_center.x - half_width,
                self.square_center.y - half_height,
            ), // Top-left
            pos2(
                self.square_center.x + half_width,
                self.square_center.y - half_height,
            ), // Top-right
            pos2(
                self.square_center.x + half_width,
                self.square_center.y + half_height,
            ), // Bottom-right
            pos2(
                self.square_center.x - half_width,
                self.square_center.y + half_height,
            ), // Bottom-left
        ];
        info!("Updated square corners: {:?}", self.square_corners);
    }

    fn get_initial_screen_size() -> (f32, f32) {
        // Static fallback when no service is available yet
        // Use a more conservative ratio that will be updated when actual screen size is detected
        #[cfg(target_os = "android")]
        {
            (1080.0, 2340.0) // Modern Android resolution (closer to actual device ratio)
        }

        #[cfg(not(target_os = "android"))]
        {
            (1920.0, 1080.0) // Fallback for non-Android
        }
    }

    fn get_actual_screen_size(&mut self) -> (f32, f32) {
        // Return cached value if available
        if let Some(cached) = self.cached_screen_size {
            return cached;
        }

        // If screen size detection previously failed, don't retry
        if self.screen_size_failed {
            return (1080.0, 1920.0); // Default mobile resolution
        }

        #[cfg(target_os = "android")]
        {
            let screen_size = self.get_screen_size_internal();
            let result = (screen_size.0 as f32, screen_size.1 as f32);
            self.cached_screen_size = Some(result);
            result
        }

        #[cfg(not(target_os = "android"))]
        {
            // For desktop platforms, use the default desktop resolution
            let result = (1920.0, 1080.0); // Default desktop resolution
            self.cached_screen_size = Some(result);
            result
        }
    }

    /// Check for updates from GitHub
    #[cfg(not(target_arch = "wasm32"))]
    fn check_for_update(&mut self) {
        self.update_checking = true;
        self.update_status.clear();
        self.update_available = false;

        #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
        {
            match crate::install::check_update() {
                Ok(info) => {
                    self.update_checking = false;
                    if info.available {
                        self.update_available = true;
                        // Store update info for later use
                        self.update_download_url = info.download_url.clone();
                        self.update_current_version = info.current_version.clone();
                        self.update_latest_version = info.latest_version.clone();
                        self.update_status =
                            format!("{} → {}", info.current_version, info.latest_version);
                    } else {
                        self.update_status = tr!("up-to-date").to_string();
                    }
                }
                Err(e) => {
                    self.update_checking = false;
                    self.update_status = format!("{}: {}", tr!("update-error"), e);
                }
            }
        }
        #[cfg(any(target_os = "android", target_arch = "wasm32"))]
        {
            self.update_checking = false;
            self.update_status = "Update checking not available on this platform".to_string();
        }
    }

    /// Perform update
    fn perform_update(&mut self) {
        // Clear the dialog flag first to prevent repeated calls
        self.dlg_about.do_perform_update = false;

        // Check if we have update info stored from check_for_update()
        if !self.update_available || self.update_download_url.is_empty() {
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
            {
                // Only show error dialog if this is a genuine attempt (dialog still open)
                // Don't warn if state was just cleared from a successful update
                if self.dlg_about.open {
                    let msg = if !self.update_available {
                        "No update available. Please check for updates first.".to_string()
                    } else {
                        format!(
                            "Update information incomplete (missing download URL). Please check for updates again.\nStatus: {}",
                            self.update_status
                        )
                    };
                    dure_warn!(
                        "Update aborted: update_available={}, download_url_empty={}",
                        self.update_available,
                        self.update_download_url.is_empty()
                    );
                    self.install_message = msg;
                    self.install_dialog_open = true;
                }
            }
            return;
        }

        #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
        {
            use crate::install_stt::InstallResult;

            let tmp_dir = if let Some(ref cfg) = self.config {
                cfg.tmp_dir.clone()
            } else {
                std::path::PathBuf::from("./tmp")
            };

            dure_info!(
                "Starting update download from: {}",
                self.update_download_url
            );
            dure_info!("Temporary directory: {}", tmp_dir.display());

            // Use the stored download URL and version from check_for_update()
            match crate::install::do_update(
                &self.update_download_url,
                &self.update_latest_version,
                &tmp_dir,
            ) {
                InstallResult::Success(msg) => {
                    dure_info!("Update successful: {}", msg);
                    self.install_message = msg;
                    self.update_available = false;
                    self.update_status.clear();
                    self.update_download_url.clear();
                    self.dlg_about.close();

                    // Give the filesystem time to sync before checking status
                    dure_debug!("Waiting for filesystem to sync after update...");
                    std::thread::sleep(std::time::Duration::from_millis(200));

                    // Refresh install status with retries (same logic as install)
                    let old_status = self.install_status;
                    let mut retries = 3;
                    loop {
                        dure_debug!(
                            "Checking install status after update (attempt {}/{})",
                            4 - retries,
                            3
                        );
                        let new_status = crate::install::check_install();

                        // After update, status should still be Installed (just newer version)
                        let status_is_correct =
                            new_status == crate::install_stt::InstallStatus::Installed;

                        if status_is_correct || retries == 0 {
                            dure_info!(
                                "Install status after update: {:?} -> {:?}",
                                old_status,
                                new_status
                            );
                            self.install_status = new_status;
                            break;
                        }

                        // Status not as expected, wait and retry
                        dure_warn!(
                            "Install status check unexpected, retrying... ({} retries left)",
                            retries
                        );
                        std::thread::sleep(std::time::Duration::from_millis(300));
                        retries -= 1;
                    }

                    if retries == 0 {
                        dure_error!("Install status check failed after all retries!");
                        self.install_message = format!(
                            "{}\n\nNote: Status may not have updated correctly. Please restart the application.",
                            self.install_message
                        );
                    }
                }
                InstallResult::Error(err) => {
                    dure_error!("Update failed: {}", err);
                    self.install_message = format!("Error: {}", err);
                }
            }
            self.install_dialog_open = true;
        }

        #[cfg(target_os = "android")]
        {
            // On Android, open browser to download page using stored URL
            if let Err(e) = webbrowser::open(&self.update_download_url) {
                dure_error!("Failed to open browser for update download: {}", e);
                self.update_status = format!("Failed to open browser: {}", e);
            } else {
                dure_info!("Opened browser for update download");
                self.dlg_about.close();
            }
        }
    }

    /// Show install dialog (desktop only)
    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
    fn show_install_dialog(&mut self, ctx: &egui::Context) {
        if !self.install_dialog_open {
            return;
        }

        let mut close_clicked = false;

        egui::Window::new(tr!("install"))
            .id(egui::Id::new("install_dialog"))
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(&self.install_message);
                ui.add_space(8.0);

                if ui.button(tr!("ok")).clicked() {
                    close_clicked = true;
                }
            });

        if close_clicked {
            self.install_dialog_open = false;
        }
    }

    /// Perform install or uninstall action based on current status
    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
    fn perform_install_action(&mut self) {
        use crate::install_stt::InstallResult;

        // Prevent concurrent operations
        if self.install_in_progress {
            dure_warn!("Install operation already in progress, ignoring duplicate request");
            return;
        }

        self.install_in_progress = true;
        dure_info!(
            "Performing install action, current status: {:?}",
            self.install_status
        );

        let result = if self.install_status == crate::install_stt::InstallStatus::Installed {
            dure_info!("Uninstalling...");
            crate::install::do_uninstall()
        } else {
            dure_info!("Installing...");
            crate::install::do_install()
        };

        match result {
            InstallResult::Success(msg) => {
                dure_info!("Operation succeeded: {}", msg);
                self.install_message = msg;

                // Give the filesystem time to sync before checking status
                // This is especially important on Windows where file operations may be asynchronous
                dure_debug!("Waiting for filesystem to sync...");
                std::thread::sleep(std::time::Duration::from_millis(200));

                // Refresh install status with retries
                let mut retries = 3;
                loop {
                    dure_debug!("Checking install status (attempt {}/{})", 4 - retries, 3);
                    let new_status = crate::install::check_install();

                    // Check if status changed as expected
                    let status_changed_correctly = match self.install_status {
                        crate::install_stt::InstallStatus::Installed => {
                            // Was installed, should now be NotInstalled after uninstall
                            new_status == crate::install_stt::InstallStatus::NotInstalled
                        }
                        crate::install_stt::InstallStatus::NotInstalled => {
                            // Was not installed, should now be Installed after install
                            new_status == crate::install_stt::InstallStatus::Installed
                        }
                        crate::install_stt::InstallStatus::Unknown => {
                            // Unknown status - just accept whatever the new status is
                            true
                        }
                    };

                    if status_changed_correctly || retries == 0 {
                        dure_info!(
                            "Install status updated: {:?} -> {:?}",
                            self.install_status,
                            new_status
                        );
                        self.install_status = new_status;
                        break;
                    }

                    // Status didn't change, wait and retry
                    dure_warn!(
                        "Install status didn't change as expected, retrying... ({} retries left)",
                        retries
                    );
                    std::thread::sleep(std::time::Duration::from_millis(300));
                    retries -= 1;
                }

                if retries == 0 {
                    dure_error!("Install status check failed after all retries!");
                    self.install_message = format!(
                        "{}\n\nNote: Status may not have updated correctly. Please restart the application.",
                        self.install_message
                    );
                }
            }
            InstallResult::Error(err) => {
                dure_error!("Operation failed: {}", err);
                self.install_message = format!("Error: {}", err);
            }
        }

        // Reset the in-progress flag
        self.install_in_progress = false;
        self.install_dialog_open = true;
    }

    /// Handle profile dialogs coordination
    ///
    /// Manages the flow between profile selection, login, create, and delete dialogs.
    fn handle_profile_dialogs(&mut self, ctx: &egui::Context) {
        // Handle pending profile selection → trigger login dialog
        if let Some(profile_name) = self.pending_profile_name.take() {
            // Only open dialog if not already open (avoid repeated logging)
            if !self.dlg_profile_login.open {
                dure_info!("Opening login dialog for profile: {}", profile_name);
                self.dlg_profile_login.open();
            }
            // Store the profile name for login processing
            self.pending_profile_name = Some(profile_name);
        }

        // Show and handle login dialog
        let was_open = self.dlg_profile_login.open;
        if self.dlg_profile_login.open {
            self.dlg_profile_login.show(ctx);

            // Process login result
            if self.dlg_profile_login.confirmed {
                if let Some(profile_name) = self.pending_profile_name.take() {
                    let password = self.dlg_profile_login.password.clone();

                    // Verify password
                    match crate::calc::profile::ProfileManager::verify_password(&profile_name, &password) {
                        Ok(()) => {
                            dure_info!("Password verified for profile: {}", profile_name);

                            // Load profile context
                            match crate::calc::profile::ProfileContext::new(&profile_name) {
                                Ok(ctx) => {
                                    // Update database path to profile's database
                                    let db_path = ctx.db_path.to_string_lossy().to_string();
                                    crate::calc::db::set_db_path(db_path);
                                    dure_info!("Database path updated to: {}", ctx.db_path.display());

                                    // Reload config from profile's config directory
                                    self.config = Config::new(Some(ctx.config_dir.clone())).ok();
                                    dure_info!("Config reloaded from profile: {}", ctx.config_file.display());

                                    // Open KeePass database handle
                                    match crate::calc::keyring::DatabaseHandle::open(
                                        ctx.kdbx_path.clone(),
                                        ctx.kpkey_path.clone(),
                                        Some(&password),
                                    ) {
                                        Ok(handle) => {
                                            // Load DB encryption key from keyring
                                            match handle.get_db_encryption_key() {
                                                Ok(db_key) => {
                                                    crate::calc::db::set_db_encryption_key(db_key);
                                                    dure_info!("SQLite encryption key loaded from keyring");
                                                }
                                                Err(e) => {
                                                    dure_warn!("Failed to load DB encryption key (profile may predate encryption): {}", e);
                                                    dure_warn!("Database will remain unencrypted. Consider migrating to encrypted DB.");
                                                }
                                            }

                                            self.current_profile_kdbx = Some(std::sync::Arc::new(handle));
                                            self.current_profile = Some(ctx);
                                            dure_info!("Profile and keyring loaded successfully: {}", profile_name);

                                            // Clear screen and reload profile configs
                                            self.clear_and_reload_profile();

                                            self.dlg_profile_login.reset();
                                        }
                                        Err(e) => {
                                            dure_error!("Failed to open keyring: {}", e);
                                            self.dlg_profile_login.set_error(format!("Failed to open keyring: {}", e));
                                            self.dlg_profile_login.confirmed = false;
                                            self.dlg_profile_login.open = true;
                                        }
                                    }
                                }
                                Err(e) => {
                                    dure_error!("Failed to load profile context: {}", e);
                                    self.dlg_profile_login.set_error(format!("Failed to load profile: {}", e));
                                    self.dlg_profile_login.confirmed = false;
                                    self.dlg_profile_login.open = true; // Reopen dialog to show error
                                }
                            }
                        }
                        Err(e) => {
                            dure_warn!("Password verification failed: {}", e);
                            self.dlg_profile_login.set_error("Incorrect password".to_string());
                            self.dlg_profile_login.confirmed = false;
                            self.dlg_profile_login.open = true; // Reopen dialog to show error
                        }
                    }
                }
            }
        }

        // Handle cancel: if dialog was open but is now closed and not confirmed, clear pending profile
        if was_open && !self.dlg_profile_login.open && !self.dlg_profile_login.confirmed {
            dure_info!("Login dialog cancelled, clearing pending profile");
            self.pending_profile_name = None;
        }

        // Show and handle create dialog
        if self.dlg_profile_create.open {
            self.dlg_profile_create.show(ctx);

            // Process create result
            if self.dlg_profile_create.confirmed {
                let profile_name = self.dlg_profile_create.profile_name.clone();
                let password = self.dlg_profile_create.password.clone();

                // Create profile
                match crate::calc::profile::ProfileManager::create_profile(&profile_name, &password) {
                    Ok(ctx) => {
                        dure_info!("Profile created successfully: {}", profile_name);

                        // Update database path to profile's database
                        let db_path = ctx.db_path.to_string_lossy().to_string();
                        crate::calc::db::set_db_path(db_path);
                        dure_info!("Database path updated to: {}", ctx.db_path.display());

                        // Reload config from profile's config directory
                        self.config = Config::new(Some(ctx.config_dir.clone())).ok();
                        dure_info!("Config reloaded from profile: {}", ctx.config_file.display());

                        // Open KeePass database handle
                        match crate::calc::keyring::DatabaseHandle::open(
                            ctx.kdbx_path.clone(),
                            ctx.kpkey_path.clone(),
                            Some(&password),
                        ) {
                            Ok(handle) => {
                                // Generate DB encryption key for new profile
                                match handle.generate_db_encryption_key() {
                                    Ok(db_key) => {
                                        crate::calc::db::set_db_encryption_key(db_key);
                                        dure_info!("Generated and stored SQLite encryption key for new profile");
                                    }
                                    Err(e) => {
                                        dure_error!("Failed to generate DB encryption key: {}", e);
                                        self.dlg_profile_create.set_error(format!("Profile created but encryption key generation failed: {}", e));
                                        self.dlg_profile_create.confirmed = false;
                                        return;
                                    }
                                }

                                // Auto-login: load the newly created profile
                                self.current_profile_kdbx = Some(std::sync::Arc::new(handle));
                                self.current_profile = Some(ctx);

                                // Clear screen and reload profile configs
                                self.clear_and_reload_profile();

                                self.dlg_profile_create.reset();
                                dure_info!("Auto-logged in to new profile: {}", profile_name);
                            }
                            Err(e) => {
                                dure_error!("Failed to open keyring for new profile: {}", e);
                                self.dlg_profile_create.set_error(format!("Profile created but failed to open keyring: {}", e));
                                self.dlg_profile_create.confirmed = false;
                            }
                        }
                    }
                    Err(e) => {
                        dure_error!("Failed to create profile: {}", e);
                        self.dlg_profile_create.set_error(format!("Failed to create profile: {}", e));
                        self.dlg_profile_create.confirmed = false;
                    }
                }
            }
        }

        // Show and handle delete dialog
        if self.dlg_profile_delete.open {
            self.dlg_profile_delete.show(ctx);

            // Process delete result
            if self.dlg_profile_delete.confirmed {
                let profile_name = self.dlg_profile_delete.profile_name.clone();

                // Check if deleting the currently active profile
                let is_active = self.current_profile
                    .as_ref()
                    .map(|p| p.name == profile_name)
                    .unwrap_or(false);

                // Delete profile
                match crate::calc::profile::ProfileManager::delete_profile(&profile_name) {
                    Ok(()) => {
                        dure_info!("Profile deleted successfully: {}", profile_name);

                        // Unload profile if it was active
                        if is_active {
                            self.current_profile_kdbx = None;
                            self.current_profile = None;

                            // Clear config - no profile means no config
                            self.config = None;
                            dure_info!("Config cleared - no active profile");

                            // Clear screen since active profile is deleted
                            self.clear_and_reload_profile();
                        }

                        self.dlg_profile_delete.reset();
                    }
                    Err(e) => {
                        dure_error!("Failed to delete profile: {}", e);
                        // Note: Delete dialog doesn't have error display, just log
                        self.dlg_profile_delete.reset();
                    }
                }
            }
        }
    }

    /// Clear screen and reload profile configurations
    ///
    /// Called when switching profiles to ensure clean state.
    fn clear_and_reload_profile(&mut self) {
        dure_info!("Clearing screen and reloading profile configs");

        // Reset all tabs to clear cached data
        self.tab_platform = crate::ui_tabs::platform::PlatformTab::default();
        self.tab_ssh = crate::ui_tabs::ssh::SshTab::default();
        self.tab_ns = crate::ui_tabs::ns::NsTab::default();
        self.tab_site = crate::ui_tabs::site::SiteTab::default();

        // Reset ViewModel to force re-initialization with new profile data
        self.viewmodel = None;

        // Reset config to reload from new profile's config file
        self.config = None;

        // Reset active tab to Platform (first tab)
        self.active_tab = crate::ui_tabs::Tab::Platform;
        self.scrolling_selected = 0;

        dure_info!("Screen cleared and ready for profile reload");
    }
}
