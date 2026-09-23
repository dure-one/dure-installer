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
static DB_ENCRYPTION_KEY: Mutex<Option<String>> = Mutex::new(None);
#[cfg(not(target_family = "wasm"))]
static MIGRATIONS_RAN: Mutex<bool> = Mutex::new(false);
#[cfg(not(target_family = "wasm"))]
static ENCRYPTION_LOGGED: Mutex<bool> = Mutex::new(false);

/// Set the database path to use for connections
#[cfg(not(target_family = "wasm"))]
pub fn set_db_path(path: String) {
    let mut db_path = DB_PATH.lock().expect("DB_PATH lock poisoned");
    *db_path = Some(path);
    // Reset flags when database path changes
    let mut migrations_ran = MIGRATIONS_RAN.lock().expect("MIGRATIONS_RAN lock poisoned");
    *migrations_ran = false;
    let mut encryption_logged = ENCRYPTION_LOGGED.lock().expect("ENCRYPTION_LOGGED lock poisoned");
    *encryption_logged = false;
}

/// Get the current database path
#[cfg(not(target_family = "wasm"))]
pub fn get_db_path() -> String {
    let db_path = DB_PATH.lock().expect("DB_PATH lock poisoned");
    db_path.as_deref().unwrap_or("dure.db").to_string()
}

/// Set the database encryption key
#[cfg(not(target_family = "wasm"))]
pub fn set_db_encryption_key(key: String) {
    let mut db_key = DB_ENCRYPTION_KEY.lock().expect("DB_ENCRYPTION_KEY lock poisoned");
    *db_key = Some(key);
}

