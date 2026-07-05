//! Dure eframe UI (Cross-platform: Desktop, Android, WASM)
//!
//! This module provides the main eframe application UI that works across all platforms.
//! Platform-specific functionality is injected via traits.

#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
use crate::api::desktop::check_user_mismatch;
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
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

    // Installation status (desktop only)
    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    pub install_status: crate::install_stt::InstallStatus,
    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    pub install_dialog_open: bool,
    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    pub install_message: String,
    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
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

    // Tabs state
    pub active_tab: crate::ui_tabs::Tab,
    pub scrolling_selected: usize,
    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    pub tab_platform: crate::ui_tabs::platform::PlatformTab,
    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    pub tab_ssh: crate::ui_tabs::ssh::SshTab,
    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    pub tab_ns: crate::ui_tabs::ns::NsTab,
    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    pub tab_site: crate::ui_tabs::site::SiteTab,
    pub tab_roles: crate::ui_tabs::roles::RolesTab,
    pub tab_members: crate::ui_tabs::members::MembersTab,
    pub tab_channel: crate::ui_tabs::channel::ChannelTab,
    pub tab_dm: crate::ui_tabs::dm::DMTab,
    pub tab_products: crate::ui_tabs::products::ProductsTab,
    pub tab_orders: crate::ui_tabs::orders::OrdersTab,
    pub tab_email: crate::ui_tabs::email::EmailTab,
    pub tab_client: crate::ui_tabs::client::ClientTab,
}

impl Default for DureApp {
    fn default() -> Self {
        let config = Config::new().ok();
        info!("Config creation result: {:?}", config.is_some());

        // Check for user mismatch (desktop user vs runtime user) - desktop only
        #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
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
            // Installation status (desktop only)
            #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
            install_status: install::check_install(),
            #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
            install_dialog_open: false,
            #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
            install_message: String::new(),
            #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
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
            // Tabs
            #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
            active_tab: crate::ui_tabs::Tab::Platform,
            #[cfg(any(target_os = "android", target_arch = "wasm32"))]
            active_tab: crate::ui_tabs::Tab::Client,
            scrolling_selected: 0,
            #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
            tab_platform: crate::ui_tabs::platform::PlatformTab::default(),
            #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
            tab_ssh: crate::ui_tabs::ssh::SshTab::default(),
            #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
            tab_ns: crate::ui_tabs::ns::NsTab::default(),
            #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
            tab_site: crate::ui_tabs::site::SiteTab::default(),
            tab_roles: crate::ui_tabs::roles::RolesTab::default(),
            tab_members: crate::ui_tabs::members::MembersTab::default(),
            tab_channel: crate::ui_tabs::channel::ChannelTab::default(),
            tab_dm: crate::ui_tabs::dm::DMTab::default(),
            tab_products: crate::ui_tabs::products::ProductsTab::default(),
            tab_orders: crate::ui_tabs::orders::OrdersTab::default(),
            tab_email: crate::ui_tabs::email::EmailTab::default(),
            tab_client: crate::ui_tabs::client::ClientTab::default(),
        }
    }
}

impl eframe::App for DureApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Initialize ViewModel on first update (lazy initialization)
        if self.viewmodel.is_none() {
            self.viewmodel = Some(crate::viewmodel::ViewModel::new(ctx.clone()));
        }

        // Poll ViewModel events
        if let Some(ref mut vm) = self.viewmodel {
            let _events = vm.poll_events(ctx);
            // Events will be processed when actors are implemented
        }

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

        egui::CentralPanel::default().show(ctx, |ui| {
            self.ui(ui);
        });

        // Show settings dialog
        self.dlg_settings.show(ctx, &mut self.settings);
        // Handle save and theme changes from settings dialog
        if self.dlg_settings.save_clicked {
            self.dlg_settings.save_clicked = false; // Reset flag after processing
            // TODO: save settings to file
            log::info!("Settings saved");
        }
        if let Some(theme_name) = self.dlg_settings.theme_to_apply.take() {
            // TODO: apply theme
            log::info!("Applying theme: {}", theme_name);
        }

        // Show about dialog
        self.dlg_about.show(
            ctx,
            self.update_checking,
            self.update_available,
            &self.update_status,
        );
        // Handle check update and perform update from about dialog
        #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
        if self.dlg_about.do_check_update {
            self.check_for_update();
        }
        if self.dlg_about.do_perform_update {
            self.perform_update();
        }

        // Show install dialog (desktop only)
        #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
        self.show_install_dialog(ctx);
    }
}

