//! Integration tests for multi-profile system
//!
//! Tests the complete workflow:
//! 1. Create profile → verify directory structure
//! 2. Login → verify password and load profile
//! 3. Multi-profile isolation → verify separate databases
//! 4. Delete → verify cleanup

use anyhow::Result;
use dure::calc::profile::{ProfileManager, ProfileContext, ProfileError};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// ============================================================================
// Test Helpers
// ============================================================================

/// Create a temporary profiles base directory for testing
fn setup_test_env() -> Result<TempDir> {
    let temp_dir = tempfile::tempdir()?;
    Ok(temp_dir)
}

/// Get test profiles base directory
fn get_test_base_dir(temp_dir: &TempDir) -> PathBuf {
    temp_dir.path().to_path_buf()
}

/// Create a test profile context with custom base directory
fn create_test_profile_context(base_dir: &PathBuf, name: &str) -> Result<ProfileContext> {
    let config_dir = base_dir.join(name);

    Ok(ProfileContext {
        name: name.to_string(),
        config_file: config_dir.join("config.yml"),
        db_path: config_dir.join("dure.db"),
        kdbx_path: config_dir.join("key.kdbx"),
        kpkey_path: config_dir.join("id_ed25519"),
        kppubkey_path: config_dir.join("id_ed25519.pub"),
        config_dir,
    })
}

/// Verify profile directory structure
fn verify_profile_structure(ctx: &ProfileContext) -> Result<()> {
    // Directory exists
    assert!(ctx.config_dir.exists(), "Profile directory should exist");
    assert!(ctx.config_dir.is_dir(), "Profile directory should be a directory");

    // Required files exist
    assert!(ctx.config_file.exists(), "config.yml should exist");
    assert!(ctx.kpkey_path.exists(), "id_ed25519 should exist");
    assert!(ctx.kppubkey_path.exists(), "id_ed25519.pub should exist");
    assert!(ctx.kdbx_path.exists(), "key.kdbx should exist");

    // Verify file types
    assert!(ctx.config_file.is_file(), "config.yml should be a file");
    assert!(ctx.kpkey_path.is_file(), "id_ed25519 should be a file");
    assert!(ctx.kppubkey_path.is_file(), "id_ed25519.pub should be a file");
    assert!(ctx.kdbx_path.is_file(), "key.kdbx should be a file");

    Ok(())
}

// ============================================================================
// Workflow Tests
// ============================================================================

#[test]
fn test_complete_workflow_create_verify_delete() -> Result<()> {
    // Arrange
    let profile_name = "test_profile_workflow";
    let password = "TestPassword123!";

    // Act 1: Create profile
    let ctx = ProfileManager::create_profile(profile_name, password)?;

    // Assert 1: Verify profile was created correctly
    assert_eq!(ctx.name, profile_name);
    verify_profile_structure(&ctx)?;

    // Act 2: Verify password (should succeed)
    let verify_result = ProfileManager::verify_password(profile_name, password);

    // Assert 2: Password verification should succeed
    assert!(verify_result.is_ok(), "Password verification should succeed");

    // Act 3: Verify wrong password (should fail)
    let wrong_password_result = ProfileManager::verify_password(profile_name, "WrongPassword");

    // Assert 3: Wrong password should fail
    assert!(wrong_password_result.is_err(), "Wrong password should fail");
    match wrong_password_result.unwrap_err().downcast::<ProfileError>() {
        Ok(ProfileError::IncorrectPassword) => {}, // Expected
        _ => panic!("Expected IncorrectPassword error"),
    }

    // Act 4: Delete profile
    ProfileManager::delete_profile(profile_name)?;

    // Assert 4: Profile directory should be gone
    assert!(!ctx.config_dir.exists(), "Profile directory should be deleted");

    Ok(())
}

