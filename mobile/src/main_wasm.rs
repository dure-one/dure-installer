//! WASM entry point for Dure
//!
//! This module contains the WASM-specific initialization code that runs
//! when the application is loaded in a web browser.

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use eframe::wasm_bindgen::JsCast as _;

/// WASM entry point — called automatically by the browser via wasm-bindgen.
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn wasm_start() {
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| {
                    // Load Material3 fonts and theme
                    use egui_material3::theme::{
                        load_fonts, load_theme_from_json_str, load_themes,
                        setup_local_fonts_from_bytes, setup_local_theme,
                    };
                    use egui_material3::*;

                    // In WASM the filesystem is unavailable, so theme files must be
                    // embedded at compile time.  We skip setup_local_theme here and
                    // call load_theme_from_json_str after load_themes() instead.

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
                    // Apply the lightblue theme embedded at compile time (filesystem
                    // is unavailable in WASM, so this must use include_str!).
                    if let Err(e) = load_theme_from_json_str(include_str!(
                        "../resources/material-theme-lightblue.json"
                    )) {
                        dure_warn!("Failed to load lightblue theme: {e}");
                    }

                    // Initialize i18n with Auto language detection
                    if let Err(e) = crate::i18n::init_i18n("Auto") {
                        dure_error!("Failed to initialize i18n: {}", e);
                    }

                    let app = crate::dure::DureApp::default();

                    dure_info!("DureApp initialized for WASM");

                    Ok(Box::new(app))
                }),
            )
            .await;

        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}
