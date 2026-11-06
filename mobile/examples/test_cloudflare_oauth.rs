//! Test Cloudflare OAuth flow
//!
//! Run with: cargo run --example test_cloudflare_oauth

extern crate dure;

use dure::api::cloudflare::CloudflareOAuthHandler;

// OAuth client ID is loaded from .env at compile time
// Redirect URI is now dynamic (random port selected at runtime)
fn get_client_id() -> &'static str {
    option_env!("CLOUDFLARE_OAUTH_CLIENT_ID")
        .unwrap_or("54d11594-84e4-41aa-b438-e81b8fa78ee7")
}

fn main() {
    println!("=== Cloudflare OAuth Test ===\n");

    println!("Configuration (from .env):");
    println!("  Client ID: {}", get_client_id());
    println!("  Redirect URI: Dynamic (random port, like GCP OAuth)");
    println!("  Source: Environment variables\n");

    println!("Creating OAuth handler...");
    let handler = CloudflareOAuthHandler::default();

    println!("✓ Handler created");
    println!("  Client ID: {}", handler.client_id());

    println!("\nStarting OAuth flow...");
    println!("This will:");
    println!("  1. Bind to http://localhost:8976");
    println!("  2. Open your browser to Cloudflare");
    println!("  3. Wait for you to authorize");
    println!("  4. Receive the callback and exchange for token\n");

    println!("Press Enter to start, or Ctrl+C to cancel...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    match handler.run_oauth_flow() {
        Ok(result) => {
            println!("\n✓ OAuth flow completed successfully!");
            println!("  Access token: {}...", &result.access_token.chars().take(20).collect::<String>());
            println!("  Refresh token: {}", result.refresh_token.is_some());
            println!("  Expires at: {}", result.expires_at);
        }
        Err(e) => {
            println!("\n✗ OAuth flow failed: {}", e);
            std::process::exit(1);
        }
    }
}
