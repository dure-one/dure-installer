//! Database helpers for the HTTPS/WSS server.
//!
//! Provides a shared, thread-safe SQLite connection via `Arc<Mutex<SqliteConnection>>`.
//! Tables are initialised on first open. Callers lock the connection and call
//! the storage-layer functions in `crate::storage::models` directly.

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use std::sync::{Arc, Mutex};

use std::io;

use diesel::prelude::*;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

use crate::storage::models::webhook;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// A cloneable, thread-safe handle to the server SQLite database.
pub type DbConn = Arc<Mutex<SqliteConnection>>;

/// Open (or create) the server database, run all pending migrations, and
/// return a shareable connection handle.
pub fn open_db(path: &str) -> io::Result<DbConn> {
    let mut conn = SqliteConnection::establish(path)
        .map_err(|e| io::Error::other(format!("DB open '{}': {}", path, e)))?;

    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|e| io::Error::other(format!("migrations: {}", e)))?;

    webhook::init_webhook_tables(&mut conn)
        .map_err(|e| io::Error::other(format!("webhook init: {}", e)))?;

    Ok(Arc::new(Mutex::new(conn)))
}
