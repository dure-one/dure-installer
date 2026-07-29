//! CRYPT command implementation for encryption/decryption operations
//!
//! Provides CLI commands for encrypting and decrypting data using X25519 + ChaCha20-Poly1305.
//! Uses device keys stored in the SQLite database.

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use crate::calc::crypt::{
    PUBLIC_KEY_SIZE, decode_base64, decrypt, encode_base64, encrypt, generate_keypair,
};
use crate::calc::db;
use crate::storage::models::crypt::{
    DeviceKeys, get_current_device_keys, init_crypt_table, store_device_keys,
};
use anyhow::{Context, Result};
use std::time::{SystemTime, UNIX_EPOCH};

/// Execute CRYPT enc command
///
/// Encrypts data for a recipient using their public key.
///
/// # Arguments
///
/// * `recipient_pubkey` - Recipient's public key (base64 or hex encoded)
/// * `data` - Data to encrypt (plain text or base64)
/// * `output_hex` - If true, output as hex instead of base64
pub fn execute_crypt_enc(recipient_pubkey: String, data: String, output_hex: bool) -> Result<()> {
    dure_info!("Encrypting data...");

    // Try to decode recipient public key from base64 or hex
    let recipient_pubkey_bytes = decode_base64(&recipient_pubkey)
        .or_else(|_| crate::calc::crypt::decode_hex(&recipient_pubkey))
        .context("Failed to decode recipient public key. Expected base64 or hex encoding.")?;

    if recipient_pubkey_bytes.len() != PUBLIC_KEY_SIZE {
        anyhow::bail!(
            "Invalid recipient public key size: expected {} bytes, got {}",
            PUBLIC_KEY_SIZE,
            recipient_pubkey_bytes.len()
        );
    }

    // Convert data to bytes (assume UTF-8 text, but also try base64)
    let data_bytes = data.as_bytes().to_vec();

    // Encrypt
    let encrypted = encrypt(&recipient_pubkey_bytes, &data_bytes)?;

    // Output encrypted data
    if output_hex {
        println!("{}", crate::calc::crypt::encode_hex(&encrypted));
    } else {
        println!("{}", encode_base64(&encrypted));
    }

    dure_info!("");
    dure_info!(" Data encrypted successfully");
    dure_info!("  Size: {} bytes plaintext → {} bytes encrypted", data_bytes.len(),
        encrypted.len()
    );

    Ok(())
}

/// Execute CRYPT dec command
///
/// Decrypts data using the device's private key.
///
/// # Arguments
///
/// * `encrypted_data` - Encrypted data (base64 or hex encoded)
/// * `output_raw` - If true, output raw bytes; otherwise interpret as UTF-8 text
pub fn execute_crypt_dec(encrypted_data: String, output_raw: bool) -> Result<()> {
    dure_info!("Decrypting data...");

    // Get database connection
    let _db_path = get_db_path()?;
    let mut conn = db::establish_connection();

    init_crypt_table(&mut conn)?;

    // Get current device keys
    let keys = get_current_device_keys(&mut conn)?.ok_or_else(|| {
        anyhow::anyhow!("No device keys found. Please run 'dure key init' first.")
    })?;

    // Decode encrypted data from base64 or hex
    let encrypted_bytes = decode_base64(&encrypted_data)
        .or_else(|_| crate::calc::crypt::decode_hex(&encrypted_data))
        .context("Failed to decode encrypted data. Expected base64 or hex encoding.")?;

    // Decrypt
    let decrypted = decrypt(&keys.private_key, &encrypted_bytes)?;

    // Output decrypted data
    if output_raw {
        use std::io::Write;
        std::io::stdout()
            .write_all(&decrypted)
            .context("Failed to write decrypted data")?;
    } else {
        let text = String::from_utf8(decrypted.clone())
            .unwrap_or_else(|_| format!("<binary data: {} bytes>", decrypted.len()));
        println!("{}", text);
    }

    dure_info!("");
    dure_info!(" Data decrypted successfully");
    dure_info!("  Device: {}", keys.device_id);

    Ok(())
}

/// Execute CRYPT init command
///
/// Initializes device encryption keys. Generates a new X25519 key pair
/// and stores it in the database.
///
/// # Arguments
///
/// * `device_id` - Optional device ID (defaults to hostname)
/// * `force` - If true, regenerate keys even if they exist
pub fn execute_crypt_init(device_id: Option<String>, force: bool) -> Result<()> {
    dure_info!("Initializing device encryption keys...");

    // Get database connection
    let db_path = get_db_path()?;
    let mut conn = db::establish_connection();

    init_crypt_table(&mut conn)?;

    // Check if keys already exist
    if !force {
        if let Some(existing) = get_current_device_keys(&mut conn)? {
            dure_info!("Device keys already exist for: {}", existing.device_id);
            dure_info!("");
            dure_info!("Public key (base64): {}", encode_base64(&existing.public_key)
            );
            dure_info!("");
            dure_info!("To regenerate keys, use --force flag");
            return Ok(());
        }
    }

    // Determine device ID
    let device_id = device_id.unwrap_or_else(|| {
        hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| format!("device-{}", chrono::Utc::now().timestamp()))
    });

    dure_info!("Device ID: {}", device_id);

    // Generate new keypair
    let (private_key, public_key) = generate_keypair();

    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let keys = DeviceKeys {
        device_id: device_id.clone(),
        private_key,
        public_key: public_key.clone(),
        created_at,
    };

    // Store in database
    store_device_keys(&mut conn, &keys)?;

    dure_info!("");
    dure_info!(" Device encryption keys initialized successfully");
    dure_info!("");
    dure_info!("Device ID: {}", device_id);
    dure_info!("Public Key (base64):");
    dure_info!("{}", encode_base64(&public_key));
    dure_info!("");
    dure_info!("Share your public key with others to receive encrypted messages.");
    dure_info!("");
    dure_warn!(" Keep your private key secure! It is stored in:");
    dure_info!("  {}", db_path.display());

    Ok(())
}

/// Execute CRYPT status command
///
/// Displays the base public key for the system.
pub fn execute_crypt_status() -> Result<()> {
    // Get database connection
    let _db_path = get_db_path()?;
    let mut conn = db::establish_connection();

    init_crypt_table(&mut conn)?;

    // Get current device keys
    let keys = get_current_device_keys(&mut conn)?.ok_or_else(|| {
        anyhow::anyhow!("No device keys found. Please run 'dure key init' first.")
    })?;

    println!("Device ID: {}", keys.device_id);
    println!();
    println!("Public Key (base64):");
    println!("{}", encode_base64(&keys.public_key));
    println!();
    println!("Public Key (hex):");
    println!("{}", crate::calc::crypt::encode_hex(&keys.public_key));
    println!();

    let created_date = chrono::DateTime::from_timestamp(keys.created_at as i64, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    println!("Created: {}", created_date);

    Ok(())
}

/// Execute CRYPT show command (deprecated - use status instead)
///
/// Displays the current device's public key.
#[deprecated(note = "Use execute_crypt_status instead")]
pub fn execute_crypt_show() -> Result<()> {
    execute_crypt_status()
}

fn get_db_path() -> Result<std::path::PathBuf> {
    let db_path = crate::calc::db::get_db_path();
    Ok(std::path::PathBuf::from(db_path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_db_path() {
        let path = get_db_path().unwrap();
        assert!(path.to_string_lossy().contains("dure"));
        assert!(path.to_string_lossy().ends_with(".db"));
    }

    #[test]
    fn test_init_and_show() {
        // This test would require a mock database or temp directory
        // Skipping for now as it requires filesystem access
    }
}
