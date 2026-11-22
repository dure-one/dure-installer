//! Mycart actor commands
//!
//! Phase 4: Cleaned up - only crontab and existing UI commands

#[derive(Debug, Clone)]
pub enum MycartCommand {
    // Existing UI commands (keep for current functionality)
    LoadStats {
        site_address: String,
    },

    // Phase 5: Installer logs (stdout filtered by site)
    LoadLogs {
        site_address: String,
        limit: usize,
    },

    // Phase 5: Operations loading command
    LoadOperations {
        site_address: String,
        limit: i64,
    },

    // Phase 5: SSH server logs (3 services)
    LoadMycartLogs {
        site_address: String,
        limit: usize,
        profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    },

    LoadXMPPProxyLogs {
        site_address: String,
        limit: usize,
        profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    },

    LoadFail2BanRSLogs {
        site_address: String,
        limit: usize,
        profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    },

    // Phase 6: Config files
    LoadMycartConfig {
        site_address: String,
        profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    },

    LoadXMPPProxyConfig {
        site_address: String,
        profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    },

    LoadFail2BanRSConfig {
        site_address: String,
        profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    },

    LoadFail2BanRSNFT {
        site_address: String,
        profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    },

    // Phase 4: Crontab management commands
    ListCrontab {
        site_address: String,
    },

    EnableCronjob {
        site_address: String,
        job_id: String,
    },

    DisableCronjob {
        site_address: String,
        job_id: String,
    },

    SetInterval {
        site_address: String,
        job_id: String,
        interval: String,
    },
}
