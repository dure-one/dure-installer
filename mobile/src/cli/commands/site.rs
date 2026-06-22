//! Site command implementation for site-to-site communication management

use crate::calc::audit;
use crate::calc::site::{add_site, delete_site, list_sites};
use anyhow::Result;

/// Execute site status command
///
/// Lists all configured sites and their connection status.
pub fn execute_site_status() -> Result<()> {
    let sites = list_sites()?;

    if sites.is_empty() {
        println!("No sites configured");
        println!();
        println!("To add a site:");
        println!("  dure site add <domain> --public-key <key>");
        return Ok(());
    }

    println!("Configured Sites:");
    println!();
    println!("{:<30} {:<15} {:<20}", "Domain", "Status", "Last Seen");
    println!("{}", "-".repeat(70));

    let total = sites.len();

    for site in sites {
        let last_seen = match site.last_seen {
            Some(ts) => chrono::DateTime::from_timestamp(ts as i64, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "Unknown".to_string()),
            None => "Never".to_string(),
        };

        println!("{:<30} {:<15} {:<20}", site.domain, site.status, last_seen);
    }

    println!();
    println!("Total: {} site(s)", total);

    Ok(())
}

/// Execute site add command
///
/// Adds a new site for site-to-site communication.
pub fn execute_site_add(domain: String, public_key: String) -> Result<()> {
    println!("Adding site: {}", domain);

    add_site(domain.clone(), public_key)?;

    // Record audit event
    let _ = audit::push_cli("system", "cli", "site add", &domain);

    println!();
    println!("✓ Site added successfully");
    println!("  Domain: {}", domain);
    println!();
    println!("Note: Ensure the site's public key is published in DNS TXT record");
    println!("      for authentication to work properly.");

    Ok(())
}

/// Execute site delete command
///
/// Removes a site from the configuration.
pub fn execute_site_del(domain: String) -> Result<()> {
    println!("Deleting site: {}", domain);

    delete_site(&domain)?;

    // Record audit event
    let _ = audit::push_cli("system", "cli", "site del", &domain);

    println!();
    println!("✓ Site deleted successfully");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    #[test]
    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    fn test_site_commands() {
        // Setup unique test database
        let test_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let test_db = format!("test-dure-site-cli-{}-{}.db", std::process::id(), test_id);
        crate::calc::db::set_db_path(test_db.clone());

        // Initialize table
        let mut conn = crate::calc::db::establish_connection();
        crate::storage::models::site::init_sites_table(&mut conn).unwrap();
        drop(conn);

        // Test add
        execute_site_add(
            "test.example.com".to_string(),
            "test-public-key".to_string(),
        )
        .unwrap();

        // Test status
        execute_site_status().unwrap();

        // Test delete
        execute_site_del("test.example.com".to_string()).unwrap();
    }
}
