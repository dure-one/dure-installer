//! Operation log storage model - DB table and CRUD operations
//!
//! Records all external API operations (GCP, Cloudflare, etc.) for auditing and debugging

use anyhow::{Context, Result};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// Status of an operation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationStatus {
    /// Operation in progress
    Running,
    /// Operation completed successfully
    Success,
    /// Operation failed with error
    Failed,
}

impl OperationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            OperationStatus::Running => "running",
            OperationStatus::Success => "success",
            OperationStatus::Failed => "failed",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "running" => OperationStatus::Running,
            "success" => OperationStatus::Success,
            "failed" => OperationStatus::Failed,
            _ => OperationStatus::Failed,
        }
    }
}

/// An operation log record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationLog {
    /// Auto-assigned row id
    pub id: i64,
    /// Project/platform identifier
    pub project_id: String,
    /// Type of operation (e.g., "create_vm", "update_firewall", "add_dns_record")
    pub operation_type: String,
    /// External system (e.g., "gcp", "cloudflare", "duckdns")
    pub external_system: String,
    /// Operation status
    pub status: String,
    /// Unix timestamp when operation started (seconds)
    pub started_at: i64,
    /// Unix timestamp when operation completed (seconds), None if still running
    pub completed_at: Option<i64>,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Additional details (JSON or text)
    pub details: Option<String>,
}

impl OperationLog {
    pub fn status(&self) -> OperationStatus {
        OperationStatus::from_str(&self.status)
    }

    pub fn duration_ms(&self) -> Option<i64> {
        self.completed_at.map(|end| (end - self.started_at) * 1000)
    }
}

/// Builder for creating a new operation log
pub struct NewOperationLog {
    pub project_id: String,
    pub operation_type: String,
    pub external_system: String,
    pub status: OperationStatus,
    pub error_message: Option<String>,
    pub details: Option<String>,
}

impl NewOperationLog {
    pub fn new(
        project_id: impl Into<String>,
        operation_type: impl Into<String>,
        external_system: impl Into<String>,
    ) -> Self {
        Self {
            project_id: project_id.into(),
            operation_type: operation_type.into(),
            external_system: external_system.into(),
            status: OperationStatus::Running,
            error_message: None,
            details: None,
        }
    }

    pub fn status(mut self, status: OperationStatus) -> Self {
        self.status = status;
        self
    }

    pub fn error(mut self, msg: impl Into<String>) -> Self {
        self.error_message = Some(msg.into());
        self.status = OperationStatus::Failed;
        self
    }

    pub fn details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

/// Ensure the operation_logs table exists (idempotent).
///
/// Note: The table is created via Diesel migrations, but this provides
/// runtime safety if migrations haven't run yet.
pub fn init_operation_logs_table(conn: &mut SqliteConnection) -> Result<()> {
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS operation_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id TEXT NOT NULL,
            operation_type TEXT NOT NULL,
            external_system TEXT NOT NULL,
            status TEXT NOT NULL,
            started_at INTEGER NOT NULL,
            completed_at INTEGER,
            error_message TEXT,
            details TEXT
        )",
    )
    .execute(conn)
    .context("Failed to create operation_logs table")?;

    // Create indexes if they don't exist
    diesel::sql_query(
        "CREATE INDEX IF NOT EXISTS idx_operation_logs_project_id ON operation_logs(project_id)",
    )
    .execute(conn)
    .context("Failed to create project_id index")?;

    diesel::sql_query(
        "CREATE INDEX IF NOT EXISTS idx_operation_logs_started_at ON operation_logs(started_at DESC)",
    )
    .execute(conn)
    .context("Failed to create started_at index")?;

    diesel::sql_query(
        "CREATE INDEX IF NOT EXISTS idx_operation_logs_project_time ON operation_logs(project_id, started_at DESC)",
    )
    .execute(conn)
    .context("Failed to create project_time index")?;

    Ok(())
}

