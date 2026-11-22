//! Mycart actor events
//! Phase 5: Added service-specific log events and operations events

#[derive(Debug, Clone)]
pub enum MycartEvent {
    // Existing UI events
    StatsLoaded {
        site_address: String,
        stats: Option<crate::api::mycart::MycartStats>,
    },

    // Phase 5: Installer logs event (stdout)
    LogsLoaded {
        site_address: String,
        logs: Vec<String>,
    },

    // Phase 5: SSH server log events (per-file storage)
    MycartLogsLoaded {
        site_address: String,
        logs: std::collections::HashMap<String, Vec<String>>,
    },

    XMPPProxyLogsLoaded {
        site_address: String,
        logs: std::collections::HashMap<String, Vec<String>>,
    },

    Fail2BanRSLogsLoaded {
        site_address: String,
        logs: std::collections::HashMap<String, Vec<String>>,
    },

    // Phase 6: Config files
    MycartConfigLoaded {
        site_address: String,
        config: String,
    },

    XMPPProxyConfigLoaded {
        site_address: String,
        config: String,
    },

    Fail2BanRSConfigLoaded {
        site_address: String,
        config: String,
    },

    Fail2BanRSNFTLoaded {
        site_address: String,
        nft_rules: String,
    },

    // Phase 5: Operations event
    OperationsLoaded {
        site_address: String,
        operations: Vec<crate::storage::models::opslog::OperationLog>,
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
