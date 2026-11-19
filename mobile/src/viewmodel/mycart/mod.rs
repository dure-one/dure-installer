//! Mycart ViewModel with Actor/Command/Event pattern
//!
//! Manages mycart operations asynchronously:
//! - Add/delete mycart sites
//! - Update admin passwords
//! - Load stats and logs

pub mod drawer_types;

use crate::api::mycart::{MycartStats, SellerInfo};
use crate::storage::models::mycart_site::MycartSiteInfo;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use drawer_types::{DrawerState, DrawerTab};

/// Commands sent to MycartActor
#[derive(Debug, Clone)]
pub enum MycartCommand {
    /// List all mycart sites
    ListSites,
    /// Add a new mycart site with verification
    AddSite {
        site: String,
        email: String,
        password: String,
    },
    /// Delete a mycart site
    DeleteSite { site: String },
    /// Update admin password
    UpdatePassword {
        site: String,
        old_password: String,
        new_password: String,
    },
    /// Load stats for a site
    LoadStats { site: String },
    /// Load logs for a site
    LoadLogs { site: String, host: String },
}

/// Events emitted by MycartActor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MycartEvent {
    /// Sites list loaded
    SitesListed { sites: Vec<MycartSiteRow> },
    /// Site added successfully
    SiteAdded { site: String },
    /// Failed to add site
    SiteAddFailed { site: String, error: String },
    /// Password updated
    PasswordUpdated { site: String },
    /// Password update failed
    PasswordUpdateFailed { site: String, error: String },
    /// Stats loaded
    StatsLoaded {
        site: String,
        stats: MycartStats,
        seller: Option<SellerInfo>,
    },
    /// Logs loaded
    LogsLoaded {
        site: String,
        logs: HashMap<String, String>,
    },
    /// Site deleted
    SiteDeleted { site: String },
    /// Error occurred
    Error { message: String },
    /// Progress update
    Progress { message: String },
}

/// Mycart site row for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MycartSiteRow {
    pub site_address: String,
    pub email: String,
    pub created_at: u64,
}

impl From<MycartSiteInfo> for MycartSiteRow {
    fn from(info: MycartSiteInfo) -> Self {
        Self {
            site_address: info.site_address,
            email: info.email,
            created_at: info.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mycart_site_row_from_info() {
        let info = MycartSiteInfo::new(
            "dure.co".to_string(),
            "admin".to_string(),
            "encrypted_pass".to_string(),
        );

        let row = MycartSiteRow::from(info.clone());
        assert_eq!(row.site_address, "dure.co");
        assert_eq!(row.email, "admin");
        assert_eq!(row.created_at, info.created_at);
    }

    #[test]
    fn test_mycart_command_variants() {
        let _cmd1 = MycartCommand::ListSites;
        let _cmd2 = MycartCommand::AddSite {
            site: "dure.co".to_string(),
            email: "admin".to_string(),
            password: "pass".to_string(),
        };
        let _cmd3 = MycartCommand::DeleteSite {
            site: "dure.co".to_string(),
        };
        // All variants compile successfully
    }
}
