pub mod actor;
pub mod opslog_actor;
pub mod opslog_commands;
pub mod opslog_events;
pub mod types;

pub use actor::log_actor_loop;
pub use opslog_actor::OpsLogActor;
pub use opslog_commands::{LogStatus, LogsCommand, StatsPeriod};
pub use opslog_events::{LogStats, LogsEvent, OperationLog};
pub use types::{LogBuffer, LogCommand, LogEvent, LogLevel};

use smol::channel::Sender;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::sync::Mutex;

const PENDING_LOG_CAPACITY: usize = 1000;

struct LogRouter {
    sender: Option<Sender<LogCommand>>,
    pending: VecDeque<LogCommand>,
    generation: u64,
    current_generation: u64,
}

impl LogRouter {
    const fn new() -> Self {
        Self {
            sender: None,
            pending: VecDeque::new(),
            generation: 0,
            current_generation: 0,
        }
    }

    fn register(&mut self, sender: Sender<LogCommand>) -> u64 {
        self.generation = self.generation.wrapping_add(1);
        self.current_generation = self.generation;
        self.sender = Some(sender);
        self.flush_pending();
        self.current_generation
    }

    fn unregister(&mut self, generation: u64) {
        if self.current_generation == generation {
            self.sender = None;
        }
    }

    fn route(&mut self, command: LogCommand) {
        let Some(sender) = self.sender.clone() else {
            self.queue(command);
            return;
        };

        if let Err(error) = sender.try_send(command) {
            self.sender = None;
            self.queue(error.into_inner());
        }
    }

    fn flush_pending(&mut self) {
        let Some(sender) = self.sender.clone() else {
            return;
        };

        while let Some(command) = self.pending.pop_front() {
            if let Err(error) = sender.try_send(command) {
                self.pending.push_front(error.into_inner());
                self.sender = None;
                break;
            }
        }
    }

    fn queue(&mut self, command: LogCommand) {
        if self.pending.len() == PENDING_LOG_CAPACITY {
            self.pending.pop_front();
        }
        self.pending.push_back(command);
    }
}

static LOG_ROUTER: Mutex<LogRouter> = Mutex::new(LogRouter::new());

thread_local! {
    static HOST_CONTEXT: RefCell<Option<String>> = RefCell::new(None);
}

pub fn init_log_sender(sender: Sender<LogCommand>) -> u64 {
    LOG_ROUTER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .register(sender)
}

pub fn clear_log_sender(registration: u64) {
    LOG_ROUTER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .unregister(registration);
}

pub fn get_log_sender() -> Option<Sender<LogCommand>> {
    LOG_ROUTER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .sender
        .clone()
}

pub fn set_host_context(host_id: impl Into<String>) {
    let id = host_id.into();
    log::debug!("[HOST_CONTEXT] Setting host context to '{}'", id);
    HOST_CONTEXT.with(|ctx| {
        *ctx.borrow_mut() = Some(id);
    });
}

pub fn clear_host_context() {
    log::debug!("[HOST_CONTEXT] Clearing host context");
    HOST_CONTEXT.with(|ctx| {
        *ctx.borrow_mut() = None;
    });
}

pub fn get_host_context() -> Option<String> {
    let ctx = HOST_CONTEXT.with(|ctx| ctx.borrow().clone());
    log::debug!("[HOST_CONTEXT] Getting host context: {:?}", ctx);
    ctx
}

pub fn append_log(host_id_override: Option<String>, level: LogLevel, message: String) {
    let host_id = host_id_override
        .or_else(get_host_context)
        .unwrap_or_else(|| "unknown".to_string());
    let command = LogCommand::AppendLog {
        host_id,
        level,
        message,
    };

    LOG_ROUTER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .route(command);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn append_command(host: &str, message: &str) -> LogCommand {
        LogCommand::AppendLog {
            host_id: host.to_string(),
            level: LogLevel::Info,
            message: message.to_string(),
        }
    }

    #[test]
    fn queues_and_flushes_logs_in_fifo_order() {
        let mut router = LogRouter::new();
        router.route(append_command("first", "one"));
        router.route(append_command("second", "two"));

        let (tx, rx) = smol::channel::unbounded();
        router.register(tx);

        assert!(matches!(
            rx.try_recv().unwrap(),
            LogCommand::AppendLog { host_id, message, .. }
                if host_id == "first" && message == "one"
        ));
        assert!(matches!(
            rx.try_recv().unwrap(),
            LogCommand::AppendLog { host_id, message, .. }
                if host_id == "second" && message == "two"
        ));
    }

    #[test]
    fn routes_only_to_latest_sender() {
        let mut router = LogRouter::new();
        let (old_tx, old_rx) = smol::channel::unbounded();
        router.register(old_tx);

        let (new_tx, new_rx) = smol::channel::unbounded();
        router.register(new_tx);
        router.route(append_command("host", "new"));

        assert!(old_rx.try_recv().is_err());
        assert!(matches!(
            new_rx.try_recv().unwrap(),
            LogCommand::AppendLog { message, .. } if message == "new"
        ));
    }

    #[test]
    fn stale_registration_cannot_unregister_latest_sender() {
        let mut router = LogRouter::new();
        let (first_tx, _) = smol::channel::unbounded();
        let first_registration = router.register(first_tx);
        let (second_tx, second_rx) = smol::channel::unbounded();
        router.register(second_tx);

        router.unregister(first_registration);
        router.route(append_command("host", "still active"));

        assert!(second_rx.try_recv().is_ok());
    }

    #[test]
    fn pending_queue_drops_oldest_log_at_capacity() {
        let mut router = LogRouter::new();
        for index in 0..=PENDING_LOG_CAPACITY {
            router.route(append_command("host", &index.to_string()));
        }

        let (tx, rx) = smol::channel::unbounded();
        router.register(tx);

        assert!(matches!(
            rx.try_recv().unwrap(),
            LogCommand::AppendLog { message, .. } if message == "1"
        ));
    }
}
