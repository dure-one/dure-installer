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
            LogCommand::AppendLog { host_id, level, message } => {
                log::debug!("[LOG_ACTOR] ===== AppendLog START =====");
                log::debug!("[LOG_ACTOR] Received AppendLog - host_id='{}', level={:?}, msg='{}'", host_id, level, message);
                let buffer = buffers.entry(host_id.clone()).or_insert_with(LogBuffer::new);

                // Add timestamp (HH:MM:SS)
                let now = chrono::Local::now();
                let timestamp = now.format("%H:%M:%S");
                let formatted_line = format!("{} [{}] {}", timestamp, level.as_str(), message);

                buffer.push(formatted_line.clone());
                log::debug!("[LOG_ACTOR] ✓ Stored log for host '{}', buffer now has {} lines", host_id, buffer.len());
                log::debug!("[LOG_ACTOR] Stored line: '{}'", formatted_line);
                log::debug!("[LOG_ACTOR] Current buffer keys: {:?}", buffers.keys().collect::<Vec<_>>());
                log::debug!("[LOG_ACTOR] ===== AppendLog END =====");
            }
            LogCommand::GetLogs { host_id } => {
                log::debug!("[LOG_ACTOR] ===== GetLogs START =====");
                log::debug!("[LOG_ACTOR] Received GetLogs - host_id='{}'", host_id);
                log::debug!("[LOG_ACTOR] Available buffer keys: {:?}", buffers.keys().collect::<Vec<_>>());

                let lines = buffers
                    .get(&host_id)
                    .map(|b| {
                        let lines = b.get_lines();
                        log::debug!("[LOG_ACTOR] Found buffer for '{}' with {} lines", host_id, lines.len());
                        lines
                    })
                    .unwrap_or_else(|| {
                        log::debug!("[LOG_ACTOR] No buffer found for '{}'", host_id);
                        vec![]
                    });

                log::debug!("[LOG_ACTOR] Returning {} lines for host '{}'", lines.len(), host_id);
                if !lines.is_empty() {
                    log::debug!("[LOG_ACTOR] First line: '{}'", lines[0]);
                }

                let _ = event_tx.send(ViewModelEvent::Logs(LogEvent::LogsRetrieved {
                    filter_id: host_id.clone(),
                    lines: lines.clone(),
                })).await;

                log::debug!("[LOG_ACTOR] ===== GetLogs END =====");
            }
            LogCommand::GetProjectLogs { project_id } => {
                log::trace!("[LOG_ACTOR] Received GetProjectLogs - project_id='{}'", project_id);
                log::trace!("[LOG_ACTOR] Available buffers: {:?}", buffers.keys().collect::<Vec<_>>());

                // Aggregate logs from all hosts matching the project_id prefix or pattern
                let mut all_lines = Vec::new();
                for (host_id, buffer) in &buffers {
                    if host_id.starts_with(&project_id) || host_id.contains(&project_id) {
                        all_lines.extend(buffer.get_lines());
                    }
                }

                // Sort by timestamp (already in HH:MM:SS format at start of each line)
                all_lines.sort();

                log::trace!("[LOG_ACTOR] Returning {} lines for project '{}'", all_lines.len(), project_id);

                let _ = event_tx.send(ViewModelEvent::Logs(LogEvent::LogsRetrieved {
                    filter_id: project_id,
                    lines: all_lines,
                })).await;
            }
            LogCommand::ClearLogs { host_id } => {
                if let Some(buffer) = buffers.get_mut(&host_id) {
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
                host_id: "test-host".to_string(),
                level: LogLevel::Info,
                message: "test message".to_string(),
            }).await.unwrap();

            // Small delay for processing
            smol::Timer::after(std::time::Duration::from_millis(10)).await;

            // Retrieve logs
            cmd_tx.send(LogCommand::GetLogs {
                host_id: "test-host".to_string(),
            }).await.unwrap();

            // Check event
            let event = event_rx.recv().await.unwrap();
            match event {
                ViewModelEvent::Logs(LogEvent::LogsRetrieved { filter_id, lines }) => {
                    assert_eq!(filter_id, "test-host");
                    assert_eq!(lines.len(), 1);
                    assert!(lines[0].contains("test message"));
                }
                _ => panic!("Expected Logs(LogsRetrieved) event"),
            }
        });
    }
}
