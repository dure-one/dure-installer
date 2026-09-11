//! Android platform integration modules
//!
//! Provides JNI bindings and Android-specific utilities.

#[cfg(target_os = "android")]
pub mod activity;

#[cfg(target_os = "android")]
pub mod clipboard;

#[cfg(target_os = "android")]
pub mod contexttheme;

#[cfg(target_os = "android")]
pub mod inputmethod;

#[cfg(target_os = "android")]
pub mod log;

#[cfg(target_os = "android")]
pub mod packagemanager;

#[cfg(target_os = "android")]
pub mod screensize;
