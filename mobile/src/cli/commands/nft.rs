//! NFTables command implementation for firewall management

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use crate::calc::db;
use crate::calc::nft::{
    WhitelistedIp, remove_whitelisted_ip as remove_ip_from_nft, show_ruleset,
    whitelist_ip as add_ip_to_nft,
};
use crate::storage::models::nft::{
    add_whitelisted_ip, init_nft_table, is_ip_whitelisted, list_whitelisted_ips,
    remove_whitelisted_ip as remove_from_db,
};
use anyhow::Result;

/// Execute NFT show command
///
/// Displays the current nftables ruleset.
/// Requires nftables to be installed.
pub fn execute_nft_show() -> Result<()> {
    dure_info!("Fetching current nftables ruleset...");
    dure_info!("");

    let ruleset = show_ruleset()?;

    if ruleset.trim().is_empty() {
        println!("No nftables rules configured");
        println!();
        println!("To set up Dure firewall rules:");
        println!("  1. Whitelist your current IP: dure nft whitelist <your-ip>");
        println!("  2. The rules will be automatically applied");
        return Ok(());
    }

    println!("{}", ruleset);

    Ok(())
}

/// Execute NFT whitelist command
///
/// Adds an IP address to the SSH whitelist and updates nftables rules.
/// Requires root/sudo privileges.
///
/// # Arguments
///
/// * `ip` - IP address to whitelist
/// * `description` - Optional description for the IP
pub fn execute_nft_whitelist(ip: String, description: Option<String>) -> Result<()> {
    // Get database connection
    let _db_path = get_db_path()?;
    let mut conn = db::establish_connection();

    init_nft_table(&mut conn)?;

    // Check if IP is already whitelisted
    if is_ip_whitelisted(&mut conn, &ip)? {
        dure_info!("IP {} is already whitelisted", ip);
        return Ok(());
    }

    let desc = description
        .unwrap_or_else(|| format!("Added on {}", chrono::Utc::now().format("%Y-%m-%d")));
    let whitelisted_ip = WhitelistedIp::new(ip.clone(), desc);

    // Add to database
    add_whitelisted_ip(&mut conn, &whitelisted_ip)?;

    dure_info!("Added {} to whitelist", ip);

    // Get all whitelisted IPs
    let all_ips = list_whitelisted_ips(&mut conn)?;
    let ip_list: Vec<String> = all_ips.iter().map(|w| w.ip.clone()).collect();

    // Update nftables rules
    dure_info!("Updating nftables rules...");
    add_ip_to_nft(&ip, &ip_list)?;

    dure_info!(" Firewall rules updated successfully");
    dure_info!("");
    dure_info!("SSH access (port 22) is now allowed from:");
    for whitelisted in &all_ips {
        dure_info!("  {} - {}", whitelisted.ip, whitelisted.description);
    }

    Ok(())
}

/// Execute NFT remove command
///
/// Removes an IP address from the SSH whitelist and updates nftables rules.
/// Requires root/sudo privileges.
///
/// # Arguments
///
/// * `ip` - IP address to remove from whitelist
pub fn execute_nft_remove(ip: String) -> Result<()> {
    // Get database connection
    let _db_path = get_db_path()?;
    let mut conn = db::establish_connection();

    init_nft_table(&mut conn)?;

    // Check if IP is whitelisted
    if !is_ip_whitelisted(&mut conn, &ip)? {
        anyhow::bail!("IP {} is not in the whitelist", ip);
    }

    // Remove from database
    remove_from_db(&mut conn, &ip)?;

    dure_info!("Removed {} from whitelist", ip);

    // Get remaining whitelisted IPs
    let remaining_ips = list_whitelisted_ips(&mut conn)?;
    let ip_list: Vec<String> = remaining_ips.iter().map(|w| w.ip.clone()).collect();

    // Update nftables rules
    dure_info!("Updating nftables rules...");
    remove_ip_from_nft(&ip, &ip_list)?;

    dure_info!(" Firewall rules updated successfully");
    dure_info!("");

    if remaining_ips.is_empty() {
        dure_warn!(
            "No IPs are whitelisted. SSH access (port 22) is now blocked for all IPs."
        );
        dure_info!("");
        dure_info!("To restore SSH access, whitelist an IP:");
        dure_info!("  dure nft whitelist <ip-address>");
    } else {
        dure_info!("SSH access (port 22) is still allowed from:");
        for whitelisted in &remaining_ips {
            dure_info!("  {} - {}", whitelisted.ip, whitelisted.description);
        }
    }

    Ok(())
}

/// Execute NFT list command
///
/// Lists all whitelisted IP addresses.
pub fn execute_nft_list() -> Result<()> {
    let _db_path = get_db_path()?;
    let mut conn = db::establish_connection();

    init_nft_table(&mut conn)?;

    let ips = list_whitelisted_ips(&mut conn)?;

    if ips.is_empty() {
        println!("No whitelisted IPs");
        println!();
        println!("To whitelist an IP:");
        println!("  dure nft whitelist <ip-address>");
        return Ok(());
    }

    println!("Whitelisted IPs for SSH access (port 22):");
    println!();

    for ip in ips {
        let added_date = chrono::DateTime::from_timestamp(ip.added_at as i64, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        println!("  {} - {} (added: {})", ip.ip, ip.description, added_date);
    }

    println!();
    println!("Ports 80 (HTTP) and 443 (HTTPS) are open to all IPs.");

    Ok(())
}

fn get_db_path() -> Result<std::path::PathBuf> {
    let db_path = crate::calc::db::get_db_path();
    Ok(std::path::PathBuf::from(db_path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_db_path() {
        let path = get_db_path().unwrap();
        assert!(path.to_string_lossy().contains("dure"));
        assert!(path.to_string_lossy().ends_with(".db"));
    }
}
