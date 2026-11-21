//! Mycart site storage model for mycart admin settings

use anyhow::{Context, Result};
use diesel::prelude::*;
use diesel::sql_types::{Integer, Text};

/// Mycart site information
#[derive(Debug, Clone)]
pub struct MycartSiteInfo {
    pub site_address: String,
    pub email: String,
    pub encrypted_password: String,
    pub jwt_token: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

impl MycartSiteInfo {
    pub fn new(site_address: String, email: String, encrypted_password: String) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_secs();

        Self {
            site_address,
            email,
            encrypted_password,
            jwt_token: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_token(mut self, token: String) -> Self {
        self.jwt_token = Some(token);
        self
    }
}

/// Initialize mycart_sites table
pub fn init_mycart_sites_table(conn: &mut SqliteConnection) -> Result<()> {
    // Check if table exists and has old schema (admin_id column)
    #[derive(QueryableByName)]
    struct ColumnCheck {
        #[diesel(sql_type = diesel::sql_types::Integer)]
        count: i32,
    }

    let has_admin_id = diesel::sql_query(
        "SELECT COUNT(*) as count FROM pragma_table_info('mycart_sites') WHERE name='admin_id'",
    )
    .get_result::<ColumnCheck>(conn)
    .map(|c| c.count > 0)
    .unwrap_or(false);

    if has_admin_id {
        // Migration path: table exists with old schema, need to migrate
        log::info!("Migrating mycart_sites table from admin_id to email column");

        // Add jwt_token if missing
        let _ =
            diesel::sql_query("ALTER TABLE mycart_sites ADD COLUMN jwt_token TEXT").execute(conn);

        // Create new table with correct schema
        diesel::sql_query(
            "CREATE TABLE mycart_sites_new (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                site_address TEXT NOT NULL UNIQUE,
                email TEXT NOT NULL,
                encrypted_password TEXT NOT NULL,
                jwt_token TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
        )
        .execute(conn)
        .context("Failed to create new mycart_sites table")?;

        // Copy data, mapping admin_id to email
        diesel::sql_query(
            "INSERT INTO mycart_sites_new (site_address, email, encrypted_password, jwt_token, created_at, updated_at)
             SELECT site_address, admin_id, encrypted_password, jwt_token, created_at, updated_at
             FROM mycart_sites"
        )
        .execute(conn)
        .context("Failed to migrate data")?;

        // Swap tables
        diesel::sql_query("DROP TABLE mycart_sites")
            .execute(conn)
            .context("Failed to drop old table")?;

        diesel::sql_query("ALTER TABLE mycart_sites_new RENAME TO mycart_sites")
            .execute(conn)
            .context("Failed to rename new table")?;

        log::info!("Migration completed successfully");
    } else {
        // Fresh install or already migrated - just ensure table exists
        diesel::sql_query(
            "CREATE TABLE IF NOT EXISTS mycart_sites (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                site_address TEXT NOT NULL UNIQUE,
                email TEXT NOT NULL,
                encrypted_password TEXT NOT NULL,
                jwt_token TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
        )
        .execute(conn)
        .context("Failed to create mycart_sites table")?;
    }

    // Create index
    diesel::sql_query(
        "CREATE INDEX IF NOT EXISTS idx_mycart_sites_site_address
         ON mycart_sites(site_address)",
    )
    .execute(conn)
    .context("Failed to create index")?;

    Ok(())
}

/// Store mycart site information
pub fn store_mycart_site(conn: &mut SqliteConnection, site: &MycartSiteInfo) -> Result<()> {
    diesel::sql_query(
        "INSERT OR REPLACE INTO mycart_sites
         (site_address, email, encrypted_password, jwt_token, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
    )
    .bind::<Text, _>(&site.site_address)
    .bind::<Text, _>(&site.email)
    .bind::<Text, _>(&site.encrypted_password)
    .bind::<diesel::sql_types::Nullable<Text>, _>(&site.jwt_token)
    .bind::<Integer, _>(site.created_at as i32)
    .bind::<Integer, _>(site.updated_at as i32)
    .execute(conn)
    .context("Failed to store mycart site")?;

    Ok(())
}

/// Get mycart site by site_address
pub fn get_mycart_site(
    conn: &mut SqliteConnection,
    site_address: &str,
) -> Result<Option<MycartSiteInfo>> {
    #[derive(QueryableByName)]
    struct MycartSiteRow {
        #[diesel(sql_type = Text)]
        site_address: String,
        #[diesel(sql_type = Text)]
        email: String,
        #[diesel(sql_type = Text)]
        encrypted_password: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<Text>)]
        jwt_token: Option<String>,
        #[diesel(sql_type = Integer)]
        created_at: i32,
        #[diesel(sql_type = Integer)]
        updated_at: i32,
    }

    let rows = diesel::sql_query(
        "SELECT site_address, email, encrypted_password, jwt_token, created_at, updated_at
         FROM mycart_sites
         WHERE site_address = ?1",
    )
    .bind::<Text, _>(site_address)
    .load::<MycartSiteRow>(conn)
    .context("Failed to query mycart site")?;

