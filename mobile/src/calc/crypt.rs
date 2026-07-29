//! Cryptography operations using X25519 + ChaCha20-Poly1305
//!
//! Provides encryption and decryption functionality for secure message exchange
//! following the industry standard ECDH + AEAD pattern.
//!
//! ## Encryption Process
//! 1. Generate ephemeral X25519 key pair
//! 2. Perform Diffie-Hellman key exchange with recipient's public key
//! 3. Use shared secret to encrypt message with ChaCha20-Poly1305
//! 4. Return: ephemeral_public_key + nonce + ciphertext
//!
//! ## Decryption Process
//! 1. Extract ephemeral_public_key, nonce, and ciphertext
//! 2. Perform Diffie-Hellman with own private key and sender's ephemeral public
//! 3. Use shared secret to decrypt with ChaCha20-Poly1305
//!
//! Reference: docs/MSG_EXCHANGE.md section "X25519 + ChaCha20-Poly1305"

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use anyhow::{Context, Result};
use chacha20poly1305::{
    ChaCha20Poly1305, Nonce,
    aead::{Aead, KeyInit, OsRng},
};
use rand::RngCore;
use x25519_dalek::{EphemeralSecret, PublicKey, StaticSecret};

/// Size of X25519 public key in bytes
pub const PUBLIC_KEY_SIZE: usize = 32;

/// Size of X25519 private key in bytes
pub const PRIVATE_KEY_SIZE: usize = 32;

/// Size of ChaCha20-Poly1305 nonce in bytes
pub const NONCE_SIZE: usize = 12;

/// Generate a new X25519 key pair
///
/// Returns (private_key, public_key) as byte vectors
///
/// # Examples
///
/// ```rust,no_run
/// use dure::calc::crypt::generate_keypair;
///
/// let (private_key, public_key) = generate_keypair();
/// println!("Generated keypair with public key: {:?}", public_key);
/// ```
pub fn generate_keypair() -> (Vec<u8>, Vec<u8>) {
    let secret = StaticSecret::random_from_rng(OsRng);
    let public = PublicKey::from(&secret);

    (secret.to_bytes().to_vec(), public.to_bytes().to_vec())
}

/// Encrypt data for a recipient
///
/// Uses X25519 Diffie-Hellman to derive a shared secret, then encrypts
/// the message with ChaCha20-Poly1305.
///
/// # Arguments
///
/// * `recipient_pubkey` - Recipient's X25519 public key (32 bytes)
/// * `plaintext` - Data to encrypt
///
/// # Returns
///
/// Encrypted bundle: ephemeral_public_key (32) + nonce (12) + ciphertext (variable)
///
/// # Examples
///
/// ```rust,no_run
/// use dure::calc::crypt::{generate_keypair, encrypt};
///
/// let (_, recipient_pubkey) = generate_keypair();
/// let message = b"Hello, World!";
/// let encrypted = encrypt(&recipient_pubkey, message).unwrap();
/// ```
pub fn encrypt(recipient_pubkey: &[u8], plaintext: &[u8]) -> Result<Vec<u8>> {
    // Validate recipient public key size
    if recipient_pubkey.len() != PUBLIC_KEY_SIZE {
        anyhow::bail!(
            "Invalid recipient public key size: expected {}, got {}",
            PUBLIC_KEY_SIZE,
            recipient_pubkey.len()
        );
    }

    // Parse recipient's public key
    let recipient_pubkey_array: [u8; PUBLIC_KEY_SIZE] = recipient_pubkey
        .try_into()
        .context("Failed to convert recipient public key")?;
    let recipient_public = PublicKey::from(recipient_pubkey_array);

    // Generate ephemeral key pair for this encryption
    let ephemeral_secret = EphemeralSecret::random_from_rng(OsRng);
    let ephemeral_public = PublicKey::from(&ephemeral_secret);

    // Perform Diffie-Hellman to get shared secret
    let shared_secret = ephemeral_secret.diffie_hellman(&recipient_public);

    // Create cipher with shared secret
    let cipher = ChaCha20Poly1305::new(shared_secret.as_bytes().into());

    // Generate random nonce
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt the plaintext
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

    // Bundle: ephemeral_public (32) + nonce (12) + ciphertext
    let mut result = Vec::with_capacity(PUBLIC_KEY_SIZE + NONCE_SIZE + ciphertext.len());
    result.extend_from_slice(ephemeral_public.as_bytes());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

/// Decrypt data from a sender
///
/// Uses the recipient's private key and the sender's ephemeral public key
/// to derive the same shared secret, then decrypts with ChaCha20-Poly1305.
///
/// # Arguments
///
/// * `my_privkey` - Recipient's X25519 private key (32 bytes)
/// * `encrypted` - Encrypted bundle from `encrypt()`
///
/// # Returns
///
/// Decrypted plaintext
///
/// # Examples
///
/// ```rust,no_run
/// use dure::calc::crypt::{generate_keypair, encrypt, decrypt};
///
/// let (recipient_privkey, recipient_pubkey) = generate_keypair();
/// let message = b"Secret message";
/// let encrypted = encrypt(&recipient_pubkey, message).unwrap();
/// let decrypted = decrypt(&recipient_privkey, &encrypted).unwrap();
/// assert_eq!(decrypted, message);
/// ```
pub fn decrypt(my_privkey: &[u8], encrypted: &[u8]) -> Result<Vec<u8>> {
    // Validate private key size
    if my_privkey.len() != PRIVATE_KEY_SIZE {
        anyhow::bail!(
            "Invalid private key size: expected {}, got {}",
            PRIVATE_KEY_SIZE,
            my_privkey.len()
        );
    }

    // Validate minimum encrypted data size
    let min_size = PUBLIC_KEY_SIZE + NONCE_SIZE;
    if encrypted.len() < min_size {
        anyhow::bail!(
            "Invalid encrypted data size: expected at least {}, got {}",
            min_size,
            encrypted.len()
        );
    }

    // Parse my private key
    let my_privkey_array: [u8; PRIVATE_KEY_SIZE] = my_privkey
        .try_into()
        .context("Failed to convert private key")?;
    let my_secret = StaticSecret::from(my_privkey_array);

    // Extract ephemeral public key (first 32 bytes)
    let ephemeral_public_bytes: [u8; PUBLIC_KEY_SIZE] = encrypted[..PUBLIC_KEY_SIZE]
        .try_into()
        .context("Failed to extract ephemeral public key")?;
    let ephemeral_public = PublicKey::from(ephemeral_public_bytes);

    // Extract nonce (next 12 bytes)
    let nonce_bytes: [u8; NONCE_SIZE] = encrypted[PUBLIC_KEY_SIZE..PUBLIC_KEY_SIZE + NONCE_SIZE]
        .try_into()
        .context("Failed to extract nonce")?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Extract ciphertext (remaining bytes)
    let ciphertext = &encrypted[PUBLIC_KEY_SIZE + NONCE_SIZE..];

    // Perform Diffie-Hellman to get shared secret
    let shared_secret = my_secret.diffie_hellman(&ephemeral_public);

    // Create cipher with shared secret
    let cipher = ChaCha20Poly1305::new(shared_secret.as_bytes().into());

    // Decrypt the ciphertext
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

    Ok(plaintext)
}

/// Encode bytes to base64 string (for display/transmission)
pub fn encode_base64(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}

/// Decode base64 string to bytes
pub fn decode_base64(encoded: &str) -> Result<Vec<u8>> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .context("Failed to decode base64")
}

