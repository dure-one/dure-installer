//! # chat - Chat Service Layer
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
//! ```rust,ignore
//! use chat::ChatService;
//!
//! smol::block_on(async {
//!     let chat = ChatService::new("./chat.db".into())
//!         .await
//!         .unwrap();
//!
//!     let mut events = chat.subscribe_events();
//!     while let Ok(event) = events.recv().await {
//!         println!("Event: {:?}", event);
//!     }
//! });
//! ```

pub mod error;
pub mod protocol;
pub mod deltachat_bridge;
pub mod chat_service;

pub use error::{ChatError, Result};
pub use protocol::{ChatEvent, Chat, Message};
pub use deltachat_bridge::DeltachatBridge;
pub use chat_service::ChatService;