/// Create a new operation log record
pub fn create_log(conn: &mut SqliteConnection, log: NewOperationLog) -> Result<i64> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let completed_at = if log.status != OperationStatus::Running {
        Some(now)
    } else {
        None
    };

    diesel::sql_query(
        "INSERT INTO operation_logs
         (project_id, operation_type, external_system, status, started_at, completed_at, error_message, details)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
    )
    .bind::<diesel::sql_types::Text, _>(&log.project_id)
    .bind::<diesel::sql_types::Text, _>(&log.operation_type)
    .bind::<diesel::sql_types::Text, _>(&log.external_system)
    .bind::<diesel::sql_types::Text, _>(log.status.as_str())
    .bind::<diesel::sql_types::BigInt, _>(now)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::BigInt>, _>(completed_at)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(log.error_message.as_deref())
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(log.details.as_deref())
    .execute(conn)
    .context("Failed to insert operation log")?;

    let id = diesel::sql_query("SELECT last_insert_rowid() AS id")
        .get_result::<LastRowId>(conn)
        .context("Failed to get last insert rowid")?;

    Ok(id.id)
}

#[derive(QueryableByName)]
struct LastRowId {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    id: i64,
}

/// Update an existing operation log (mark as completed)
pub fn update_log_status(
    conn: &mut SqliteConnection,
    id: i64,
    status: OperationStatus,
    error_message: Option<String>,
) -> Result<()> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    diesel::sql_query(
        "UPDATE operation_logs
         SET status = ?1, completed_at = ?2, error_message = ?3
         WHERE id = ?4",
    )
    .bind::<diesel::sql_types::Text, _>(status.as_str())
    .bind::<diesel::sql_types::BigInt, _>(now)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(error_message.as_deref())
    .bind::<diesel::sql_types::BigInt, _>(id)
    .execute(conn)
    .context("Failed to update operation log")?;

    Ok(())
}

/// List operation logs for a specific project, newest first
pub fn list_by_project(
    conn: &mut SqliteConnection,
    project_id: &str,
    limit: i64,
) -> Result<Vec<OperationLog>> {
    use diesel::sql_types::{BigInt, Nullable, Text};

    #[derive(QueryableByName)]
    struct Row {
        #[diesel(sql_type = BigInt)]
        id: i64,
        #[diesel(sql_type = Text)]
        project_id: String,
        #[diesel(sql_type = Text)]
        operation_type: String,
        #[diesel(sql_type = Text)]
        external_system: String,
        #[diesel(sql_type = Text)]
        status: String,
        #[diesel(sql_type = BigInt)]
        started_at: i64,
        #[diesel(sql_type = Nullable<BigInt>)]
        completed_at: Option<i64>,
        #[diesel(sql_type = Nullable<Text>)]
        error_message: Option<String>,
        #[diesel(sql_type = Nullable<Text>)]
        details: Option<String>,
    }

    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, project_id, operation_type, external_system, status, started_at, completed_at, error_message, details
         FROM operation_logs
         WHERE project_id = ?1
         ORDER BY started_at DESC
         LIMIT ?2",
    )
    .bind::<Text, _>(project_id)
    .bind::<BigInt, _>(limit)
    .load(conn)
    .context("Failed to list operation logs by project")?;

    Ok(rows
        .into_iter()
        .map(|r| OperationLog {
            id: r.id,
            project_id: r.project_id,
            operation_type: r.operation_type,
            external_system: r.external_system,
            status: r.status,
            started_at: r.started_at,
            completed_at: r.completed_at,
            error_message: r.error_message,
            details: r.details,
        })
        .collect())
}

/// List all operation logs, newest first
pub fn list_all(conn: &mut SqliteConnection, limit: i64) -> Result<Vec<OperationLog>> {
    use diesel::sql_types::{BigInt, Nullable, Text};

    #[derive(QueryableByName)]
    struct Row {
        #[diesel(sql_type = BigInt)]
        id: i64,
        #[diesel(sql_type = Text)]
        project_id: String,
        #[diesel(sql_type = Text)]
        operation_type: String,
        #[diesel(sql_type = Text)]
        external_system: String,
        #[diesel(sql_type = Text)]
        status: String,
        #[diesel(sql_type = BigInt)]
        started_at: i64,
        #[diesel(sql_type = Nullable<BigInt>)]
        completed_at: Option<i64>,
        #[diesel(sql_type = Nullable<Text>)]
        error_message: Option<String>,
        #[diesel(sql_type = Nullable<Text>)]
        details: Option<String>,
    }

    let rows: Vec<Row> = diesel::sql_query(
        "SELECT id, project_id, operation_type, external_system, status, started_at, completed_at, error_message, details
         FROM operation_logs
         ORDER BY started_at DESC
         LIMIT ?1",
    )
    .bind::<BigInt, _>(limit)
    .load(conn)
    .context("Failed to list all operation logs")?;

    Ok(rows
        .into_iter()
        .map(|r| OperationLog {
            id: r.id,
            project_id: r.project_id,
            operation_type: r.operation_type,
            external_system: r.external_system,
            status: r.status,
            started_at: r.started_at,
            completed_at: r.completed_at,
            error_message: r.error_message,
            details: r.details,
        })
        .collect())
}

