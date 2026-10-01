-- Add operation_source and relevant_id columns to operation_logs (nullable initially)
ALTER TABLE operation_logs ADD COLUMN operation_source TEXT;
ALTER TABLE operation_logs ADD COLUMN relevant_id TEXT;

-- Backfill existing records with platform defaults (handle both NULL and empty string)
UPDATE operation_logs
SET operation_source = 'platform',
    relevant_id = project_id
WHERE operation_source IS NULL OR operation_source = '';

-- Create composite index for efficient filtering
CREATE INDEX IF NOT EXISTS idx_operation_logs_source_id
ON operation_logs(operation_source, relevant_id);
