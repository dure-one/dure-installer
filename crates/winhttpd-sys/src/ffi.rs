//! Raw FFI bindings to winhttpd C functions
//!
//! This module contains unsafe extern "C" declarations.
//! Consumers should use the safe wrapper in lib.rs instead.

#![allow(non_camel_case_types)]
#![allow(dead_code)]

use std::os::raw::{c_char, c_int};

#[cfg(target_os = "windows")]
extern "C" {
    /// Initialize winhttpd with given command-line arguments
    ///
    /// # Safety
    /// - `argv` must be a valid pointer to an array of `argc` C strings
    /// - Each string in `argv` must be null-terminated
    /// - The strings must remain valid for the duration of the call
    pub fn winhttpd_init(argc: c_int, argv: *mut *mut c_char) -> c_int;

    /// Run one iteration of the poll loop
    ///
    /// # Safety
    /// - Must be called after `winhttpd_init`
    /// - Should only be called from a single thread
    pub fn winhttpd_poll_once();

    /// Start the server (sets the running flag)
    ///
    /// # Safety
    /// - Must be called after `winhttpd_init`
    pub fn winhttpd_start();

    /// Stop the server (clears the running flag)
    ///
    /// # Safety
    /// - Can be called at any time after `winhttpd_init`
    pub fn winhttpd_stop();

    /// Check if the server is running
    ///
    /// # Safety
    /// - Must be called after `winhttpd_init`
    pub fn winhttpd_is_running() -> c_int;

    /// Cleanup and shutdown winhttpd
    ///
    /// # Safety
    /// - Must be called after `winhttpd_init`
    /// - Should only be called once
    /// - No other winhttpd functions should be called after this
    pub fn winhttpd_cleanup();
}

// Stub implementations for non-Windows platforms (tests won't run but will compile)
#[cfg(not(target_os = "windows"))]
pub unsafe fn winhttpd_init(_argc: c_int, _argv: *mut *mut c_char) -> c_int {
    -1 // Always fail on non-Windows
}

#[cfg(not(target_os = "windows"))]
pub unsafe fn winhttpd_poll_once() {}

#[cfg(not(target_os = "windows"))]
pub unsafe fn winhttpd_start() {}

#[cfg(not(target_os = "windows"))]
pub unsafe fn winhttpd_stop() {}

#[cfg(not(target_os = "windows"))]
pub unsafe fn winhttpd_is_running() -> c_int {
    0
}

#[cfg(not(target_os = "windows"))]
pub unsafe fn winhttpd_cleanup() {}
