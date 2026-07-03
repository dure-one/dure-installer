//! i18n (Internationalization) module for Dure
//!
//! This module handles fluent translation initialization and language switching.
//! Translation files are embedded at compile time from mobile/assets/languages/fluent/

use anyhow::Result;

/// Embedded fluent translation files
#[cfg(feature = "gui")]
const EN_US_FTL: &str = include_str!("../assets/languages/fluent/en-US.ftl");
#[cfg(feature = "gui")]
const KO_KR_FTL: &str = include_str!("../assets/languages/fluent/ko-KR.ftl");

/// Initialize the i18n system
///
/// This loads all embedded translation files and sets the initial language.
/// Should be called once during application startup.
///
/// # Arguments
/// * `language` - Language code ("en-US", "ko-KR", or "Auto")
#[cfg(feature = "gui")]
pub fn init_i18n(language: &str) -> Result<()> {
    log::info!("Initializing i18n system with language: {}", language);

    // On Windows, Fluent wraps placeables in Unicode directionality marks
    // (U+2068 / U+2069) that some native text renderers display as garbage.
    // Disable them before loading any bundles.
    #[cfg(target_os = "windows")]
    egui_i18n::set_use_isolating(false);

    // Load all translation bundles
    egui_i18n::load_translations_from_text("en-US", EN_US_FTL)
        .map_err(|e| anyhow::anyhow!("Failed to load en-US translations: {}", e))?;
    log::debug!("Loaded en-US translations");

    egui_i18n::load_translations_from_text("ko-KR", KO_KR_FTL)
        .map_err(|e| anyhow::anyhow!("Failed to load ko-KR translations: {}", e))?;
    log::debug!("Loaded ko-KR translations");

    // Set fallback language
    egui_i18n::set_fallback("en-US");

    // Set initial language
    let lang_to_use = if language == "Auto" {
        detect_system_language()
    } else {
        language.to_string()
    };

    set_language(&lang_to_use)?;

    log::info!(
        "i18n initialized successfully with language: {}",
        lang_to_use
    );
    Ok(())
}

/// Initialize the i18n system (no-op for headless builds)
#[cfg(not(feature = "gui"))]
pub fn init_i18n(_language: &str) -> Result<()> {
    log::debug!("i18n disabled (headless build)");
    Ok(())
}

/// Set the current language
///
/// # Arguments
/// * `language` - Language code ("en-US", "ko-KR", or "Auto")
#[cfg(feature = "gui")]
pub fn set_language(language: &str) -> Result<()> {
    let lang_code = if language == "Auto" {
        detect_system_language()
    } else {
        language.to_string()
    };

    log::info!("Setting language to: {}", lang_code);
    egui_i18n::set_language(&lang_code);

    Ok(())
}

/// Set the current language (no-op for headless builds)
#[cfg(not(feature = "gui"))]
pub fn set_language(_language: &str) -> Result<()> {
    Ok(())
}

/// Detect the system language
///
/// Returns a language code string like "en-US" or "ko-KR".
/// Falls back to "en-US" if system locale cannot be detected.
pub fn detect_system_language() -> String {
    let locale = detect_locale();
    map_locale_to_language(&locale)
}

/// Detect the raw locale string from the platform.
///
/// On WASM, reads `navigator.language` from the browser.
/// On other platforms, uses `sys_locale`.
fn detect_locale() -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        let lang = web_sys::window().and_then(|w| w.navigator().language());
        if lang.is_some() {
            log::debug!("Detected browser language: {:?}", lang);
        } else {
            log::debug!("Could not detect browser language");
        }
        return lang;
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let locale = sys_locale::get_locale();
        if let Some(ref l) = locale {
            log::debug!("Detected system locale: {}", l);
        } else {
            log::debug!("Could not detect system locale");
        }
        locale
    }
}

/// Map a raw locale string to a supported language code.
fn map_locale_to_language(locale: &Option<String>) -> String {
    match locale {
        Some(locale) => {
            if locale.starts_with("ko") {
                "ko-KR".to_string()
            } else if locale.starts_with("en") {
                "en-US".to_string()
            } else {
                log::debug!("Unsupported locale {}, falling back to en-US", locale);
                "en-US".to_string()
            }
        }
        None => {
            log::debug!("No locale detected, falling back to en-US");
            "en-US".to_string()
        }
    }
}

/// Get list of available languages
pub fn get_available_languages() -> Vec<String> {
    vec!["en-US".to_string(), "ko-KR".to_string()]
}
