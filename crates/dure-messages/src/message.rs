//! Messaging operation message types

use asyncapi_rust::schemars::JsonSchema;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Send message request
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct MessageSendRequest {
    /// Channel ID to send to
    pub channel_id: String,
    /// Message content
    pub content: String,
    /// Optional reply to message ID
    pub reply_to: Option<String>,
    /// Optional file attachments (base64 encoded)
    pub attachments: Option<Vec<MessageAttachment>>,
}

/// Message attachment
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct MessageAttachment {
    /// File name
    pub filename: String,
    /// MIME type
    pub content_type: String,
    /// File data (base64 encoded)
    pub data: String,
    /// File size in bytes
    pub size: usize,
}

/// Message sent response
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct MessageSentResponse {
    /// Whether send was successful
    pub success: bool,
    /// Sent message ID
    pub message_id: Option<String>,
    /// Message details
    pub message: Option<MessageData>,
    /// Error if failed
    pub error: Option<String>,
}

/// Message data
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct MessageData {
    /// Message ID
    pub message_id: String,
    /// Channel ID
    pub channel_id: String,
    /// Author member ID
    pub author_id: String,
    /// Message content
    pub content: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Edited timestamp
    pub edited_at: Option<DateTime<Utc>>,
    /// Reply to message ID
    pub reply_to: Option<String>,
    /// Attachments
    pub attachments: Option<Vec<MessageAttachment>>,
    /// Reactions
    pub reactions: Option<Vec<MessageReaction>>,
}

/// Message reaction
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct MessageReaction {
    /// Emoji
    pub emoji: String,
    /// Count of reactions
    pub count: u32,
    /// Whether current user reacted
    pub me: bool,
}

/// List messages request
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct MessageListRequest {
    /// Channel ID
    pub channel_id: String,
    /// Number of messages to retrieve (default: 10)
    pub limit: Option<usize>,
    /// Message ID to start before (for pagination)
    pub before: Option<String>,
    /// Message ID to start after (for pagination)
    pub after: Option<String>,
}

/// List messages response
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct MessageListResponse {
    /// List of messages
    pub messages: Vec<MessageData>,
    /// Total count (if available)
    pub total: Option<usize>,
    /// Whether there are more messages
    pub has_more: bool,
}

/// Message received notification (broadcast to subscribers)
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct MessageReceivedNotification {
    /// Message data
    pub message: MessageData,
}

/// Edit message request
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct MessageEditRequest {
    /// Channel ID
    pub channel_id: String,
    /// Message ID to edit
    pub message_id: String,
    /// New content
    pub content: String,
}

/// Message edited notification
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct MessageEditedNotification {
    /// Edited message data
    pub message: MessageData,
}

/// Delete message request
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct MessageDeleteRequest {
    /// Channel ID
    pub channel_id: String,
    /// Message ID to delete
    pub message_id: String,
    /// Confirmation flag
    pub confirm: bool,
}

/// Message deleted notification
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct MessageDeletedNotification {
    /// Channel ID
    pub channel_id: String,
    /// Deleted message ID
    pub message_id: String,
    /// Who deleted the message
    pub deleted_by: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Reply to message request
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct MessageReplyRequest {
    /// Channel ID
    pub channel_id: String,
    /// Message ID to reply to
    pub message_id: String,
    /// Reply content
    pub content: String,
    /// Optional attachments
    pub attachments: Option<Vec<MessageAttachment>>,
}
