//! Chat service layer with deltachat-core integration
//!
//! Provides email-based chat functionality via async-compat bridge

pub mod error;
pub mod protocol;
pub mod deltachat_bridge;
pub mod chat_service;

pub use error::{ServiceError, Result};
pub use protocol::{ChatEvent, Chat, Message};
pub use deltachat_bridge::DeltachatBridge;
pub use chat_service::ChatService;
