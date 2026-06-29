//! Platform command implementation
//!
//! CLI commands for managing cloud platforms (GCP, Firebase, Supabase)
//! including OAuth setup, project configuration, and resource provisioning.

use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::calc::audit;
use crate::calc::platform::{PlatformType, init_platform, validate_platform_config};
use crate::config::{AppConfig, CloudPlatformConfig};

/// Get config file path
fn get_config_path() -> Result<PathBuf> {
    let proj_dirs = directories::ProjectDirs::from("pe", "nikescar", "dure")
        .context("Failed to get project directories")?;
    Ok(proj_dirs.config_dir().join("config.yml"))
}

/// Load application config
fn load_config() -> Result<(AppConfig, PathBuf)> {
    let config_path = get_config_path()?;
    let app_config = AppConfig::load_or_default(&config_path);
    Ok((app_config, config_path))
}

/// Execute platform status command
pub fn execute_platform_status() -> Result<()> {
    let (app_config, _config_path) = load_config()?;

    if app_config.platforms.is_empty() {
        println!("No platforms configured");
        println!();
        println!("Add a platform with: dure platform add <name> <type>");
        return Ok(());
    }

    println!("Platform Status:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("{:<20} {:<15} {:<40}", "Name", "Type", "Details");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    for platform in &app_config.platforms {
        println!("{:<20} {:<15}", platform.name, platform.platform_type,);

        // Show details based on platform type
        match platform.platform_type.as_str() {
            "gcp" => {
                // GCP details are now in VMs
            }
            "firebase" => {
                if let Some(project_id) = &platform.firebase_project_id {
                    println!("  └─ Project: {}", project_id);
                }
            }
            "supabase" => {
                if let Some(project_ref) = &platform.supabase_project_ref {
                    println!("  └─ Project: {}", project_ref);
                }
                if let Some(api_url) = &platform.supabase_api_url {
                    println!("  └─ URL: {}", api_url);
                }
            }
            _ => {}
        }
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("Total platforms: {}", app_config.platforms.len());

    Ok(())
}

/// Execute platform add command
pub fn execute_platform_add(name: String, platform_type: String) -> Result<()> {
    // Validate platform type
    if PlatformType::from_str(&platform_type).is_none() {
        return Err(anyhow::anyhow!(
            "Invalid platform type: {}. Valid types: gcp, firebase, supabase",
            platform_type
        ));
    }

    let (mut app_config, config_path) = load_config()?;

    // Check if platform already exists
    if app_config.platforms.iter().any(|p| p.name == name) {
        return Err(anyhow::anyhow!("Platform '{}' already exists", name));
    }

    // Create new platform
    let platform = CloudPlatformConfig {
        name: name.clone(),
        platform_type: platform_type.clone(),
        gcp_oauth_access_token: None,
        gcp_oauth_refresh_token: None,
        gcp_oauth_token_expiry: None,
        gcp_connected_email: None,
        gcp_selected_project_id: None,
        firebase_project_id: None,
        firebase_api_key: None,
        supabase_project_ref: None,
        supabase_api_url: None,
        supabase_anon_key: None,
        api_token: None,
        service_account_json: None,
        vms: Vec::new(),
    };

    // Add to config
    app_config.platforms.push(platform);

    // Save config
    app_config.save(&config_path)?;

    // Record audit event
    let _ = audit::push_cli("system", "cli", "platform add", &name);

    println!("✓ Platform '{}' added successfully", name);
    println!("  Type: {}", platform_type);
    println!("  Config: {}", config_path.display());
    println!();
    println!("Next steps:");
    println!(
        "  1. Run 'dure platform init {}' to initialize the platform",
        name
    );
    println!("  2. Follow the OAuth flow to connect your account");

    Ok(())
}

/// Execute platform del command
pub fn execute_platform_del(name: String) -> Result<()> {
    let (mut app_config, config_path) = load_config()?;

    // Find platform
    let platform_idx = app_config
        .platforms
        .iter()
        .position(|p| p.name == name)
        .ok_or_else(|| anyhow::anyhow!("Platform '{}' not found", name))?;

    let platform = &app_config.platforms[platform_idx];

    // Confirm deletion
    println!(
        "⚠️  This will delete platform '{}' ({})",
        name, platform.platform_type
    );
    println!("Are you sure? (y/N): ");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if !input.trim().eq_ignore_ascii_case("y") {
        println!("Deletion cancelled");
        return Ok(());
    }

    // Remove from config
    app_config.platforms.remove(platform_idx);

    // Save config
    app_config.save(&config_path)?;

    // Record audit event
    let _ = audit::push_cli("system", "cli", "platform del", &name);

    println!("✓ Platform '{}' deleted successfully", name);

    Ok(())
}

/// Execute platform init command
pub fn execute_platform_init(name: String) -> Result<()> {
    let (mut app_config, config_path) = load_config()?;

    // Find platform
    let platform = app_config
        .platforms
        .iter_mut()
        .find(|p| p.name == name)
        .ok_or_else(|| anyhow::anyhow!("Platform '{}' not found", name))?;

    println!("Initializing platform '{}'...", name);
    println!("Type: {}", platform.platform_type);
    println!();

    // Validate configuration
    let validation = validate_platform_config(platform);
    if !validation.valid {
        println!("❌ Configuration errors:");
        for error in &validation.errors {
            println!("   - {}", error);
        }
        return Err(anyhow::anyhow!("Platform configuration is invalid"));
    }

    // Run initialization based on platform type
    match init_platform(platform) {
        Ok(progress) => {
            println!();
            println!("Initialization progress:");
            for step in progress {
                println!(
                    "  [{}%] {}: {}",
                    (step.progress * 100.0) as u8,
                    step.step,
                    step.message
                );
            }

            // Save updated config
            app_config.save(&config_path)?;

            println!();
            println!("✓ Platform '{}' initialized successfully", name);
        }
        Err(e) => {
            println!();
            println!("❌ Initialization failed: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

/// Display help for platform commands
pub fn display_platform_help() {
    println!("Platform Management Commands:");
    println!();
    println!("  dure platform status");
    println!("    List all configured platforms and their status");
    println!();
    println!("  dure platform add <name> <type>");
    println!("    Add a new platform configuration");
    println!("    Types: gcp, firebase, supabase");
    println!();
    println!("  dure platform del <name>");
    println!("    Delete a platform configuration");
    println!();
    println!("  dure platform init <name>");
    println!("    Initialize a platform (OAuth, project setup, resources)");
    println!();
    println!("Examples:");
    println!("  dure platform add my-gcp gcp");
    println!("  dure platform init my-gcp");
    println!("  dure platform status");
    println!("  dure platform del my-gcp");
}
