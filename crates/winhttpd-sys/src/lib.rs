//! Rust FFI bindings for winhttpd - Windows HTTP server
//!
//! Provides safe Rust wrapper around winhttpd C library for serving
//! static files on Windows. API matches darkhttpd-sys for cross-platform compatibility.

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
}
