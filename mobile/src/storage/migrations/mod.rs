//! Database migrations module
//! Migrations are automatically embedded and run by diesel_migrations

#[cfg(test)]
mod migration_tests {
    use crate::calc::db;
    use diesel::prelude::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    #[test]
    fn test_migration_backfills_platform_operations() {
        // Setup unique test database
        let test_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let test_db = format!(
            "test-migration-backfill-{}-{}.db",
            std::process::id(),
            test_id
        );
        db::set_db_path(test_db.clone());

        {
            let mut conn = db::establish_connection();

            // Initialize old table without migration columns
            diesel::sql_query("DROP TABLE IF EXISTS operation_logs")
                .execute(&mut conn)
                .ok();

            diesel::sql_query(
                "CREATE TABLE operation_logs (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    project_id TEXT NOT NULL,
                    operation_type TEXT NOT NULL,
                    external_system TEXT NOT NULL,
                    status TEXT NOT NULL,
                    started_at INTEGER NOT NULL,
                    completed_at INTEGER,
                    error_message TEXT,
                    details TEXT
                )"
            )
            .execute(&mut conn)
            .expect("Failed to create operation_logs table");

            // Insert old-schema operation
            diesel::sql_query(
                "INSERT INTO operation_logs (project_id, operation_type, external_system, status, started_at)
                 VALUES ('test-project', 'test_op', 'gcp', 'success', 1234567890)"
            )
            .execute(&mut conn)
            .expect("Failed to insert test operation");

            // Manually run migration SQL
            diesel::sql_query(
                "ALTER TABLE operation_logs ADD COLUMN operation_source TEXT"
            )
            .execute(&mut conn)
            .expect("Failed to add operation_source column");

            diesel::sql_query(
                "ALTER TABLE operation_logs ADD COLUMN relevant_id TEXT"
            )
            .execute(&mut conn)
            .expect("Failed to add relevant_id column");

            // Backfill existing records
            diesel::sql_query(
                "UPDATE operation_logs
                 SET operation_source = 'platform',
                     relevant_id = project_id
                 WHERE operation_source IS NULL"
            )
            .execute(&mut conn)
            .expect("Failed to backfill operations");

            // Verify backfill worked
            #[derive(QueryableByName)]
            struct Row {
                #[diesel(sql_type = diesel::sql_types::BigInt)]
                id: i64,
                #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
                operation_source: Option<String>,
                #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
                relevant_id: Option<String>,
            }

            let ops: Vec<Row> = diesel::sql_query(
                "SELECT id, operation_source, relevant_id
                 FROM operation_logs
                 WHERE operation_source = 'platform' AND relevant_id = 'test-project'"
            )
            .load(&mut conn)
            .expect("Failed to load operations");

            assert_eq!(ops.len(), 1, "Expected 1 operation after backfill");
            assert_eq!(ops[0].operation_source, Some("platform".to_string()));
            assert_eq!(ops[0].relevant_id, Some("test-project".to_string()));
        }

        // Cleanup
        let _ = std::fs::remove_file(&test_db);
    }
}
