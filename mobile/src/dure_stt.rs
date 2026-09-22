#[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
use crate::install_stt::InstallStatus;
use crate::{Config, Settings};
use eframe::egui::{Align2, Pos2, Vec2};
use std::sync::Arc;

// ViewModel (MVVM architecture)
#[cfg(feature = "gui")]
use crate::viewmodel::ViewModel;

// Profile management
#[cfg(feature = "gui")]
use crate::calc::profile::ProfileContext;

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

    // Profile state (gui feature-gated)
    #[cfg(feature = "gui")]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub current_profile: Option<ProfileContext>,
    #[cfg(feature = "gui")]
    pub pending_profile_name: Option<String>,

    // Installation status (desktop only)
    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
    pub install_status: InstallStatus,
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
            #[cfg(feature = "gui")]
            current_profile: None,
            #[cfg(feature = "gui")]
            pending_profile_name: None,
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
            install_status: InstallStatus::default(),
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
            install_dialog_open: false,
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
            install_message: String::new(),
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
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

#[cfg(all(test, feature = "gui"))]
mod tests {
    use super::*;

    #[test]
    fn test_dure_app_default_initialization() {
        // Arrange & Act
        let app = DureApp::default();

        // Assert
        assert_eq!(app.title, "DureApp Window");
        assert!(!app.title_bar);
        assert!(!app.collapsible);
        assert!(!app.resizable);
        assert!(!app.constrain);
        assert!(app.anchored);
    }

    #[test]
    fn test_profile_state_default_none() {
        // Arrange & Act
        let app = DureApp::default();

        // Assert - profile state should be None
        assert!(app.current_profile.is_none(), "current_profile should be None on default");
        assert!(app.pending_profile_name.is_none(), "pending_profile_name should be None on default");
    }

    #[test]
    fn test_profile_state_fields_exist() {
        // Arrange & Act
        let mut app = DureApp::default();

        // Assert - can set profile state
        app.current_profile = ProfileContext::new("test-profile").ok();
        app.pending_profile_name = Some("test-profile".to_string());

        // Verify we can read them back
        assert!(app.current_profile.is_some());
        assert!(app.pending_profile_name.is_some());
        assert_eq!(app.pending_profile_name.as_ref().unwrap(), "test-profile");
    }

    #[test]
    fn test_profile_state_option_types() {
        // Arrange & Act
        let app = DureApp::default();

        // Assert - types are Options
        // These would fail at compile time if the types were wrong,
        // but we test the runtime behavior here
        assert!(app.current_profile.as_ref().is_none());
        assert!(app.pending_profile_name.as_ref().is_none());

        // Test that they can be used with Option combinators
        let name = app.pending_profile_name.as_deref();
        assert!(name.is_none());
    }

    #[test]
    fn test_profile_state_independence() {
        // Arrange
        let mut app1 = DureApp::default();
        let mut app2 = DureApp::default();

        // Act
        app1.pending_profile_name = Some("profile1".to_string());
        app2.pending_profile_name = Some("profile2".to_string());

        // Assert - ensure state is independent per instance
        assert_eq!(app1.pending_profile_name.as_ref().unwrap(), "profile1");
        assert_eq!(app2.pending_profile_name.as_ref().unwrap(), "profile2");
    }
}
