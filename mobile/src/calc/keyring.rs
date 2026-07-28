//! Keyring management using KeePass format without database
//!
//! This module provides key management functionality using KeePass database files
//! directly, without relying on SQLite. All operations work on a KeePass file
//! (default: key.kdbx) protected by a keyfile (default: id_ed25519).
//!
//! ## Architecture
//!
//! - **Storage**: KeePass 4 format (.kdbx) file
//! - **Protection**: Ed25519 keyfile (id_ed25519) + optional password
//! - **Auto-init**: Creates keyfile if missing using go-webauthn
//! - **No Database**: All operations are file-based
//!
//! ## Key Entry Structure
//!
//! Each key is stored as a KeePass entry with:
//! - **Title**: domain/URL (e.g., "www.dure.app")
//! - **UserName**: username/email (e.g., "nikescar@gmail.com")
//! - **Password**: the actual password/credential (protected)
//! - **created_at**: Unix timestamp (custom field)
//!
//! ## Default Paths
//!
//! - KeePass DB: `~/.config/dure/key.kdbx`
//! - KPKey: `~/.config/dure/id_ed25519`
//! - KPPubKey: `~/.config/dure/id_ed25519.pub`

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use anyhow::{Context, Result};
#[cfg(not(target_arch = "wasm32"))]
use directories::ProjectDirs;
use keepass::{
    Database, DatabaseKey,
    db::{Entry, Group, Value},
};
use std::fs::{File, create_dir_all};
use std::io::{Cursor, Write as IoWrite};
use std::path::{Path, PathBuf};

const KEEPASS_GROUP_NAME: &str = "Dure Keys";
const DEFAULT_KDBX_NAME: &str = "key.kdbx";
const DEFAULT_KPKEY_NAME: &str = "id_ed25519";
const DEFAULT_KPPUBKEY_NAME: &str = "id_ed25519.pub";

/// A key entry in the keyring
#[derive(Debug, Clone)]
pub struct KeyEntry {
    pub domain: String,
    pub username: String,
    pub password: String,
    pub created_at: u64,
    pub last_modification: Option<i64>,
    pub last_access: Option<i64>,
    pub notes: Option<String>,
    pub ssh_key: Option<Vec<u8>>, // Binary SSH private key
}

/// Get the default config directory for dure
#[cfg(not(target_arch = "wasm32"))]
fn get_config_dir() -> Result<PathBuf> {
    ProjectDirs::from("com", "dure", "dure")
        .map(|proj_dirs| proj_dirs.config_dir().to_path_buf())
        .context("Failed to determine config directory")
}

/// WASM builds do not have native project directories, so keep path generation local.
#[cfg(target_arch = "wasm32")]
fn get_config_dir() -> Result<PathBuf> {
    Ok(PathBuf::from(".dure"))
}

/// Get the default KeePass database path
pub fn get_default_kdbx_path() -> Result<PathBuf> {
    Ok(get_config_dir()?.join(DEFAULT_KDBX_NAME))
}

/// Get the default KPKey path
pub fn get_default_kpkey_path() -> Result<PathBuf> {
    Ok(get_config_dir()?.join(DEFAULT_KPKEY_NAME))
}

/// Get the default KPPubKey path
pub fn get_default_kppubkey_path() -> Result<PathBuf> {
    Ok(get_config_dir()?.join(DEFAULT_KPPUBKEY_NAME))
}

/// Generate Ed25519 key pair if it doesn't exist (using go-webauthn)
///
/// Uses ed25519-dalek directly for key generation.
/// Available on desktop (except OpenBSD) and Android.
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "openbsd")))]
pub fn ensure_kpkey_exists() -> Result<PathBuf> {
    use ed25519_dalek::SigningKey;
    use rand::RngCore;

    let kpkey_path = get_default_kpkey_path()?;
    let kppubkey_path = get_default_kppubkey_path()?;

    // If KPKey already exists, return it
    if kpkey_path.exists() {
        return Ok(kpkey_path);
    }

    eprintln!("Generating Ed25519 keypair for KeePass...");

    // Create config directory if it doesn't exist
    let config_dir = get_config_dir()?;
    create_dir_all(&config_dir)?;

    // Generate random 32 bytes for the key
    let mut rng = rand::thread_rng();
    let mut secret_bytes = [0u8; 32];
    rng.fill_bytes(&mut secret_bytes);

    // Generate key pair
    let signing_key = SigningKey::from_bytes(&secret_bytes);
    let verifying_key = signing_key.verifying_key();

    // Save private key
    let mut priv_file = File::create(&kpkey_path).context("Failed to create KPKey file")?;
    priv_file
        .write_all(&secret_bytes)
        .context("Failed to write private key")?;

    // Save public key
    let mut pub_file = File::create(&kppubkey_path).context("Failed to create KPPubKey file")?;
    pub_file
        .write_all(verifying_key.as_bytes())
        .context("Failed to write public key")?;

    eprintln!("✓ Generated new Ed25519 key pair:");
    eprintln!("  KPKey (private): {}", kpkey_path.display());
    eprintln!("  KPPubKey:        {}", kppubkey_path.display());

    Ok(kpkey_path)
}