impl DureApp {
    fn ui(&mut self, ui: &mut egui::Ui) {
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

        // Tabs navigation
        #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
        ui.add(
            tabs_primary(&mut self.scrolling_selected)
                .id_salt("scrolling_primary")
                .tab(tr!("tab-client"))
                .tab(tr!("tab-platform"))
                .tab(tr!("tab-ssh"))
                .tab(tr!("tab-domains"))
                .tab(tr!("tab-site"))
                // .tab(tr!("tab-roles"))
                .tab(tr!("tab-members"))
                .tab(tr!("tab-channel"))
                .tab(tr!("tab-dm"))
                .tab(tr!("tab-products"))
                .tab(tr!("tab-orders"))
                .tab(tr!("tab-email")),
        );
        #[cfg(any(target_os = "android", target_arch = "wasm32"))]
        ui.add(
            tabs_primary(&mut self.scrolling_selected)
                .id_salt("scrolling_primary")
                .tab(tr!("tab-client"))
                // .tab(tr!("tab-roles"))
                .tab(tr!("tab-members"))
                .tab(tr!("tab-channel"))
                .tab(tr!("tab-dm"))
                .tab(tr!("tab-products"))
                .tab(tr!("tab-orders"))
                .tab(tr!("tab-email")),
        );

        // Sync scrolling_selected with active_tab enum
        use crate::ui_tabs::Tab;
        #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
        {
            self.active_tab = match self.scrolling_selected {
                0 => Tab::Client,
                1 => Tab::Platform,
                2 => Tab::Ssh,
                3 => Tab::Ns,
                4 => Tab::Site,
                5 => Tab::Members,
                6 => Tab::Channel,
                7 => Tab::DM,
                8 => Tab::Products,
                9 => Tab::Orders,
                10 => Tab::Email,
                _ => Tab::Platform,
            };
        }
        #[cfg(any(target_os = "android", target_arch = "wasm32"))]
        {
            self.active_tab = match self.scrolling_selected {
                0 => Tab::Client,
                1 => Tab::Members,
                2 => Tab::Channel,
                3 => Tab::DM,
                4 => Tab::Products,
                5 => Tab::Orders,
                6 => Tab::Email,
                _ => Tab::Client,
            };
        }

        ui.add_space(10.0);

        // Render active tab content
        match self.active_tab {
            Tab::Client => self.tab_client.ui(ui),
            #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
            Tab::Platform => self.tab_platform.ui(ui, self.viewmodel.as_mut()),
            #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
            Tab::Ssh => self.tab_ssh.ui(ui),
            #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
            Tab::Ns => self.tab_ns.ui(ui),
            #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
            Tab::Site => self.tab_site.ui(ui),
            Tab::Roles => self.tab_roles.ui(ui),
            Tab::Members => self.tab_members.ui(ui),
            Tab::Channel => self.tab_channel.ui(ui),
            Tab::DM => self.tab_dm.ui(ui),
            Tab::Products => self.tab_products.ui(ui),
            Tab::Orders => self.tab_orders.ui(ui),
            Tab::Email => self.tab_email.ui(ui),
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
        let theme = self.get_theme();

        let mut visuals = match theme.theme_mode {
            ThemeMode::Light => egui::Visuals::light(),
            ThemeMode::Dark => egui::Visuals::dark(),
            ThemeMode::Auto => {
                // Use system preference or default to light
                if ctx.style().visuals.dark_mode {
                    egui::Visuals::dark()
                } else {
                    egui::Visuals::light()
                }
            }
        };

        // Apply Material Design 3 colors from theme
        let primary_color = theme.get_primary_color();
        let on_primary = theme.get_on_primary_color();
        let surface = theme.get_surface_color(visuals.dark_mode);

        // Apply colors to visuals
        visuals.selection.bg_fill = primary_color;
        visuals.selection.stroke.color = primary_color;
        visuals.hyperlink_color = primary_color;

        // Button and widget colors
        visuals.widgets.noninteractive.bg_fill = surface;

        visuals.widgets.inactive.bg_fill = Color32::from_rgba_unmultiplied(
            primary_color.r(),
            primary_color.g(),
            primary_color.b(),
            20,
        );

        visuals.widgets.hovered.bg_fill = Color32::from_rgba_unmultiplied(
            primary_color.r(),
            primary_color.g(),
            primary_color.b(),
            40,
        );

        visuals.widgets.active.bg_fill = primary_color;
        visuals.widgets.active.fg_stroke.color = on_primary;

        // Window background
        visuals.window_fill = surface;
        visuals.panel_fill = theme.get_color_by_name("surfaceContainer");

        // Apply surface colors
        visuals.extreme_bg_color = theme.get_color_by_name("surfaceContainerLowest");

        ctx.set_visuals(visuals);
    }
}

/// Open a directory in the file manager (Desktop only)
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
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
    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    fn check_for_update(&mut self) {
        self.update_checking = true;
        self.update_status.clear();
        self.update_available = false;

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

    /// Perform update
    fn perform_update(&mut self) {
        // Clear the dialog flag first to prevent repeated calls
        self.dlg_about.do_perform_update = false;

        // Check if we have update info stored from check_for_update()
        if !self.update_available || self.update_download_url.is_empty() {
            #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
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
                    log::warn!(
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

        #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
        {
            use crate::install_stt::InstallResult;

            let tmp_dir = if let Some(ref cfg) = self.config {
                cfg.tmp_dir.clone()
            } else {
                std::path::PathBuf::from("./tmp")
            };

            log::info!(
                "Starting update download from: {}",
                self.update_download_url
            );
            log::info!("Temporary directory: {}", tmp_dir.display());

            // Use the stored download URL and version from check_for_update()
            match crate::install::do_update(
                &self.update_download_url,
                &self.update_latest_version,
                &tmp_dir,
            ) {
                InstallResult::Success(msg) => {
                    log::info!("Update successful: {}", msg);
                    self.install_message = msg;
                    self.update_available = false;
                    self.update_status.clear();
                    self.update_download_url.clear();
                    self.dlg_about.close();

                    // Give the filesystem time to sync before checking status
                    log::debug!("Waiting for filesystem to sync after update...");
                    std::thread::sleep(std::time::Duration::from_millis(200));

                    // Refresh install status with retries (same logic as install)
                    let old_status = self.install_status;
                    let mut retries = 3;
                    loop {
                        log::debug!(
                            "Checking install status after update (attempt {}/{})",
                            4 - retries,
                            3
                        );
                        let new_status = crate::install::check_install();

                        // After update, status should still be Installed (just newer version)
                        let status_is_correct =
                            new_status == crate::install_stt::InstallStatus::Installed;

                        if status_is_correct || retries == 0 {
                            log::info!(
                                "Install status after update: {:?} -> {:?}",
                                old_status,
                                new_status
                            );
                            self.install_status = new_status;
                            break;
                        }

                        // Status not as expected, wait and retry
                        log::warn!(
                            "Install status check unexpected, retrying... ({} retries left)",
                            retries
                        );
                        std::thread::sleep(std::time::Duration::from_millis(300));
                        retries -= 1;
                    }

                    if retries == 0 {
                        log::error!("Install status check failed after all retries!");
                        self.install_message = format!(
                            "{}\n\nNote: Status may not have updated correctly. Please restart the application.",
                            self.install_message
                        );
                    }
                }
                InstallResult::Error(err) => {
                    log::error!("Update failed: {}", err);
                    self.install_message = format!("Error: {}", err);
                }
            }
            self.install_dialog_open = true;
        }

        #[cfg(target_os = "android")]
        {
            // On Android, open browser to download page using stored URL
            if let Err(e) = webbrowser::open(&self.update_download_url) {
                log::error!("Failed to open browser for update download: {}", e);
                self.update_status = format!("Failed to open browser: {}", e);
            } else {
                log::info!("Opened browser for update download");
                self.dlg_about.close();
            }
        }
    }

    /// Show install dialog (desktop only)
    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
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
    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    fn perform_install_action(&mut self) {
        use crate::install_stt::InstallResult;

        // Prevent concurrent operations
        if self.install_in_progress {
            log::warn!("Install operation already in progress, ignoring duplicate request");
            return;
        }

        self.install_in_progress = true;
        log::info!(
            "Performing install action, current status: {:?}",
            self.install_status
        );

        let result = if self.install_status == crate::install_stt::InstallStatus::Installed {
            log::info!("Uninstalling...");
            crate::install::do_uninstall()
        } else {
            log::info!("Installing...");
            crate::install::do_install()
        };

        match result {
            InstallResult::Success(msg) => {
                log::info!("Operation succeeded: {}", msg);
                self.install_message = msg;

                // Give the filesystem time to sync before checking status
                // This is especially important on Windows where file operations may be asynchronous
                log::debug!("Waiting for filesystem to sync...");
                std::thread::sleep(std::time::Duration::from_millis(200));

                // Refresh install status with retries
                let mut retries = 3;
                loop {
                    log::debug!("Checking install status (attempt {}/{})", 4 - retries, 3);
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
                        log::info!(
                            "Install status updated: {:?} -> {:?}",
                            self.install_status,
                            new_status
                        );
                        self.install_status = new_status;
                        break;
                    }

                    // Status didn't change, wait and retry
                    log::warn!(
                        "Install status didn't change as expected, retrying... ({} retries left)",
                        retries
                    );
                    std::thread::sleep(std::time::Duration::from_millis(300));
                    retries -= 1;
                }

                if retries == 0 {
                    log::error!("Install status check failed after all retries!");
                    self.install_message = format!(
                        "{}\n\nNote: Status may not have updated correctly. Please restart the application.",
                        self.install_message
                    );
                }
            }
            InstallResult::Error(err) => {
                log::error!("Operation failed: {}", err);
                self.install_message = format!("Error: {}", err);
            }
        }

        // Reset the in-progress flag
        self.install_in_progress = false;
        self.install_dialog_open = true;
    }
}