#[test]
fn test_multi_profile_isolation() -> Result<()> {
    // Arrange
    let profile1_name = "test_profile_isolation_1";
    let profile2_name = "test_profile_isolation_2";
    let password1 = "Password1!";
    let password2 = "Password2!";

    // Act 1: Create two profiles
    let ctx1 = ProfileManager::create_profile(profile1_name, password1)?;
    let ctx2 = ProfileManager::create_profile(profile2_name, password2)?;

    // Assert 1: Both profiles exist with separate directories
    assert!(ctx1.config_dir.exists());
    assert!(ctx2.config_dir.exists());
    assert_ne!(ctx1.config_dir, ctx2.config_dir, "Profiles should have separate directories");

    // Assert 2: Verify directory structure for both
    verify_profile_structure(&ctx1)?;
    verify_profile_structure(&ctx2)?;

    // Assert 3: Database files are separate
    assert_ne!(ctx1.db_path, ctx2.db_path, "Database paths should be different");

    // Assert 4: KeePass databases are separate
    assert_ne!(ctx1.kdbx_path, ctx2.kdbx_path, "KeePass paths should be different");

    // Assert 5: SSH keys are separate
    assert_ne!(ctx1.kpkey_path, ctx2.kpkey_path, "SSH key paths should be different");

    // Assert 6: Each profile only accepts its own password
    assert!(ProfileManager::verify_password(profile1_name, password1).is_ok());
    assert!(ProfileManager::verify_password(profile2_name, password2).is_ok());
    assert!(ProfileManager::verify_password(profile1_name, password2).is_err());
    assert!(ProfileManager::verify_password(profile2_name, password1).is_err());

    // Cleanup
    ProfileManager::delete_profile(profile1_name)?;
    ProfileManager::delete_profile(profile2_name)?;

    Ok(())
}

