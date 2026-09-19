use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}

pub struct LogBuffer {
    lines: VecDeque<String>,
    capacity: usize,
}

impl LogBuffer {
    pub fn new() -> Self {
        Self::with_capacity(1000)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            lines: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, line: String) {
        if self.lines.len() >= self.capacity {
            self.lines.pop_front();
        }
        self.lines.push_back(line);
    }

    pub fn get_lines(&self) -> Vec<String> {
        self.lines.iter().cloned().collect()
    }

    pub fn clear(&mut self) {
        self.lines.clear();
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}

impl Default for LogBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub enum LogCommand {
    AppendLog {
        project_id: String,
        level: LogLevel,
        message: String,
    },
    GetLogs {
        project_id: String,
    },
    ClearLogs {
        project_id: String,
    },
    ListProjects,
}

#[derive(Debug, Clone)]
pub enum LogEvent {
    LogsRetrieved {
        project_id: String,
        lines: Vec<String>,
    },
    ProjectList {
        project_ids: Vec<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_buffer_respects_capacity() {
        let mut buffer = LogBuffer::with_capacity(3);

        buffer.push("line 1".to_string());
        buffer.push("line 2".to_string());
        buffer.push("line 3".to_string());
        buffer.push("line 4".to_string()); // Should evict "line 1"

        let lines = buffer.get_lines();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "line 2");
        assert_eq!(lines[2], "line 4");
    }
}