    Ok(rows.into_iter().next().map(|row| MycartSiteInfo {
        site_address: row.site_address,
        email: row.email,
        encrypted_password: row.encrypted_password,
        jwt_token: row.jwt_token,
        created_at: row.created_at as u64,
        updated_at: row.updated_at as u64,
    }))
}

/// List all mycart sites
pub fn list_mycart_sites(conn: &mut SqliteConnection) -> Result<Vec<MycartSiteInfo>> {
    #[derive(QueryableByName)]
    struct MycartSiteRow {
        #[diesel(sql_type = Text)]
        site_address: String,
        #[diesel(sql_type = Text)]
        email: String,
        #[diesel(sql_type = Text)]
        encrypted_password: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<Text>)]
        jwt_token: Option<String>,
        #[diesel(sql_type = Integer)]
        created_at: i32,
        #[diesel(sql_type = Integer)]
        updated_at: i32,
    }

    let rows = diesel::sql_query(
        "SELECT site_address, email, encrypted_password, jwt_token, created_at, updated_at
         FROM mycart_sites
         ORDER BY created_at DESC",
    )
    .load::<MycartSiteRow>(conn)
    .context("Failed to list mycart sites")?;

    Ok(rows
        .into_iter()
        .map(|row| MycartSiteInfo {
            site_address: row.site_address,
            email: row.email,
            encrypted_password: row.encrypted_password,
            jwt_token: row.jwt_token,
            created_at: row.created_at as u64,
            updated_at: row.updated_at as u64,
        })
        .collect())
}

/// Update admin password for a mycart site
pub fn update_mycart_password(
    conn: &mut SqliteConnection,
    site_address: &str,
    encrypted_password: &str,
) -> Result<()> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(std::time::Duration::from_secs(0))
        .as_secs();

    diesel::sql_query(
        "UPDATE mycart_sites
         SET encrypted_password = ?1, updated_at = ?2
         WHERE site_address = ?3",
    )
    .bind::<Text, _>(encrypted_password)
    .bind::<Integer, _>(now as i32)
    .bind::<Text, _>(site_address)
    .execute(conn)
    .context("Failed to update mycart password")?;

    Ok(())
}

/// Update JWT token for a mycart site
pub fn update_jwt_token(
    conn: &mut SqliteConnection,
    site_address: &str,
    jwt_token: Option<&str>,
) -> Result<()> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(std::time::Duration::from_secs(0))
        .as_secs();

    diesel::sql_query(
        "UPDATE mycart_sites
         SET jwt_token = ?1, updated_at = ?2
         WHERE site_address = ?3",
    )
    .bind::<diesel::sql_types::Nullable<Text>, _>(&jwt_token)
    .bind::<Integer, _>(now as i32)
    .bind::<Text, _>(site_address)
    .execute(conn)
    .context("Failed to update JWT token")?;

    Ok(())
}

