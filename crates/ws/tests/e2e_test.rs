use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use ws::{ServerConfig, WsServer};

#[cfg(feature = "chat-service")]
#[smol_potat::test]
async fn test_server_with_static_files_and_chat() {
    // Setup test environment
    let temp_dir = TempDir::new().unwrap();

    // Create static files
    let static_dir = temp_dir.path().join("static");
    fs::create_dir(&static_dir).unwrap();
    fs::write(static_dir.join("index.html"), b"<h1>Test</h1>").unwrap();

    // Create chat database
    let chat_db = temp_dir.path().join("chat.db");

    // Initialize chat chat
    let chat_chat = chat::ChatService::new(chat_db).await.unwrap();

    // Create server configuration
    let mut config = ServerConfig::new("test.example.com");
    config.static_dir = static_dir;
    config.bind_addr = "127.0.0.1:0".parse().unwrap(); // Random port

    // Create server with chat chat
    let server = WsServer::new(config).with_chat_chat(chat_chat);

    // Verify configuration
    assert_eq!(server.config.domain, "test.example.com");
}

#[smol_potat::test]
async fn test_server_without_chat() {
    let temp_dir = TempDir::new().unwrap();
    let static_dir = temp_dir.path().join("static");
    fs::create_dir(&static_dir).unwrap();

    let mut config = ServerConfig::new("test.example.com");
    config.static_dir = static_dir;

    let server = WsServer::new(config);
    assert_eq!(server.config.domain, "test.example.com");
}
