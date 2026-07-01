//! Raw FFI bindings to winhttpd C functions
//!
//! This module contains unsafe extern "C" declarations.
//! Consumers should use the safe wrapper in lib.rs instead.

#![allow(dead_code)] // FFI functions used via unsafe blocks

use std::os::raw::{c_char, c_int};

// FFI functions will be added in Task 4
