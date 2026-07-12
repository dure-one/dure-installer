//! DeltaChat actor implementation

use crate::viewmodel::common::ViewModelEvent;
use super::{DeltaChatCommand, DeltaChatEvent};
use smol::channel::{Receiver, Sender};
use std::path::PathBuf;

pub struct DeltaChatActor {
    command_rx: Receiver<DeltaChatCommand>,
    event_tx: Sender<ViewModelEvent>,
    context: Option<deltachat::Context>,
    tokio_runtime: tokio::runtime::Runtime,
    database_path: PathBuf,
    is_configured: bool,
    is_connected: bool,
}

impl DeltaChatActor {
    pub fn new(
        command_rx: Receiver<DeltaChatCommand>,
        event_tx: Sender<ViewModelEvent>,
        database_path: PathBuf,
    ) -> Self {
        let tokio_runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("deltachat-tokio")
            .build()
            .expect("Failed to create tokio runtime");

        Self {
            command_rx,
            event_tx,
            context: None,
            tokio_runtime,
            database_path,
            is_configured: false,
            is_connected: false,
        }
    }

    pub async fn run(mut self) {
        log::info!("DeltaChatActor started");

        while let Ok(cmd) = self.command_rx.recv().await {
            log::debug!("DeltaChatActor received command: {:?}", cmd);

            let result = smol::unblock({
                let rt = &self.tokio_runtime;
                move || {
                    rt.block_on(async {
                        // TODO: handle commands
                        Ok::<(), String>(())
                    })
                }
            }).await;

            if let Err(e) = result {
                log::error!("DeltaChat command failed: {}", e);
                self.emit_event(DeltaChatEvent::Error {
                    operation: "command".to_string(),
                    error: e,
                });
            }
        }

        log::info!("DeltaChatActor stopped");
    }

    fn emit_event(&self, event: DeltaChatEvent) {
        let event_tx = self.event_tx.clone();
        smol::block_on(async move {
            let _ = event_tx.send(ViewModelEvent::DeltaChat(event)).await;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smol::channel::unbounded;
    use std::path::PathBuf;

    #[test]
    fn test_actor_creation() {
        let (cmd_tx, cmd_rx) = unbounded();
        let (event_tx, event_rx) = unbounded();
        let db_path = PathBuf::from("/tmp/test.db");

        let actor = DeltaChatActor::new(cmd_rx, event_tx, db_path.clone());

        assert_eq!(actor.database_path, db_path);
        assert!(!actor.is_configured);
        assert!(!actor.is_connected);
        assert!(actor.context.is_none());
    }

    #[test]
    fn test_actor_receives_commands() {
        smol::block_on(async {
            let (cmd_tx, cmd_rx) = unbounded();
            let (event_tx, _event_rx) = unbounded();

            cmd_tx.send(DeltaChatCommand::GetConnectionStatus).await.unwrap();

            let received = cmd_rx.recv().await.unwrap();
            assert!(matches!(received, DeltaChatCommand::GetConnectionStatus));
        });
    }
}
