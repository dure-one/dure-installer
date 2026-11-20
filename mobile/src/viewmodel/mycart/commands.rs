//! Mycart actor commands
//!
//! Phase 4: Cleaned up - only crontab and existing UI commands

#[derive(Debug, Clone)]
pub enum MycartCommand {
    // Existing UI commands (keep for current functionality)
    LoadStats {
        site_address: String,
    },

    LoadLogs {
        site_address: String,
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
