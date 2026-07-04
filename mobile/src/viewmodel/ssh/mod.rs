//! SSH actor for host and container management

mod commands;
mod events;

pub use commands::SshCommand;
pub use events::SshEvent;

use smol::channel::{Receiver, Sender};
use crate::viewmodel::ViewModelEvent;

pub struct SshActor {
    command_rx: Receiver<SshCommand>,
    event_tx: Sender<ViewModelEvent>,
}

impl SshActor {
    pub fn new(command_rx: Receiver<SshCommand>, event_tx: Sender<ViewModelEvent>) -> Self {
        Self { command_rx, event_tx }
    }

    pub async fn run(mut self) {
        log::info!("SshActor started");
        loop {
            match self.command_rx.recv().await {
                Ok(_cmd) => {
                    // TODO: implement in Week 2
                }
                Err(_) => {
                    log::info!("SshActor: channel closed");
                    break;
                }
            }
        }
    }
}
