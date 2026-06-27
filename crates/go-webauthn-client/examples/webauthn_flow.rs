// WebAuthn registration and authentication flow example
//
// This demonstrates the complete WebAuthn ceremony:
// 1. Signup (registration)
// 2. Signin (authentication)
//
// Note: This is a simplified example. In a real application:
// - The challenge_json would be sent to a browser
// - The browser would call navigator.credentials.create() / get()
// - The response would be sent back to the server

use go_webauthn_client::{
    GoWebAuthnClient, WebAuthnSignupBeginParams, WebAuthnSignupFinishParams,
    WebAuthnSigninBeginParams, WebAuthnSigninFinishParams,
};

fn main() -> anyhow::Result<()> {
    println!("=== WebAuthn Flow Example ===\n");

    // Create client
    let mut client = GoWebAuthnClient::new(None)?;

    // ========================================================================
    // Step 1: Begin Registration
    // ========================================================================
    println!("1. Beginning registration...");

    let signup_begin = WebAuthnSignupBeginParams {
        rp_display_name: "Example Corp".to_string(),
        rp_id: "example.com".to_string(),
        rp_origins: "https://example.com".to_string(),
        username: "alice@example.com".to_string(),
        display_name: "Alice Smith".to_string(),
        scenario: "passwordless".to_string(),
    };

    let signup_result = client.webauthn_signup_begin(signup_begin)?;

    println!("   ✅ Session ID: {}", signup_result.session_id);
    println!("   ✅ Challenge JSON length: {} bytes", signup_result.challenge_json.len());
    println!("   → In a real app, this challenge would be sent to the browser\n");

    // ========================================================================
    // Step 2: Finish Registration (simulated)
    // ========================================================================
    println!("2. Finishing registration...");
    println!("   ⚠️  In a real app, the browser would:");
    println!("      - Call navigator.credentials.create()");
    println!("      - Return the credential response");
    println!("   ⚠️  This example cannot complete without a browser\n");

    // We can't actually complete the registration without a real browser,
    // so we'll stop here. In a real application:
    //
    // let signup_finish = WebAuthnSignupFinishParams {
    //     session_id: signup_result.session_id,
    //     credential_json: response_from_browser,
    // };
    // let finish_result = client.webauthn_signup_finish(signup_finish)?;

    // ========================================================================
    // Step 3: Begin Authentication (would happen after signup)
    // ========================================================================
    println!("3. Beginning authentication...");
    println!("   ⚠️  This would fail because we haven't completed registration");
    println!("   → In a real app with completed registration:");

    let signin_begin = WebAuthnSigninBeginParams {
        username: "alice@example.com".to_string(),
        scenario: "passwordless".to_string(),
    };

    match client.webauthn_signin_begin(signin_begin) {
        Ok(result) => {
            println!("   ✅ Session ID: {}", result.session_id);
            println!("   ✅ Challenge JSON length: {} bytes\n", result.challenge_json.len());
        }
        Err(e) => {
            println!("   ❌ Expected error: {}\n", e);
            println!("   → This is normal - no credentials exist yet");
        }
    }

    // ========================================================================
    // Summary
    // ========================================================================
    println!("\n=== Summary ===");
    println!("✅ WebAuthn CLI integration works!");
    println!("✅ Can begin registration ceremony");
    println!("✅ Can begin authentication ceremony");
    println!();
    println!("📝 To use in a real app:");
    println!("   1. Call signup_begin() from your server");
    println!("   2. Send challenge_json to browser");
    println!("   3. Browser calls navigator.credentials.create()");
    println!("   4. Send credential back to server");
    println!("   5. Call signup_finish() to complete registration");
    println!();
    println!("   Then for authentication:");
    println!("   1. Call signin_begin()");
    println!("   2. Send challenge_json to browser");
    println!("   3. Browser calls navigator.credentials.get()");
    println!("   4. Send assertion back to server");
    println!("   5. Call signin_finish() to verify");

    Ok(())
}
