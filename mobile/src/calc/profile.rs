//! Profile management for multi-profile support
//!
//! Provides business logic for creating, loading, and managing profiles.
//! Each profile has isolated config, database, and credentials in its own directory:
//! ~/.config/dure_installer/{profile_name}/
//!
//! Each profile contains:
//! - config.yml - Application configuration
//! - id_ed25519, id_ed25519.pub - SSH keys
//! - key.kdbx - KeePass database (password protected)
//! - dure.db - SQLite database

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

/// Profile metadata and paths
#[derive(Debug, Clone)]
pub struct ProfileContext {
    pub name: String,
    pub config_dir: PathBuf,       // ~/.config/dure_installer/{name}/
    pub config_file: PathBuf,      // .../config.yml
    pub db_path: PathBuf,          // .../dure.db
    pub kdbx_path: PathBuf,        // .../key.kdbx
    pub kpkey_path: PathBuf,       // .../id_ed25519
    pub kppubkey_path: PathBuf,    // .../id_ed25519.pub
}

impl ProfileContext {
    /// Create ProfileContext from profile name
    ///
    /// Constructs all file paths for the profile based on the base directory.
    /// Does NOT create files or verify they exist - use ProfileManager for that.
    pub fn new(name: impl Into<String>) -> Result<Self> {
        let name = name.into();
        let config_dir = crate::get_profiles_base_dir()?.join(&name);

        Ok(Self {
            name,
            config_file: config_dir.join("config.yml"),
            db_path: config_dir.join("dure.db"),
            kdbx_path: config_dir.join("key.kdbx"),
            kpkey_path: config_dir.join("id_ed25519"),
            kppubkey_path: config_dir.join("id_ed25519.pub"),
            config_dir,
        })
    }
}

/// Profile-related errors
#[derive(Error, Debug)]
pub enum ProfileError {
    #[error("Profile '{0}' not found")]
    NotFound(String),

    #[error("Profile '{0}' already exists")]
    AlreadyExists(String),

    #[error("Invalid profile name: {0}")]
    InvalidName(String),

    #[error("Incorrect password")]
    IncorrectPassword,

    #[error("Profile directory corrupted: missing {0}")]
    CorruptedProfile(String),

    #[error("Failed to create profile: {0}")]
    CreationFailed(String),

    #[error("Failed to delete profile: {0}")]
    DeletionFailed(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Profile manager - stateless operations
pub struct ProfileManager;

impl ProfileManager {
    /// Validate profile name
    ///
    /// Rules:
    /// - Only a-z, A-Z, 0-9, -, _ allowed
    /// - Length 1-64 characters
    /// - Case-sensitive
    pub fn validate_name(name: &str) -> Result<(), ProfileError> {
        // Check empty
        if name.is_empty() {
            return Err(ProfileError::InvalidName("name cannot be empty".to_string()));
        }

        // Check length
        if name.len() > 64 {
            return Err(ProfileError::InvalidName(format!(
                "name too long ({} chars, max 64)",
                name.len()
            )));
        }

        // Check valid characters: a-z, A-Z, 0-9, -, _
        for ch in name.chars() {
            if !ch.is_ascii_alphanumeric() && ch != '-' && ch != '_' {
                return Err(ProfileError::InvalidName(format!(
                    "invalid character '{}' in name (only a-z, A-Z, 0-9, -, _ allowed)",
                    ch
                )));
            }
        }

        Ok(())
    }

    /// List all valid profiles
    ///
    /// Scans the profiles base directory and returns names of all valid profiles.
    /// A valid profile directory must contain: config.yml, id_ed25519, id_ed25519.pub, key.kdbx
    pub fn list_profiles() -> Result<Vec<String>> {
        let base_dir = crate::get_profiles_base_dir()?;

        // If directory doesn't exist, return empty list
        if !base_dir.exists() {
            return Ok(Vec::new());
        }

        let mut profiles = Vec::new();

        // Scan directory for valid profiles
        for entry in fs::read_dir(&base_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Skip if not a directory
            if !path.is_dir() {
                continue;
            }

            // Get directory name as profile name
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                // Validate directory contains required files
                if Self::validate_profile_dir(&path).is_ok() {
                    profiles.push(name.to_string());
                }
            }
        }

        Ok(profiles)
    }

    /// Validate that a profile directory contains all required files
    ///
    /// Required files: config.yml, id_ed25519, id_ed25519.pub, key.kdbx
    fn validate_profile_dir(dir: &PathBuf) -> Result<()> {
        let required_files = ["config.yml", "id_ed25519", "id_ed25519.pub", "key.kdbx"];

        for file in &required_files {
            let file_path = dir.join(file);
            if !file_path.exists() {
                return Err(ProfileError::CorruptedProfile(file.to_string()).into());
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_context_new() {
        std::env::set_var("DURE_TEST_PROFILES_DIR", "/tmp/test");

        let profile = ProfileContext::new("test-profile").unwrap();

        assert_eq!(profile.name, "test-profile");
        assert!(profile.config_dir.to_string_lossy().contains("test-profile"));
        assert!(profile.config_file.to_string_lossy().ends_with("config.yml"));
        assert!(profile.db_path.to_string_lossy().ends_with("dure.db"));
        assert!(profile.kdbx_path.to_string_lossy().ends_with("key.kdbx"));

        std::env::remove_var("DURE_TEST_PROFILES_DIR");
    }

    #[test]
    fn test_validate_name_valid() {
        // Valid names: a-z, A-Z, 0-9, -, _, 1-64 chars
        assert!(ProfileManager::validate_name("profile1").is_ok());
        assert!(ProfileManager::validate_name("my-profile").is_ok());
        assert!(ProfileManager::validate_name("my_profile").is_ok());
        assert!(ProfileManager::validate_name("Profile123").is_ok());
        assert!(ProfileManager::validate_name("a").is_ok());
        assert!(ProfileManager::validate_name("a".repeat(64).as_str()).is_ok());
    }

    #[test]
    fn test_validate_name_invalid() {
        // Empty name
        assert!(ProfileManager::validate_name("").is_err());

        // Too long (>64 chars)
        assert!(ProfileManager::validate_name(&"a".repeat(65)).is_err());

        // Invalid characters
        assert!(ProfileManager::validate_name("profile name").is_err()); // space
        assert!(ProfileManager::validate_name("profile.name").is_err()); // dot
        assert!(ProfileManager::validate_name("profile/name").is_err()); // slash
        assert!(ProfileManager::validate_name("profile@name").is_err()); // @
        assert!(ProfileManager::validate_name("profile#name").is_err()); // #
    }

    #[test]
    fn test_list_profiles_empty() {
        use std::fs;

        // Use temporary directory for test
        let test_dir = std::env::temp_dir().join("dure_test_list_empty");
        std::env::set_var("DURE_TEST_PROFILES_DIR", &test_dir);

        // Clean up any existing test directory
        let _ = fs::remove_dir_all(&test_dir);

        // Create empty profiles directory
        fs::create_dir_all(&test_dir).unwrap();

        // List should return empty vector
        let profiles = ProfileManager::list_profiles().unwrap();
        assert_eq!(profiles.len(), 0);

        // Cleanup
        fs::remove_dir_all(&test_dir).unwrap();
        std::env::remove_var("DURE_TEST_PROFILES_DIR");
    }
}
