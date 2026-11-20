//! Logs actor commands

#[derive(Debug, Clone)]
pub enum LogsCommand {
    /// Create log entry
    CreateLog {
        project_id: String,
        operation_type: String,
        operation_subtype: String,
        status: LogStatus,
        metadata: Option<String>, // JSON metadata
    },

    /// Update log entry
    UpdateLog {
        log_id: i64,
        status: LogStatus,
        error_message: Option<String>,
        metadata: Option<String>,
    },

    /// Query logs
    ListLogs {
        project_id: Option<String>,
        operation_type: Option<String>,
        status: Option<LogStatus>,
        limit: usize,
        offset: usize,
    },

    /// Get single log
    GetLog { log_id: i64 },

    /// Statistics
    GetLogStats {
        project_id: String,
        period: StatsPeriod,
    },

    /// Cleanup
    PurgeLogs { older_than_days: i64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogStatus {
    Running,
    Success,
    Failed,
    Cancelled,
}

impl LogStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Success => "success",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum StatsPeriod {
    Today,
    Week,
    Month,
    Year,
}
