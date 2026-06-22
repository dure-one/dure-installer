//! Site storage model for site-to-site communication

use anyhow::{Context, Result};
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Nullable, Text};

/// Site information for site-to-site communication
#[derive(Debug, Clone)]
pub struct SiteInfo {
    pub domain: String,
    pub public_key: String,
    pub status: String,
    pub last_seen: Option<u64>,
    pub created_at: u64,
    pub updated_at: u64,
}

impl SiteInfo {
    pub fn new(domain: String, public_key: String) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_secs();

        Self {
            domain,
            public_key,
            status: "disconnected".to_string(),
            last_seen: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn is_connected(&self) -> bool {
        self.status == "connected"
    }
}

/// Initialize sites table (migration handled by diesel_migrations)
pub fn init_sites_table(_conn: &mut SqliteConnection) -> Result<()> {
    // Table creation is handled by Diesel migrations
    // This function is kept for API compatibility
    Ok(())
}

/// Store site information
pub fn store_site(conn: &mut SqliteConnection, site: &SiteInfo) -> Result<()> {
    let last_seen_value: Option<i64> = site.last_seen.map(|v| v as i64);

    diesel::sql_query(
        "INSERT OR REPLACE INTO sites
         (domain, public_key, status, last_seen, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    )
    .bind::<Text, _>(&site.domain)
    .bind::<Text, _>(&site.public_key)
    .bind::<Text, _>(&site.status)
    .bind::<Nullable<BigInt>, _>(last_seen_value)
    .bind::<BigInt, _>(site.created_at as i64)
    .bind::<BigInt, _>(site.updated_at as i64)
    .execute(conn)
    .context("Failed to store site")?;

    Ok(())
}

/// Get site by domain
pub fn get_site(conn: &mut SqliteConnection, domain: &str) -> Result<Option<SiteInfo>> {
    #[derive(QueryableByName)]
    struct SiteRow {
        #[diesel(sql_type = Text)]
        domain: String,
        #[diesel(sql_type = Text)]
        public_key: String,
        #[diesel(sql_type = Text)]
        status: String,
        #[diesel(sql_type = Nullable<BigInt>)]
        last_seen: Option<i64>,
        #[diesel(sql_type = BigInt)]
        created_at: i64,
        #[diesel(sql_type = BigInt)]
        updated_at: i64,
    }

    let rows = diesel::sql_query(
        "SELECT domain, public_key, status, last_seen, created_at, updated_at
         FROM sites
         WHERE domain = ?1",
    )
    .bind::<Text, _>(domain)
    .load::<SiteRow>(conn)
    .context("Failed to query site")?;

    Ok(rows.into_iter().next().map(|row| SiteInfo {
        domain: row.domain,
        public_key: row.public_key,
        status: row.status,
        last_seen: row.last_seen.map(|v| v as u64),
        created_at: row.created_at as u64,
        updated_at: row.updated_at as u64,
    }))
}

/// List all sites
pub fn list_sites(conn: &mut SqliteConnection) -> Result<Vec<SiteInfo>> {
    #[derive(QueryableByName)]
    struct SiteRow {
        #[diesel(sql_type = Text)]
        domain: String,
        #[diesel(sql_type = Text)]
        public_key: String,
        #[diesel(sql_type = Text)]
        status: String,
        #[diesel(sql_type = Nullable<BigInt>)]
        last_seen: Option<i64>,
        #[diesel(sql_type = BigInt)]
        created_at: i64,
        #[diesel(sql_type = BigInt)]
        updated_at: i64,
    }

    let rows = diesel::sql_query(
        "SELECT domain, public_key, status, last_seen, created_at, updated_at
         FROM sites
         ORDER BY domain ASC",
    )
    .load::<SiteRow>(conn)
    .context("Failed to list sites")?;

    Ok(rows
        .into_iter()
        .map(|row| SiteInfo {
            domain: row.domain,
            public_key: row.public_key,
            status: row.status,
            last_seen: row.last_seen.map(|v| v as u64),
            created_at: row.created_at as u64,
            updated_at: row.updated_at as u64,
        })
        .collect())
}

/// Delete site by domain
pub fn delete_site(conn: &mut SqliteConnection, domain: &str) -> Result<bool> {
    let deleted = diesel::sql_query("DELETE FROM sites WHERE domain = ?1")
        .bind::<Text, _>(domain)
        .execute(conn)
        .context("Failed to delete site")?;

    Ok(deleted > 0)
}

/// Update site status
pub fn update_site_status(conn: &mut SqliteConnection, domain: &str, status: &str) -> Result<()> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(std::time::Duration::from_secs(0))
        .as_secs();

    diesel::sql_query(
        "UPDATE sites SET status = ?1, last_seen = ?2, updated_at = ?3 WHERE domain = ?4",
    )
    .bind::<Text, _>(status)
    .bind::<BigInt, _>(now as i64)
    .bind::<BigInt, _>(now as i64)
    .bind::<Text, _>(domain)
    .execute(conn)
    .context("Failed to update site status")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calc::db;
    use std::sync::atomic::{AtomicU32, Ordering};

    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn setup_test_db() -> diesel::SqliteConnection {
        let test_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let test_db = format!("test-dure-site-{}-{}.db", std::process::id(), test_id);
        db::set_db_path(test_db);
        db::establish_connection()
    }

    #[test]
    fn test_site_crud() {
        let mut conn = setup_test_db();
        init_sites_table(&mut conn).unwrap();

        // Create
        let site = SiteInfo::new("example.com".to_string(), "pubkey123".to_string());
        store_site(&mut conn, &site).unwrap();

        // Read
        let retrieved = get_site(&mut conn, "example.com").unwrap().unwrap();
        assert_eq!(retrieved.domain, "example.com");
        assert_eq!(retrieved.public_key, "pubkey123");

        // List
        let sites = list_sites(&mut conn).unwrap();
        assert!(!sites.is_empty());

        // Update status
        update_site_status(&mut conn, "example.com", "connected").unwrap();
        let updated = get_site(&mut conn, "example.com").unwrap().unwrap();
        assert_eq!(updated.status, "connected");

        // Delete
        let deleted = delete_site(&mut conn, "example.com").unwrap();
        assert!(deleted);
    }
}
