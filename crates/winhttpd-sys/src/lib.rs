//! Rust FFI bindings for winhttpd - Windows HTTP server
//!
//! This crate provides safe Rust bindings to [winhttpd](https://github.com/FmasterofU/winhttpd),
//! a Windows port of the darkhttpd static file server. It enables serving static files
//! on Windows with an API identical to `darkhttpd-sys`.
//!
//! # Platform Support
//!
//! This crate is **Windows-only** (MSVC compiler required). For Unix-like systems,
//! use `darkhttpd-sys` which provides an identical API.
//!
//! # Example
//!
//! ```no_run
//! use winhttpd_sys::WinHttpd;
//!
//! let mut server = WinHttpd::new();
//! server.serve("./www", 8080).expect("Failed to start server");
//!
//! // Server runs until stopped
//! server.stop().expect("Failed to stop server");
//! // Or server automatically stops when dropped
//! ```
//!
//! # Cross-Platform Usage
//!
//! For cross-platform code, use type aliasing:
//!
//! ```ignore
//! #[cfg(not(target_os = "windows"))]
//! use darkhttpd_sys::DarkHttpd as HttpServer;
//!
//! #[cfg(target_os = "windows")]
//! use winhttpd_sys::WinHttpd as HttpServer;
//!
//! fn start_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
//!     let mut server = HttpServer::new();
//!     server.serve("./www", port)?;
//!     Ok(())
//! }
//! ```
//!
//! # Safety
//!
//! This crate uses FFI to call C functions. All unsafe code is encapsulated
//! in the `ffi` module. The public API is safe to use.
//!
//! # Build Requirements
//!
//! - Windows with MSVC compiler
//! - `cc` crate for building C source
//! - Links against `ws2_32.lib` (Windows sockets)

#![allow(unsafe_code)] // Required for FFI

use std::ffi::NulError;

/// Errors that can occur when working with WinHttpd
#[derive(Debug, thiserror::Error)]
pub enum WinHttpdError {
    #[error("Failed to convert string to C string: {0}")]
    StringConversion(#[from] NulError),

    #[error("Initialization failed with code: {0}")]
    InitializationFailed(i32),

    #[error("Server is already initialized")]
    AlreadyInitialized,

    #[error("Server is not initialized")]
    NotInitialized,
}

/// Type alias for Results using WinHttpdError
pub type Result<T> = std::result::Result<T, WinHttpdError>;

mod ffi;

use std::ffi::CString;
use std::os::raw::c_int;
use std::ptr;

/// A safe wrapper around the winhttpd C library
///
/// This provides an idiomatic Rust interface to winhttpd while managing
/// the underlying C resources safely.
pub struct WinHttpd {
    initialized: bool,
    running: bool,
}

impl WinHttpd {
    /// Create a new WinHttpd instance
    pub fn new() -> Self {
        Self {
            initialized: false,
            running: false,
        }
    }

    /// Start serving files from the specified directory on the given port
    ///
    /// # Arguments
    /// * `path` - The directory to serve files from
    /// * `port` - The port to listen on
    pub fn serve(&mut self, path: &str, port: u16) -> Result<()> {
        if self.initialized {
            return Err(WinHttpdError::AlreadyInitialized);
        }

        let args = vec![
            CString::new("winhttpd")?,
            CString::new(path)?,
            CString::new("--port")?,
            CString::new(port.to_string())?,
        ];

        self.init_with_args(&args)?;
        self.start();

        Ok(())
    }

    /// Internal method to initialize with C strings
    fn init_with_args(&mut self, args: &[CString]) -> Result<()> {
        let mut argv: Vec<*mut c_int> = args
            .iter()
            .map(|s| s.as_ptr() as *mut c_int)
            .collect();
        argv.push(ptr::null_mut());

        let argc = args.len() as c_int;

        // SAFETY: We've constructed valid C strings and a null-terminated argv array
        let result = unsafe { ffi::winhttpd_init(argc, argv.as_mut_ptr() as *mut *mut _) };

        if result != 0 {
            return Err(WinHttpdError::InitializationFailed(result));
        }

        self.initialized = true;
        Ok(())
    }

    /// Start the server (begin accepting connections)
    pub fn start(&mut self) {
        if self.initialized && !self.running {
            // SAFETY: We've verified initialization
            unsafe { ffi::winhttpd_start() };
            self.running = true;
        }
    }

    /// Stop the server (stop accepting new connections)
    pub fn stop(&mut self) -> Result<()> {
        if !self.running {
            return Err(WinHttpdError::NotInitialized);
        }

        // SAFETY: We've verified the server is running
        unsafe { ffi::winhttpd_stop() };
        self.running = false;

        Ok(())
    }

    /// Check if the server is currently running
    pub fn is_running(&self) -> bool {
        self.running
    }
}

impl Default for WinHttpd {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for WinHttpd {
    fn drop(&mut self) {
        if self.initialized {
            let _ = self.stop();
            // SAFETY: We've verified initialization and stopped the server
            unsafe { ffi::winhttpd_cleanup() };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_string_conversion() {
        let nul_error = std::ffi::CString::new("test\0string")
            .unwrap_err();
        let error: WinHttpdError = nul_error.into();
        assert!(error.to_string().contains("Failed to convert string"));
    }

    #[test]
    fn test_error_initialization_failed() {
        let error = WinHttpdError::InitializationFailed(-1);
        assert_eq!(error.to_string(), "Initialization failed with code: -1");
    }

    #[test]
    fn test_error_already_initialized() {
        let error = WinHttpdError::AlreadyInitialized;
        assert_eq!(error.to_string(), "Server is already initialized");
    }

    #[test]
    fn test_error_not_initialized() {
        let error = WinHttpdError::NotInitialized;
        assert_eq!(error.to_string(), "Server is not initialized");
    }

    #[test]
    fn test_winhttpd_new() {
        let server = WinHttpd::new();
        assert!(!server.initialized, "New server should not be initialized");
        assert!(!server.running, "New server should not be running");
    }

    #[test]
    fn test_winhttpd_default() {
        let server = WinHttpd::default();
        assert!(!server.is_running());
    }

    // These tests require Windows to run since they need the C code to compile
    #[test]
    #[cfg(target_os = "windows")]
    fn test_winhttpd_serve_and_stop() {
        let mut server = WinHttpd::new();
        // Use current directory for testing
        assert!(server.serve(".", 8080).is_ok());
        assert!(server.stop().is_ok());
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_winhttpd_double_serve_fails() {
        let mut server = WinHttpd::new();
        server.serve(".", 8080).ok();
        let result = server.serve(".", 8081);
        assert!(result.is_err(), "Double serve should fail");
        server.stop().ok();
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_winhttpd_stop_not_running() {
        let mut server = WinHttpd::new();
        let result = server.stop();
        assert!(result.is_err(), "Stop without start should fail");
    }
}
