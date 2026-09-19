//! Drawer actor implementation for tab management and operation logging

use super::{DrawerCommand, DrawerEvent, DrawerRepository, DrawerState};
use crate::viewmodel::logs::LogCommand;
use crate::viewmodel::{runtime, ViewModelEvent};
use crate::{dure_debug, dure_error, dure_info};
use smol::channel::{Receiver, Sender};

/// Actor for managing drawer state and operation logs
pub struct DrawerActor {
    command_rx: Receiver<DrawerCommand>,
    event_tx: Sender<ViewModelEvent>,
    logs_tx: Sender<LogCommand>,
    repository: DrawerRepository,
    state: DrawerState,
}

impl DrawerActor {
    pub fn new(
        command_rx: Receiver<DrawerCommand>,
        event_tx: Sender<ViewModelEvent>,
        logs_tx: Sender<LogCommand>,
    ) -> Self {
        Self {
            command_rx,
            event_tx,
            logs_tx,
            repository: DrawerRepository::new(),
            state: DrawerState::new(),
        }
    }

    /// Run the actor loop
    pub async fn run(mut self) {
        dure_info!("DrawerActor started");

        // Initialize repository
        if let Err(e) = self.repository.init().await {
            dure_error!("DrawerActor: failed to initialize repository: {}", e);
        }

        loop {
            match self.command_rx.recv().await {
                Ok(cmd) => {
                    if let Err(e) = self.handle_command(cmd).await {
                        dure_error!("DrawerActor command failed: {}", e);
                    }
                }
                Err(_) => {
                    dure_info!("DrawerActor: channel closed, shutting down");
                    break;
                }
            }
        }
    }

    /// Handle a drawer command
    async fn handle_command(&mut self, cmd: DrawerCommand) -> anyhow::Result<()> {
        dure_debug!("DrawerActor: handling command {:?}", cmd);

        let result = match cmd {
            DrawerCommand::SwitchTab { tab } => self.switch_tab(tab).await,
            DrawerCommand::LoadOperations { project_id, limit } => {
                self.load_operations(project_id, limit).await
            }
            DrawerCommand::LoadLogs { project_id, limit } => {
                self.load_logs(project_id, limit).await
            }
            DrawerCommand::Refresh => self.refresh().await,
        };

        if let Err(e) = result {
            let error_msg = format!("{:#}", e);
            dure_error!("DrawerActor: command error: {}", error_msg);

            let event = DrawerEvent::Error {
                operation: "drawer_command".to_string(),
                error: error_msg,
            };

            self.send_event(event).await?;
        }

        Ok(())
    }

    /// Switch to a different tab
    async fn switch_tab(&mut self, tab: super::DrawerTab) -> anyhow::Result<()> {
        dure_debug!("DrawerActor: switching to tab {:?}", tab);

        self.state.switch_tab(tab);

        let event = DrawerEvent::TabSwitched { tab };
        self.send_event(event).await
    }

    /// Load operation logs for a project
    async fn load_operations(&mut self, project_id: String, limit: i64) -> anyhow::Result<()> {
        dure_debug!(
            "DrawerActor: loading operations for project {} (limit {})",
            project_id,
            limit
        );

        self.state.set_loading(true);
        self.state.set_project(&project_id);

        let logs = self.repository.get_project_logs(&project_id, limit).await?;

        dure_info!(
            "DrawerActor: loaded {} operation logs for project {}",
            logs.len(),
            project_id
        );

        self.state.set_operations(logs.clone());

        let event = DrawerEvent::OperationsLoaded {
            project_id: project_id.clone(),
            logs,
        };
        self.send_event(event).await
    }

