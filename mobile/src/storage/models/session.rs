//! Session storage model for HTTP/WSS sessions

use anyhow::{Context, Result};
use diesel::prelude::*;

/// Session information for HTTP/WSS connections
#[derive(Debug, Clone)]
pub struct Session {
    pub session_id: String,
    pub domain: String,
    pub session_type: String, // "http" or "wss"
    pub connected_at: u64,
    pub last_seen: u64,
    pub request_count: u64,
    pub remote_addr: String,
}

impl Session {
    pub fn new(
        session_id: String,
        domain: String,
        session_type: String,
        remote_addr: String,
    ) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_secs();

        Self {
            session_id,
            domain,
            session_type,
            connected_at: now,
            last_seen: now,
            request_count: 0,
            remote_addr,
        }
    }
}

/// Initialize session tables (migration handled by diesel_migrations)
pub fn init_session_tables(_conn: &mut SqliteConnection) -> Result<()> {
    // Table creation is handled by Diesel migrations
    // This function is kept for API compatibility
    Ok(())
}

/// Store or update session
pub fn store_session(conn: &mut SqliteConnection, session: &Session) -> Result<()> {
    diesel::sql_query(
        "INSERT OR REPLACE INTO sessions
         (session_id, domain, session_type, connected_at, last_seen, request_count, remote_addr)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
    )
    .bind::<diesel::sql_types::Text, _>(&session.session_id)
    .bind::<diesel::sql_types::Text, _>(&session.domain)
    .bind::<diesel::sql_types::Text, _>(&session.session_type)
    .bind::<diesel::sql_types::BigInt, _>(session.connected_at as i64)
    .bind::<diesel::sql_types::BigInt, _>(session.last_seen as i64)
    .bind::<diesel::sql_types::BigInt, _>(session.request_count as i64)
    .bind::<diesel::sql_types::Text, _>(&session.remote_addr)
    .execute(conn)
    .context("Failed to store session")?;

    Ok(())
}

/// Get session by ID
pub fn get_session(conn: &mut SqliteConnection, session_id: &str) -> Result<Option<Session>> {
    use diesel::sql_types::{BigInt, Text};

    #[derive(QueryableByName)]
    struct SessionRow {
        #[diesel(sql_type = Text)]
        session_id: String,
        #[diesel(sql_type = Text)]
        domain: String,
        #[diesel(sql_type = Text)]
        session_type: String,
        #[diesel(sql_type = BigInt)]
        connected_at: i64,
        #[diesel(sql_type = BigInt)]
        last_seen: i64,
        #[diesel(sql_type = BigInt)]
        request_count: i64,
        #[diesel(sql_type = Text)]
        remote_addr: String,
    }

    let rows = diesel::sql_query(
        "SELECT session_id, domain, session_type, connected_at, last_seen, request_count, remote_addr
         FROM sessions
         WHERE session_id = ?1",
    )
    .bind::<Text, _>(session_id)
    .load::<SessionRow>(conn)
    .context("Failed to query session")?;

    if rows.is_empty() {
        return Ok(None);
    }

    let row = &rows[0];
    Ok(Some(Session {
        session_id: row.session_id.clone(),
        domain: row.domain.clone(),
        session_type: row.session_type.clone(),
        connected_at: row.connected_at as u64,
        last_seen: row.last_seen as u64,
        request_count: row.request_count as u64,
        remote_addr: row.remote_addr.clone(),
    }))
}

/// Update session last_seen and increment request count
pub fn update_session_activity(conn: &mut SqliteConnection, session_id: &str) -> Result<()> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(std::time::Duration::from_secs(0))
        .as_secs();

    diesel::sql_query(
        "UPDATE sessions SET last_seen = ?1, request_count = request_count + 1 WHERE session_id = ?2",
    )
    .bind::<diesel::sql_types::BigInt, _>(now as i64)
    .bind::<diesel::sql_types::Text, _>(session_id)
    .execute(conn)
    .context("Failed to update session activity")?;

    Ok(())
}

