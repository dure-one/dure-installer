//! Android entry point for Dure
//!
//! Initializes the eframe app with Android-specific services injected

// Allow unsafe code for FFI entry point (android_main with #[no_mangle])
#![allow(unsafe_code)]

use android_activity::AndroidApp;
use eframe::NativeOptions;

use crate::dure::DureApp;
use crate::{dure_info, dure_debug, dure_warn, dure_error};

/// Android entry point
///
/// # Safety
/// This function is called by the Android system and must have a stable ABI,
/// hence the use of `no_mangle`. The function is safe to call from Android.
#[unsafe(no_mangle)]
pub fn android_main(app: AndroidApp) {
    // Initialize Android logger
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Debug)
            .with_tag("Dure")
    );

    dure_info!("Dure v{} starting on Android", env!("CARGO_PKG_VERSION"));

    // Get Android internal data path and set it as an environment variable
    // so the rest of the app can access it via get_profiles_base_dir() and similar functions
    let internal_data_path = app.internal_data_path();
    dure_info!("Android internal data path: {:?}", internal_data_path);

    // Set environment variable for the app's data directory
    // This will be used by get_profiles_base_dir() and other path functions
    std::env::set_var("ANDROID_INTERNAL_DATA_PATH", internal_data_path.to_string_lossy().to_string());

    // NOTE: Config and database are NOT loaded here - they are loaded after profile selection
    // See mobile/src/dure.rs profile login/create handlers for config/DB initialization

    // Set up panic handler
    std::panic::set_hook(Box::new(|panic_info| {
        let payload = panic_info.payload();

        // Check if this is the expected "winit window doesn't exist" panic
        // This happens when Android destroys the activity after background operations
        let is_expected_window_panic = if let Some(s) = payload.downcast_ref::<&str>() {
            s.contains("winit window doesn't exist")
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.contains("winit window doesn't exist")
        } else {
            false
        };

        if is_expected_window_panic {
            dure_warn!(
                "Expected window destruction during activity lifecycle change: {}",
                panic_info
            );
            // Don't treat this as a critical error - it's normal during activity lifecycle
            return;
        }

        // For other panics, log as errors
        dure_error!("PANIC: {}", panic_info);
        if let Some(location) = panic_info.location() {
            dure_error!("Location: {}:{}", location.file(), location.line());
        }
    }));

    let options = NativeOptions {
        android_app: Some(app),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };

    dure_info!("🔥 CALLING eframe::run_native");
    match eframe::run_native(
        "Dure",
        options,
        Box::new(|cc| {
            // Load Material3 fonts and theme
            use egui_material3::theme::{
                load_fonts, load_themes, setup_google_fonts, setup_local_fonts_from_bytes,
                setup_local_theme,
            };
            use egui_material3::*;

            // Setup theme from file FIRST (before fonts)
            setup_local_theme(Some("resources/material-theme-lightblue.json"));

            // Prepare local fonts including Material Symbols (using include_bytes!)
            // setup_local_fonts_from_bytes(
            //     "MaterialSymbolsOutlined",
            //     include_bytes!("../resources/MaterialSymbolsOutlined[FILL,GRAD,opsz,wght].ttf"),
            // );
            setup_local_fonts_from_bytes(
                "NotoSansKr",
                include_bytes!("../resources/noto-sans-kr.ttf"),
            );

            // Register Korean font with egui for proper text rendering
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert(
                "NotoSansKr".to_owned(),
                std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
                    "../resources/noto-sans-kr.ttf"
                ))),
            );
            // Put Korean font first in proportional and monospace families
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "NotoSansKr".to_owned());
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push("NotoSansKr".to_owned());
            cc.egui_ctx.set_fonts(fonts);

            // Prepare themes from build-time constants
            setup_local_theme(None);
            // Install image loaders
            egui_extras::install_image_loaders(&cc.egui_ctx);
            // Load all prepared fonts and themes
            load_fonts(&cc.egui_ctx);
            load_themes();

            // Initialize i18n with Auto language detection
            if let Err(e) = crate::i18n::init_i18n("Auto") {
                dure_error!("Failed to initialize i18n: {}", e);
            }

            let app = DureApp::default();

            dure_info!("🔥 DureApp initialized with Android services");
            dure_info!("🔥 About to return app to eframe");

            Ok(Box::new(app))
        }),
    ) {
        Ok(_) => {
            dure_info!("🔥 eframe::run_native RETURNED OK");
        }
        Err(e) => {
            dure_error!("🔥 eframe::run_native RETURNED ERROR: {}", e);
        }
    }
    dure_info!("🔥 android_main EXITING");
}
