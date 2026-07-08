//! Billing command implementation

use crate::cli::commands::platform::helpers::*;
use crate::cli::commands::platform::runner::PlatformCliRunner;
use crate::config::AppConfig;
use crate::viewmodel::platform::{PlatformCommand, PlatformEvent};
use anyhow::{Result, anyhow};
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

/// Execute billing command (inner function for testing)
#[cfg(test)]
pub async fn execute_billing_inner(
    runner: &mut crate::cli::commands::platform::tests::MockPlatformRunner,
    platform: &crate::config::CloudPlatformConfig,
) -> Result<()> {
    // For testing, use placeholder values
    let project_id = platform
        .gcp_selected_project_id
        .as_ref()
        .ok_or_else(|| anyhow!("No project selected"))?;

    let event = runner
        .execute_command(PlatformCommand::FetchBilling {
            platform_name: project_id.clone(),
            project_id: project_id.clone(),
            dataset: "billing_export".to_string(),
            table: "gcp_billing_export".to_string(),
        })
        .await?;

    if let PlatformEvent::BillingFetched { records, .. } = event {
        println!("Billing Summary (Last 3 Months):");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("{:<12} {}", "Month", "Cost (USD)");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        let mut total = 0.0;
        for record in &records {
            println!("{:<12} ${:.2}", record.month, record.total_net_cost);
            total += record.total_net_cost;
        }

        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("{:<12} ${:.2}", "Total:", total);

        Ok(())
    } else {
        Err(anyhow!("Unexpected event: {:?}", event))
    }
}

/// Execute billing command
pub fn execute_billing_command(name: String) -> Result<()> {
    let config = load_config()?;

    let platform = config
        .platforms
        .iter()
        .find(|p| p.gcp_selected_project_id.as_ref() == Some(&name))
        .ok_or_else(|| anyhow!("Platform '{}' not found", name))?;

    // Validate platform is ready
    validate_platform_ready(platform, "billing")?;

    // Check billing configuration
    let project_id = platform
        .gcp_selected_project_id
        .as_ref()
        .ok_or_else(|| anyhow!("No project selected for platform '{}'", name))?;

    // Use hardcoded billing export settings (could be made configurable later)
    let dataset = "billing_export".to_string();
    let table = "gcp_billing_export".to_string();

    println!("Fetching billing data...");

    smol::block_on(async {
        let mut runner = PlatformCliRunner::new();

        let event = runner
            .execute_command(PlatformCommand::FetchBilling {
                platform_name: project_id.clone(),
                project_id: project_id.clone(),
                dataset,
                table,
            })
            .await?;

        if let PlatformEvent::BillingFetched { records, .. } = event {
            println!("\nBilling Summary (Last 3 Months):");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("{:<12} {}", "Month", "Cost (USD)");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

            let mut total = 0.0;
            for record in &records {
                println!("{:<12} ${:.2}", record.month, record.total_net_cost);
                total += record.total_net_cost;
            }

            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("{:<12} ${:.2}", "Total:", total);

            Ok(())
        } else {
            Err(anyhow!("Unexpected event: {:?}", event))
        }
    })
}
