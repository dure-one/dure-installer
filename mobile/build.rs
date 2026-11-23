// build.rs

extern crate winres;

fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("resources/logo.ico");
        res.compile().unwrap();
    }

    // Font subsetting: Material Symbols (reduce 9.2MB → ~50KB)
    subset_material_symbols().expect("Failed to subset Material Symbols font");

    // Inject git version info for dev builds
    inject_git_version();

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

    // Cloudflare OAuth client ID (PKCE - no client_secret needed)
    // Redirect URI is now dynamic (random port), so only client_id is needed
    if let Ok(client_id) = std::env::var("CLOUDFLARE_OAUTH_CLIENT_ID") {
        println!("cargo:rustc-env=CLOUDFLARE_OAUTH_CLIENT_ID={}", client_id);
    } else {
        println!("cargo:warning=CLOUDFLARE_OAUTH_CLIENT_ID not set - using wrangler's public client as fallback");
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
                        println!(
                            "cargo:warning=Failed to build go-webauthn-cli (exit code: {:?})",
                            s.code()
                        );
                        println!("cargo:warning=Run: cd crates/go-webauthn && ./build-cli.sh");
                    }
                    Err(e) => {
                        println!("cargo:warning=Failed to execute go build: {}", e);
                        println!("cargo:warning=Run: cd crates/go-webauthn && ./build-cli.sh");
                    }
                }
            } else {
                println!("cargo:warning=Go compiler not found - skipping go-webauthn-cli build");
                println!(
                    "cargo:warning=Install Go or run: cd crates/go-webauthn && ./build-cli.sh"
                );
            }
        }
    }

    // Validate translation keys at compile time
    validate_translations();
}

fn validate_translations() {
    use std::collections::HashSet;

    println!("cargo:rerun-if-changed=assets/languages/fluent");
    println!("cargo:rerun-if-changed=src");

    // Parse all translation keys from .ftl files
    let mut keys = HashSet::new();
    for locale in &["en-US", "ko-KR"] {
        let path = format!("assets/languages/fluent/{}.ftl", locale);
        if let Ok(content) = std::fs::read_to_string(&path) {
            for line in content.lines() {
                if let Some(key) = line.split('=').next() {
                    let key = key.trim();
                    if !key.is_empty() && !key.starts_with('#') {
                        keys.insert(key.to_string());
                    }
                }
            }
        }
    }

    // Scan source files for tr!() calls
    let mut missing_keys = Vec::new();
    if let Ok(entries) = glob::glob("src/**/*.rs") {
        for entry in entries.flatten() {
            if let Ok(content) = std::fs::read_to_string(&entry) {
                // Match tr!("key") and tr!("key", ...) - must start with "tr!"
                let re = regex::Regex::new(r#"\btr!\s*\(\s*"([^"]+)""#).unwrap();
                for cap in re.captures_iter(&content) {
                    let key = &cap[1];
                    // Skip file paths (contain / or .)
                    if key.contains('/') || key.contains('.') {
                        continue;
                    }
                    if !keys.contains(key) {
                        missing_keys.push((entry.display().to_string(), key.to_string()));
                    }
                }
            }
        }
    }

    if !missing_keys.is_empty() {
        eprintln!("\n❌ Missing translation keys:");
        for (file, key) in &missing_keys {
            eprintln!("  {} → \"{}\"", file, key);
        }
        panic!(
            "Build failed: {} missing translation keys",
            missing_keys.len()
        );
    }
}

fn subset_material_symbols() -> Result<(), Box<dyn std::error::Error>> {
    use std::collections::HashSet;
    use std::fs;
    use std::path::Path;

    let source_font = Path::new("resources/MaterialSymbolsOutlined[FILL,GRAD,opsz,wght].ttf");
    let subset_font = Path::new("resources/MaterialSymbolsOutlined_subset.ttf");

    // Load original font binary data
    let font_data = fs::read(source_font)?;

    // Define Material Icons used in the app
    // To add more icons:
    // 1. Find icon name at https://fonts.google.com/icons
    // 2. Get Unicode codepoint from Material Symbols reference
    // 3. Add '\u{XXXX}' here
    let chars_to_keep: HashSet<char> = [
        '\u{E8B8}', // settings
        '\u{E5D2}', // menu
        '\u{E88E}', // info
        '\u{E838}', // star
        '\u{F06F}', // star_outline
        '\u{E14B}', // block
    ]
    .iter()
    .copied()
    .collect();

    // Perform font subsetting using fontcull (no OpenType features needed)
    let subsetted_bytes = fontcull::subset_font_data(&font_data, &chars_to_keep, &[])?;

    // Write the subsetted font
    fs::write(subset_font, subsetted_bytes)?;

    // Rebuild if source font changes
    println!("cargo:rerun-if-changed={}", source_font.display());

    Ok(())
}

fn inject_git_version() {
    // Try to get the latest git tag
    let git_tag = std::process::Command::new("git")
        .args(["describe", "--tags", "--exact-match", "HEAD"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string());

    // Get commit hash (short)
    let git_commit = std::process::Command::new("git")
        .args(["rev-parse", "--short=8", "HEAD"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .unwrap_or_else(|| "unknown".to_string())
        .trim()
        .to_string();

    // Determine app version
    let app_version = if let Some(tag) = git_tag {
        // Tagged build: use tag (remove 'v' prefix if present)
        tag.strip_prefix('v').unwrap_or(&tag).to_string()
    } else {
        // Untagged build: use master-{hash}
        format!("master-{}", git_commit)
    };

    // Get git describe for additional info
    let git_describe = std::process::Command::new("git")
        .args(["describe", "--tags", "--always", "--dirty"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .unwrap_or_else(|| format!("master-{}", git_commit))
        .trim()
        .to_string();

    // Get build date (UTC)
    let build_date = chrono::Utc::now().format("%Y-%m-%d").to_string();

    println!("cargo:rustc-env=APP_VERSION={}", app_version);
    println!("cargo:rustc-env=GIT_DESCRIBE={}", git_describe);
    println!("cargo:rustc-env=GIT_COMMIT={}", git_commit);
    println!("cargo:rustc-env=BUILD_DATE={}", build_date);
    println!("cargo:rerun-if-changed=../.git/HEAD");
    println!("cargo:rerun-if-changed=../.git/refs/");
}