/// Clear the database encryption key (on logout)
#[cfg(not(target_family = "wasm"))]
pub fn clear_db_encryption_key() {
    let mut db_key = DB_ENCRYPTION_KEY.lock().expect("DB_ENCRYPTION_KEY lock poisoned");
    *db_key = None;
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
            let (vfs, once) = &*VFS.lock().expect("VFS lock poisoned");
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
            let db_path = DB_PATH.lock().expect("DB_PATH lock poisoned");
            let url = db_path.as_deref().unwrap_or("dure.db").to_string();

            // Ensure parent directory exists before connecting
            if let Some(parent) = std::path::Path::new(&url).parent() {
                // Only create directory if path has a real parent (not empty/current dir)
                if !parent.as_os_str().is_empty() && parent != std::path::Path::new(".") {
                    if !parent.exists() {
                        dure_warn!("Database parent directory doesn't exist: {}", parent.display());
                        dure_warn!("This usually means the profile was deleted. Attempting to create directory...");
                        std::fs::create_dir_all(parent)
                            .unwrap_or_else(|e| panic!("Failed to create database directory {}: {}", parent.display(), e));
                    }
                }
            }

            let mut conn = SqliteConnection::establish(&url)
                .unwrap_or_else(|e| panic!("Error connecting to {}: {}", url, e));

            // CRITICAL: Set encryption BEFORE any other operations
            let db_key = DB_ENCRYPTION_KEY.lock().expect("DB_ENCRYPTION_KEY lock poisoned");
            if let Some(key) = db_key.as_ref() {
                // Set cipher (AES-256-CBC for hardware acceleration)
                diesel::sql_query("PRAGMA cipher = 'aes256cbc';")
                    .execute(&mut conn)
                    .expect("Failed to set cipher");

                // Set encryption key
                diesel::sql_query(format!("PRAGMA key = '{}';", key))
                    .execute(&mut conn)
                    .expect("Failed to set encryption key");

                // Log encryption only once
                let mut encryption_logged = ENCRYPTION_LOGGED.lock().unwrap();
                if !*encryption_logged {
                    dure_info!("SQLite encryption enabled (AES-256-CBC)");
                    *encryption_logged = true;
                }
            }
            drop(db_key); // Release lock

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

    #[cfg(not(any(feature = "postgres", target_family = "wasm")))]
    #[test]
    fn test_sqlite_encryption() {
        use tempfile::tempdir;
        use diesel::prelude::*;

        // NOTE: This test verifies the encryption *setup* works correctly.
        // Full encryption verification requires libsqlite3-hotbundle with encryption compiled in.

        // Arrange: Create temporary database and encryption key
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test-encrypted.db");
        set_db_path(db_path.to_str().unwrap().to_string());

        // Generate a test encryption key (32 bytes base64-encoded)
        use rand::RngCore;
        let mut key_bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key_bytes);
        let encryption_key = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, key_bytes);

        // Act 1: Test encryption key management
        set_db_encryption_key(encryption_key.clone());

        // Verify key is set
        let db_key_guard = DB_ENCRYPTION_KEY.lock().unwrap();
        assert!(db_key_guard.is_some(), "Encryption key should be set");
        assert_eq!(db_key_guard.as_ref().unwrap(), &encryption_key, "Key mismatch");
        drop(db_key_guard);

        // Act 2: Establish connection with encryption (PRAGMA commands executed)
        {
            let mut conn = establish_connection();

            // Create test table
            diesel::sql_query("CREATE TABLE test_secrets (id INTEGER PRIMARY KEY, data TEXT)")
                .execute(&mut conn)
                .expect("Failed to create table");

            // Insert test data
            diesel::sql_query("INSERT INTO test_secrets (id, data) VALUES (1, 'test data')")
                .execute(&mut conn)
                .expect("Failed to insert data");

            // Verify data can be queried
            let result = diesel::sql_query("SELECT COUNT(*) FROM test_secrets")
                .execute(&mut conn);

            assert!(result.is_ok(), "Should be able to query database");
        }

        // Act 3: Test key clearing
        clear_db_encryption_key();
        let db_key_guard = DB_ENCRYPTION_KEY.lock().unwrap();
        assert!(db_key_guard.is_none(), "Encryption key should be cleared");
        drop(db_key_guard);

        // Cleanup
        set_db_path("dure.db".to_string());

        println!("✓ Encryption key management works correctly");
        println!("✓ PRAGMA commands execute without error");
        println!("Note: Full encryption verification requires libsqlite3-hotbundle with encryption support");
    }

    #[cfg(not(any(feature = "postgres", target_family = "wasm")))]
    #[test]
    fn test_encryption_key_storage() {
        use tempfile::tempdir;
        use std::io::Cursor;

        // Arrange: Create temporary keyring
        let temp_dir = tempdir().unwrap();
        let kdbx_path = temp_dir.path().join("test.kdbx");
        let kpkey_path = temp_dir.path().join("id_ed25519");

        // Generate test keyfile
        use rand::RngCore;
        let mut kpkey_bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut kpkey_bytes);
        std::fs::write(&kpkey_path, &kpkey_bytes).unwrap();

        // Create new KeePass database
        let mut db = keepass::Database::new(Default::default());
        db.meta.database_name = Some("Test".to_string());

        // Create key from password + keyfile
        let mut key = keepass::DatabaseKey::new().with_password("testpass");
        let mut cursor = Cursor::new(&kpkey_bytes);
        key = key.with_keyfile(&mut cursor).unwrap();

        // Save database
        let mut file = std::fs::File::create(&kdbx_path).unwrap();
        db.save(&mut file, key).unwrap();
        drop(file);

        // Open database handle
        let handle = crate::calc::keyring::DatabaseHandle::open(
            kdbx_path.clone(),
            kpkey_path.clone(),
            Some("testpass"),
        ).expect("Failed to open test database");

        // Act: Generate DB encryption key
        let generated_key = handle.generate_db_encryption_key()
            .expect("Failed to generate DB encryption key");

        // Assert: Key should be 44 characters (32 bytes base64-encoded)
        assert_eq!(generated_key.len(), 44, "Generated key should be 32 bytes base64-encoded (44 chars)");

        // Act: Retrieve the key
        let retrieved_key = handle.get_db_encryption_key()
            .expect("Failed to retrieve DB encryption key");

        // Assert: Retrieved key should match generated key
        assert_eq!(retrieved_key, generated_key, "Retrieved key should match generated key");

        // Act: Try to generate again (should fail - key exists)
        let result = handle.generate_db_encryption_key();
        assert!(result.is_err(), "Should not be able to generate key twice");
    }
}
