CREATE TABLE operation_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id TEXT NOT NULL,
    operation_type TEXT NOT NULL,
    external_system TEXT NOT NULL,
    status TEXT NOT NULL,
    started_at INTEGER NOT NULL,
    completed_at INTEGER,
    error_message TEXT,
    details TEXT
);
CREATE INDEX idx_operation_logs_project_id ON operation_logs(project_id);
CREATE INDEX idx_operation_logs_started_at ON operation_logs(started_at DESC);
CREATE INDEX idx_operation_logs_project_time ON operation_logs(project_id, started_at DESC);
