//! Operation logs actor - manages audit trail in database

use super::opslog_commands::{LogStatus, LogsCommand, StatsPeriod};
use super::opslog_events::{LogStats, LogsEvent, OperationLog};
use crate::storage::models::opslog;
use crate::viewmodel::{runtime, ViewModelEvent};
use crate::{dure_debug, dure_error, dure_info};
use smol::channel::{Receiver, Sender};

pub struct OpsLogActor {
    command_rx: Receiver<LogsCommand>,
    event_tx: Sender<ViewModelEvent>,
    database_initialized: bool,
}

impl OpsLogActor {
    pub fn new(command_rx: Receiver<LogsCommand>, event_tx: Sender<ViewModelEvent>) -> Self {
        Self {
            command_rx,
            event_tx,
            database_initialized: false,
        }
    }

    pub async fn run(mut self) {
        dure_info!("OpsLogActor started");

        loop {
            match self.command_rx.recv().await {
                Ok(cmd) => {
                    if let Err(e) = self.handle_command(cmd).await {
                        dure_error!("OpsLogActor command failed: {}", e);
                    }
                }
                Err(_) => {
                    dure_info!("OpsLogActor: channel closed, shutting down");
                    break;
                }
            }
        }
    }

    async fn init_database(&self) -> anyhow::Result<()> {
        runtime::unblock(|| -> anyhow::Result<()> {
            use crate::calc::db;
            let mut conn = db::establish_connection();
            opslog::init_operation_logs_table(&mut conn)
        })
        .await
    }

    async fn handle_command(&mut self, cmd: LogsCommand) -> anyhow::Result<()> {
        let operation = match &cmd {
            LogsCommand::CreateLog { .. } => "create_log",
            LogsCommand::UpdateLog { .. } => "update_log",
            LogsCommand::ListLogs { .. } => "list_logs",
            LogsCommand::GetLog { .. } => "get_log",
            LogsCommand::GetLogStats { .. } => "get_log_stats",
            LogsCommand::PurgeLogs { .. } => "purge_logs",
        };

        if !self.database_initialized {
            if let Err(e) = self.init_database().await {
                let error = e.to_string();
                dure_error!("{} failed: {}", operation, error);
                self.send_event(LogsEvent::Error {
                    operation: operation.to_string(),
                    error,
                })
                .await;
                return Err(e);
            }
            self.database_initialized = true;
        }

        let result = match cmd {
            LogsCommand::CreateLog {
                project_id,
                operation_type,
                operation_subtype,
                status,
                metadata,
            } => {
                self.create_log(
                    project_id,
                    operation_type,
                    operation_subtype,
                    status,
                    metadata,
                )
                .await
            }
            LogsCommand::UpdateLog {
                log_id,
                status,
                error_message,
                metadata,
            } => {
                self.update_log(log_id, status, error_message, metadata)
                    .await
            }
            LogsCommand::ListLogs {
                project_id,
                operation_type,
                status,
                limit,
                offset,
            } => {
                self.list_logs(project_id, operation_type, status, limit, offset)
                    .await
            }
            LogsCommand::GetLog { log_id } => self.get_log(log_id).await,
            LogsCommand::GetLogStats { project_id, period } => {
                self.get_log_stats(project_id, period).await
            }
            LogsCommand::PurgeLogs { older_than_days } => self.purge_logs(older_than_days).await,
        };

        if let Err(ref e) = result {
            dure_error!("{} failed: {}", operation, e);
            self.send_event(LogsEvent::Error {
                operation: operation.to_string(),
                error: e.to_string(),
            })
            .await;
        }

        result
    }

