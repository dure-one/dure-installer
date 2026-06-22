//! NFTables IP whitelist storage model
//! Desktop-only module

#![cfg(not(any(target_os = "android", target_arch = "wasm32")))]

use crate::calc::nft::WhitelistedIp;
use anyhow::{Context, Result};
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Text};

/// Row struct for querying whitelisted IPs
#[derive(QueryableByName)]
struct WhitelistedIpRow {
    #[diesel(sql_type = Text)]
    ip: String,
    #[diesel(sql_type = Text)]
    description: String,
    #[diesel(sql_type = BigInt)]
    added_at: i64,
}

impl From<WhitelistedIpRow> for WhitelistedIp {
    fn from(row: WhitelistedIpRow) -> Self {
        WhitelistedIp {
            ip: row.ip,
            description: row.description,
            added_at: row.added_at as u64,
        }
    }
}

/// Initialize NFTables whitelist table (migration handled by diesel_migrations)
pub fn init_nft_table(_conn: &mut SqliteConnection) -> Result<()> {
    // Table creation is handled by Diesel migrations
    // This function is kept for API compatibility
    Ok(())
}

/// Add an IP to the whitelist
pub fn add_whitelisted_ip(conn: &mut SqliteConnection, ip: &WhitelistedIp) -> Result<()> {
    diesel::sql_query(
        "INSERT OR REPLACE INTO nft_whitelist (ip, description, added_at)
         VALUES (?1, ?2, ?3)",
    )
    .bind::<Text, _>(&ip.ip)
    .bind::<Text, _>(&ip.description)
    .bind::<BigInt, _>(ip.added_at as i64)
    .execute(conn)
    .context("Failed to add whitelisted IP")?;

    Ok(())
}

/// Remove an IP from the whitelist
pub fn remove_whitelisted_ip(conn: &mut SqliteConnection, ip: &str) -> Result<()> {
    let deleted = diesel::sql_query("DELETE FROM nft_whitelist WHERE ip = ?1")
        .bind::<Text, _>(ip)
        .execute(conn)
        .context("Failed to remove whitelisted IP")?;

    if deleted == 0 {
        anyhow::bail!("IP {} not found in whitelist", ip);
    }

    Ok(())
}

/// Check if an IP is whitelisted
pub fn is_ip_whitelisted(conn: &mut SqliteConnection, ip: &str) -> Result<bool> {
    #[derive(QueryableByName)]
    struct CountRow {
        #[diesel(sql_type = BigInt)]
        count: i64,
    }

    let rows = diesel::sql_query("SELECT COUNT(*) as count FROM nft_whitelist WHERE ip = ?1")
        .bind::<Text, _>(ip)
        .load::<CountRow>(conn)
        .context("Failed to check if IP is whitelisted")?;

    Ok(rows.first().map(|r| r.count > 0).unwrap_or(false))
}

/// List all whitelisted IPs
pub fn list_whitelisted_ips(conn: &mut SqliteConnection) -> Result<Vec<WhitelistedIp>> {
    let rows = diesel::sql_query(
        "SELECT ip, description, added_at
         FROM nft_whitelist
         ORDER BY added_at DESC",
    )
    .load::<WhitelistedIpRow>(conn)
    .context("Failed to list whitelisted IPs")?;

    Ok(rows.into_iter().map(WhitelistedIp::from).collect())
}

/// Get whitelisted IP details
pub fn get_whitelisted_ip(conn: &mut SqliteConnection, ip: &str) -> Result<Option<WhitelistedIp>> {
    let rows = diesel::sql_query(
        "SELECT ip, description, added_at
         FROM nft_whitelist
         WHERE ip = ?1",
    )
    .bind::<Text, _>(ip)
    .load::<WhitelistedIpRow>(conn)
    .context("Failed to get whitelisted IP")?;

    Ok(rows.into_iter().next().map(WhitelistedIp::from))
}

/// Clear all whitelisted IPs
pub fn clear_whitelist(conn: &mut SqliteConnection) -> Result<usize> {
    let deleted = diesel::sql_query("DELETE FROM nft_whitelist")
        .execute(conn)
        .context("Failed to clear whitelist")?;

    Ok(deleted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calc::db;
    use std::sync::atomic::{AtomicU32, Ordering};

    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn setup_test_db() -> diesel::SqliteConnection {
        let test_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let test_db = format!("test-dure-nft-{}-{}.db", std::process::id(), test_id);
        db::set_db_path(test_db);
        db::establish_connection()
    }

    #[test]
    fn test_init_table() {
        let mut conn = setup_test_db();
        init_nft_table(&mut conn).unwrap();
    }

    #[test]
    fn test_add_and_list_ips() {
        let mut conn = setup_test_db();
        init_nft_table(&mut conn).unwrap();

        let ip1 = WhitelistedIp::new("192.168.1.1".to_string(), "Office".to_string());
        let ip2 = WhitelistedIp::new("10.0.0.1".to_string(), "Home".to_string());

        add_whitelisted_ip(&mut conn, &ip1).unwrap();
        add_whitelisted_ip(&mut conn, &ip2).unwrap();

        let ips = list_whitelisted_ips(&mut conn).unwrap();
        assert_eq!(ips.len(), 2);
    }

    #[test]
    fn test_remove_ip() {
        let mut conn = setup_test_db();
        init_nft_table(&mut conn).unwrap();

        let ip = WhitelistedIp::new("192.168.1.1".to_string(), "Test".to_string());
        add_whitelisted_ip(&mut conn, &ip).unwrap();

        assert!(is_ip_whitelisted(&mut conn, "192.168.1.1").unwrap());

        remove_whitelisted_ip(&mut conn, "192.168.1.1").unwrap();

        assert!(!is_ip_whitelisted(&mut conn, "192.168.1.1").unwrap());
    }
}
