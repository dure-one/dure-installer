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
}