/// Delete operation logs older than the given timestamp
pub fn delete_older_than(conn: &mut SqliteConnection, timestamp: i64) -> Result<usize> {
    let deleted = diesel::sql_query(
        "DELETE FROM operation_logs WHERE started_at < ?1",
    )
    .bind::<diesel::sql_types::BigInt, _>(timestamp)
    .execute(conn)
    .context("Failed to delete old operation logs")?;

    Ok(deleted)
}

/// Delete all operation logs for a specific project
pub fn delete_by_project(conn: &mut SqliteConnection, project_id: &str) -> Result<usize> {
    let deleted = diesel::sql_query(
        "DELETE FROM operation_logs WHERE project_id = ?1",
    )
    .bind::<diesel::sql_types::Text, _>(project_id)
    .execute(conn)
    .context("Failed to delete operation logs for project")?;

    Ok(deleted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calc::db;
    use std::sync::atomic::{AtomicU32, Ordering};

    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    #[test]
    fn test_operation_log_crud() {
        // Setup unique test database
        let test_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let test_db = format!(
            "test-operation-log-{}-{}.db",
            std::process::id(),
            test_id
        );
        db::set_db_path(test_db.clone());

        // Initialize table
        let mut conn = db::establish_connection();
        init_operation_logs_table(&mut conn).unwrap();

        // Create a log
        let new_log = NewOperationLog::new("test-project-123", "create_vm", "gcp")
            .details("Creating VM instance in us-central1-a");

        let log_id = create_log(&mut conn, new_log).unwrap();
        assert!(log_id > 0);

        // List logs by project
        let logs = list_by_project(&mut conn, "test-project-123", 10).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].id, log_id);
        assert_eq!(logs[0].project_id, "test-project-123");
        assert_eq!(logs[0].operation_type, "create_vm");
        assert_eq!(logs[0].external_system, "gcp");
        assert_eq!(logs[0].status(), OperationStatus::Running);
        assert!(logs[0].completed_at.is_none());

        // Update log status to success
        update_log_status(&mut conn, log_id, OperationStatus::Success, None).unwrap();

        // Verify update
        let logs = list_by_project(&mut conn, "test-project-123", 10).unwrap();
        assert_eq!(logs[0].status(), OperationStatus::Success);
        assert!(logs[0].completed_at.is_some());

        // Create a failed log
        let failed_log = NewOperationLog::new("test-project-123", "update_firewall", "gcp")
            .error("Permission denied");

        create_log(&mut conn, failed_log).unwrap();

        // List all logs
        let logs = list_all(&mut conn, 100).unwrap();
        assert_eq!(logs.len(), 2);

        // Delete logs by project
        let deleted = delete_by_project(&mut conn, "test-project-123").unwrap();
        assert_eq!(deleted, 2);

        // Verify deletion
        let logs = list_by_project(&mut conn, "test-project-123", 10).unwrap();
        assert_eq!(logs.len(), 0);
    }

    #[test]
    fn test_operation_status_conversion() {
        assert_eq!(OperationStatus::Running.as_str(), "running");
        assert_eq!(OperationStatus::Success.as_str(), "success");
        assert_eq!(OperationStatus::Failed.as_str(), "failed");

        assert_eq!(OperationStatus::from_str("running"), OperationStatus::Running);
        assert_eq!(OperationStatus::from_str("success"), OperationStatus::Success);
        assert_eq!(OperationStatus::from_str("failed"), OperationStatus::Failed);
        assert_eq!(OperationStatus::from_str("unknown"), OperationStatus::Failed);
    }
}
