/// Integration test for SQLite3MultipleCiphers encryption features
/// Adapted from https://github.com/nikescar/libsqlite3-hotbundle/blob/main/tests/test_encryption.rs
/// Modified to use Diesel ORM instead of rusqlite
use diesel::prelude::*;
use diesel::sql_types::{Integer, Text};
use std::fs;
use tempfile::TempDir;

#[derive(QueryableByName, Debug)]
struct VersionResult {
    #[diesel(sql_type = Text)]
    version: String,
}

#[derive(QueryableByName, Debug)]
struct CountResult {
    #[diesel(sql_type = Integer)]
    count: i32,
}

#[derive(QueryableByName, Debug)]
struct UserResult {
    #[diesel(sql_type = Text)]
    name: String,
}

#[derive(QueryableByName, Debug)]
struct DataResult {
    #[diesel(sql_type = Text)]
    data: String,
}

#[derive(QueryableByName, Debug)]
struct ValueResult {
    #[diesel(sql_type = Text)]
    value: String,
}

#[test]
#[ignore] // TODO: Fix libsqlite3-hotbundle integration - currently using system SQLite without encryption
fn test_sqlite3mc_encryption() {
    eprintln!("\n=== SQLite3MultipleCiphers Encryption Test ===\n");

    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // Test 1: Verify SQLite version includes SQLite3MultipleCiphers
    test_version(&temp_dir);

    // Test 2: Create and use encrypted database
    test_basic_encryption(&temp_dir);

    // Test 3: Verify encryption actually works (can't open without key)
    test_encryption_required(&temp_dir);

    // Test 4: Test different ciphers
    test_different_ciphers(&temp_dir);

    // Test 5: Verify plaintext databases still work
    test_plaintext_database(&temp_dir);

    eprintln!("\n✅ All tests passed! SQLite3MultipleCiphers is working correctly.\n");
}

fn test_version(temp_dir: &TempDir) {
    eprintln!("📋 Test 1: Checking SQLite version...");

    let db_path = temp_dir.path().join("test_version.db");
    let database_url = db_path.to_str().unwrap();
    let mut conn =
        SqliteConnection::establish(database_url).expect("Failed to establish connection");

    let version: VersionResult = diesel::sql_query("SELECT sqlite_version() as version")
        .get_result(&mut conn)
        .expect("Failed to get SQLite version");

    eprintln!("   SQLite version: {}", version.version);

    // Try to get SQLite3MC version if available
    let mc_version_result: Result<VersionResult, _> =
        diesel::sql_query("SELECT sqlite3mc_version() as version").get_result(&mut conn);

    match mc_version_result {
        Ok(mc_version) => {
            eprintln!(
                "   ✅ SQLite3MultipleCiphers version: {}",
                mc_version.version
            );
        }
        Err(_) => {
            eprintln!(
                "   ⚠️  SQLite3MC version function not found (might be using standard SQLite)"
            );
        }
    }

    eprintln!();
}

fn test_basic_encryption(temp_dir: &TempDir) {
    eprintln!("🔐 Test 2: Creating encrypted database with ChaCha20...");

    let db_path = temp_dir.path().join("test_encrypted.db");
    let database_url = format!("file:{}?key=my-secret-password", db_path.display());
    let mut conn = SqliteConnection::establish(&database_url)
        .expect("Failed to establish encrypted connection");

    // Create table and insert data
    diesel::sql_query("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, email TEXT)")
        .execute(&mut conn)
        .expect("Failed to create table");

    diesel::sql_query("INSERT INTO users (name, email) VALUES ('Alice', 'alice@example.com')")
        .execute(&mut conn)
        .expect("Failed to insert Alice");

    diesel::sql_query("INSERT INTO users (name, email) VALUES ('Bob', 'bob@example.com')")
        .execute(&mut conn)
        .expect("Failed to insert Bob");

    // Verify we can read the data
    let count: CountResult = diesel::sql_query("SELECT COUNT(*) as count FROM users")
        .get_result(&mut conn)
        .expect("Failed to count users");
    assert_eq!(count.count, 2, "Expected 2 users");

    let name: UserResult = diesel::sql_query("SELECT name FROM users WHERE id = 1")
        .get_result(&mut conn)
        .expect("Failed to get user name");
    assert_eq!(name.name, "Alice", "Expected Alice");

    eprintln!("   ✅ Created encrypted database");
    eprintln!("   ✅ Inserted 2 records");
    eprintln!("   ✅ Successfully queried encrypted data");
    eprintln!();
}