/// OpenBSD - use ed25519-dalek directly for key generation
#[cfg(target_os = "openbsd")]
pub fn ensure_kpkey_exists() -> Result<PathBuf> {
    use ed25519_dalek::SigningKey;
    use rand::RngCore;

    let kpkey_path = get_default_kpkey_path()?;
    let kppubkey_path = get_default_kppubkey_path()?;

    // If KPKey already exists, return it
    if kpkey_path.exists() {
        return Ok(kpkey_path);
    }

    eprintln!("Generating Ed25519 keypair for KeePass...");

    // Create config directory if it doesn't exist
    let config_dir = get_config_dir()?;
    create_dir_all(&config_dir)?;

    // Generate random 32 bytes for the key
    let mut rng = rand::thread_rng();
    let mut secret_bytes = [0u8; 32];
    rng.fill_bytes(&mut secret_bytes);

    // Generate key pair
    let signing_key = SigningKey::from_bytes(&secret_bytes);
    let verifying_key = signing_key.verifying_key();

    // Save private key
    let mut priv_file = File::create(&kpkey_path).context("Failed to create KPKey file")?;
    priv_file
        .write_all(&secret_bytes)
        .context("Failed to write KPKey")?;

    // Save public key
    let mut pub_file = File::create(&kppubkey_path).context("Failed to create KPPubKey file")?;
    pub_file
        .write_all(&verifying_key.to_bytes())
        .context("Failed to write KPPubKey")?;

    eprintln!("✓ Generated KPKey at: {}", kpkey_path.display());

    Ok(kpkey_path)
}

/// WASM stub - key generation not supported
#[cfg(target_arch = "wasm32")]
pub fn ensure_kpkey_exists() -> Result<PathBuf> {
    anyhow::bail!(
        "KPKey generation not supported on WASM. Please import a KPKey from desktop/Android."
    );
}

/// Initialize the KeePass database if it doesn't exist
///
/// Creates a new KeePass database with the default KPKey.
/// If KPKey doesn't exist, generates it first.
pub fn ensure_kdbx_exists() -> Result<PathBuf> {
    let kdbx_path = get_default_kdbx_path()?;

    // If database already exists, return it
    if kdbx_path.exists() {
        return Ok(kdbx_path);
    }

    // Ensure KPKey exists (generate if needed)
    let kpkey_path = ensure_kpkey_exists()?;

    // Create config directory if it doesn't exist
    let config_dir = get_config_dir()?;
    create_dir_all(&config_dir)?;

    // Create new KeePass database
    let mut db = Database::new(Default::default());
    db.root.name = KEEPASS_GROUP_NAME.to_string();

    // Save to file with KPKey
    let mut file = File::create(&kdbx_path).context("Failed to create KeePass database file")?;

    let kpkey_data = std::fs::read(&kpkey_path).context("Failed to read KPKey")?;

    let mut kpkey_cursor = Cursor::new(kpkey_data);
    let key = DatabaseKey::new().with_keyfile(&mut kpkey_cursor)?;

    db.save(&mut file, key)
        .context("Failed to save KeePass database")?;

    eprintln!("✓ Created new KeePass database: {}", kdbx_path.display());

    Ok(kdbx_path)
}

