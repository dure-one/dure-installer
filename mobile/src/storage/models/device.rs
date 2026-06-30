//! Device authentication storage model

use anyhow::{Context, Result};
use diesel::prelude::*;

/// Authenticated device information
#[derive(Debug, Clone)]
pub struct AuthenticatedDevice {
    pub device_id: String,
    pub public_key: String,
    pub session_id: String,
    pub authenticated_at: u64,
    pub last_seen: u64,
}

/// Store or update authenticated device
pub fn store_device_auth(conn: &mut SqliteConnection, device: &AuthenticatedDevice) -> Result<()> {
    diesel::sql_query(
        "INSERT OR REPLACE INTO authenticated_devices
         (device_id, public_key, session_id, authenticated_at, last_seen)
         VALUES (?1, ?2, ?3, ?4, ?5)",
    )
    .bind::<diesel::sql_types::Text, _>(&device.device_id)
    .bind::<diesel::sql_types::Text, _>(&device.public_key)
    .bind::<diesel::sql_types::Text, _>(&device.session_id)
    .bind::<diesel::sql_types::BigInt, _>(device.authenticated_at as i64)
    .bind::<diesel::sql_types::BigInt, _>(device.last_seen as i64)
    .execute(conn)
    .context("Failed to store device authentication")?;

    Ok(())
}

/// Get authenticated device by device_id
pub fn get_device_auth(
    conn: &mut SqliteConnection,
    device_id: &str,
) -> Result<Option<AuthenticatedDevice>> {
    use diesel::sql_types::{BigInt, Text};

    #[derive(QueryableByName)]
    struct DeviceRow {
        #[diesel(sql_type = Text)]
        device_id: String,
        #[diesel(sql_type = Text)]
        public_key: String,
        #[diesel(sql_type = Text)]
        session_id: String,
        #[diesel(sql_type = BigInt)]
        authenticated_at: i64,
        #[diesel(sql_type = BigInt)]
        last_seen: i64,
    }

    let rows = diesel::sql_query(
        "SELECT device_id, public_key, session_id, authenticated_at, last_seen
         FROM authenticated_devices
         WHERE device_id = ?1",
    )
    .bind::<Text, _>(device_id)
    .load::<DeviceRow>(conn)
    .context("Failed to query device authentication")?;

    if rows.is_empty() {
        return Ok(None);
    }

    let row = &rows[0];
    Ok(Some(AuthenticatedDevice {
        device_id: row.device_id.clone(),
        public_key: row.public_key.clone(),
        session_id: row.session_id.clone(),
        authenticated_at: row.authenticated_at as u64,
        last_seen: row.last_seen as u64,
    }))
}

/// Get device_id by session_id
pub fn get_device_by_session(
    conn: &mut SqliteConnection,
    session_id: &str,
) -> Result<Option<String>> {
    use diesel::sql_types::Text;

    #[derive(QueryableByName)]
    struct DeviceIdRow {
        #[diesel(sql_type = Text)]
        device_id: String,
    }

    let rows =
        diesel::sql_query("SELECT device_id FROM authenticated_devices WHERE session_id = ?1")
            .bind::<Text, _>(session_id)
            .load::<DeviceIdRow>(conn)
            .context("Failed to query device by session")?;

    Ok(rows.first().map(|row| row.device_id.clone()))
}

/// Update device last_seen timestamp
pub fn update_device_activity(conn: &mut SqliteConnection, device_id: &str) -> Result<()> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(std::time::Duration::from_secs(0))
        .as_secs();

    diesel::sql_query("UPDATE authenticated_devices SET last_seen = ?1 WHERE device_id = ?2")
        .bind::<diesel::sql_types::BigInt, _>(now as i64)
        .bind::<diesel::sql_types::Text, _>(device_id)
        .execute(conn)
        .context("Failed to update device activity")?;

    Ok(())
}

/// Delete authenticated device
pub fn delete_device_auth(conn: &mut SqliteConnection, device_id: &str) -> Result<()> {
    diesel::sql_query("DELETE FROM authenticated_devices WHERE device_id = ?1")
        .bind::<diesel::sql_types::Text, _>(device_id)
        .execute(conn)
        .context("Failed to delete device authentication")?;

    Ok(())
}

/// List all authenticated devices
pub fn list_authenticated_devices(conn: &mut SqliteConnection) -> Result<Vec<AuthenticatedDevice>> {
    use diesel::sql_types::{BigInt, Text};

    #[derive(QueryableByName)]
    struct DeviceRow {
        #[diesel(sql_type = Text)]
        device_id: String,
        #[diesel(sql_type = Text)]
        public_key: String,
        #[diesel(sql_type = Text)]
        session_id: String,
        #[diesel(sql_type = BigInt)]
        authenticated_at: i64,
        #[diesel(sql_type = BigInt)]
        last_seen: i64,
    }

    let rows = diesel::sql_query(
        "SELECT device_id, public_key, session_id, authenticated_at, last_seen
         FROM authenticated_devices
         ORDER BY last_seen DESC",
    )
    .load::<DeviceRow>(conn)
    .context("Failed to list authenticated devices")?;

    Ok(rows
        .into_iter()
        .map(|row| AuthenticatedDevice {
            device_id: row.device_id,
            public_key: row.public_key,
            session_id: row.session_id,
            authenticated_at: row.authenticated_at as u64,
            last_seen: row.last_seen as u64,
        })
        .collect())
}

/// Clean up old device authentications (older than max_age_secs)
pub fn cleanup_old_devices(conn: &mut SqliteConnection, max_age_secs: u64) -> Result<usize> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(std::time::Duration::from_secs(0))
        .as_secs();

    let cutoff = now - max_age_secs;

    let deleted = diesel::sql_query("DELETE FROM authenticated_devices WHERE last_seen < ?1")
        .bind::<diesel::sql_types::BigInt, _>(cutoff as i64)
        .execute(conn)
        .context("Failed to cleanup old device authentications")?;

    Ok(deleted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calc::db;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn setup_test_db() -> diesel::SqliteConnection {
        let test_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let test_db = format!("test-dure-device-{}-{}.db", std::process::id(), test_id);
        db::set_db_path(test_db);
        db::establish_connection()
    }

    #[test]
    fn test_store_and_get_device() {
        let mut conn = setup_test_db();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let device = AuthenticatedDevice {
            device_id: "test-device".to_string(),
            public_key: "test-public-key".to_string(),
            session_id: "test-session".to_string(),
            authenticated_at: now,
            last_seen: now,
        };

        store_device_auth(&mut conn, &device).unwrap();

        let retrieved = get_device_auth(&mut conn, "test-device").unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.device_id, "test-device");
        assert_eq!(retrieved.public_key, "test-public-key");
    }

    #[test]
    fn test_get_device_by_session() {
        let mut conn = setup_test_db();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let device = AuthenticatedDevice {
            device_id: "test-device-2".to_string(),
            public_key: "test-key".to_string(),
            session_id: "test-session-2".to_string(),
            authenticated_at: now,
            last_seen: now,
        };

        store_device_auth(&mut conn, &device).unwrap();

        let device_id = get_device_by_session(&mut conn, "test-session-2").unwrap();
        assert_eq!(device_id, Some("test-device-2".to_string()));
    }
}
