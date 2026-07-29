//! Android entry point for Dure
//!
//! Initializes the eframe app with Android-specific services injected

use android_activity::AndroidApp;
use eframe::NativeOptions;
use std::sync::Arc;

use crate::dure::{DureApp, WallpaperSetter};

/// Android background image setter service (legacy compatibility)
struct AndroidWallpaperService;

impl WallpaperSetter for AndroidWallpaperService {
    fn set_wallpaper_from_bytes(&self, bytes: &[u8]) -> std::io::Result<bool> {
        // Delegate to android_wallpaper module (legacy feature)
        crate::android_wallpaper::set_wallpaper_from_bytes(bytes)
    }
}

/// Android entry point
#[no_mangle]
pub fn android_main(app: AndroidApp) {
    // Initialize Android logger
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Debug)
            .with_tag("Dure")
    );

    dure_info!("Dure v{} starting on Android", env!("CARGO_PKG_VERSION"));

    // Initialize application configuration and database
    let config = crate::Config::new().unwrap_or_else(|e| {
        dure_error!("Failed to initialize application config: {}", e);
        std::panic::panic_any("Failed to initialize config");
    });
    let db_path = config.data_dir.join("dure.db");
    crate::calc::db::set_db_path(db_path.to_string_lossy().to_string());
    dure_info!("Database path set to: {}", db_path.display());

    // Initialize background image bridge (legacy feature)
    crate::android_wallpaper::init_wallpaper_bridge();

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

            let mut app = DureApp::default();

            // Inject Android background image setter (legacy feature)
            app.set_wallpaper_setter(Arc::new(AndroidWallpaperService));

            dure_info!("DureApp initialized with Android services");

            Ok(Box::new(app))
        }),
    ) {
        Ok(_) => {
            dure_info!("DureApp exited successfully");
        }
        Err(e) => {
            dure_error!("DureApp failed: {}", e);
        }
    }
}
