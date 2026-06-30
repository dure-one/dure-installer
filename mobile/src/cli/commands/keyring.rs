//! KEY command implementation for key management using KeePass format
//!
//! Provides CLI commands for managing credentials (domain/username/password)
//! in a KeePass database file without using SQLite.

use crate::calc::keyring::{
    add_key, delete_key, ensure_kdbx_exists, get_default_kdbx_path, get_default_kpkey_path,
    list_keys,
};
use anyhow::{Context, Result};
use std::path::Path;

/// Execute KEY SAVE command
///
/// Saves the keyring to a KeePass file (export).
///
/// # Arguments
///
/// * `output_path` - Optional output path (defaults to ./exported_keys.kdbx)
pub fn execute_key_save(output_path: Option<String>) -> Result<()> {
    eprintln!("Saving keyring to KeePass database...");

    // Get source database path
    let source_path = get_default_kdbx_path()?;
    if !source_path.exists() {
        anyhow::bail!("No keyring found. Run 'dure key add' to create keys first.");
    }

    // Determine output path
    let output_path = output_path.unwrap_or_else(|| "exported_keys.kdbx".to_string());
    let output_path = Path::new(&output_path);

    // Check if file already exists
    if output_path.exists() {
        eprintln!("⚠ Warning: File already exists: {}", output_path.display());
        eprintln!("  It will be overwritten.");
        eprintln!();
    }

    // Copy the database file
    std::fs::copy(&source_path, output_path)
        .with_context(|| format!("Failed to save keyring to {}", output_path.display()))?;

    eprintln!();
    eprintln!("✓ Keyring saved successfully");
    eprintln!("  Output: {}", output_path.display());
    eprintln!();
    eprintln!("⚠ Keep this file secure! It contains your credentials.");
    eprintln!(
        "  The file is protected by your KPKey: {}",
        get_default_kpkey_path()?.display()
    );

    Ok(())
}

/// Execute KEY LOAD command
///
/// Loads a keyring from a KeePass file (import/replace).
///
/// # Arguments
///
/// * `input_path` - Path to the KeePass database file (.kdbx)
pub fn execute_key_load(input_path: String) -> Result<()> {
    eprintln!("Loading keyring from KeePass database...");

    let input_path = Path::new(&input_path);

    // Check if file exists
    if !input_path.exists() {
        anyhow::bail!("KeePass file not found: {}", input_path.display());
    }

    // Get destination path
    let dest_path = get_default_kdbx_path()?;

    // Warn if replacing existing keyring
    if dest_path.exists() {
        eprintln!("⚠ Warning: This will replace your current keyring!");
        eprintln!("  Current keyring: {}", dest_path.display());
        eprintln!();
    }

    // Ensure config directory exists
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent).context("Failed to create config directory")?;
    }

    // Copy the file
    std::fs::copy(input_path, &dest_path)
        .with_context(|| format!("Failed to load keyring from {}", input_path.display()))?;

    eprintln!();
    eprintln!("✓ Keyring loaded successfully");
    eprintln!("  Loaded from: {}", input_path.display());
    eprintln!("  Installed to: {}", dest_path.display());
    eprintln!();
    eprintln!("Use 'dure key status' to view loaded keys.");

    Ok(())
}

/// Execute KEY STATUS command
///
/// Lists all keys in the current keyring.
pub fn execute_key_status() -> Result<()> {
    // Ensure database exists
    let kdbx_path = ensure_kdbx_exists()?;
    let kpkey_path = get_default_kpkey_path()?;

    // List all keys
    let keys = list_keys(&kdbx_path, Some(&kpkey_path))?;

    if keys.is_empty() {
        eprintln!("No keys found in keyring.");
        eprintln!();
        eprintln!("Use 'dure key add <domain> <username> <password>' to add keys.");
    } else {
        eprintln!("Keyring status:");
        eprintln!("  Database: {}", kdbx_path.display());
        eprintln!("  KPKey:    {}", kpkey_path.display());
        eprintln!();
        eprintln!("Keys ({} total):", keys.len());
        eprintln!();

        // Print header
        eprintln!("{:<30} {:<30} {:<15}", "Domain", "Username", "Created");
        eprintln!("{:-<75}", "");

        // Print keys
        for key in &keys {
            let created = format_timestamp(key.created_at);
            eprintln!(
                "{:<30} {:<30} {:<15}",
                truncate(&key.domain, 28),
                truncate(&key.username, 28),
                created
            );
        }
        eprintln!();
    }

    Ok(())
}

/// Execute KEY ADD command
///
/// Adds a new key to the keyring.
///
/// # Arguments
///
/// * `domain` - Domain/URL for the key (e.g., "www.dure.app")
/// * `username` - Username/email (e.g., "nikescar@gmail.com")
/// * `password` - The password/credential
pub fn execute_key_add(domain: String, username: String, password: String) -> Result<()> {
    eprintln!("Adding key to keyring...");

    // Ensure database exists
    let kdbx_path = ensure_kdbx_exists()?;
    let kpkey_path = get_default_kpkey_path()?;

    // Add the key
    add_key(&kdbx_path, Some(&kpkey_path), &domain, &username, &password)?;

    eprintln!();
    eprintln!("✓ Key added successfully");
    eprintln!("  Domain:   {}", domain);
    eprintln!("  Username: {}", username);
    eprintln!();
    eprintln!("Use 'dure key status' to view all keys.");

    Ok(())
}

/// Execute KEY DEL command
///
/// Deletes a key from the keyring.
///
/// # Arguments
///
/// * `domain` - Domain/URL of the key to delete
pub fn execute_key_del(domain: String) -> Result<()> {
    eprintln!("Deleting key from keyring...");

    // Ensure database exists
    let kdbx_path = ensure_kdbx_exists()?;
    let kpkey_path = get_default_kpkey_path()?;

    // Delete the key
    let deleted = delete_key(&kdbx_path, Some(&kpkey_path), &domain)?;

    if deleted {
        eprintln!();
        eprintln!("✓ Key deleted successfully");
        eprintln!("  Domain: {}", domain);
    } else {
        eprintln!();
        eprintln!("⚠ No key found with domain: {}", domain);
        eprintln!();
        eprintln!("Use 'dure key status' to view available keys.");
    }

    Ok(())
}

/// Format Unix timestamp as human-readable date
fn format_timestamp(timestamp: u64) -> String {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    let datetime = UNIX_EPOCH + Duration::from_secs(timestamp);
    let now = SystemTime::now();

    // Simple relative time formatting
    if let Ok(duration) = now.duration_since(datetime) {
        let days = duration.as_secs() / 86400;
        if days == 0 {
            return "Today".to_string();
        } else if days == 1 {
            return "Yesterday".to_string();
        } else if days < 7 {
            return format!("{} days ago", days);
        } else if days < 30 {
            return format!("{} weeks ago", days / 7);
        } else if days < 365 {
            return format!("{} months ago", days / 30);
        } else {
            return format!("{} years ago", days / 365);
        }
    }

    "Unknown".to_string()
}

/// Truncate string to max length with ellipsis
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len - 1])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("this is a very long string", 10), "this is a…");
    }

    #[test]
    fn test_format_timestamp() {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        assert_eq!(format_timestamp(now), "Today");
        assert_eq!(format_timestamp(now - 86400), "Yesterday");
        assert_eq!(format_timestamp(now - 86400 * 3), "3 days ago");
    }
}
