//! List and show commands for platforms

use crate::config::AppConfig;
use crate::cli::commands::platform::helpers::*;
use anyhow::{anyhow, Result};
use std::path::PathBuf;

/// Get config file path
fn get_config_path() -> Result<PathBuf> {
    let proj_dirs = directories::ProjectDirs::from("pe", "nikescar", "dure")
        .ok_or_else(|| anyhow!("Failed to get project directories"))?;
    Ok(proj_dirs.config_dir().join("config.yml"))
}

/// Load application config
fn load_config() -> Result<AppConfig> {
    let config_path = get_config_path()?;
    Ok(AppConfig::load_or_default(&config_path))
}

/// Format platform list for display
pub fn format_platform_list(config: &AppConfig) -> String {
    if config.platforms.is_empty() {
        return "No platforms configured\n\nAdd a platform with: dure platform add <name> <type>".to_string();
    }

    let mut output = String::new();
    output.push_str("Platform Status:\n");
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    output.push_str(&format!("{:<20} {:<8} {}\n", "Name", "Type", "Steps"));
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    for platform in &config.platforms {
        let steps = format_steps(platform);
        output.push_str(&format!("{:<20} {:<8} {}\n",
            platform.name,
            platform.platform_type.to_uppercase(),
            steps.chars().take(50).collect::<String>()
        ));
    }

    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    output.push_str(&format!("\nSteps: Connected → Project → VM → Firewall → SSH\n"));
    output.push_str(&format!("\nTotal platforms: {}\n", config.platforms.len()));
    output.push_str("\nUse 'dure platform <name>' to see details and available actions.\n");

    output
}

/// Format platform show output
pub fn format_platform_show(config: &AppConfig, name: &str) -> Result<String> {
    let platform = config.platforms.iter()
        .find(|p| p.name == name)
        .ok_or_else(|| {
            let available: Vec<_> = config.platforms.iter().map(|p| &p.name).collect();
            anyhow!(
                "Platform '{}' not found\n\nAvailable platforms:\n{}",
                name,
                available.iter().map(|n| format!("  • {}", n)).collect::<Vec<_>>().join("\n")
            )
        })?;

    let mut output = String::new();
    output.push_str(&format!("Platform: {}\n", platform.name));
    output.push_str(&format!("Type: {}\n", platform.platform_type.to_uppercase()));
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n");

    output.push_str("Connection Steps:\n");
    let steps = format_steps(platform);
    for step in steps.split("→") {
        output.push_str(&format!("  {}\n", step.trim()));
    }
    output.push('\n');

    output.push_str("Details:\n");
    let details = format_drawer_content(platform);
    for line in details.lines() {
        output.push_str(&format!("  {}\n", line));
    }
    output.push('\n');

    output.push_str("Available Actions:\n");
    output.push_str("  refresh   - Refresh platform data\n");
    if platform.vms.is_empty() {
        output.push_str("  addvm     - Add a new VM\n");
    } else {
        output.push_str("  addvm     - Add a new VM (disabled: VM already exists)\n");
    }
    output.push_str("  firewall  - Update firewall rules\n");
    if !platform.vms.is_empty() {
        output.push_str("  restart   - Restart VM\n");
        output.push_str("  delvm     - Delete VM\n");
    }
    if platform.gcp_selected_project_id.is_some() {
        output.push_str("  billing   - Show billing information\n");
    }
    output.push_str("  delete    - Delete platform\n");
    output.push_str(&format!("\nRun: dure platform {} <action>\n", name));

    Ok(output)
}

/// Execute platform list command
pub fn execute_platform_list() -> Result<()> {
    let config = load_config()?;
    let output = format_platform_list(&config);
    println!("{}", output);
    Ok(())
}

/// Execute platform show command
pub fn execute_platform_show(name: String) -> Result<()> {
    let config = load_config()?;
    let output = format_platform_show(&config, &name)?;
    println!("{}", output);
    Ok(())
}
