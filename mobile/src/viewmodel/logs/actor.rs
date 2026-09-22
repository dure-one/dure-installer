use crate::viewmodel::logs::types::{LogBuffer, LogCommand, LogEvent, LogLevel};
use crate::viewmodel::ViewModelEvent;
use smol::channel::{Receiver, Sender};
use std::collections::HashMap;

pub async fn log_actor_loop(
    cmd_rx: Receiver<LogCommand>,
    event_tx: Sender<ViewModelEvent>,
) {
    let mut buffers: HashMap<String, LogBuffer> = HashMap::new();

    while let Ok(cmd) = cmd_rx.recv().await {
        match cmd {
            LogCommand::AppendLog { project_id, level, message } => {
                log::trace!("[LOG_ACTOR] Received AppendLog - project_id='{}', level={:?}, msg='{}'", project_id, level, message);
                let buffer = buffers.entry(project_id.clone()).or_insert_with(LogBuffer::new);

                // Add timestamp (HH:MM:SS)
                let now = chrono::Local::now();
                let timestamp = now.format("%H:%M:%S");
                let formatted_line = format!("{} [{}] {}", timestamp, level.as_str(), message);

                buffer.push(formatted_line);
                log::trace!("[LOG_ACTOR] ✓ Stored log for project '{}', buffer now has {} lines", project_id, buffer.len());
                log::trace!("[LOG_ACTOR] Current buffers: {:?}", buffers.keys().collect::<Vec<_>>());
            }
            LogCommand::GetLogs { project_id } => {
                log::trace!("[LOG_ACTOR] Received GetLogs - project_id='{}'", project_id);
                log::trace!("[LOG_ACTOR] Available buffers: {:?}", buffers.keys().collect::<Vec<_>>());

                let lines = buffers
                    .get(&project_id)
                    .map(|b| b.get_lines())
                    .unwrap_or_default();

                log::trace!("[LOG_ACTOR] Returning {} lines for project '{}'", lines.len(), project_id);

                let _ = event_tx.send(ViewModelEvent::Logs(LogEvent::LogsRetrieved {
                    project_id,
                    lines,
                })).await;
            }
            LogCommand::ClearLogs { project_id } => {
                if let Some(buffer) = buffers.get_mut(&project_id) {
                    buffer.clear();
                }
            }
            LogCommand::ListProjects => {
                let project_ids: Vec<String> = buffers.keys().cloned().collect();
                let _ = event_tx.send(ViewModelEvent::Logs(LogEvent::ProjectList { project_ids })).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_actor_append_and_retrieve() {
        smol::block_on(async {
            let (cmd_tx, cmd_rx) = smol::channel::unbounded();
            let (event_tx, event_rx) = smol::channel::unbounded();

            // Spawn actor
            smol::spawn(log_actor_loop(cmd_rx, event_tx)).detach();

            // Append log
            cmd_tx.send(LogCommand::AppendLog {
                project_id: "test-project".to_string(),
                level: LogLevel::Info,
                message: "test message".to_string(),
            }).await.unwrap();

            // Small delay for processing
            smol::Timer::after(std::time::Duration::from_millis(10)).await;

            // Retrieve logs
            cmd_tx.send(LogCommand::GetLogs {
                project_id: "test-project".to_string(),
            }).await.unwrap();

            // Check event
            let event = event_rx.recv().await.unwrap();
            match event {
                ViewModelEvent::Logs(LogEvent::LogsRetrieved { project_id, lines }) => {
                    assert_eq!(project_id, "test-project");
                    assert_eq!(lines.len(), 1);
                    assert!(lines[0].contains("test message"));
                }
                _ => panic!("Expected Logs(LogsRetrieved) event"),
            }
        });
    }
}
