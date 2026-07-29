//! Unified database connection and migration management
//!
//! This module provides a centralized database management system using Diesel ORM.
//! It supports multiple backends:
//! - SQLite for desktop and Android (with WAL mode and connection pooling)
//! - SQLite for WASM
//! - PostgreSQL for desktop (optional)
//!
//! All migrations are embedded and run automatically on connection establishment.

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use diesel::prelude::*;
use std::sync::Mutex;

use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

#[cfg(target_family = "wasm")]
use std::sync::Once;

#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;

// Embed migrations at compile time
const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// WASM VFS selector: 0 = Memory, 1 = OPFS SAH Pool, 2 = Relaxed IndexedDB
#[cfg(target_family = "wasm")]
static VFS: Mutex<(i32, Once)> = Mutex::new((0, Once::new()));


#[cfg(not(target_family = "wasm"))]
static DB_PATH: Mutex<Option<String>> = Mutex::new(None);
#[cfg(not(target_family = "wasm"))]
static MIGRATIONS_RAN: Mutex<bool> = Mutex::new(false);

/// Set the database path to use for connections
#[cfg(not(target_family = "wasm"))]
pub fn set_db_path(path: String) {
    let mut db_path = DB_PATH.lock().unwrap();
    *db_path = Some(path);
    // Reset migrations flag when database path changes
    let mut migrations_ran = MIGRATIONS_RAN.lock().unwrap();
    *migrations_ran = false;
}

/// Get the current database path
#[cfg(not(target_family = "wasm"))]
pub fn get_db_path() -> String {
    let db_path = DB_PATH.lock().unwrap();
    db_path.as_deref().unwrap_or("dure.db").to_string()
}

#[cfg(feature = "postgres")]
pub mod postgres {
    use diesel::prelude::*;
    use dotenvy::dotenv;
    use std::env;

    pub fn establish_connection() -> PgConnection {
        dotenv().ok();

        let database_url = env::var("PG_DATABASE_URL")
            .or_else(|_| env::var("DATABASE_URL"))
            .expect("DATABASE_URL must be set for PostgreSQL");

        PgConnection::establish(&database_url)
            .unwrap_or_else(|e| panic!("Failed to connect to PostgreSQL: {}", e))
    }
}

#[cfg(not(feature = "postgres"))]
pub mod sqlite {
    use super::*;
    use diesel::prelude::*;

    pub fn establish_connection() -> SqliteConnection {
        #[cfg(target_family = "wasm")]
        {
            let (vfs, once) = &*VFS.lock().unwrap();
            let url = match vfs {
                0 => "dure.db",
                1 => "file:dure.db?vfs=opfs-sahpool",
                2 => "file:dure.db?vfs=relaxed-idb",
                _ => unreachable!(),
            };
            let mut conn = SqliteConnection::establish(url)
                .unwrap_or_else(|_| panic!("Error connecting to {url}"));
            once.call_once(|| {
                conn.run_pending_migrations(MIGRATIONS).unwrap();
                dure_info!("WASM database migrations completed successfully");
            });
            return conn;
        }

        #[cfg(not(target_family = "wasm"))]
        {
            // Get the database path from the static or use default
            let db_path = DB_PATH.lock().unwrap();
            let url = db_path.as_deref().unwrap_or("dure.db").to_string();

            let mut conn = SqliteConnection::establish(&url)
                .unwrap_or_else(|_| panic!("Error connecting to {}", url));

            // Enable WAL mode for better concurrent access
            diesel::sql_query("PRAGMA journal_mode=WAL;")
                .execute(&mut conn)
                .ok();

            // Set busy timeout to wait up to 30 seconds when database is locked
            // This prevents "database is locked" errors during concurrent access
            diesel::sql_query("PRAGMA busy_timeout=30000;")
                .execute(&mut conn)
                .ok();

            // Set cache size to 8MB for better performance
            diesel::sql_query("PRAGMA cache_size=-8000;")
                .execute(&mut conn)
                .ok();

            // Enable foreign keys
            diesel::sql_query("PRAGMA foreign_keys=ON;")
                .execute(&mut conn)
                .ok();

            // Run migrations only once per database
            let mut migrations_ran = MIGRATIONS_RAN.lock().unwrap();
            if !*migrations_ran {
                conn.run_pending_migrations(MIGRATIONS)
                    .expect("Failed to run database migrations");
                *migrations_ran = true;
                dure_info!("Database migrations completed successfully");
            }

            conn
        }
    }
}

/// Install the OPFS Synchronous Access Handle Pool VFS for persistent storage.
#[cfg(all(target_family = "wasm", target_os = "unknown"))]
#[wasm_bindgen(js_name = installOpfsSahpool)]
pub async fn install_opfs_sahpool() {
    use sqlite_wasm_vfs::sahpool::{OpfsSAHPoolCfg, install};
    install::<sqlite_wasm_rs::WasmOsCallback>(&OpfsSAHPoolCfg::default(), false)
        .await
        .unwrap();
}