/// List sessions for a domain
pub fn list_sessions_for_domain(conn: &mut SqliteConnection, domain: &str) -> Result<Vec<Session>> {
    use diesel::sql_types::{BigInt, Text};

    #[derive(QueryableByName)]
    struct SessionRow {
        #[diesel(sql_type = Text)]
        session_id: String,
        #[diesel(sql_type = Text)]
        domain: String,
        #[diesel(sql_type = Text)]
        session_type: String,
        #[diesel(sql_type = BigInt)]
        connected_at: i64,
        #[diesel(sql_type = BigInt)]
        last_seen: i64,
        #[diesel(sql_type = BigInt)]
        request_count: i64,
        #[diesel(sql_type = Text)]
        remote_addr: String,
    }

    let rows = diesel::sql_query(
        "SELECT session_id, domain, session_type, connected_at, last_seen, request_count, remote_addr
         FROM sessions
         WHERE domain = ?1
         ORDER BY last_seen DESC",
    )
    .bind::<Text, _>(domain)
    .load::<SessionRow>(conn)
    .context("Failed to list sessions")?;

    Ok(rows
        .into_iter()
        .map(|row| Session {
            session_id: row.session_id,
            domain: row.domain,
            session_type: row.session_type,
            connected_at: row.connected_at as u64,
            last_seen: row.last_seen as u64,
            request_count: row.request_count as u64,
            remote_addr: row.remote_addr,
        })
        .collect())
}

/// List sessions by type
pub fn list_sessions_by_type(
    conn: &mut SqliteConnection,
    session_type: &str,
) -> Result<Vec<Session>> {
    use diesel::sql_types::{BigInt, Text};

    #[derive(QueryableByName)]
    struct SessionRow {
        #[diesel(sql_type = Text)]
        session_id: String,
        #[diesel(sql_type = Text)]
        domain: String,
        #[diesel(sql_type = Text)]
        session_type: String,
        #[diesel(sql_type = BigInt)]
        connected_at: i64,
        #[diesel(sql_type = BigInt)]
        last_seen: i64,
        #[diesel(sql_type = BigInt)]
        request_count: i64,
        #[diesel(sql_type = Text)]
        remote_addr: String,
    }

    let rows = diesel::sql_query(
        "SELECT session_id, domain, session_type, connected_at, last_seen, request_count, remote_addr
         FROM sessions
         WHERE session_type = ?1
         ORDER BY last_seen DESC",
    )
    .bind::<Text, _>(session_type)
    .load::<SessionRow>(conn)
    .context("Failed to list sessions by type")?;

    Ok(rows
        .into_iter()
        .map(|row| Session {
            session_id: row.session_id,
            domain: row.domain,
            session_type: row.session_type,
            connected_at: row.connected_at as u64,
            last_seen: row.last_seen as u64,
            request_count: row.request_count as u64,
            remote_addr: row.remote_addr,
        })
        .collect())
}

/// List all active sessions
pub fn list_active_sessions(
    conn: &mut SqliteConnection,
    timeout_secs: u64,
) -> Result<Vec<Session>> {
    use diesel::sql_types::{BigInt, Text};
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(std::time::Duration::from_secs(0))
        .as_secs();

    let cutoff = now - timeout_secs;

    #[derive(QueryableByName)]
    struct SessionRow {
        #[diesel(sql_type = Text)]
        session_id: String,
        #[diesel(sql_type = Text)]
        domain: String,
        #[diesel(sql_type = Text)]
        session_type: String,
        #[diesel(sql_type = BigInt)]
        connected_at: i64,
        #[diesel(sql_type = BigInt)]
        last_seen: i64,
        #[diesel(sql_type = BigInt)]
        request_count: i64,
        #[diesel(sql_type = Text)]
        remote_addr: String,
    }

    let rows = diesel::sql_query(
        "SELECT session_id, domain, session_type, connected_at, last_seen, request_count, remote_addr
         FROM sessions
         WHERE last_seen > ?1
         ORDER BY last_seen DESC",
    )
    .bind::<BigInt, _>(cutoff as i64)
    .load::<SessionRow>(conn)
    .context("Failed to list active sessions")?;

    Ok(rows
        .into_iter()
        .map(|row| Session {
            session_id: row.session_id,
            domain: row.domain,
            session_type: row.session_type,
            connected_at: row.connected_at as u64,
            last_seen: row.last_seen as u64,
            request_count: row.request_count as u64,
            remote_addr: row.remote_addr,
        })
        .collect())
}

