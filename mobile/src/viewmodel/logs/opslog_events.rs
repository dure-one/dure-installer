//! Logs actor events

use super::opslog_commands::LogStatus;

#[derive(Debug, Clone)]
pub struct OperationLog {
    pub id: i64,
    pub project_id: String,
    pub operation_type: String,
    pub operation_subtype: String,
    pub status: String,
    pub error_message: Option<String>,
    pub metadata: Option<String>,
    pub started_at: i64,
    pub completed_at: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct LogStats {
    pub total_operations: i64,
    pub successful_operations: i64,
    pub failed_operations: i64,
    pub running_operations: i64,
}

#[derive(Debug, Clone)]
pub enum LogsEvent {
    LogCreated {
        log_id: i64,
        project_id: String,
        operation_type: String,
    },

    LogUpdated {
        log_id: i64,
        status: LogStatus,
    },

    LogsListed {
        logs: Vec<OperationLog>,
        total: usize,
    },

    LogRetrieved {
        log: OperationLog,
    },

    LogStatsRetrieved {
        project_id: String,
        stats: LogStats,
    },

    LogsPurged {
        count: i64,
    },

    Error {
        operation: String,
        error: String,
    },
}