    /// Load stdout logs filtered by project
    async fn load_logs(&mut self, project_id: String, _limit: usize) -> anyhow::Result<()> {
        dure_debug!(
            "DrawerActor: requesting stdout logs for project {}",
            project_id
        );

        self.state.set_loading(true);
        self.state.set_project(&project_id);

        // Send command to LogActor to retrieve logs
        self.logs_tx
            .send(LogCommand::GetLogs {
                project_id: project_id.clone(),
            })
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send GetLogs command: {}", e))?;

        dure_debug!(
            "DrawerActor: GetLogs command sent for project {}",
            project_id
        );

        // LogActor will respond via ViewModelEvent::Log(LogEvent::LogsRetrieved)
        // which will be handled by ViewModel and forwarded back to drawer
        Ok(())
    }

    /// Refresh current tab data
    async fn refresh(&mut self) -> anyhow::Result<()> {
        dure_debug!("DrawerActor: refreshing current tab");

        if let Some(project_id) = self.state.project_id.clone() {
            match self.state.active_tab {
                super::DrawerTab::Status => {
                    // Status tab doesn't need refresh (static content)
                    Ok(())
                }
                super::DrawerTab::Logs => {
                    self.load_logs(project_id, 100).await
                }
                super::DrawerTab::Operations => {
                    self.load_operations(project_id, 100).await
                }
            }
        } else {
            dure_debug!("DrawerActor: no project selected, skipping refresh");
            Ok(())
        }
    }

    /// Send an event to the ViewModel
    async fn send_event(&self, event: DrawerEvent) -> anyhow::Result<()> {
        let vm_event = ViewModelEvent::Drawer(event);
        self.event_tx
            .send(vm_event)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send drawer event: {}", e))
    }

    /// Get current state (for testing)
    #[cfg(test)]
    pub fn state(&self) -> &DrawerState {
        &self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calc::db;
    use smol::channel::unbounded;
    use std::sync::atomic::{AtomicU32, Ordering};

    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    #[test]
    fn test_drawer_actor_switch_tab() {
        smol::block_on(async {
            let test_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
            let test_db = format!("test-drawer-actor-tab-{}-{}.db", std::process::id(), test_id);
            db::set_db_path(test_db);

            let (cmd_tx, cmd_rx) = unbounded();
            let (evt_tx, evt_rx) = unbounded();
            let (logs_tx, _logs_rx) = unbounded();

            let actor = DrawerActor::new(cmd_rx, evt_tx, logs_tx);

            // Spawn actor
            smol::spawn(async move {
                actor.run().await;
            })
            .detach();

            // Send switch tab command
            cmd_tx
                .send(DrawerCommand::SwitchTab {
                    tab: super::super::DrawerTab::Operations,
                })
                .await
                .unwrap();

            // Receive event
            let vm_event = evt_rx.recv().await.unwrap();
            match vm_event {
                ViewModelEvent::Drawer(DrawerEvent::TabSwitched { tab }) => {
                    assert_eq!(tab, super::super::DrawerTab::Operations);
                }
                _ => panic!("Expected TabSwitched event"),
            }
        })
    }

    #[test]
    fn test_drawer_actor_load_operations() {
        smol::block_on(async {
            let test_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
            let test_db = format!("test-drawer-actor-ops-{}-{}.db", std::process::id(), test_id);
            db::set_db_path(test_db);

            let (cmd_tx, cmd_rx) = unbounded();
            let (evt_tx, evt_rx) = unbounded();
            let (logs_tx, _logs_rx) = unbounded();

            let actor = DrawerActor::new(cmd_rx, evt_tx, logs_tx);

            // Spawn actor
            smol::spawn(async move {
                actor.run().await;
            })
            .detach();

            // Send load operations command
            cmd_tx
                .send(DrawerCommand::LoadOperations {
                    project_id: "test-project".to_string(),
                    limit: 10,
                })
                .await
                .unwrap();

            // Receive event
            let vm_event = evt_rx.recv().await.unwrap();
            match vm_event {
                ViewModelEvent::Drawer(DrawerEvent::OperationsLoaded { project_id, logs }) => {
                    assert_eq!(project_id, "test-project");
                    assert_eq!(logs.len(), 0); // Empty initially
                }
                _ => panic!("Expected OperationsLoaded event"),
            }
        })
    }
}
