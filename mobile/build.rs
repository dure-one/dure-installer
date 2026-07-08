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
        println!(
            "cargo:rustc-env=GOOGLE_OAUTH_CLIENT_SECRET={}",
            client_secret
        );
    } else {
        println!("cargo:warning=GOOGLE_OAUTH_CLIENT_SECRET not set - OAuth will not work");
    }

    // rust2go bridge is now in the go-webauthn crate

    // Build go-webauthn-cli executable
    #[cfg(not(target_arch = "wasm32"))]
    {
        let go_webauthn_dir = std::path::PathBuf::from("../crates/go-webauthn");
        if go_webauthn_dir.exists() {
            println!("cargo:rerun-if-changed=../crates/go-webauthn/cmd");
            println!("cargo:rerun-if-changed=../crates/go-webauthn/go");

            // Check if go is available
            if std::process::Command::new("go")
                .arg("version")
                .output()
                .is_ok()
            {
                println!("cargo:warning=Building go-webauthn-cli...");

                // Create bin directory
                let bin_dir = go_webauthn_dir.join("bin");
                let _ = std::fs::create_dir_all(&bin_dir);

                // Build the CLI
                let status = std::process::Command::new("go")
                    .arg("build")
                    .arg("-o")
                    .arg(bin_dir.join("go-webauthn-cli"))
                    .arg("../cmd")
                    .current_dir(go_webauthn_dir.join("go"))
                    .status();

                match status {
                    Ok(s) if s.success() => {
                        println!("cargo:warning=Successfully built go-webauthn-cli");
                    }
                    Ok(s) => {
                        println!("cargo:warning=Failed to build go-webauthn-cli (exit code: {:?})", s.code());
                        println!("cargo:warning=Run: cd crates/go-webauthn && ./build-cli.sh");
                    }
                    Err(e) => {
                        println!("cargo:warning=Failed to execute go build: {}", e);
                        println!("cargo:warning=Run: cd crates/go-webauthn && ./build-cli.sh");
                    }
                }
            } else {
                println!("cargo:warning=Go compiler not found - skipping go-webauthn-cli build");
                println!("cargo:warning=Install Go or run: cd crates/go-webauthn && ./build-cli.sh");
            }
        }
    }
}