/// Encode bytes to hex string (for display/transmission)
pub fn encode_hex(data: &[u8]) -> String {
    hex::encode(data)
}

/// Decode hex string to bytes
pub fn decode_hex(encoded: &str) -> Result<Vec<u8>> {
    hex::decode(encoded).context("Failed to decode hex")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keypair() {
        let (privkey, pubkey) = generate_keypair();
        assert_eq!(privkey.len(), PRIVATE_KEY_SIZE);
        assert_eq!(pubkey.len(), PUBLIC_KEY_SIZE);
    }

    #[test]
    fn test_encrypt_decrypt() {
        let (privkey, pubkey) = generate_keypair();
        let message = b"Hello, World!";

        let encrypted = encrypt(&pubkey, message).unwrap();
        let decrypted = decrypt(&privkey, &encrypted).unwrap();

        assert_eq!(decrypted, message);
    }

    #[test]
    fn test_encrypt_decrypt_empty() {
        let (privkey, pubkey) = generate_keypair();
        let message = b"";

        let encrypted = encrypt(&pubkey, message).unwrap();
        let decrypted = decrypt(&privkey, &encrypted).unwrap();

        assert_eq!(decrypted, message);
    }

    #[test]
    fn test_encrypt_decrypt_large() {
        let (privkey, pubkey) = generate_keypair();
        let message = vec![42u8; 10000];

        let encrypted = encrypt(&pubkey, &message).unwrap();
        let decrypted = decrypt(&privkey, &encrypted).unwrap();

        assert_eq!(decrypted, message);
    }

    #[test]
    fn test_invalid_recipient_pubkey() {
        let message = b"test";
        let invalid_pubkey = vec![0u8; 16]; // Wrong size

        let result = encrypt(&invalid_pubkey, message);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_privkey() {
        let (_, pubkey) = generate_keypair();
        let message = b"test";
        let encrypted = encrypt(&pubkey, message).unwrap();

        let invalid_privkey = vec![0u8; 16]; // Wrong size
        let result = decrypt(&invalid_privkey, &encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_encrypted_data() {
        let (privkey, _) = generate_keypair();
        let invalid_encrypted = vec![0u8; 10]; // Too small

        let result = decrypt(&privkey, &invalid_encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_corrupted_ciphertext() {
        let (privkey, pubkey) = generate_keypair();
        let message = b"test";

        let mut encrypted = encrypt(&pubkey, message).unwrap();
        // Corrupt the ciphertext
        if let Some(byte) = encrypted.last_mut() {
            *byte ^= 0xFF;
        }

        let result = decrypt(&privkey, &encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_base64_encoding() {
        let data = b"Hello, World!";
        let encoded = encode_base64(data);
        let decoded = decode_base64(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_hex_encoding() {
        let data = b"Hello, World!";
        let encoded = encode_hex(data);
        let decoded = decode_hex(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_different_recipients() {
        let (_, pubkey1) = generate_keypair();
        let (privkey2, pubkey2) = generate_keypair();
        let message = b"test";

        // Encrypt for pubkey1
        let encrypted = encrypt(&pubkey1, message).unwrap();

        // Try to decrypt with privkey2 (should fail or give garbage)
        let result = decrypt(&privkey2, &encrypted);
        // Decryption will technically succeed but the auth tag will fail
        assert!(result.is_err());
    }
}