/// Open a KeePass database with KPKey
///
/// # Arguments
///
/// * `kdbx_path` - Path to the .kdbx file
/// * `kpkey_path` - Optional path to KPKey (defaults to id_ed25519)
/// * `password` - Optional password (can be empty for KPKey-only auth)
pub fn open_kdbx(
    kdbx_path: &Path,
    kpkey_path: Option<&Path>,
    password: Option<&str>,
) -> Result<Database> {
    let mut file = File::open(kdbx_path)
        .with_context(|| format!("Failed to open KeePass file: {}", kdbx_path.display()))?;

    let mut key = DatabaseKey::new();

    // Add password if provided
    if let Some(pwd) = password {
        if !pwd.is_empty() {
            key = key.with_password(pwd);
        }
    }

    // Add KPKey if provided
    if let Some(kf_path) = kpkey_path {
        let kpkey_data = std::fs::read(kf_path)
            .with_context(|| format!("Failed to read KPKey: {}", kf_path.display()))?;
        let mut kpkey_cursor = Cursor::new(kpkey_data);
        key = key.with_keyfile(&mut kpkey_cursor)?;
    }

    Database::open(&mut file, key).context("Failed to open KeePass database. Check password/KPKey.")
}

/// Save a KeePass database to file
///
/// # Arguments
///
/// * `db` - The database to save
/// * `kdbx_path` - Path to save the .kdbx file
/// * `kpkey_path` - Optional path to KPKey (defaults to id_ed25519)
/// * `password` - Optional password (can be empty for KPKey-only auth)
pub fn save_kdbx(
    db: &mut Database,
    kdbx_path: &Path,
    kpkey_path: Option<&Path>,
    password: Option<&str>,
) -> Result<()> {
    let mut file = File::create(kdbx_path)
        .with_context(|| format!("Failed to create KeePass file: {}", kdbx_path.display()))?;

    let mut key = DatabaseKey::new();

    // Add password if provided
    if let Some(pwd) = password {
        if !pwd.is_empty() {
            key = key.with_password(pwd);
        }
    }

    // Add KPKey if provided
    if let Some(kf_path) = kpkey_path {
        let kpkey_data = std::fs::read(kf_path)
            .with_context(|| format!("Failed to read KPKey: {}", kf_path.display()))?;
        let mut kpkey_cursor = Cursor::new(kpkey_data);
        key = key.with_keyfile(&mut kpkey_cursor)?;
    }

    db.save(&mut file, key)
        .context("Failed to save KeePass database")
}

/// List all keys from KeePass database
pub fn list_keys(kdbx_path: &Path, kpkey_path: Option<&Path>) -> Result<Vec<KeyEntry>> {
    let db = open_kdbx(kdbx_path, kpkey_path, None)?;
    let mut keys = Vec::new();
    collect_keys_from_group(&db.root, &mut keys)?;
    Ok(keys)
}

/// Recursively collect keys from a group and its children
fn collect_keys_from_group(group: &Group, keys: &mut Vec<KeyEntry>) -> Result<()> {
    for entry in &group.entries {
        if let Some(key) = parse_key_entry(entry)? {
            keys.push(key);
        }
    }

    for child_group in &group.groups {
        collect_keys_from_group(child_group, keys)?;
    }

    Ok(())
}

/// Parse a KeePass entry into a KeyEntry
fn parse_key_entry(entry: &Entry) -> Result<Option<KeyEntry>> {
    // Get domain from Title field
    let domain = match entry.fields.get("Title") {
        Some(value) => value.get().to_string(),
        None => return Ok(None),
    };

    if domain.is_empty() {
        return Ok(None);
    }

    // Get username from UserName field
    let username = match entry.fields.get("UserName") {
        Some(value) => value.get().to_string(),
        None => String::new(),
    };

    // Get password from Password field
    let password = match entry.fields.get("Password") {
        Some(value) => value.get().to_string(),
        None => String::new(),
    };

    // Get created_at from custom field or times.creation
    let created_at = match entry.fields.get("created_at") {
        Some(value) => value.get().parse::<u64>().unwrap_or_else(|_| {
            entry
                .times
                .creation
                .map(|dt| dt.and_utc().timestamp() as u64)
                .unwrap_or_else(|| {
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                })
        }),
        None => entry
            .times
            .creation
            .map(|dt| dt.and_utc().timestamp() as u64)
            .unwrap_or_else(|| {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            }),
    };

    // Get last modification time
    let last_modification = entry
        .times
        .last_modification
        .map(|dt| dt.and_utc().timestamp());

    // Get last access time
    let last_access = entry.times.last_access.map(|dt| dt.and_utc().timestamp());

    // Get notes from Notes field
    let notes = entry
        .fields
        .get("Notes")
        .map(|value| value.get().to_string())
        .filter(|s| !s.is_empty());

    // Get SSH key from attachments (look for "ssh_key" or "private_key" attachment)
    let ssh_key = entry
        .attachments
        .get("ssh_key")
        .or_else(|| entry.attachments.get("private_key"))
        .or_else(|| entry.attachments.get("id_rsa"))
        .or_else(|| entry.attachments.get("id_ed25519"))
        .map(|attachment| attachment.data.get().clone());

    Ok(Some(KeyEntry {
        domain,
        username,
        password,
        created_at,
        last_modification,
        last_access,
        notes,
        ssh_key,
    }))
}

