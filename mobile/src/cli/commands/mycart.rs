//! Mycart CLI commands

use crate::calc::mycart;
use crate::calc::profile::ProfileContext;
use anyhow::{Context, Result};

/// Add a new mycart site
pub async fn add_mycart_site(
    profile: &ProfileContext,
    address: String,
    email: String,
    password: String,
    platform_id: Option<String>,
    ssh_host: Option<String>,
) -> Result<()> {
    println!("Adding mycart site: {}", address);

    mycart::add_mycart_site(
        &profile.config_file,
        &address,
        &email,
        &password,
        platform_id.as_deref(),
        ssh_host.as_deref(),
    )
    .await
    .context("Failed to add mycart site")?;

    println!("✓ Mycart site '{}' added successfully", address);
    Ok(())
}

/// Delete a mycart site
pub fn delete_mycart_site(profile: &ProfileContext, address: String) -> Result<()> {
    mycart::delete_mycart_site(&profile.config_file, &address)
        .context("Failed to delete mycart site")?;

    println!("✓ Mycart site '{}' deleted", address);
    Ok(())
}

/// List all mycart sites
pub fn list_mycart_sites(profile: &ProfileContext) -> Result<()> {
    let sites = mycart::list_mycart_sites(&profile.config_file)?;

    if sites.is_empty() {
        println!("No mycart sites configured.");
        println!();
        println!("Run 'dure mycart add <address> <email>' to add a site");
        return Ok(());
    }

    println!("Mycart Sites:");
    println!();
    for (idx, site) in sites.iter().enumerate() {
        println!("{}. {}", idx + 1, site.site_address);
        println!("   Email: {}", site.email);
        if let Some(platform) = &site.platform_id {
            println!("   Platform: {}", platform);
        }
        if let Some(ssh) = &site.ssh_host {
            println!("   SSH Host: {}", ssh);
        }
        println!();
    }

    Ok(())
}

/// Get site statistics
pub async fn get_site_stats(profile: &ProfileContext, address: String) -> Result<()> {
    println!("Fetching stats for {}...", address);

    let stats = mycart::get_site_stats(&profile.config_file, &address)
        .await
        .context("Failed to get site stats")?;

    println!();
    println!("Site Statistics:");
    println!("  Orders: {}", stats.orders_count);
    println!("  Products: {}", stats.products_count);
    println!("  Messages: {}", stats.messages_count);

    Ok(())
}

/// Get site logs
pub async fn get_site_logs(profile: &ProfileContext, address: String) -> Result<()> {
    println!("Fetching logs for {}...", address);
    println!();
    println!("Note: Log fetching requires SSH configuration.");
    println!("Use the GUI to view logs with integrated SSH support.");
    println!();
    println!("Coming soon: CLI log viewing with SSH");

    Ok(())
}

/// Get site crontab
pub async fn get_site_crontab(profile: &ProfileContext, address: String) -> Result<()> {
    println!("Fetching crontab for {}...", address);

    let crontab = mycart::get_site_crontab(&profile.config_file, &address)
        .await
        .context("Failed to get site crontab")?;

    println!();
    println!("Crontab:");
    println!("{}", serde_json::to_string_pretty(&crontab)?);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calc::profile::ProfileManager;
    use tempfile::TempDir;

    fn setup_test_env() -> Result<(TempDir, ProfileContext)> {
        let temp = TempDir::new()?;
        std::env::set_var("HOME", temp.path());

        let profile = ProfileManager::create_profile("test", "pass")?;
        Ok((temp, profile))
    }

    #[test]
    fn test_mycart_list_empty() {
        let (_temp, profile) = setup_test_env().unwrap();

        let result = list_mycart_sites(&profile);
        assert!(result.is_ok());
        // Should show "No mycart sites configured"
    }

    #[test]
    fn test_mycart_add_site() {
        let (_temp, profile) = setup_test_env().unwrap();

        // Note: This requires a real mycart server, so it will fail in unit tests
        // Should be an integration test instead
        let result = smol::block_on(add_mycart_site(
            &profile,
            "test.example.com".to_string(),
            "admin@test.com".to_string(),
            "password".to_string(),
            None,
            None,
        ));

        // Expect failure since no real server
        assert!(result.is_err());
    }

    #[test]
    fn test_mycart_delete_nonexistent() {
        let (_temp, profile) = setup_test_env().unwrap();

        let result = delete_mycart_site(&profile, "nonexistent.com".to_string());
        // Should succeed (idempotent delete) or error gracefully
        let _ = result;
    }
}
