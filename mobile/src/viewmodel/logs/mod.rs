pub mod types;
pub mod actor;

pub use types::{LogBuffer, LogCommand, LogEvent, LogLevel};
pub use actor::log_actor_loop;

use smol::channel::Sender;
use std::sync::RwLock;

static LOG_SENDER: RwLock<Option<Sender<LogCommand>>> = RwLock::new(None);

pub fn init_log_sender(sender: Sender<LogCommand>) {
    if let Ok(mut guard) = LOG_SENDER.write() {
        *guard = Some(sender);
        log::debug!("[LOG_INIT] Log sender initialized/updated");
    } else {
        log::error!("[LOG_INIT] Failed to acquire write lock for log sender");
    }
}

pub fn get_log_sender() -> Option<Sender<LogCommand>> {
    LOG_SENDER.read().ok().and_then(|guard| guard.clone())
}

pub fn append_log(project_id: impl Into<String>, level: LogLevel, message: impl Into<String>) {
    let pid = project_id.into();
    let msg = message.into();

    if let Some(sender) = get_log_sender() {
        log::trace!("[APPEND_LOG] Sending AppendLog command - project_id='{}', level={:?}", pid, level);
        let cmd = LogCommand::AppendLog {
            project_id: pid,
            level,
            message: msg,
        };
        match sender.try_send(cmd) {
            Ok(_) => log::trace!("[APPEND_LOG] ✓ Command sent successfully"),
            Err(e) => log::error!("[APPEND_LOG] ✗ Failed to send command: {}", e),
        }
    } else {
        log::trace!("[APPEND_LOG] Log sender not initialized yet (profile not loaded)");
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

    #[test]
    fn test_log_sender_reinitialization() {
        smol::block_on(async {
            // First initialization
            let (tx1, rx1) = smol::channel::unbounded();
            init_log_sender(tx1);

            append_log("test-1", LogLevel::Info, "msg 1");
            smol::Timer::after(std::time::Duration::from_millis(10)).await;

            let cmd1 = rx1.try_recv().unwrap();
            assert!(matches!(cmd1, LogCommand::AppendLog { .. }));

            // Re-initialization (simulating profile switch)
            let (tx2, rx2) = smol::channel::unbounded();
            init_log_sender(tx2);

            append_log("test-2", LogLevel::Info, "msg 2");
            smol::Timer::after(std::time::Duration::from_millis(10)).await;

            // Should receive on new channel
            let cmd2 = rx2.try_recv().unwrap();
            match cmd2 {
                LogCommand::AppendLog { project_id, .. } => {
                    assert_eq!(project_id, "test-2");
                }
                _ => panic!("Expected AppendLog on new channel"),
            }

            // Old channel should be empty
            assert!(rx1.try_recv().is_err());
        });
    }
}
