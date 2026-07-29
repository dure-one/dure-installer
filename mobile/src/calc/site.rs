//! Site management functionality for site-to-site communication
//!
//! Provides functionality for managing sites that can communicate with each other
//! via WebSocket with DNS TXT record-based authentication.

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Site configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteConfig {
    pub domain: String,
    pub public_key: String,
    pub status: String,
    pub last_seen: Option<u64>,
}

impl SiteConfig {
    pub fn new(domain: String, public_key: String) -> Self {
        Self {
            domain,
            public_key,
            status: "disconnected".to_string(),
            last_seen: None,
        }
    }

    pub fn is_connected(&self) -> bool {
        self.status == "connected"
    }
}

/// List all configured sites
pub fn list_sites() -> Result<Vec<SiteConfig>> {
    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    {
        use crate::calc::db;
        use crate::storage::models::site;

        let mut conn = db::establish_connection();
        site::init_sites_table(&mut conn)?;

        let sites = site::list_sites(&mut conn)?;
        Ok(sites
            .into_iter()
            .map(|s| SiteConfig {
                domain: s.domain,
                public_key: s.public_key,
                status: s.status,
                last_seen: s.last_seen,
            })
            .collect())
    }

    #[cfg(any(target_os = "android", target_arch = "wasm32"))]
    {
        anyhow::bail!("Site management is not available on this platform")
    }
}

/// Add a new site
pub fn add_site(domain: String, public_key: String) -> Result<()> {
    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    {
        use crate::calc::db;
        use crate::storage::models::site;

        let mut conn = db::establish_connection();
        site::init_sites_table(&mut conn)?;

        // Check if site already exists
        if let Some(_existing) = site::get_site(&mut conn, &domain)? {
            anyhow::bail!("Site {} already exists", domain);
        }

        let site_info = site::SiteInfo::new(domain, public_key);
        site::store_site(&mut conn, &site_info)?;

        Ok(())
    }

    #[cfg(any(target_os = "android", target_arch = "wasm32"))]
    {
        anyhow::bail!("Site management is not available on this platform")
    }
}

/// Delete a site
pub fn delete_site(domain: &str) -> Result<()> {
    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    {
        use crate::calc::db;
        use crate::storage::models::site;

        let mut conn = db::establish_connection();
        site::init_sites_table(&mut conn)?;

        let deleted = site::delete_site(&mut conn, domain)?;
        if !deleted {
            anyhow::bail!("Site {} not found", domain);
        }

        Ok(())
    }

    #[cfg(any(target_os = "android", target_arch = "wasm32"))]
    {
        anyhow::bail!("Site management is not available on this platform")
    }
}

/// Update site status
pub fn update_site_status(domain: &str, status: &str) -> Result<()> {
    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    {
        use crate::calc::db;
        use crate::storage::models::site;

        let mut conn = db::establish_connection();
        site::init_sites_table(&mut conn)?;

        site::update_site_status(&mut conn, domain, status)?;

        Ok(())
    }

    #[cfg(any(target_os = "android", target_arch = "wasm32"))]
    {
        anyhow::bail!("Site management is not available on this platform")
    }
}

/// Get site by domain
pub fn get_site(domain: &str) -> Result<Option<SiteConfig>> {
    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    {
        use crate::calc::db;
        use crate::storage::models::site;

        let mut conn = db::establish_connection();
        site::init_sites_table(&mut conn)?;

        let site = site::get_site(&mut conn, domain)?;
        Ok(site.map(|s| SiteConfig {
            domain: s.domain,
            public_key: s.public_key,
            status: s.status,
            last_seen: s.last_seen,
        }))
    }

    #[cfg(any(target_os = "android", target_arch = "wasm32"))]
    {
        anyhow::bail!("Site management is not available on this platform")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calc::db;
    use std::sync::atomic::{AtomicU32, Ordering};

    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    #[test]
    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    fn test_site_operations() {
        // Setup unique test database
        let test_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let test_db = format!("test-dure-site-calc-{}-{}.db", std::process::id(), test_id);
        db::set_db_path(test_db.clone());

        // Initialize table
        let mut conn = db::establish_connection();
        crate::storage::models::site::init_sites_table(&mut conn).unwrap();
        drop(conn);

        // Add site
        add_site("test.example.com".to_string(), "test-pubkey".to_string()).unwrap();

        // List sites
        let sites = list_sites().unwrap();
        assert!(!sites.is_empty());

        // Get site
        let site = get_site("test.example.com").unwrap();
        assert!(site.is_some());

        // Update status
        update_site_status("test.example.com", "connected").unwrap();

        // Delete site
        delete_site("test.example.com").unwrap();
    }
}
