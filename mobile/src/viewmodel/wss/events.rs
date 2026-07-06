//! WSS actor events

#[derive(Debug, Clone)]
pub enum WssEvent {
    Error { operation: String, error: String },
}
