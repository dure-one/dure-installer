//! NS actor for DNS management

mod commands;
mod events;

pub use commands::NsCommand;
pub use events::NsEvent;

use smol::channel::{Receiver, Sender};
use crate::viewmodel::ViewModelEvent;

pub struct NsActor {
    command_rx: Receiver<NsCommand>,
    event_tx: Sender<ViewModelEvent>,
}

impl NsActor {
    pub fn new(command_rx: Receiver<NsCommand>, event_tx: Sender<ViewModelEvent>) -> Self {
        Self { command_rx, event_tx }
    }

    pub async fn run(mut self) {
        log::info!("NsActor started");
        loop {
            match self.command_rx.recv().await {
                Ok(_cmd) => {
                    // TODO: implement in Week 2
                }
                Err(_) => {
                    log::info!("NsActor: channel closed");
                    break;
                }
            }
        }
    }
}
