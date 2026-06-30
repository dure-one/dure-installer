//! Integration tests for WebAuthn client functionality
//!
//! These tests verify the complete WebAuthn flow including:
//! - Client initialization
//! - Ed25519 key generation
//! - SSH key format conversion
//! - WebAuthn ceremonies (registration, authentication)
//!
//! Run with: PATH="$PWD/crates/go-webauthn/bin:$PATH" cargo test --test test_webauthn_integration -- --ignored

#[cfg(not(target_arch = "wasm32"))]
mod webauthn_tests {
    use go_webauthn_client::{
        GoWebAuthnClient, WebAuthnListCredentialsParams, WebAuthnPasskeyLoginBeginParams,
        WebAuthnSigninBeginParams, WebAuthnSignupBeginParams,
    };

    #[test]
    #[ignore] // Requires go-webauthn-cli to be built
    fn test_client_initialization() {
        let result = GoWebAuthnClient::new(None);
        if let Err(e) = &result {
            panic!("Failed to create WebAuthn client: {}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    #[ignore]
    fn test_ed25519_key_generation() {
        let mut client = GoWebAuthnClient::new(None).expect("Failed to create client");

        let result = client.ed25519_generate_key();
        assert!(result.is_ok(), "Failed to generate key: {:?}", result);

        let keypair = result.unwrap();

        // Ed25519 keys should have specific lengths
        assert_eq!(
            keypair.public_key.len(),
            32,
            "Public key should be 32 bytes"
        );
        assert_eq!(
            keypair.private_key.len(),
            64,
            "Private key should be 64 bytes"
        );

        // Keys should not be all zeros
        assert!(
            keypair.public_key.iter().any(|&b| b != 0),
            "Public key should not be all zeros"
        );
        assert!(
            keypair.private_key.iter().any(|&b| b != 0),
            "Private key should not be all zeros"
        );
    }

    #[test]
    #[ignore]
    fn test_ed25519_sign_and_verify() {
        let mut client = GoWebAuthnClient::new(None).expect("Failed to create client");

        // Generate key pair
        let keypair = client
            .ed25519_generate_key()
            .expect("Failed to generate key");

        let message = b"Hello, World!";

        // Sign message
        let signature = client
            .ed25519_sign(&keypair.private_key, message)
            .expect("Failed to sign message");

        // Signature should be 64 bytes for Ed25519
        assert_eq!(signature.len(), 64, "Signature should be 64 bytes");

        // Verify signature
        let valid = client
            .ed25519_verify(&keypair.public_key, message, &signature)
            .expect("Failed to verify signature");

        assert!(valid, "Signature should be valid");

        // Verify with wrong message should fail
        let wrong_message = b"Wrong message";
        let valid = client
            .ed25519_verify(&keypair.public_key, wrong_message, &signature)
            .expect("Failed to verify signature");

        assert!(!valid, "Signature should be invalid for wrong message");
    }

    #[test]
    #[ignore]
    fn test_chacha20_encrypt_decrypt() {
        let mut client = GoWebAuthnClient::new(None).expect("Failed to create client");

        let key = vec![1u8; 32]; // 32-byte key
        let nonce = vec![2u8; 24]; // 24-byte nonce
        let plaintext = b"Secret message";

        // Encrypt
        let ciphertext = client
            .chacha20_encrypt(&key, &nonce, plaintext, None)
            .expect("Failed to encrypt");

        // Ciphertext should be different from plaintext
        assert_ne!(ciphertext.as_slice(), plaintext);

        // Ciphertext should be longer (includes auth tag)
        assert!(ciphertext.len() > plaintext.len());

        // Decrypt
        let decrypted = client
            .chacha20_decrypt(&key, &nonce, &ciphertext, None)
            .expect("Failed to decrypt");

        // Should match original plaintext
        assert_eq!(decrypted.as_slice(), plaintext);
    }

    #[test]
    #[ignore]
    fn test_chacha20_with_additional_data() {
        let mut client = GoWebAuthnClient::new(None).expect("Failed to create client");

        let key = vec![1u8; 32];
        let nonce = vec![2u8; 24];
        let plaintext = b"Secret message";
        let additional_data = b"metadata";

        // Encrypt with additional data
        let ciphertext = client
            .chacha20_encrypt(&key, &nonce, plaintext, Some(additional_data))
            .expect("Failed to encrypt");

        // Decrypt with same additional data
        let decrypted = client
            .chacha20_decrypt(&key, &nonce, &ciphertext, Some(additional_data))
            .expect("Failed to decrypt");

        assert_eq!(decrypted.as_slice(), plaintext);

        // Decrypt with different additional data should fail
        let wrong_additional_data = b"wrong";
        let result =
            client.chacha20_decrypt(&key, &nonce, &ciphertext, Some(wrong_additional_data));

        assert!(
            result.is_err(),
            "Decryption with wrong additional data should fail"
        );
    }

    #[test]
    #[ignore]
    fn test_webauthn_signup_begin() {
        let mut client = GoWebAuthnClient::new(None).expect("Failed to create client");

        let params = WebAuthnSignupBeginParams {
            rp_display_name: "Test App".to_string(),
            rp_id: "example.com".to_string(),
            rp_origins: "https://example.com".to_string(),
            username: "alice@example.com".to_string(),
            display_name: "Alice Smith".to_string(),
            scenario: "passwordless".to_string(),
        };

        let result = client.webauthn_signup_begin(params);
        assert!(result.is_ok(), "Failed to begin signup: {:?}", result);

        let signup_result = result.unwrap();

        // Should have session ID
        assert!(!signup_result.session_id.is_empty());

        // Should have challenge JSON
        assert!(!signup_result.challenge_json.is_empty());

        // Challenge should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&signup_result.challenge_json)
            .expect("Challenge should be valid JSON");

        // Should contain publicKey (WebAuthn structure)
        assert!(
            parsed.get("publicKey").is_some(),
            "Challenge should have publicKey"
        );

        // Should have challenge field
        let public_key = parsed.get("publicKey").unwrap();
        assert!(
            public_key.get("challenge").is_some(),
            "Should have challenge"
        );
    }

    #[test]
    #[ignore]
    fn test_webauthn_signin_begin_no_user() {
        let mut client = GoWebAuthnClient::new(None).expect("Failed to create client");

        let params = WebAuthnSigninBeginParams {
            username: "nonexistent@example.com".to_string(),
            scenario: "passwordless".to_string(),
        };

        let result = client.webauthn_signin_begin(params);

        // Should fail because WebAuthn is not initialized (fresh process)
        assert!(result.is_err(), "Signin should fail for fresh process");
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("WebAuthn not initialized") || error_msg.contains("user not found"),
            "Error should indicate WebAuthn not initialized or user not found, got: {}",
            error_msg
        );
    }

    #[test]
    #[ignore]
    fn test_webauthn_passkey_login_begin() {
        let mut client = GoWebAuthnClient::new(None).expect("Failed to create client");

        let params = WebAuthnPasskeyLoginBeginParams {
            rp_display_name: "Test App".to_string(),
            rp_id: "example.com".to_string(),
            rp_origins: "https://example.com".to_string(),
        };

        let result = client.webauthn_passkey_login_begin(params);
        assert!(
            result.is_ok(),
            "Failed to begin passkey login: {:?}",
            result
        );

        let login_result = result.unwrap();

        // Should have session ID
        assert!(!login_result.session_id.is_empty());

        // Should have challenge JSON
        assert!(!login_result.challenge_json.is_empty());

        // Challenge should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&login_result.challenge_json)
            .expect("Challenge should be valid JSON");

        // Should contain publicKey for WebAuthn
        assert!(
            parsed.get("publicKey").is_some(),
            "Challenge should have publicKey"
        );

        let public_key = parsed.get("publicKey").unwrap();
        assert!(
            public_key.get("challenge").is_some(),
            "Should have challenge"
        );
    }

    #[test]
    #[ignore]
    fn test_webauthn_credentials_list_empty() {
        let mut client = GoWebAuthnClient::new(None).expect("Failed to create client");

        // First initialize WebAuthn by calling signup.begin
        let signup_params = WebAuthnSignupBeginParams {
            rp_display_name: "Test App".to_string(),
            rp_id: "example.com".to_string(),
            rp_origins: "https://example.com".to_string(),
            username: "newuser@example.com".to_string(),
            display_name: "New User".to_string(),
            scenario: "passwordless".to_string(),
        };

        // This initializes WebAuthn state
        let _ = client
            .webauthn_signup_begin(signup_params)
            .expect("Signup begin should succeed");

        // Now list credentials
        let params = WebAuthnListCredentialsParams {
            username: "newuser@example.com".to_string(),
        };

        let result = client.webauthn_list_credentials(params);

        // For a new user (registration not completed), should return empty list
        assert!(
            result.is_ok(),
            "Listing credentials should succeed: {:?}",
            result
        );

        let creds = result.unwrap();
        assert_eq!(
            creds.credentials.len(),
            0,
            "New user should have no credentials"
        );
    }

    #[test]
    #[ignore]
    fn test_multiple_clients() {
        // Test that multiple clients can be created and used independently
        let mut client1 = GoWebAuthnClient::new(None).expect("Failed to create client 1");

        let mut client2 = GoWebAuthnClient::new(None).expect("Failed to create client 2");

        // Generate keys with both clients
        let key1 = client1
            .ed25519_generate_key()
            .expect("Failed to generate key with client 1");

        let key2 = client2
            .ed25519_generate_key()
            .expect("Failed to generate key with client 2");

        // Keys should be different (extremely unlikely to be the same)
        assert_ne!(key1.public_key, key2.public_key);
        assert_ne!(key1.private_key, key2.private_key);
    }

    #[test]
    #[ignore]
    fn test_client_process_lifecycle() {
        // Create client
        let client = GoWebAuthnClient::new(None).expect("Failed to create client");

        // Client should spawn a process
        // When client is dropped, process should be killed
        drop(client);

        // Create a new client - should work fine
        let mut client2 = GoWebAuthnClient::new(None).expect("Failed to create second client");

        // Should still be able to use it
        let _key = client2
            .ed25519_generate_key()
            .expect("Failed to generate key with second client");
    }

    #[test]
    #[ignore]
    fn test_error_handling_invalid_key_length() {
        let mut client = GoWebAuthnClient::new(None).expect("Failed to create client");

        // Try to sign with invalid key length
        let invalid_key = vec![1u8; 10]; // Too short
        let message = b"test";

        let result = client.ed25519_sign(&invalid_key, message);

        // Should fail
        assert!(result.is_err(), "Should fail with invalid key length");
    }
}