/// Delete session
pub fn delete_session(conn: &mut SqliteConnection, session_id: &str) -> Result<()> {
    diesel::sql_query("DELETE FROM sessions WHERE session_id = ?1")
        .bind::<diesel::sql_types::Text, _>(session_id)
        .execute(conn)
        .context("Failed to delete session")?;

    Ok(())
}

/// Clean up old sessions (older than max_age_secs)
pub fn cleanup_old_sessions(conn: &mut SqliteConnection, max_age_secs: u64) -> Result<usize> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(std::time::Duration::from_secs(0))
        .as_secs();

    let cutoff = now - max_age_secs;

    let deleted = diesel::sql_query("DELETE FROM sessions WHERE last_seen < ?1")
        .bind::<diesel::sql_types::BigInt, _>(cutoff as i64)
        .execute(conn)
        .context("Failed to cleanup old sessions")?;

    Ok(deleted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calc::db;
    use std::sync::atomic::{AtomicU32, Ordering};

    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn setup_test_db() -> diesel::SqliteConnection {
        let test_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let test_db = format!("test-dure-session-{}-{}.db", std::process::id(), test_id);
        db::set_db_path(test_db);
        db::establish_connection()
    }

    #[test]
    fn test_init_tables() {
        let mut conn = setup_test_db();
        init_session_tables(&mut conn).unwrap();
    }

    #[test]
    fn test_store_and_get_session() {
        let mut conn = setup_test_db();
        init_session_tables(&mut conn).unwrap();

        let session = Session::new(
            "test-session".to_string(),
            "example.com".to_string(),
            "http".to_string(),
            "127.0.0.1".to_string(),
        );

        store_session(&mut conn, &session).unwrap();

        let retrieved = get_session(&mut conn, "test-session").unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.session_id, "test-session");
        assert_eq!(retrieved.domain, "example.com");
        assert_eq!(retrieved.session_type, "http");
    }

    #[test]
    fn test_update_session_activity() {
        let mut conn = setup_test_db();
        init_session_tables(&mut conn).unwrap();

        let session = Session::new(
            "test-session".to_string(),
            "example.com".to_string(),
            "http".to_string(),
            "127.0.0.1".to_string(),
        );

        store_session(&mut conn, &session).unwrap();

        update_session_activity(&mut conn, "test-session").unwrap();

        let retrieved = get_session(&mut conn, "test-session").unwrap().unwrap();
        assert_eq!(retrieved.request_count, 1);
    }

    #[test]
    fn test_list_sessions_by_type() {
        let mut conn = setup_test_db();
        init_session_tables(&mut conn).unwrap();

        let http_session = Session::new(
            "http-session".to_string(),
            "example.com".to_string(),
            "http".to_string(),
            "127.0.0.1".to_string(),
        );

        let wss_session = Session::new(
            "wss-session".to_string(),
            "example.com".to_string(),
            "wss".to_string(),
            "127.0.0.2".to_string(),
        );

        store_session(&mut conn, &http_session).unwrap();
        store_session(&mut conn, &wss_session).unwrap();

        let http_sessions = list_sessions_by_type(&mut conn, "http").unwrap();
        assert_eq!(http_sessions.len(), 1);

        let wss_sessions = list_sessions_by_type(&mut conn, "wss").unwrap();
        assert_eq!(wss_sessions.len(), 1);
    }

    #[test]
    fn test_cleanup_old_sessions() {
        let mut conn = setup_test_db();
        init_session_tables(&mut conn).unwrap();

        let mut old_session = Session::new(
            "old-session".to_string(),
            "example.com".to_string(),
            "http".to_string(),
            "127.0.0.1".to_string(),
        );
        old_session.last_seen = 1000; // Very old timestamp

        store_session(&mut conn, &old_session).unwrap();

        let deleted = cleanup_old_sessions(&mut conn, 3600).unwrap();
        assert_eq!(deleted, 1);
    }
}