/// Delete mycart site
pub fn delete_mycart_site(conn: &mut SqliteConnection, site_address: &str) -> Result<()> {
    diesel::sql_query("DELETE FROM mycart_sites WHERE site_address = ?1")
        .bind::<Text, _>(site_address)
        .execute(conn)
        .context("Failed to delete mycart site")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_db() -> SqliteConnection {
        let mut conn =
            SqliteConnection::establish(":memory:").expect("Failed to create in-memory database");
        init_mycart_sites_table(&mut conn).expect("Failed to initialize table");
        conn
    }

    #[test]
    fn test_mycart_site_table_exists() {
        let mut conn = setup_test_db();

        // Table should exist and be queryable
        let result =
            diesel::sql_query("SELECT COUNT(*) as count FROM mycart_sites").execute(&mut conn);

        assert!(result.is_ok(), "mycart_sites table should exist");
    }

    #[test]
    fn test_mycart_site_create_and_find() {
        let mut conn = setup_test_db();

        // Arrange
        let site = MycartSiteInfo::new(
            "dure.co".to_string(),
            "admin".to_string(),
            "encrypted_password_123".to_string(),
        );

        // Act - Store site
        store_mycart_site(&mut conn, &site).expect("Failed to store site");

        // Assert - Find site
        let found = get_mycart_site(&mut conn, "dure.co")
            .expect("Failed to query site")
            .expect("Site should exist");

        assert_eq!(found.site_address, "dure.co");
        assert_eq!(found.email, "admin");
        assert_eq!(found.encrypted_password, "encrypted_password_123");
        assert!(found.created_at > 0);
        assert_eq!(found.created_at, found.updated_at);
    }

    #[test]
    fn test_mycart_site_update_password() {
        let mut conn = setup_test_db();

        // Arrange - Create initial site
        let site = MycartSiteInfo::new(
            "dure.co".to_string(),
            "admin".to_string(),
            "old_password".to_string(),
        );
        store_mycart_site(&mut conn, &site).expect("Failed to store site");

        // Act - Update password
        update_mycart_password(&mut conn, "dure.co", "new_password")
            .expect("Failed to update password");

        // Assert - Verify password changed
        let found = get_mycart_site(&mut conn, "dure.co")
            .expect("Failed to query site")
            .expect("Site should exist");

        assert_eq!(found.encrypted_password, "new_password");
        assert!(found.updated_at >= found.created_at);
    }

    #[test]
    fn test_mycart_site_list_all() {
        let mut conn = setup_test_db();

        // Arrange - Create multiple sites
        let site1 = MycartSiteInfo::new(
            "site1.com".to_string(),
            "admin1".to_string(),
            "pass1".to_string(),
        );
        let site2 = MycartSiteInfo::new(
            "site2.com".to_string(),
            "admin2".to_string(),
            "pass2".to_string(),
        );

        store_mycart_site(&mut conn, &site1).expect("Failed to store site1");
        store_mycart_site(&mut conn, &site2).expect("Failed to store site2");

        // Act - List all sites
        let sites = list_mycart_sites(&mut conn).expect("Failed to list sites");

        // Assert - Should have 2 sites
        assert_eq!(sites.len(), 2);
        assert!(sites.iter().any(|s| s.site_address == "site1.com"));
        assert!(sites.iter().any(|s| s.site_address == "site2.com"));
    }

    #[test]
    fn test_mycart_site_delete() {
        let mut conn = setup_test_db();

        // Arrange - Create site
        let site = MycartSiteInfo::new(
            "dure.co".to_string(),
            "admin".to_string(),
            "password".to_string(),
        );
        store_mycart_site(&mut conn, &site).expect("Failed to store site");

        // Act - Delete site
        delete_mycart_site(&mut conn, "dure.co").expect("Failed to delete site");

        // Assert - Site should not exist
        let found = get_mycart_site(&mut conn, "dure.co").expect("Query should succeed");
        assert!(found.is_none(), "Site should be deleted");
    }

    #[test]
    fn test_jwt_token_storage() {
        let mut conn = setup_test_db();

        // Arrange - Create site with token
        let site = MycartSiteInfo::new(
            "dure.co".to_string(),
            "admin".to_string(),
            "password".to_string(),
        )
        .with_token("jwt_token_abc123".to_string());

        // Act - Store site
        store_mycart_site(&mut conn, &site).expect("Failed to store site");

        // Assert - Token should be stored
        let found = get_mycart_site(&mut conn, "dure.co")
            .expect("Failed to query site")
            .expect("Site should exist");

        assert_eq!(found.jwt_token, Some("jwt_token_abc123".to_string()));
    }

    #[test]
    fn test_jwt_token_update() {
        let mut conn = setup_test_db();

        // Arrange - Create site without token
        let site = MycartSiteInfo::new(
            "dure.co".to_string(),
            "admin".to_string(),
            "password".to_string(),
        );
        store_mycart_site(&mut conn, &site).expect("Failed to store site");

        // Act - Update token
        update_jwt_token(&mut conn, "dure.co", Some("new_token_xyz"))
            .expect("Failed to update token");

        // Assert - Token should be updated
        let found = get_mycart_site(&mut conn, "dure.co")
            .expect("Failed to query site")
            .expect("Site should exist");

        assert_eq!(found.jwt_token, Some("new_token_xyz".to_string()));
    }

    #[test]
    fn test_jwt_token_clear() {
        let mut conn = setup_test_db();

        // Arrange - Create site with token
        let site = MycartSiteInfo::new(
            "dure.co".to_string(),
            "admin".to_string(),
            "password".to_string(),
        )
        .with_token("token_to_clear".to_string());
        store_mycart_site(&mut conn, &site).expect("Failed to store site");

        // Act - Clear token
        update_jwt_token(&mut conn, "dure.co", None).expect("Failed to clear token");

        // Assert - Token should be None
        let found = get_mycart_site(&mut conn, "dure.co")
            .expect("Failed to query site")
            .expect("Site should exist");

        assert!(found.jwt_token.is_none(), "Token should be cleared");

        // Assert - Site should not exist
        let found = get_mycart_site(&mut conn, "dure.co").expect("Failed to query site");
        assert!(found.is_none(), "Site should be deleted");
    }

    #[test]
    fn test_mycart_site_unique_constraint() {
        let mut conn = setup_test_db();

        // Arrange - Create site
        let site = MycartSiteInfo::new(
            "dure.co".to_string(),
            "admin".to_string(),
            "password".to_string(),
        );
        store_mycart_site(&mut conn, &site).expect("Failed to store site");

        // Act - Try to insert duplicate (should replace due to INSERT OR REPLACE)
        let site2 = MycartSiteInfo::new(
            "dure.co".to_string(),
            "admin2".to_string(),
            "password2".to_string(),
        );
        store_mycart_site(&mut conn, &site2).expect("Failed to replace site");

        // Assert - Should have only one site with updated data
        let sites = list_mycart_sites(&mut conn).expect("Failed to list sites");
        assert_eq!(sites.len(), 1);
        assert_eq!(sites[0].email, "admin2");
    }
}
