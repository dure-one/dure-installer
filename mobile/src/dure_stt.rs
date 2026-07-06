#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
use crate::install_stt::InstallStatus;
use crate::{Config, Settings};
use eframe::egui::{Align2, Pos2, Vec2};
use std::sync::Arc;

// ViewModel (MVVM architecture)
#[cfg(feature = "gui")]
use crate::viewmodel::ViewModel;

#[doc(hidden)]
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
    pub anchor: Align2,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub anchor_offset: Vec2,

    #[cfg_attr(feature = "serde", serde(skip))]
    pub config: Option<Config>,

    // Settings
    pub settings: Settings,

    // Dialog states
    pub dlg_settings: crate::ui_dlg::DlgSettings,
    pub dlg_about: crate::ui_dlg::DlgAbout,

    // Installation status (desktop only)
    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    pub install_status: InstallStatus,
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
    pub square_center: Pos2,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub square_corners: [Pos2; 4],

    // HTTP cache
    #[cfg_attr(feature = "serde", serde(skip))]
    pub ehttp_cache: Option<Arc<crate::api::ehttp_cache::EhttpCache>>,

    // ViewModel (MVVM architecture)
    #[cfg(feature = "gui")]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub viewmodel: Option<ViewModel>,
}

impl Default for DureApp {
    fn default() -> Self {
        Self {
            title: "DureApp Window".to_owned(),
            title_bar: false,
            collapsible: false,
            resizable: false,
            constrain: false,
            anchored: true,
            anchor: Align2::CENTER_TOP,
            anchor_offset: Vec2::ZERO,
            config: None,
            settings: Settings::default(),
            dlg_settings: crate::ui_dlg::DlgSettings::default(),
            dlg_about: crate::ui_dlg::DlgAbout::default(),
            #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
            install_status: InstallStatus::default(),
            #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
            install_dialog_open: false,
            #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
            install_message: String::new(),
            #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
            install_in_progress: false,
            update_status: String::new(),
            update_available: false,
            update_checking: false,
            update_download_url: String::new(),
            update_current_version: String::new(),
            update_latest_version: String::new(),
            user_mismatch_warning: None,
            screen_size_provider: None,
            cached_screen_size: None,
            screen_size_failed: false,
            screen_ratio: 1.0,
            square_size_factor: 0.3,
            square_center: Pos2::ZERO,
            square_corners: [Pos2::ZERO; 4],
            ehttp_cache: None,
            #[cfg(feature = "gui")]
            viewmodel: None,
        }
    }
}
