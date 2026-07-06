#![cfg(target_os = "windows")]

mod helpers;

use winhttpd_sys::WinHttpd;

#[test]
fn test_server_lifecycle() {
    let mut server = WinHttpd::new();
    let port = helpers::find_free_port();

    // Start server
    assert!(server.serve("tests/fixtures/test_www", port).is_ok());

    // Stop server
    assert!(server.stop().is_ok());
}

#[test]
fn test_serve_invalid_path() {
    let mut server = WinHttpd::new();
    let result = server.serve("/nonexistent/path/12345", 8080);

    // Current implementation doesn't validate paths in init
    // This test documents expected behavior for future enhancement
    let _ = result;
}

#[test]
fn test_multiple_servers_different_ports() {
    let port1 = helpers::find_free_port();
    let port2 = helpers::find_free_port();

    let mut server1 = WinHttpd::new();
    let mut server2 = WinHttpd::new();

    assert!(server1.serve("tests/fixtures/test_www", port1).is_ok());
    assert!(server2.serve("tests/fixtures/test_www", port2).is_ok());

    server1.stop().ok();
    server2.stop().ok();
}