/// Add a new key to the KeePass database
pub fn add_key(
    kdbx_path: &Path,
    kpkey_path: Option<&Path>,
    domain: &str,
    username: &str,
    password: &str,
) -> Result<()> {
    add_key_with_ssh(
        kdbx_path, kpkey_path, domain, username, password, None, None,
    )
}

/// Add a new key with optional SSH key and notes to the KeePass database
pub fn add_key_with_ssh(
    kdbx_path: &Path,
    kpkey_path: Option<&Path>,
    domain: &str,
    username: &str,
    password: &str,
    ssh_key: Option<&[u8]>,
    notes: Option<&str>,
) -> Result<()> {
    let mut db = open_kdbx(kdbx_path, kpkey_path, None)?;

    // Check if key with same domain already exists
    let existing_keys = {
        let mut keys = Vec::new();
        collect_keys_from_group(&db.root, &mut keys)?;
        keys
    };

    if existing_keys.iter().any(|k| k.domain == domain) {
        anyhow::bail!(
            "Key with domain '{}' already exists. Delete it first.",
            domain
        );
    }

    // Create new entry
    let mut entry = Entry::default();
    entry
        .fields
        .insert("Title".to_string(), Value::unprotected(domain.to_string()));
    entry.fields.insert(
        "UserName".to_string(),
        Value::unprotected(username.to_string()),
    );
    entry.fields.insert(
        "Password".to_string(),
        Value::protected(password.to_string()),
    );
    entry.fields.insert(
        "created_at".to_string(),
        Value::unprotected(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .to_string(),
        ),
    );

    // Add notes if provided
    if let Some(notes_text) = notes {
        if !notes_text.is_empty() {
            entry.fields.insert(
                "Notes".to_string(),
                Value::unprotected(notes_text.to_string()),
            );
        }
    }

    // Add SSH key as binary attachment if provided
    if let Some(key_data) = ssh_key {
        use keepass::db::{Attachment, Value};
        entry.attachments.insert(
            "ssh_key".to_string(),
            Attachment {
                data: Value::unprotected(key_data.to_vec()),
            },
        );
    }

    // Add entry to root group
    db.root.entries.push(entry);

    // Save database
    save_kdbx(&mut db, kdbx_path, kpkey_path, None)?;

    Ok(())
}

/// Delete a key from the KeePass database
pub fn delete_key(kdbx_path: &Path, kpkey_path: Option<&Path>, domain: &str) -> Result<bool> {
    let mut db = open_kdbx(kdbx_path, kpkey_path, None)?;

    // Find and remove the entry
    let initial_count = db.root.entries.len();
    db.root
        .entries
        .retain(|entry| match entry.fields.get("Title") {
            Some(value) => value.get() != domain,
            None => true,
        });

    let deleted = db.root.entries.len() < initial_count;

    if deleted {
        // Save database
        save_kdbx(&mut db, kdbx_path, kpkey_path, None)?;
    }

    Ok(deleted)
}

/// Update or add a key to the KeePass database
///
/// If a key with the same domain exists, it will be replaced.
/// Otherwise, a new key will be added.
pub fn update_key(
    kdbx_path: &Path,
    kpkey_path: Option<&Path>,
    domain: &str,
    username: &str,
    password: &str,
) -> Result<()> {
    update_key_with_ssh(
        kdbx_path, kpkey_path, domain, username, password, None, None,
    )
}

