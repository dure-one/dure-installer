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

    /// Delete a profile
    ///
    /// Removes the profile directory and all its contents.
    /// Returns NotFound error if profile doesn't exist.
    pub fn delete_profile(name: &str) -> Result<()> {
        // Get profile context
        let ctx = ProfileContext::new(name)?;

        // Check if profile exists
        if !ctx.config_dir.exists() {
            return Err(ProfileError::NotFound(name.to_string()).into());
        }

        // Remove profile directory and all contents
        fs::remove_dir_all(&ctx.config_dir)
            .with_context(|| format!("failed to delete profile directory: {}", name))?;

        dure_info!("Deleted profile: {}", name);
        Ok(())
    }

    /// Verify profile password
    ///
    /// Attempts to open the KeePass database with the given password.
    /// Returns Ok if password is correct, IncorrectPassword error if wrong.
    pub fn verify_password(name: &str, password: &str) -> Result<()> {
        // Get profile context
        let ctx = ProfileContext::new(name)?;

        // Check if profile exists
        if !ctx.config_dir.exists() {
            return Err(ProfileError::NotFound(name.to_string()).into());
        }

        // Try to open KeePass database with password
        use keepass::{Database, DatabaseKey};
        use std::fs::File;

        let mut file = File::open(&ctx.kdbx_path)
            .context("failed to open kdbx file")?;

        let key = DatabaseKey::new().with_password(password);

        // Attempt to open database
        match Database::open(&mut file, key) {
            Ok(_) => Ok(()),
            Err(_) => Err(ProfileError::IncorrectPassword.into()),
        }
    }

    /// Create a new profile
    ///
    /// Creates profile directory with:
    /// - config.yml (default configuration)
    /// - id_ed25519, id_ed25519.pub (SSH keypair)
    /// - key.kdbx (KeePass database)
    pub fn create_profile(name: &str, password: &str) -> Result<ProfileContext> {
        // Validate name
        Self::validate_name(name)?;

        // Get profile context
        let ctx = ProfileContext::new(name)?;

        // Check if already exists
        if ctx.config_dir.exists() {
            return Err(ProfileError::AlreadyExists(name.to_string()).into());
        }

        // Create profile directory
        fs::create_dir_all(&ctx.config_dir).context("failed to create profile directory")?;

        // Generate SSH keypair
        let (private_key, public_key) = Self::generate_keypair()?;

        // Write private key
        fs::write(&ctx.kpkey_path, private_key).context("failed to write private key")?;

        // Write public key
        fs::write(&ctx.kppubkey_path, public_key).context("failed to write public key")?;

        // Create KeePass database
        Self::create_kdbx(&ctx.kdbx_path, password)?;

        // Create default config.yml
        let default_config = "# Profile configuration\n";
        fs::write(&ctx.config_file, default_config).context("failed to write config file")?;

        dure_info!("Created profile: {}", name);
        Ok(ctx)
    }

    /// Generate Ed25519 SSH keypair
    ///
    /// Returns (private_key_pem, public_key_openssh)
    fn generate_keypair() -> Result<(String, String)> {
        use ed25519_dalek::{SigningKey, VerifyingKey};
        use rand::rngs::OsRng;

        // Generate keypair
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key: VerifyingKey = (&signing_key).into();

        // Format private key (PEM-like format for OpenSSH)
        let private_bytes = signing_key.to_bytes();
        let private_key = format!(
            "-----BEGIN OPENSSH PRIVATE KEY-----\n{}\n-----END OPENSSH PRIVATE KEY-----\n",
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, private_bytes)
        );

        // Format public key (OpenSSH format)
        let public_bytes = verifying_key.to_bytes();
        let public_key = format!(
            "ssh-ed25519 {} dure-profile\n",
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, public_bytes)
        );

        Ok((private_key, public_key))
    }

    /// Create KeePass database
    ///
    /// Creates a new KDBX4 database with the given password
    fn create_kdbx(path: &PathBuf, password: &str) -> Result<()> {
        use keepass::{Database, DatabaseKey};

        // Create new database
        let mut db = Database::new(Default::default());
        db.root.name = "Dure Profile".to_string();

        // Create database key with password
        let key = DatabaseKey::new().with_password(password);

        // Save database
        let mut file = fs::File::create(path).context("failed to create kdbx file")?;
        db.save(&mut file, key)
            .context("failed to save kdbx database")?;

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
        use std::time::{SystemTime, UNIX_EPOCH};

        // Use unique temporary directory for test
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let test_dir = std::env::temp_dir().join(format!("dure_test_list_empty_{}", timestamp));
        std::env::set_var("DURE_TEST_PROFILES_DIR", &test_dir);

        // Create empty profiles directory
        fs::create_dir_all(&test_dir).unwrap();

        // List should return empty vector
        let profiles = ProfileManager::list_profiles().unwrap();
        assert_eq!(profiles.len(), 0);

        // Cleanup
        fs::remove_dir_all(&test_dir).unwrap();
        std::env::remove_var("DURE_TEST_PROFILES_DIR");
    }

    #[test]
    fn test_create_profile_success() {
        use std::fs;
        use std::time::{SystemTime, UNIX_EPOCH};

        // Use unique temporary directory for test (avoid parallel test conflicts)
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let test_dir = std::env::temp_dir().join(format!("dure_test_create_success_{}", timestamp));
        std::env::set_var("DURE_TEST_PROFILES_DIR", &test_dir);

        // Create base directory
        fs::create_dir_all(&test_dir).unwrap();

        // Create profile
        let profile_name = "test-profile";
        let password = "test-password-123";
        let ctx = ProfileManager::create_profile(profile_name, password).unwrap();

        // Verify profile directory was created
        let profile_dir = test_dir.join(profile_name);
        assert!(profile_dir.exists());

        // Verify all required files exist
        assert!(profile_dir.join("config.yml").exists());
        assert!(profile_dir.join("id_ed25519").exists());
        assert!(profile_dir.join("id_ed25519.pub").exists());
        assert!(profile_dir.join("key.kdbx").exists());

        // Verify it appears in list
        let profiles = ProfileManager::list_profiles().unwrap();
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0], profile_name);

        // Cleanup
        fs::remove_dir_all(&test_dir).unwrap();
        std::env::remove_var("DURE_TEST_PROFILES_DIR");
    }

    #[test]
    fn test_create_profile_invalid_name() {
        use std::fs;

        // Use temporary directory for test
        let test_dir = std::env::temp_dir().join("dure_test_create_invalid");
        std::env::set_var("DURE_TEST_PROFILES_DIR", &test_dir);

        // Clean up any existing test directory
        let _ = fs::remove_dir_all(&test_dir);

        // Try to create profile with invalid name
        let result = ProfileManager::create_profile("invalid name", "password");
        assert!(result.is_err());

        // Verify error is InvalidName
        match result.unwrap_err().downcast_ref::<ProfileError>() {
            Some(ProfileError::InvalidName(_)) => {}
            _ => panic!("Expected InvalidName error"),
        }

        // Cleanup
        let _ = fs::remove_dir_all(&test_dir);
        std::env::remove_var("DURE_TEST_PROFILES_DIR");
    }

    #[test]
    fn test_verify_password_correct() {
        use std::fs;
        use std::time::{SystemTime, UNIX_EPOCH};

        // Use unique temporary directory for test
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let test_dir = std::env::temp_dir().join(format!("dure_test_verify_correct_{}", timestamp));
        std::env::set_var("DURE_TEST_PROFILES_DIR", &test_dir);

        // Create base directory and profile
        fs::create_dir_all(&test_dir).unwrap();
        let profile_name = "test-profile";
        let password = "test-password-123";
        ProfileManager::create_profile(profile_name, password).unwrap();

        // Verify correct password
        let result = ProfileManager::verify_password(profile_name, password);
        assert!(result.is_ok());

        // Cleanup
        fs::remove_dir_all(&test_dir).unwrap();
        std::env::remove_var("DURE_TEST_PROFILES_DIR");
    }

    #[test]
    fn test_verify_password_incorrect() {
        use std::fs;
        use std::time::{SystemTime, UNIX_EPOCH};

        // Use unique temporary directory for test
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let test_dir = std::env::temp_dir().join(format!("dure_test_verify_incorrect_{}", timestamp));
        std::env::set_var("DURE_TEST_PROFILES_DIR", &test_dir);

        // Create base directory and profile
        fs::create_dir_all(&test_dir).unwrap();
        let profile_name = "test-profile";
        let password = "test-password-123";
        ProfileManager::create_profile(profile_name, password).unwrap();

        // Verify incorrect password fails
        let result = ProfileManager::verify_password(profile_name, "wrong-password");
        assert!(result.is_err());

        // Verify error is IncorrectPassword
        match result.unwrap_err().downcast_ref::<ProfileError>() {
            Some(ProfileError::IncorrectPassword) => {}
            _ => panic!("Expected IncorrectPassword error"),
        }

        // Cleanup
        fs::remove_dir_all(&test_dir).unwrap();
        std::env::remove_var("DURE_TEST_PROFILES_DIR");
    }

    #[test]
    fn test_delete_profile() {
        use std::fs;
        use std::time::{SystemTime, UNIX_EPOCH};

        // Use unique temporary directory for test
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let test_dir = std::env::temp_dir().join(format!("dure_test_delete_{}", timestamp));
        std::env::set_var("DURE_TEST_PROFILES_DIR", &test_dir);

        // Create base directory and profile
        fs::create_dir_all(&test_dir).unwrap();
        let profile_name = "test-profile";
        let password = "test-password-123";
        ProfileManager::create_profile(profile_name, password).unwrap();

        // Verify profile exists
        let profiles = ProfileManager::list_profiles().unwrap();
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0], profile_name);

        // Delete profile
        let result = ProfileManager::delete_profile(profile_name);
        assert!(result.is_ok());

        // Verify profile no longer exists
        let profiles = ProfileManager::list_profiles().unwrap();
        assert_eq!(profiles.len(), 0);

        // Cleanup
        fs::remove_dir_all(&test_dir).unwrap();
        std::env::remove_var("DURE_TEST_PROFILES_DIR");
    }

    #[test]
    fn test_delete_profile_not_found() {
        use std::fs;
        use std::time::{SystemTime, UNIX_EPOCH};

        // Use unique temporary directory for test
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let test_dir = std::env::temp_dir().join(format!("dure_test_delete_not_found_{}", timestamp));
        std::env::set_var("DURE_TEST_PROFILES_DIR", &test_dir);

        // Create base directory
        fs::create_dir_all(&test_dir).unwrap();

        // Try to delete non-existent profile
        let result = ProfileManager::delete_profile("nonexistent");
        assert!(result.is_err());

        // Verify error is NotFound
        match result.unwrap_err().downcast_ref::<ProfileError>() {
            Some(ProfileError::NotFound(_)) => {}
            _ => panic!("Expected NotFound error"),
        }

        // Cleanup
        fs::remove_dir_all(&test_dir).unwrap();
        std::env::remove_var("DURE_TEST_PROFILES_DIR");
    }
}
