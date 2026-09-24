//! Android entry point for Dure
//!
//! Initializes the eframe app with Android-specific services injected

// Allow unsafe code for FFI entry point (android_main with #[no_mangle])
#![allow(unsafe_code)]

use android_activity::AndroidApp;
use eframe::NativeOptions;
use jni::JNIEnv;

use crate::dure::DureApp;
use crate::{dure_info, dure_debug, dure_warn, dure_error};

/// Get Android internal files directory using JNI
fn get_android_files_dir(app: &AndroidApp) -> Option<String> {
    unsafe {
        // Get the Activity from AndroidApp
        let activity = app.activity_as_ptr();
        if activity.is_null() {
            dure_error!("Activity pointer is null");
            return None;
        }

        // Get JNI environment (ndk-context is already initialized by android-activity)
        let vm_ptr = app.vm_as_ptr() as *mut jni::sys::JavaVM;
        let vm = jni::JavaVM::from_raw(vm_ptr).ok()?;
        let mut env = vm.get_env().ok()?;

        // Get the Activity object
        let activity_obj = jni::objects::JObject::from_raw(activity as jni::sys::jobject);

        // Call getFilesDir() on the Activity
        let files_dir = env.call_method(
            &activity_obj,
            "getFilesDir",
            "()Ljava/io/File;",
            &[]
        ).ok()?;

        // Get the File object
        let file_obj = files_dir.l().ok()?;

        // Call getAbsolutePath() on the File object
        let path_result = env.call_method(
            &file_obj,
            "getAbsolutePath",
            "()Ljava/lang/String;",
            &[]
        ).ok()?;

        // Get the String object
        let path_jstring = path_result.l().ok()?;
        let jstring = jni::objects::JString::from(path_jstring);
        let path_string = env.get_string(&jstring).ok()?;

        Some(path_string.to_string_lossy().to_string())
    }
}

/// Get Android status bar height via JNI
fn get_status_bar_height(app: &AndroidApp) -> f32 {
    unsafe {
        let activity = app.activity_as_ptr();
        if activity.is_null() {
            dure_warn!("Activity pointer is null - cannot get status bar height");
            return 0.0;
        }

        let vm_ptr = app.vm_as_ptr() as *mut jni::sys::JavaVM;
        let vm = match jni::JavaVM::from_raw(vm_ptr) {
            Ok(vm) => vm,
            Err(e) => {
                dure_warn!("Failed to get JavaVM: {:?}", e);
                return 0.0;
            }
        };

        let mut env = match vm.get_env() {
            Ok(env) => env,
            Err(e) => {
                dure_warn!("Failed to get JNI env: {:?}", e);
                return 0.0;
            }
        };

        let activity_obj = jni::objects::JObject::from_raw(activity as jni::sys::jobject);

        // Get Resources: activity.getResources()
        let resources = match env.call_method(&activity_obj, "getResources", "()Landroid/content/res/Resources;", &[]) {
            Ok(r) => match r.l() {
                Ok(obj) => obj,
                Err(e) => {
                    dure_warn!("Failed to get Resources object: {:?}", e);
                    return 0.0;
                }
            },
            Err(e) => {
                dure_warn!("Failed to call getResources(): {:?}", e);
                return 0.0;
            }
        };

        // Get resource ID: resources.getIdentifier("status_bar_height", "dimen", "android")
        let name = match env.new_string("status_bar_height") {
            Ok(s) => s,
            Err(_) => return 0.0,
        };
        let def_type = match env.new_string("dimen") {
            Ok(s) => s,
            Err(_) => return 0.0,
        };
        let def_package = match env.new_string("android") {
            Ok(s) => s,
            Err(_) => return 0.0,
        };

        let resource_id = match env.call_method(
            &resources,
            "getIdentifier",
            "(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)I",
            &[
                jni::objects::JValue::Object(&name),
                jni::objects::JValue::Object(&def_type),
                jni::objects::JValue::Object(&def_package),
            ]
        ) {
            Ok(id) => match id.i() {
                Ok(i) => i,
                Err(e) => {
                    dure_warn!("Failed to get resource ID: {:?}", e);
                    return 0.0;
                }
            },
            Err(e) => {
                dure_warn!("Failed to call getIdentifier(): {:?}", e);
                return 0.0;
            }
        };

        if resource_id == 0 {
            dure_warn!("❌ status_bar_height resource not found");
            return 0.0;
        }

        dure_info!("📏 status_bar_height resource_id: {}", resource_id);

        // Get dimension in pixels: resources.getDimensionPixelSize(resource_id)
        let height_px = match env.call_method(
            &resources,
            "getDimensionPixelSize",
            "(I)I",
            &[jni::objects::JValue::Int(resource_id)]
        ) {
            Ok(h) => match h.i() {
                Ok(i) => i,
                Err(e) => {
                    dure_warn!("Failed to get height: {:?}", e);
                    return 0.0;
                }
            },
            Err(e) => {
                dure_warn!("Failed to call getDimensionPixelSize(): {:?}", e);
                return 0.0;
            }
        };

        dure_info!("✅ Status bar height: {} px", height_px);
        height_px as f32
    }
}

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

    // Get Android internal files directory using JNI and set it as an environment variable
    // so the rest of the app can access it via get_profiles_base_dir() and similar functions
    if let Some(files_dir) = get_android_files_dir(&app) {
        dure_info!("Android files directory: {}", files_dir);
        // Set environment variable for the app's data directory
        // This will be used by get_profiles_base_dir() and other path functions
        std::env::set_var("ANDROID_INTERNAL_DATA_PATH", files_dir);
    } else {
        dure_error!("Failed to get Android files directory via JNI!");
        // Fallback: try to use a reasonable default
        let fallback_path = "/data/data/app.dure.installer/files";
        dure_warn!("Using fallback path: {}", fallback_path);
        std::env::set_var("ANDROID_INTERNAL_DATA_PATH", fallback_path);
    }

    // NOTE: Config and database are NOT loaded here - they are loaded after profile selection
    // See mobile/src/dure.rs profile login/create handlers for config/DB initialization

    // Get status bar height for applying top padding in egui
    let status_bar_height = get_status_bar_height(&app);
    dure_info!("Status bar height: {} px", status_bar_height);
    std::env::set_var("ANDROID_STATUS_BAR_HEIGHT", status_bar_height.to_string());

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
