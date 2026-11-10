//! Language initialization persistence tests
//!
//! Tests that language settings persist across profile loads

use dure::{Settings, i18n};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[cfg(feature = "gui")]
#[test]
fn test_language_setting_persists_after_save_and_reload() {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let settings_path = temp_dir.path().join("settings.yml");

    // Create settings with Korean language
    let mut settings = Settings::default();
    settings.language = "ko-KR".to_string();

    // Act - Save settings
    settings.save(&settings_path).unwrap();

    // Act - Load settings from disk
    let loaded_settings = Settings::load(&settings_path).unwrap();

    // Assert
    assert_eq!(loaded_settings.language, "ko-KR", "Language setting should persist");
}

#[cfg(feature = "gui")]
#[test]
fn test_default_language_is_auto() {
    // Arrange & Act
    let settings = Settings::default();

    // Assert
    assert_eq!(settings.language, "Auto", "Default language should be Auto");
}

#[cfg(feature = "gui")]
#[test]
fn test_language_applied_to_i18n_system() {
    // Arrange
    i18n::init_i18n("Auto").unwrap();

    // Act - Set language to Korean
    i18n::set_language("ko-KR").unwrap();

    // Assert - Verify i18n system accepted the language
    // (i18n system doesn't provide a getter, so this test verifies no error occurred)
    // In real usage, UI would display Korean text
}

#[cfg(feature = "gui")]
#[test]
fn test_load_or_default_with_missing_file() {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let missing_path = temp_dir.path().join("nonexistent.yml");

    // Act
    let settings = Settings::load_or_default(&missing_path);

    // Assert - Should return default settings (not error)
    assert_eq!(settings.language, "Auto");
    assert_eq!(settings.log_level, "Error");
}

#[cfg(feature = "gui")]
#[test]
fn test_korean_language_setting_roundtrip() {
    // Arrange
    let temp_dir = TempDir::new().unwrap();
    let settings_path = temp_dir.path().join("settings.yml");

    // Create settings with Korean
    let mut original = Settings::default();
    original.language = "ko-KR".to_string();
    original.log_level = "Debug".to_string();
    original.show_logs = true;

    // Act - Save and reload
    original.save(&settings_path).unwrap();
    let loaded = Settings::load(&settings_path).unwrap();

    // Assert - All settings preserved
    assert_eq!(loaded.language, "ko-KR");
    assert_eq!(loaded.log_level, "Debug");
    assert_eq!(loaded.show_logs, true);
}

#[cfg(not(feature = "gui"))]
#[test]
fn test_headless_build_compiles() {
    // Headless builds don't support i18n
    // This test verifies the module compiles without GUI feature
    assert!(true);
}
