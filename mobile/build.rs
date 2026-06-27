// build.rs

extern crate winres;

fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("resources/logo.ico");
        res.compile().unwrap();
    }

    // Embed OAuth credentials at compile time
    // Load from .env file if it exists (development)
    // Or from environment variables (CI/CD with GitHub Secrets)

    // Try workspace root first (../.env), then current dir (.env)
    if dotenvy::from_filename("../.env").is_err() {
        let _ = dotenvy::dotenv(); // Fallback to current dir
    }

    // Read from environment and pass to rustc
    if let Ok(client_id) = std::env::var("GOOGLE_OAUTH_CLIENT_ID") {
        println!("cargo:rustc-env=GOOGLE_OAUTH_CLIENT_ID={}", client_id);
    } else {
        println!("cargo:warning=GOOGLE_OAUTH_CLIENT_ID not set - OAuth will not work");
    }

    if let Ok(client_secret) = std::env::var("GOOGLE_OAUTH_CLIENT_SECRET") {
        println!("cargo:rustc-env=GOOGLE_OAUTH_CLIENT_SECRET={}", client_secret);
    } else {
        println!("cargo:warning=GOOGLE_OAUTH_CLIENT_SECRET not set - OAuth will not work");
    }

    // rust2go bridge is now in the go-webauthn crate
}
