//! Platform-specific HTTP server for OAuth callbacks
//!
//! Uses darkhttpd-sys on Unix-like systems and winhttpd-sys on Windows.
//! Both provide identical API via type aliasing.

#[cfg(not(target_os = "windows"))]
use darkhttpd_sys::DarkHttpd as HttpServer;

#[cfg(target_os = "windows")]
use winhttpd_sys::WinHttpd as HttpServer;

// Example usage (not implemented in this task):
// fn start_oauth_server(port: u16) -> Result<()> {
//     let mut server = HttpServer::new();
//     server.serve("./oauth_callback", port)?;
//     Ok(())
// }