#[test]
fn test_create_list_delete_multiple_profiles() -> Result<()> {
    // Arrange
    let profiles = vec![
        ("test_profile_list_1", "Pass1!"),
        ("test_profile_list_2", "Pass2!"),
        ("test_profile_list_3", "Pass3!"),
    ];

    // Act 1: Create multiple profiles
    for (name, password) in &profiles {
        ProfileManager::create_profile(name, password)?;
    }

    // Act 2: List profiles
    let profile_list = ProfileManager::list_profiles()?;

    // Assert 1: All created profiles should be in the list
    for (name, _) in &profiles {
        assert!(
            profile_list.contains(&name.to_string()),
            "Profile {} should be in list",
            name
        );
    }

    // Act 3: Delete one profile
    ProfileManager::delete_profile(profiles[0].0)?;

    // Act 4: List profiles again
    let updated_list = ProfileManager::list_profiles()?;

    // Assert 2: Deleted profile should not be in list
    assert!(
        !updated_list.contains(&profiles[0].0.to_string()),
        "Deleted profile should not be in list"
    );

    // Assert 3: Other profiles should still be in list
    assert!(updated_list.contains(&profiles[1].0.to_string()));
    assert!(updated_list.contains(&profiles[2].0.to_string()));

    // Cleanup remaining profiles
    ProfileManager::delete_profile(profiles[1].0)?;
    ProfileManager::delete_profile(profiles[2].0)?;

    Ok(())
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_create_profile_already_exists() -> Result<()> {
    // Arrange
    let profile_name = "test_profile_duplicate";
    let password = "Password123!";

    // Act 1: Create profile
    ProfileManager::create_profile(profile_name, password)?;

    // Act 2: Try to create same profile again
    let result = ProfileManager::create_profile(profile_name, password);

    // Assert: Should fail with AlreadyExists error
    assert!(result.is_err());
    match result.unwrap_err().downcast::<ProfileError>() {
        Ok(ProfileError::AlreadyExists(name)) => {
            assert_eq!(name, profile_name);
        }
        _ => panic!("Expected AlreadyExists error"),
    }

    // Cleanup
    ProfileManager::delete_profile(profile_name)?;

    Ok(())
}

#[test]
fn test_delete_nonexistent_profile() {
    // Arrange
    let profile_name = "nonexistent_profile";

    // Act
    let result = ProfileManager::delete_profile(profile_name);

    // Assert: Should fail with NotFound error
    assert!(result.is_err());
    match result.unwrap_err().downcast::<ProfileError>() {
        Ok(ProfileError::NotFound(name)) => {
            assert_eq!(name, profile_name);
        }
        _ => panic!("Expected NotFound error"),
    }
}

#[test]
fn test_verify_password_nonexistent_profile() {
    // Arrange
    let profile_name = "nonexistent_profile";
    let password = "AnyPassword";

    // Act
    let result = ProfileManager::verify_password(profile_name, password);

    // Assert: Should fail with NotFound error
    assert!(result.is_err());
    match result.unwrap_err().downcast::<ProfileError>() {
        Ok(ProfileError::NotFound(name)) => {
            assert_eq!(name, profile_name);
        }
        _ => panic!("Expected NotFound error"),
    }
}

// ============================================================================
// Validation Tests
// ============================================================================

#[test]
fn test_validate_name_valid_names() {
    // Valid names
    let valid_names = vec![
        "a",
        "test",
        "Test123",
        "user-profile",
        "user_profile",
        "Profile-2024",
    ];

    for name in valid_names {
        assert!(
            ProfileManager::validate_name(name).is_ok(),
            "Name '{}' should be valid",
            name
        );
    }

    // Test max length separately (64 chars)
    let max_length_name = "a".repeat(64);
    assert!(
        ProfileManager::validate_name(&max_length_name).is_ok(),
        "64 character name should be valid"
    );
}

#[test]
fn test_validate_name_invalid_names() {
    // Invalid names (using &str slices)
    let invalid_names: Vec<&str> = vec![
        "",                    // Empty
        "user profile",        // Space
        "user@profile",        // Special char
        "user.profile",        // Dot
        "user/profile",        // Slash
        "user\\profile",       // Backslash
        "user!profile",        // Exclamation
        "用户",                // Non-ASCII
    ];

    for name in invalid_names {
        assert!(
            ProfileManager::validate_name(name).is_err(),
            "Name '{}' should be invalid",
            name
        );
    }

    // Test too long name separately (65 chars)
    let too_long_name = "a".repeat(65);
    assert!(
        ProfileManager::validate_name(&too_long_name).is_err(),
        "65 character name should be invalid"
    );
}

// ============================================================================
// Profile Directory Structure Tests
// ============================================================================

#[test]
fn test_profile_directory_contains_all_required_files() -> Result<()> {
    // Arrange
    let profile_name = "test_profile_files";
    let password = "Password123!";

    // Act
    let ctx = ProfileManager::create_profile(profile_name, password)?;

    // Assert: All required files exist
    let required_files = vec![
        ("config.yml", &ctx.config_file),
        ("id_ed25519", &ctx.kpkey_path),
        ("id_ed25519.pub", &ctx.kppubkey_path),
        ("key.kdbx", &ctx.kdbx_path),
    ];

    for (name, path) in required_files {
        assert!(path.exists(), "{} should exist", name);
        assert!(path.is_file(), "{} should be a file", name);

        // Verify file is not empty (except config.yml which has minimal content)
        let metadata = fs::metadata(path)?;
        assert!(metadata.len() > 0, "{} should not be empty", name);
    }

    // Cleanup
    ProfileManager::delete_profile(profile_name)?;

    Ok(())
}

#[test]
fn test_ssh_keypair_format() -> Result<()> {
    // Arrange
    let profile_name = "test_ssh_keys";
    let password = "Password123!";

    // Act
    let ctx = ProfileManager::create_profile(profile_name, password)?;

    // Assert: Private key has correct format
    let private_key = fs::read_to_string(&ctx.kpkey_path)?;
    assert!(
        private_key.contains("-----BEGIN OPENSSH PRIVATE KEY-----"),
        "Private key should have OpenSSH header"
    );
    assert!(
        private_key.contains("-----END OPENSSH PRIVATE KEY-----"),
        "Private key should have OpenSSH footer"
    );

    // Assert: Public key has correct format
    let public_key = fs::read_to_string(&ctx.kppubkey_path)?;
    assert!(
        public_key.starts_with("ssh-ed25519 "),
        "Public key should start with ssh-ed25519"
    );
    assert!(
        public_key.contains("dure-profile"),
        "Public key should contain comment"
    );

    // Cleanup
    ProfileManager::delete_profile(profile_name)?;

    Ok(())
}

// ============================================================================
// Cleanup Tests
// ============================================================================

#[test]
fn test_delete_profile_removes_all_files() -> Result<()> {
    // Arrange
    let profile_name = "test_profile_cleanup";
    let password = "Password123!";

    // Act 1: Create profile
    let ctx = ProfileManager::create_profile(profile_name, password)?;

    // Remember paths before deletion
    let config_dir = ctx.config_dir.clone();
    let config_file = ctx.config_file.clone();
    let kpkey_path = ctx.kpkey_path.clone();
    let kppubkey_path = ctx.kppubkey_path.clone();
    let kdbx_path = ctx.kdbx_path.clone();

    // Verify files exist before deletion
    assert!(config_dir.exists());
    assert!(config_file.exists());
    assert!(kpkey_path.exists());
    assert!(kppubkey_path.exists());
    assert!(kdbx_path.exists());

    // Act 2: Delete profile
    ProfileManager::delete_profile(profile_name)?;

    // Assert: All files and directory should be gone
    assert!(!config_dir.exists(), "Profile directory should be deleted");
    assert!(!config_file.exists(), "config.yml should be deleted");
    assert!(!kpkey_path.exists(), "id_ed25519 should be deleted");
    assert!(!kppubkey_path.exists(), "id_ed25519.pub should be deleted");
    assert!(!kdbx_path.exists(), "key.kdbx should be deleted");

    Ok(())
}