    async fn create_log(
        &mut self,
        project_id: String,
        operation_type: String,
        operation_subtype: String,
        status: LogStatus,
        metadata: Option<String>,
    ) -> anyhow::Result<()> {
        let project_id_clone = project_id.clone();
        let operation_type_clone = operation_type.clone();

        let log_id = runtime::unblock(move || -> anyhow::Result<i64> {
            use crate::calc::db;
            let mut conn = db::establish_connection();

            let new_log = opslog::NewOperationLog::new(
                &project_id_clone,
                &operation_type_clone,
                &operation_subtype,
            )
            .status(match status {
                LogStatus::Running => opslog::OperationStatus::Running,
                LogStatus::Success => opslog::OperationStatus::Success,
                LogStatus::Failed | LogStatus::Cancelled => opslog::OperationStatus::Failed,
            });

            let new_log = if let Some(meta) = metadata {
                new_log.details(meta)
            } else {
                new_log
            };

            opslog::create_log(&mut conn, new_log)
        })
        .await?;

        self.send_event(LogsEvent::LogCreated {
            log_id,
            project_id,
            operation_type,
        })
        .await;

        Ok(())
    }

    async fn update_log(
        &mut self,
        log_id: i64,
        status: LogStatus,
        error_message: Option<String>,
        _metadata: Option<String>,
    ) -> anyhow::Result<()> {
        runtime::unblock(move || -> anyhow::Result<()> {
            use crate::calc::db;
            let mut conn = db::establish_connection();

            let status_enum = match status {
                LogStatus::Running => opslog::OperationStatus::Running,
                LogStatus::Success => opslog::OperationStatus::Success,
                LogStatus::Failed | LogStatus::Cancelled => opslog::OperationStatus::Failed,
            };

            opslog::update_log_status(&mut conn, log_id, status_enum, error_message)
        })
        .await?;

        self.send_event(LogsEvent::LogUpdated { log_id, status })
            .await;

        Ok(())
    }

    async fn list_logs(
        &mut self,
        project_id: Option<String>,
        _operation_type: Option<String>,
        _status: Option<LogStatus>,
        limit: usize,
        _offset: usize,
    ) -> anyhow::Result<()> {
        let logs = runtime::unblock(move || -> anyhow::Result<Vec<opslog::OperationLog>> {
            use crate::calc::db;
            let mut conn = db::establish_connection();

            if let Some(pid) = project_id {
                opslog::list_by_project(&mut conn, &pid, limit as i64)
            } else {
                opslog::list_all(&mut conn, limit as i64)
            }
        })
        .await?;

        let total = logs.len();
        let event_logs: Vec<OperationLog> = logs
            .into_iter()
            .map(|l| OperationLog {
                id: l.id,
                project_id: l.project_id,
                operation_type: l.operation_type,
                operation_subtype: l.external_system,
                status: l.status,
                error_message: l.error_message,
                metadata: l.details,
                started_at: l.started_at,
                completed_at: l.completed_at,
            })
            .collect();

        self.send_event(LogsEvent::LogsListed {
            logs: event_logs,
            total,
        })
        .await;

        Ok(())
    }

    async fn get_log(&mut self, _log_id: i64) -> anyhow::Result<()> {
        // ponytail: stub - single log retrieval not implemented in opslog
        // upgrade when needed for log details view
        Ok(())
    }

    async fn get_log_stats(
        &mut self,
        _project_id: String,
        _period: StatsPeriod,
    ) -> anyhow::Result<()> {
        // ponytail: stub - statistics not implemented
        // upgrade when needed for dashboard
        let stats = LogStats {
            total_operations: 0,
            successful_operations: 0,
            failed_operations: 0,
            running_operations: 0,
        };

        self.send_event(LogsEvent::LogStatsRetrieved {
            project_id: _project_id,
            stats,
        })
        .await;

        Ok(())
    }

    async fn purge_logs(&mut self, older_than_days: i64) -> anyhow::Result<()> {
        use std::time::{SystemTime, UNIX_EPOCH};

        let count = runtime::unblock(move || -> anyhow::Result<i64> {
            use crate::calc::db;
            let mut conn = db::establish_connection();

            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;

            let cutoff = now - (older_than_days * 86400); // days to seconds

            opslog::delete_older_than(&mut conn, cutoff).map(|c| c as i64)
        })
        .await?;

        self.send_event(LogsEvent::LogsPurged { count }).await;

        Ok(())
    }

    async fn send_event(&self, event: LogsEvent) {
        let _ = self.event_tx.send(ViewModelEvent::OpsLog(event)).await;
    }
}
