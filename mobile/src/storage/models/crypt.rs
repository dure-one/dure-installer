//! Cryptography key storage model
//!
//! Stores device identity (device_id) and encryption keys (private key, public key)
//! for X25519 + ChaCha20-Poly1305 encryption/decryption operations.

use anyhow::{Context, Result};
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Binary, Text};

/// Device identity and encryption keys
#[derive(Debug, Clone)]
pub struct DeviceKeys {
    pub device_id: String,
    pub private_key: Vec<u8>,
    pub public_key: Vec<u8>,
    pub created_at: u64,
}

/// Row struct for querying device keys
#[derive(QueryableByName)]
struct DeviceKeysRow {
    #[diesel(sql_type = Text)]
    device_id: String,
    #[diesel(sql_type = Binary)]
    private_key: Vec<u8>,
    #[diesel(sql_type = Binary)]
    public_key: Vec<u8>,
    #[diesel(sql_type = BigInt)]
    created_at: i64,
}

impl From<DeviceKeysRow> for DeviceKeys {
    fn from(row: DeviceKeysRow) -> Self {
        DeviceKeys {
            device_id: row.device_id,
            private_key: row.private_key,
            public_key: row.public_key,
            created_at: row.created_at as u64,
        }
    }
}

/// Initialize the crypt_keys table (migration handled by diesel_migrations)
pub fn init_crypt_table(_conn: &mut SqliteConnection) -> Result<()> {
    // Table creation is handled by Diesel migrations
    // This function is kept for API compatibility
    Ok(())
}

/// Store device keys
pub fn store_device_keys(conn: &mut SqliteConnection, keys: &DeviceKeys) -> Result<()> {
    diesel::sql_query(
        "INSERT OR REPLACE INTO crypt_keys (device_id, private_key, public_key, created_at)
         VALUES (?1, ?2, ?3, ?4)",
    )
    .bind::<Text, _>(&keys.device_id)
    .bind::<Binary, _>(&keys.private_key)
    .bind::<Binary, _>(&keys.public_key)
    .bind::<BigInt, _>(keys.created_at as i64)
    .execute(conn)
    .context("Failed to store device keys")?;

    Ok(())
}

/// Get device keys by device_id
pub fn get_device_keys(conn: &mut SqliteConnection, device_id: &str) -> Result<Option<DeviceKeys>> {
    let rows = diesel::sql_query(
        "SELECT device_id, private_key, public_key, created_at
         FROM crypt_keys
         WHERE device_id = ?1",
    )
    .bind::<Text, _>(device_id)
    .load::<DeviceKeysRow>(conn)
    .context("Failed to get device keys")?;

    Ok(rows.into_iter().next().map(DeviceKeys::from))
}

/// Get the current device's keys
pub fn get_current_device_keys(conn: &mut SqliteConnection) -> Result<Option<DeviceKeys>> {
    let rows = diesel::sql_query(
        "SELECT device_id, private_key, public_key, created_at
         FROM crypt_keys
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .load::<DeviceKeysRow>(conn)
    .context("Failed to get current device keys")?;

    Ok(rows.into_iter().next().map(DeviceKeys::from))
}

/// List all device keys
pub fn list_device_keys(conn: &mut SqliteConnection) -> Result<Vec<DeviceKeys>> {
    let rows = diesel::sql_query(
        "SELECT device_id, private_key, public_key, created_at
         FROM crypt_keys
         ORDER BY created_at DESC",
    )
    .load::<DeviceKeysRow>(conn)
    .context("Failed to list device keys")?;

    Ok(rows.into_iter().map(DeviceKeys::from).collect())
}

/// Delete device keys
pub fn delete_device_keys(conn: &mut SqliteConnection, device_id: &str) -> Result<()> {
    let deleted = diesel::sql_query("DELETE FROM crypt_keys WHERE device_id = ?1")
        .bind::<Text, _>(device_id)
        .execute(conn)
        .context("Failed to delete device keys")?;

    if deleted == 0 {
        anyhow::bail!("Device keys for {} not found", device_id);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calc::db;
    use std::sync::atomic::{AtomicU32, Ordering};

    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn setup_test_db() -> diesel::SqliteConnection {
        let test_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let test_db = format!("test-dure-crypt-{}-{}.db", std::process::id(), test_id);
        db::set_db_path(test_db);
        db::establish_connection()
    }

    #[test]
    fn test_init_table() {
        let mut conn = setup_test_db();
        init_crypt_table(&mut conn).unwrap();
    }

    #[test]
    fn test_store_and_get_keys() {
        let mut conn = setup_test_db();
        init_crypt_table(&mut conn).unwrap();

        let keys = DeviceKeys {
            device_id: "test-device".to_string(),
            private_key: vec![1, 2, 3, 4],
            public_key: vec![5, 6, 7, 8],
            created_at: 1234567890,
        };

        store_device_keys(&mut conn, &keys).unwrap();

        let retrieved = get_device_keys(&mut conn, "test-device").unwrap();
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.device_id, "test-device");
        assert_eq!(retrieved.private_key, vec![1, 2, 3, 4]);
        assert_eq!(retrieved.public_key, vec![5, 6, 7, 8]);
    }

    #[test]
    fn test_delete_keys() {
        let mut conn = setup_test_db();
        init_crypt_table(&mut conn).unwrap();

        let keys = DeviceKeys {
            device_id: "test-device".to_string(),
            private_key: vec![1, 2, 3, 4],
            public_key: vec![5, 6, 7, 8],
            created_at: 1234567890,
        };

        store_device_keys(&mut conn, &keys).unwrap();
        delete_device_keys(&mut conn, "test-device").unwrap();

        let retrieved = get_device_keys(&mut conn, "test-device").unwrap();
        assert!(retrieved.is_none());
    }
}