fn test_encryption_required(temp_dir: &TempDir) {
    eprintln!("🔒 Test 3: Verifying encryption is enforced...");

    let db_path = temp_dir.path().join("test_encrypted.db");

    // Try to open the encrypted database without a key
    let database_url_no_key = db_path.to_str().unwrap();
    match SqliteConnection::establish(database_url_no_key) {
        Ok(mut conn) => {
            // Try to query - this should fail
            let result: Result<CountResult, _> =
                diesel::sql_query("SELECT COUNT(*) as count FROM users").get_result(&mut conn);

            match result {
                Ok(_) => {
                    panic!("❌ ERROR: Could read encrypted database without key!");
                }
                Err(_) => {
                    eprintln!("   ✅ Cannot read encrypted database without key (expected)");
                }
            }
        }
        Err(e) => {
            eprintln!("   ✅ Cannot open encrypted database: {:?}", e);
        }
    }

    // Now open with correct key
    let database_url_with_key = format!("file:{}?key=my-secret-password", db_path.display());
    let mut conn = SqliteConnection::establish(&database_url_with_key)
        .expect("Failed to open with correct key");

    let count: CountResult = diesel::sql_query("SELECT COUNT(*) as count FROM users")
        .get_result(&mut conn)
        .expect("Failed to count with correct key");
    assert_eq!(count.count, 2);

    eprintln!("   ✅ Successfully opened with correct key");

    // Try with wrong key
    let copy_path = temp_dir.path().join("test_encrypted_copy.db");
    fs::copy(&db_path, &copy_path).expect("Failed to copy database");

    let database_url_wrong_key = format!("file:{}?key=wrong-password", copy_path.display());
    let mut conn2 = SqliteConnection::establish(&database_url_wrong_key)
        .expect("Opened connection with wrong key");

    let result: Result<CountResult, _> =
        diesel::sql_query("SELECT COUNT(*) as count FROM users").get_result(&mut conn2);

    match result {
        Ok(_) => {
            panic!("❌ ERROR: Could read with wrong password!");
        }
        Err(_) => {
            eprintln!("   ✅ Cannot read with wrong password (expected)");
        }
    }

    eprintln!();
}

fn test_different_ciphers(temp_dir: &TempDir) {
    eprintln!("🔑 Test 4: Testing different cipher algorithms...");

    // Test AES-256
    eprintln!("   Testing AES-256-CBC...");
    let aes_path = temp_dir.path().join("test_aes256.db");
    let database_url = format!(
        "file:{}?cipher=aes256cbc&key=aes-password",
        aes_path.display()
    );
    let mut conn =
        SqliteConnection::establish(&database_url).expect("Failed to create AES256 database");

    diesel::sql_query("CREATE TABLE test (id INTEGER, data TEXT)")
        .execute(&mut conn)
        .expect("Failed to create AES256 table");

    diesel::sql_query("INSERT INTO test VALUES (1, 'AES encrypted')")
        .execute(&mut conn)
        .expect("Failed to insert AES256 data");

    let data: DataResult = diesel::sql_query("SELECT data FROM test WHERE id = 1")
        .get_result(&mut conn)
        .expect("Failed to query AES256 data");
    assert_eq!(data.data, "AES encrypted");
    eprintln!("      ✅ AES-256-CBC works");

    // Test SQLCipher compatibility
    eprintln!("   Testing SQLCipher...");
    let sqlcipher_path = temp_dir.path().join("test_sqlcipher.db");
    let database_url = format!(
        "file:{}?cipher=sqlcipher&key=sqlcipher-password",
        sqlcipher_path.display()
    );
    let mut conn =
        SqliteConnection::establish(&database_url).expect("Failed to create SQLCipher database");

    diesel::sql_query("CREATE TABLE test (id INTEGER, data TEXT)")
        .execute(&mut conn)
        .expect("Failed to create SQLCipher table");

    diesel::sql_query("INSERT INTO test VALUES (1, 'SQLCipher encrypted')")
        .execute(&mut conn)
        .expect("Failed to insert SQLCipher data");

    let data: DataResult = diesel::sql_query("SELECT data FROM test WHERE id = 1")
        .get_result(&mut conn)
        .expect("Failed to query SQLCipher data");
    assert_eq!(data.data, "SQLCipher encrypted");
    eprintln!("      ✅ SQLCipher works");

    eprintln!("   ✅ Multiple cipher algorithms working");
    eprintln!();
}

fn test_plaintext_database(temp_dir: &TempDir) {
    eprintln!("📝 Test 5: Verifying plaintext databases still work...");

    // Create unencrypted database
    let plaintext_path = temp_dir.path().join("test_plaintext.db");
    let database_url = plaintext_path.to_str().unwrap();
    let mut conn =
        SqliteConnection::establish(database_url).expect("Failed to create plaintext database");

    diesel::sql_query("CREATE TABLE data (id INTEGER PRIMARY KEY, value TEXT)")
        .execute(&mut conn)
        .expect("Failed to create plaintext table");

    diesel::sql_query("INSERT INTO data (value) VALUES ('plaintext data')")
        .execute(&mut conn)
        .expect("Failed to insert plaintext data");

    let value: ValueResult = diesel::sql_query("SELECT value FROM data WHERE id = 1")
        .get_result(&mut conn)
        .expect("Failed to query plaintext data");

    assert_eq!(value.value, "plaintext data");
    eprintln!("   ✅ Unencrypted databases work normally");

    // Verify the file is actually unencrypted by reading raw bytes
    drop(conn);
    if let Ok(file_content) = fs::read(&plaintext_path) {
        let file_str = String::from_utf8_lossy(&file_content[..100.min(file_content.len())]);
        if file_str.contains("SQLite") {
            eprintln!("   ✅ Database file contains SQLite magic string (not encrypted)");
        }
    }

    eprintln!();
}
