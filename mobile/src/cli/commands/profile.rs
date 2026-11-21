//! Profile management CLI commands

use crate::calc::profile::{ProfileContext, ProfileManager};
use anyhow::{Context, Result};

/// List all available profiles
pub fn list_profiles() -> Result<Vec<String>> {
    ProfileManager::list_profiles()
}

/// Create a new profile
pub fn create_profile(name: String, password: String) -> Result<()> {
    ProfileManager::create_profile(&name, &password)?;
    Ok(())
}

/// Delete an existing profile
pub fn delete_profile(name: String) -> Result<()> {
    ProfileManager::delete_profile(&name)
}

/// Login to a profile (verify password)
pub fn login_profile(name: String, password: String) -> Result<ProfileContext> {
    ProfileManager::verify_password(&name, &password)?;
    ProfileContext::new(name)
}

/// Execute profile list command
pub fn execute_profile_list() -> Result<()> {
    let profiles = list_profiles()?;

    if profiles.is_empty() {
        println!("No profiles found.");
        println!();
        println!("Run 'dure profile create <name>' to create a profile");
        return Ok(());
    }

    println!("Available profiles:");
    println!();
    for (idx, name) in profiles.iter().enumerate() {
        println!("{}. {}", idx + 1, name);
    }
    println!();
    println!("Run 'dure profile login <name>' to load a profile");

    Ok(())
}

/// Execute profile create command
pub fn execute_profile_create(name: String, password: String) -> Result<()> {
    create_profile(name.clone(), password)?;
    println!("✓ Profile '{}' created successfully", name);
    println!();
    println!("Run 'dure profile login {}' to use this profile", name);
    Ok(())
}

/// Execute profile delete command
pub fn execute_profile_delete(name: String) -> Result<()> {
    delete_profile(name.clone())?;
    println!("✓ Profile '{}' deleted", name);
    Ok(())
}

/// Execute profile login command
pub fn execute_profile_login(name: String, password: String) -> Result<ProfileContext> {
    let profile = login_profile(name, password)?;
    Ok(profile)
}

/// Logout from current profile
pub fn logout_profile() -> Result<()> {
    ProfileManager::clear_active_profile()
}

/// Execute profile logout command
pub fn execute_profile_logout() -> Result<()> {
    logout_profile()?;
    println!("✓ Logged out successfully");
    println!();
    println!("Run 'dure profile login <name>' to login again");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Helper: Create temp profiles base dir
    fn setup_test_env() -> Result<TempDir> {
        let temp_dir = TempDir::new()?;
        std::env::set_var("HOME", temp_dir.path());
        Ok(temp_dir)
    }

    #[test]
    fn test_profile_list_empty() {
        let _temp = setup_test_env().unwrap();

        // Should return empty list when no profiles
        let profiles = list_profiles().unwrap();
        assert!(profiles.is_empty());
    }

    #[test]
    fn test_profile_list_with_profiles() {
        let _temp = setup_test_env().unwrap();

        // Create test profiles
        ProfileManager::create_profile("test1", "pass1").unwrap();
        ProfileManager::create_profile("test2", "pass2").unwrap();

        let profiles = list_profiles().unwrap();
        assert_eq!(profiles.len(), 2);
        assert!(profiles.contains(&"test1".to_string()));
        assert!(profiles.contains(&"test2".to_string()));
    }

    #[test]
    fn test_profile_create_new() {
        let _temp = setup_test_env().unwrap();

        // Create profile with valid name
        let result = create_profile("my-profile".to_string(), "password123".to_string());
        assert!(result.is_ok());

        // Verify profile exists
        let ctx = ProfileContext::new("my-profile").unwrap();
        assert!(ctx.config_dir.exists());
        assert!(ctx.kdbx_path.exists());
    }

    #[test]
    fn test_profile_create_invalid_name() {
        let _temp = setup_test_env().unwrap();

        // Invalid characters
        let result = create_profile("test@profile".to_string(), "pass".to_string());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("invalid character"));

        // Empty name
        let result = create_profile("".to_string(), "pass".to_string());
        assert!(result.is_err());

        // Too long
        let long_name = "a".repeat(65);
        let result = create_profile(long_name, "pass".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_profile_delete_existing() {
        let _temp = setup_test_env().unwrap();

        // Create then delete
        ProfileManager::create_profile("to-delete", "pass").unwrap();
        let result = delete_profile("to-delete".to_string());
        assert!(result.is_ok());

        // Verify deleted
        let ctx = ProfileContext::new("to-delete").unwrap();
        assert!(!ctx.config_dir.exists());
    }

    #[test]
    fn test_profile_delete_nonexistent() {
        let _temp = setup_test_env().unwrap();

        let result = delete_profile("nonexistent".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_profile_login_valid() {
        let _temp = setup_test_env().unwrap();

        // Create profile
        ProfileManager::create_profile("test-login", "correct-pass").unwrap();

        // Login with correct password
        let result = login_profile("test-login".to_string(), "correct-pass".to_string());
        assert!(result.is_ok());

        let profile = result.unwrap();
        assert_eq!(profile.name, "test-login");
    }

    #[test]
    fn test_profile_login_invalid() {
        let _temp = setup_test_env().unwrap();

        // Create profile
        ProfileManager::create_profile("test-login", "correct-pass").unwrap();

        // Wrong password
        let result = login_profile("test-login".to_string(), "wrong-pass".to_string());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Incorrect password"));

        // Nonexistent profile
        let result = login_profile("missing".to_string(), "pass".to_string());
        assert!(result.is_err());
    }
}
