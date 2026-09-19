//! Repository for drawer data (operation logs and status)
//!
//! Provides async interface to operation log persistence layer

use crate::calc::db;
use crate::storage::models::opslog::{
    create_log, init_operation_logs_table, list_by_project, update_log_status, NewOperationLog,
    OperationLog, OperationStatus,
};
use anyhow::{Context, Result};

/// Repository for managing operation logs and drawer data
#[derive(Clone)]
pub struct DrawerRepository;

impl DrawerRepository {
    /// Create a new repository instance
    pub fn new() -> Self {
        Self
    }

    /// Initialize the operation logs table (idempotent)
    pub async fn init(&self) -> Result<()> {
        smol::unblock(|| {
            let mut conn = db::establish_connection();
            init_operation_logs_table(&mut conn)
        })
        .await
        .context("Failed to initialize operation logs table")
    }

    /// Log a new operation (returns log ID)
    pub async fn log_operation(
        &self,
        project_id: impl Into<String> + Send,
        operation_type: impl Into<String> + Send,
        external_system: impl Into<String> + Send,
        details: Option<String>,
    ) -> Result<i64> {
        let project_id = project_id.into();
        let operation_type = operation_type.into();
        let external_system = external_system.into();

        smol::unblock(move || {
            let mut conn = db::establish_connection();
            let mut log = NewOperationLog::new(&project_id, &operation_type, &external_system);
            if let Some(d) = details {
                log = log.details(d);
            }
            create_log(&mut conn, log)
        })
        .await
        .context("Failed to create operation log")
    }

    /// Mark an operation as completed with success
    pub async fn mark_success(&self, log_id: i64) -> Result<()> {
        smol::unblock(move || {
            let mut conn = db::establish_connection();
            update_log_status(&mut conn, log_id, OperationStatus::Success, None)
        })
        .await
        .context("Failed to mark operation as success")
    }

    /// Mark an operation as failed with error message
    pub async fn mark_failed(&self, log_id: i64, error: impl Into<String> + Send) -> Result<()> {
        let error_msg = error.into();
        smol::unblock(move || {
            let mut conn = db::establish_connection();
            update_log_status(
                &mut conn,
                log_id,
                OperationStatus::Failed,
                Some(error_msg),
            )
        })
        .await
        .context("Failed to mark operation as failed")
    }

    /// Get operation logs for a specific project (newest first)
    pub async fn get_project_logs(
        &self,
        project_id: impl Into<String> + Send,
        limit: i64,
    ) -> Result<Vec<OperationLog>> {
        let project_id = project_id.into();
        smol::unblock(move || {
            let mut conn = db::establish_connection();
            list_by_project(&mut conn, &project_id, limit)
        })
        .await
        .context("Failed to get project operation logs")
    }
}

impl Default for DrawerRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    #[test]
    fn test_repository_log_operation() {
        smol::block_on(async {
            // Setup unique test database
            let test_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
            let test_db = format!(
                "test-drawer-repo-{}-{}.db",
                std::process::id(),
                test_id
            );
            db::set_db_path(test_db);

            let repo = DrawerRepository::new();
            repo.init().await.unwrap();

            // Log an operation
            let log_id = repo
                .log_operation(
                    "test-project-456",
                    "create_vm",
                    "gcp",
                    Some("Creating test VM".to_string()),
                )
                .await
                .unwrap();

            assert!(log_id > 0);

            // Get logs
            let logs = repo.get_project_logs("test-project-456", 10).await.unwrap();
            assert_eq!(logs.len(), 1);
            assert_eq!(logs[0].id, log_id);
            assert_eq!(logs[0].project_id, "test-project-456");
            assert_eq!(logs[0].operation_type, "create_vm");
            assert_eq!(logs[0].external_system, "gcp");
            assert_eq!(logs[0].status(), OperationStatus::Running);

            // Mark as success
            repo.mark_success(log_id).await.unwrap();

            // Verify status update
            let logs = repo.get_project_logs("test-project-456", 10).await.unwrap();
            assert_eq!(logs[0].status(), OperationStatus::Success);
            assert!(logs[0].completed_at.is_some());
        })
    }

    #[test]
    fn test_repository_mark_failed() {
        smol::block_on(async {
            // Setup unique test database
            let test_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
            let test_db = format!(
                "test-drawer-repo-fail-{}-{}.db",
                std::process::id(),
                test_id
            );
            db::set_db_path(test_db);

            let repo = DrawerRepository::new();
            repo.init().await.unwrap();

            // Log an operation
            let log_id = repo
                .log_operation("test-project-789", "update_firewall", "gcp", None)
                .await
                .unwrap();

            // Mark as failed
            repo.mark_failed(log_id, "Permission denied").await.unwrap();

            // Verify failure
            let logs = repo.get_project_logs("test-project-789", 10).await.unwrap();
            assert_eq!(logs[0].status(), OperationStatus::Failed);
            assert_eq!(logs[0].error_message, Some("Permission denied".to_string()));
            assert!(logs[0].completed_at.is_some());
        })
    }

    #[test]
    fn test_repository_multiple_projects() {
        smol::block_on(async {
            // Setup unique test database
            let test_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
            let test_db = format!(
                "test-drawer-repo-multi-{}-{}.db",
                std::process::id(),
                test_id
            );
            db::set_db_path(test_db);

            let repo = DrawerRepository::new();
            repo.init().await.unwrap();

            // Log operations for multiple projects
            repo.log_operation("project-a", "op1", "gcp", None)
                .await
                .unwrap();
            repo.log_operation("project-a", "op2", "gcp", None)
                .await
                .unwrap();
            repo.log_operation("project-b", "op3", "cloudflare", None)
                .await
                .unwrap();

            // Get logs for project-a only
            let logs_a = repo.get_project_logs("project-a", 10).await.unwrap();
            assert_eq!(logs_a.len(), 2);
            assert!(logs_a.iter().all(|l| l.project_id == "project-a"));

            // Get logs for project-b only
            let logs_b = repo.get_project_logs("project-b", 10).await.unwrap();
            assert_eq!(logs_b.len(), 1);
            assert_eq!(logs_b[0].project_id, "project-b");
            assert_eq!(logs_b[0].external_system, "cloudflare");
        })
    }
}