/// Update or add a key with SSH key and notes to the KeePass database
///
/// If a key with the same domain exists, it will be replaced.
/// Otherwise, a new key will be added.
pub fn update_key_with_ssh(
    kdbx_path: &Path,
    kpkey_path: Option<&Path>,
    domain: &str,
    username: &str,
    password: &str,
    ssh_key: Option<&[u8]>,
    notes: Option<&str>,
) -> Result<()> {
    let mut db = open_kdbx(kdbx_path, kpkey_path, None)?;

    // Remove existing entry with same domain (if exists)
    db.root
        .entries
        .retain(|entry| match entry.fields.get("Title") {
            Some(value) => value.get() != domain,
            None => true,
        });

    // Create new entry
    let mut entry = Entry::default();
    entry
        .fields
        .insert("Title".to_string(), Value::unprotected(domain.to_string()));
    entry.fields.insert(
        "UserName".to_string(),
        Value::unprotected(username.to_string()),
    );
    entry.fields.insert(
        "Password".to_string(),
        Value::protected(password.to_string()),
    );
    entry.fields.insert(
        "created_at".to_string(),
        Value::unprotected(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .to_string(),
        ),
    );

    // Add notes if provided
    if let Some(notes_text) = notes {
        if !notes_text.is_empty() {
            entry.fields.insert(
                "Notes".to_string(),
                Value::unprotected(notes_text.to_string()),
            );
        }
    }

    // Add SSH key as binary attachment if provided
    if let Some(key_data) = ssh_key {
        use keepass::db::{Attachment, Value};
        entry.attachments.insert(
            "ssh_key".to_string(),
            Attachment {
                data: Value::unprotected(key_data.to_vec()),
            },
        );
    }

    // Add entry to root group
    db.root.entries.push(entry);

    // Save database
    save_kdbx(&mut db, kdbx_path, kpkey_path, None)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_paths() {
        let config_dir = get_config_dir().unwrap();
        assert!(config_dir.to_string_lossy().contains("dure"));

        let kdbx_path = get_default_kdbx_path().unwrap();
        assert!(kdbx_path.to_string_lossy().ends_with("key.kdbx"));

        let kpkey_path = get_default_kpkey_path().unwrap();
        assert!(kpkey_path.to_string_lossy().ends_with("id_ed25519"));
    }

    #[test]
    #[cfg(all(not(target_arch = "wasm32"), not(target_os = "openbsd")))]
    fn test_add_list_delete_key() {
        use std::env;

        // Use temp directory for test
        let temp_dir = env::temp_dir();
        let test_kdbx = temp_dir.join("test_keyring.kdbx");
        let test_kpkey = temp_dir.join("test_keyring.key");

        // Generate test KPKey
        use go_webauthn::*;
        use pollster::block_on;

        let gen_req = Ed25519GenerateKeyRequest {};
        let gen_resp = block_on(crypto_ed25519_generate_key(&gen_req));
        assert!(gen_resp.success);

        std::fs::write(&test_kpkey, &gen_resp.private_key).unwrap();

        // Create empty database
        let mut db = Database::new(Default::default());
        db.root.name = KEEPASS_GROUP_NAME.to_string();
        save_kdbx(&mut db, &test_kdbx, Some(&test_kpkey), None).unwrap();

        // Add a key
        add_key(
            &test_kdbx,
            Some(&test_kpkey),
            "example.com",
            "user@example.com",
            "secretpass123",
        )
        .unwrap();

        // List keys
        let keys = list_keys(&test_kdbx, Some(&test_kpkey)).unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].domain, "example.com");
        assert_eq!(keys[0].username, "user@example.com");
        assert_eq!(keys[0].password, "secretpass123");

        // Try to add duplicate
        let result = add_key(
            &test_kdbx,
            Some(&test_kpkey),
            "example.com",
            "another@example.com",
            "pass",
        );
        assert!(result.is_err());

        // Delete key
        let deleted = delete_key(&test_kdbx, Some(&test_kpkey), "example.com").unwrap();
        assert!(deleted);

        // Verify deletion
        let keys = list_keys(&test_kdbx, Some(&test_kpkey)).unwrap();
        assert_eq!(keys.len(), 0);

        // Cleanup
        std::fs::remove_file(test_kdbx).ok();
        std::fs::remove_file(test_kpkey).ok();
    }
}
