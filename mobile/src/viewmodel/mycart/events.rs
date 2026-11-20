//! Mycart actor events
//! Phase 4: Cleaned up - only crontab and UI events

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum MycartEvent {
    // Existing UI events
    StatsLoaded {
        site_address: String,
        stats: Option<crate::api::mycart::MycartStats>,
    },

    LogsLoaded {
        site_address: String,
        logs: HashMap<String, String>,
    },

    // Phase 4: Crontab management events
    CrontabLoaded {
        site_address: String,
        jobs: Vec<crate::api::mycart::CrontabJob>,
    },

    CronjobUpdated {
        site_address: String,
        job: crate::api::mycart::CrontabJob,
    },

    // Error
    Error {
        operation: String,
        error: String,
    },
}
