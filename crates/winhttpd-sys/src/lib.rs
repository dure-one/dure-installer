//! Rust FFI bindings for winhttpd - Windows HTTP server
//!
//! Provides safe Rust wrapper around winhttpd C library for serving
//! static files on Windows. API matches darkhttpd-sys for cross-platform compatibility.

#![allow(unsafe_code)] // Required for FFI

// Modules will be added in later tasks
