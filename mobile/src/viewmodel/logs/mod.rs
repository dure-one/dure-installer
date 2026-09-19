pub mod types;
pub mod actor;

pub use types::{LogBuffer, LogCommand, LogEvent, LogLevel};
pub use actor::log_actor_loop;

use smol::channel::Sender;
use std::sync::OnceLock;

static LOG_SENDER: OnceLock<Sender<LogCommand>> = OnceLock::new();

pub fn init_log_sender(sender: Sender<LogCommand>) {
    LOG_SENDER.set(sender).ok();
}

pub fn get_log_sender() -> Option<&'static Sender<LogCommand>> {
    LOG_SENDER.get()
}

pub fn append_log(project_id: impl Into<String>, level: LogLevel, message: impl Into<String>) {
    let pid = project_id.into();
    let msg = message.into();

    if let Some(sender) = get_log_sender() {
        log::debug!("[APPEND_LOG] Sending AppendLog command - project_id='{}', level={:?}", pid, level);
        let cmd = LogCommand::AppendLog {
            project_id: pid,
            level,
            message: msg,
        };
        match sender.try_send(cmd) {
            Ok(_) => log::debug!("[APPEND_LOG] ✓ Command sent successfully"),
            Err(e) => log::error!("[APPEND_LOG] ✗ Failed to send command: {}", e),
        }
    } else {
        log::warn!("[APPEND_LOG] ✗ Log sender not initialized!");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_log_sender_and_append() {
        smol::block_on(async {
            let (tx, rx) = smol::channel::unbounded();
            init_log_sender(tx);

            append_log("test-proj", LogLevel::Info, "test msg");

            // Small delay for message to be sent
            smol::Timer::after(std::time::Duration::from_millis(10)).await;

            let cmd = rx.try_recv().unwrap();
            match cmd {
                LogCommand::AppendLog { project_id, level, message } => {
                    assert_eq!(project_id, "test-proj");
                    assert_eq!(level, LogLevel::Info);
                    assert_eq!(message, "test msg");
                }
                _ => panic!("Expected AppendLog command"),
            }
        });
    }
}
