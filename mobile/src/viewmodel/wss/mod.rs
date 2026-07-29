//! WSS actor (stub - not implemented yet)

mod commands;
mod events;

pub use commands::WssCommand;
pub use events::WssEvent;

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use crate::viewmodel::ViewModelEvent;
use smol::channel::{Receiver, Sender};

pub struct WssActor {
    command_rx: Receiver<WssCommand>,
    event_tx: Sender<ViewModelEvent>,
}

impl WssActor {
    pub fn new(command_rx: Receiver<WssCommand>, event_tx: Sender<ViewModelEvent>) -> Self {
        Self {
            command_rx,
            event_tx,
        }
    }

    pub async fn run(mut self) {
        dure_info!("WssActor stub - not implemented yet");
        loop {
            match self.command_rx.recv().await {
                Ok(cmd) => {
                    dure_warn!(
                        "WssActor received command but is not implemented: {:?}",
                        cmd
                    );
                    let _ = self
                        .event_tx
                        .send(ViewModelEvent::Wss(WssEvent::Error {
                            operation: format!("{:?}", cmd),
                            error: "WSS not implemented yet".to_string(),
                        }))
                        .await;
                }
                Err(_) => {
                    dure_info!("WssActor: channel closed");
                    break;
                }
            }
        }
    }
}
