//! # service - Chat Service Layer
//!
//! Email-based chat service using deltachat-core with async-compat bridge.
//!
//! ## Features
//!
//! - deltachat-core integration (SMTP/IMAP)
//! - async-compat bridge (tokio ↔ smol)
//! - Event broadcasting
//! - End-to-end encryption (rPGP, Autocrypt)
//!
//! ## Example
//!
//! ```rust,no_run
//! use service::ChatService;
//!
//! #[smol::main]
//! async fn main() {
//!     let service = ChatService::new("./chat.db".into())
//!         .await
//!         .unwrap();
//!
//!     let mut events = service.subscribe_events();
//!     while let Ok(event) = events.recv().await {
//!         println!("Event: {:?}", event);
//!     }
//! }
//! ```

pub mod error;
pub mod protocol;
pub mod deltachat_bridge;
pub mod chat_service;

pub use error::{ServiceError, Result};
pub use protocol::{ChatEvent, Chat, Message};
pub use deltachat_bridge::DeltachatBridge;
pub use chat_service::ChatService;
