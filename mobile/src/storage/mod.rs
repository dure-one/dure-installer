//! Storage layer for DNS caching
//!
//! This module provides persistence using `fsqlite`

pub mod diesel_schema;
pub mod migrations;
pub mod models;

// Re-export schema
pub use diesel_schema as schema;
