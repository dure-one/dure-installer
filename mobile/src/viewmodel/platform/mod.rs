//! Platform actor for GCP operations

mod commands;
mod events;

pub use commands::PlatformCommand;
pub use events::PlatformEvent;

use smol::channel::{Receiver, Sender};
use crate::viewmodel::ViewModelEvent;

pub struct PlatformActor {
    command_rx: Receiver<PlatformCommand>,
    event_tx: Sender<ViewModelEvent>,
}

impl PlatformActor {
    pub fn new(command_rx: Receiver<PlatformCommand>, event_tx: Sender<ViewModelEvent>) -> Self {
        Self { command_rx, event_tx }
    }

    pub async fn run(mut self) {
        log::info!("PlatformActor started");
        loop {
            match self.command_rx.recv().await {
                Ok(_cmd) => {
                    // TODO: implement in Week 2
                }
                Err(_) => {
                    log::info!("PlatformActor: channel closed");
                    break;
                }
            }
        }
    }
}
