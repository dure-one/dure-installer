//! Chat service layer with deltachat-core integration
//!
//! Provides email-based chat functionality via async-compat bridge

pub mod error;
pub mod protocol;

pub use error::{ServiceError, Result};
pub use protocol::{ChatEvent, Chat, Message};
