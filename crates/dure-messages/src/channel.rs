//! Channel management message types

use asyncapi_rust::schemars::JsonSchema;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// List channels request
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ChannelListRequest {
    /// Optional server ID filter
    pub server_id: Option<String>,
}

/// List channels response
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ChannelListResponse {
    /// List of channels
    pub channels: Vec<ChannelInfo>,
    /// Total count
    pub total: usize,
}

/// Channel information
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ChannelInfo {
    /// Channel ID
    pub channel_id: String,
    /// Channel name
    pub name: String,
    /// Channel type
    pub channel_type: ChannelType,
    /// Server ID this channel belongs to
    pub server_id: String,
    /// Channel topic/description
    pub topic: Option<String>,
    /// Whether channel is NSFW
    pub is_nsfw: bool,
    /// Slowmode delay in seconds
    pub slowmode_secs: Option<u32>,
    /// Created timestamp
    pub created_at: Option<DateTime<Utc>>,
}

/// Channel type
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ChannelType {
    /// Text channel
    Text,
    /// Voice channel
    Voice,
    /// Forum channel
    Forum,
    /// Order channel (auto-created for orders)
    Order,
}

/// Get channel info request
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ChannelInfoRequest {
    /// Channel ID or name (#channel)
    pub channel_id: String,
}

/// Get channel info response
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ChannelInfoResponse {
    /// Channel information
    pub channel: Option<ChannelInfo>,
    /// Error message if not found
    pub error: Option<String>,
}

/// Create channel request
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ChannelCreateRequest {
    /// Server ID
    pub server_id: String,
    /// Channel name
    pub name: String,
    /// Channel type
    pub channel_type: ChannelType,
    /// Optional topic
    pub topic: Option<String>,
    /// Optional slowmode delay
    pub slowmode_secs: Option<u32>,
}

/// Channel created notification
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ChannelCreatedNotification {
    /// Created channel information
    pub channel: ChannelInfo,
    /// Who created the channel
    pub created_by: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Edit channel request
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ChannelEditRequest {
    /// Channel ID to edit
    pub channel_id: String,
    /// New name (optional)
    pub name: Option<String>,
    /// New topic (optional)
    pub topic: Option<String>,
    /// New slowmode delay (optional)
    pub slowmode_secs: Option<u32>,
}

/// Channel edited notification
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ChannelEditedNotification {
    /// Edited channel information
    pub channel: ChannelInfo,
    /// Who edited the channel
    pub edited_by: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Delete channel request
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ChannelDeleteRequest {
    /// Channel ID to delete
    pub channel_id: String,
    /// Confirmation flag
    pub confirm: bool,
}

/// Channel deleted notification
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ChannelDeletedNotification {
    /// Deleted channel ID
    pub channel_id: String,
    /// Server ID
    pub server_id: String,
    /// Who deleted the channel
    pub deleted_by: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}