/// Install the Relaxed IndexedDB VFS for persistent storage.
#[cfg(all(target_family = "wasm", target_os = "unknown"))]
#[wasm_bindgen(js_name = installRelaxedIdb)]
pub async fn install_relaxed_idb() {
    use sqlite_wasm_vfs::relaxed_idb::{RelaxedIdbCfg, install};
    install::<sqlite_wasm_rs::WasmOsCallback>(&RelaxedIdbCfg::default(), false)
        .await
        .unwrap();
}

/// Switch the active VFS: 0 = Memory, 1 = OPFS SAH Pool, 2 = Relaxed IndexedDB.
#[cfg(target_family = "wasm")]
#[wasm_bindgen(js_name = switchVfs)]
pub fn switch_vfs(id: i32) {
    use std::sync::Once;
    *VFS.lock().unwrap() = (id, Once::new());
}

/// Establish a database connection (convenience function)
#[cfg(not(feature = "postgres"))]
pub fn establish_connection() -> SqliteConnection {
    sqlite::establish_connection()
}

#[cfg(feature = "postgres")]
pub fn establish_connection() -> PgConnection {
    postgres::establish_connection()
}

/// Establish a database connection with Result return type
#[cfg(not(feature = "postgres"))]
pub fn establish_connection_result() -> Result<SqliteConnection, anyhow::Error> {
    #[cfg(target_family = "wasm")]
    {
        let (vfs, once) = &*VFS.lock().unwrap();
        let url = match vfs {
            0 => "dure.db",
            1 => "file:dure.db?vfs=opfs-sahpool",
            2 => "file:dure.db?vfs=relaxed-idb",
            _ => unreachable!(),
        };
        let mut conn = SqliteConnection::establish(url)
            .map_err(|e| anyhow::anyhow!("Failed to connect to {}: {}", url, e))?;
        once.call_once(|| {
            conn.run_pending_migrations(MIGRATIONS).unwrap();
            console_log!("WASM database migrations completed successfully");
        });
        return Ok(conn);
    }

    #[cfg(not(target_family = "wasm"))]
    {
        let db_path = DB_PATH.lock().unwrap();
        let url = db_path.as_deref().unwrap_or("dure.db").to_string();

        let mut conn = SqliteConnection::establish(&url)
            .map_err(|e| anyhow::anyhow!("Failed to connect to {}: {}", url, e))?;

        // Enable WAL mode and pragmas
        diesel::sql_query("PRAGMA journal_mode=WAL;")
            .execute(&mut conn)
            .ok();
        diesel::sql_query("PRAGMA busy_timeout=30000;")
            .execute(&mut conn)
            .ok();
        diesel::sql_query("PRAGMA cache_size=-8000;")
            .execute(&mut conn)
            .ok();
        diesel::sql_query("PRAGMA foreign_keys=ON;")
            .execute(&mut conn)
            .ok();

        // Run migrations only once per database
        let mut migrations_ran = MIGRATIONS_RAN.lock().unwrap();
        if !*migrations_ran {
            conn.run_pending_migrations(MIGRATIONS)
                .map_err(|e| anyhow::anyhow!("Failed to run database migrations: {}", e))?;
            *migrations_ran = true;
            dure_info!("Database migrations completed successfully");
        }

        Ok(conn)
    }
}

#[cfg(feature = "postgres")]
pub fn establish_connection_result() -> Result<PgConnection, anyhow::Error> {
    use dotenvy::dotenv;
    use std::env;

    dotenv().ok();

    let database_url = env::var("PG_DATABASE_URL")
        .or_else(|_| env::var("DATABASE_URL"))
        .map_err(|_| anyhow::anyhow!("DATABASE_URL must be set for PostgreSQL"))?;

    PgConnection::establish(&database_url)
        .map_err(|e| anyhow::anyhow!("Failed to connect to PostgreSQL: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(target_family = "wasm"))]
    #[test]
    fn test_db_path() {
        set_db_path("test-dure.db".to_string());
        assert_eq!(get_db_path(), "test-dure.db");
        // Reset to default
        set_db_path("dure.db".to_string());
    }

    #[cfg(not(any(feature = "postgres", target_family = "wasm")))]
    #[test]
    fn test_establish_connection() {
        use tempfile::tempdir;
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().join("test-dure-connection.db");
        set_db_path(path.to_str().unwrap().to_string());

        let conn = establish_connection();
        // If we got here, connection was established successfully
        drop(conn);

        // Reset to default
        set_db_path("dure.db".to_string());
    }
}
