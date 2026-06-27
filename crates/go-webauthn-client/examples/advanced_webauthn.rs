// Advanced WebAuthn features example
//
// Demonstrates:
// 1. Passkey Login (usernameless, discoverable credentials)
// 2. MFA Login (second factor authentication)
// 3. Credential Management (list, delete)

use go_webauthn_client::{
    GoWebAuthnClient,
    WebAuthnSignupBeginParams,
    WebAuthnPasskeyLoginBeginParams,
    WebAuthnMfaLoginBeginParams,
    WebAuthnListCredentialsParams,
};

fn main() -> anyhow::Result<()> {
    println!("=== Advanced WebAuthn Features Example ===\n");

    let mut client = GoWebAuthnClient::new(None)?;

    // ========================================================================
    // Example 1: Passkey Login (Discoverable Credentials)
    // ========================================================================
    println!("1. Passkey Login (Usernameless)");
    println!("   This allows login WITHOUT entering a username first\n");

    let passkey_params = WebAuthnPasskeyLoginBeginParams {
        rp_display_name: "Example Corp".to_string(),
        rp_id: "example.com".to_string(),
        rp_origins: "https://example.com".to_string(),
    };

    match client.webauthn_passkey_login_begin(passkey_params) {
        Ok(result) => {
            println!("   ✅ Session ID: {}", result.session_id);
            println!("   ✅ Challenge JSON length: {} bytes", result.challenge_json.len());
            println!("   → Browser would show available passkeys to select from");
            println!("   → No username entry required!\n");
        }
        Err(e) => {
            println!("   ℹ️  Expected: {}", e);
            println!("   → This is normal - no credentials exist yet\n");
        }
    }

    // ========================================================================
    // Example 2: MFA Login (Second Factor)
    // ========================================================================
    println!("2. MFA Login (Second Factor Authentication)");
    println!("   This adds passkey as second factor AFTER password login\n");

    let mfa_params = WebAuthnMfaLoginBeginParams {
        rp_display_name: "Example Corp".to_string(),
        rp_id: "example.com".to_string(),
        rp_origins: "https://example.com".to_string(),
        username: "alice@example.com".to_string(),
    };

    match client.webauthn_mfa_login_begin(mfa_params) {
        Ok(result) => {
            println!("   ✅ Session ID: {}", result.session_id);
            println!("   ✅ Challenge JSON length: {} bytes", result.challenge_json.len());
            println!("   → Use case: User already logged in with password");
            println!("   → Now verify with passkey as 2FA\n");
        }
        Err(e) => {
            println!("   ℹ️  Expected: {}", e);
            println!("   → This is normal - no credentials exist yet\n");
        }
    }

    // ========================================================================
    // Example 3: Credential Management
    // ========================================================================
    println!("3. Credential Management");
    println!("   List and manage registered passkeys\n");

    // First, try to register a user to demonstrate the flow
    println!("   3a. Setting up a user for demonstration...");
    let signup_params = WebAuthnSignupBeginParams {
        rp_display_name: "Example Corp".to_string(),
        rp_id: "example.com".to_string(),
        rp_origins: "https://example.com".to_string(),
        username: "bob@example.com".to_string(),
        display_name: "Bob Smith".to_string(),
        scenario: "passwordless".to_string(),
    };

    match client.webauthn_signup_begin(signup_params) {
        Ok(result) => {
            println!("      ✅ Registration started for bob@example.com");
            println!("      → Session ID: {}", result.session_id);
            println!("      ⚠️  Cannot complete without browser\n");
        }
        Err(e) => {
            println!("      ℹ️  {}\n", e);
        }
    }

    // Try to list credentials (will be empty until registration is completed)
    println!("   3b. Listing credentials...");
    let list_params = WebAuthnListCredentialsParams {
        username: "bob@example.com".to_string(),
    };

    match client.webauthn_list_credentials(list_params) {
        Ok(result) => {
            println!("      ✅ Found {} credentials", result.credentials.len());
            for (i, cred) in result.credentials.iter().enumerate() {
                println!("      Credential {}:", i + 1);
                println!("         ID: {}", cred.id);
                println!("         AAGUID: {}", cred.aaguid);
                println!("         Sign count: {}", cred.sign_count);
            }
        }
        Err(e) => {
            println!("      ℹ️  {}", e);
            println!("      → This is normal - user doesn't exist yet");
        }
    }

    // ========================================================================
    // Summary
    // ========================================================================
    println!("\n=== Summary ===");
    println!("✅ All advanced WebAuthn features are implemented!");
    println!();
    println!("Available features:");
    println!("  1. Passkey Login");
    println!("     - Usernameless authentication");
    println!("     - Discoverable credentials");
    println!("     - Browser shows list of available passkeys");
    println!();
    println!("  2. MFA Login");
    println!("     - Second factor authentication");
    println!("     - Use after password login");
    println!("     - Stronger security compliance");
    println!();
    println!("  3. Credential Management");
    println!("     - List user's registered passkeys");
    println!("     - Delete old/lost devices");
    println!("     - View credential metadata");
    println!();
    println!("📝 Real-world usage:");
    println!("   - Complete registration in a browser first");
    println!("   - Then use passkey login (no username!)");
    println!("   - Manage credentials as devices change");

    Ok(())
}
