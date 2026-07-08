//! Member/directory management message types

use asyncapi_rust::schemars::JsonSchema;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// List members request
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct MemberListRequest {
    /// Server/hosting ID
    pub server_id: String,
    /// Maximum number of members to return
    pub limit: Option<usize>,
    /// Offset for pagination
    pub offset: Option<usize>,
}

/// List members response
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct MemberListResponse {
    /// List of members
    pub members: Vec<MemberInfo>,
    /// Total member count
    pub total: usize,
    /// Whether there are more members
    pub has_more: bool,
}

/// Member information
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct MemberInfo {
    /// Member ID (domain name for stores, device ID for owner devices)
    pub member_id: String,
    /// Member display name
    pub display_name: Option<String>,
    /// Member type (owner or store)
    pub member_type: MemberType,
    /// Roles assigned to this member
    pub roles: Vec<String>,
    /// Join date
    pub joined_at: Option<DateTime<Utc>>,
    /// Last seen timestamp
    pub last_seen: Option<DateTime<Utc>>,
    /// Whether member is online
    pub is_online: bool,
}

/// Member type
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum MemberType {
    /// Owner device member (no dots in ID)
    Owner,
    /// Store member (dots allowed in ID)
    Store,
}

/// Get member info request
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct MemberInfoRequest {
    /// Server/hosting ID
    pub server_id: String,
    /// Member ID or user mention (@username)
    pub member_id: String,
}

/// Get member info response
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct MemberInfoResponse {
    /// Member information
    pub member: Option<MemberInfo>,
    /// Error message if not found
    pub error: Option<String>,
}

/// Kick member request
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct MemberKickRequest {
    /// Server/hosting ID
    pub server_id: String,
    /// Member ID to kick
    pub member_id: String,
    /// Reason for kicking
    pub reason: Option<String>,
}

/// Member kicked notification
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct MemberKickedNotification {
    /// Server ID
    pub server_id: String,
    /// Kicked member ID
    pub member_id: String,
    /// Who kicked the member
    pub kicked_by: String,
    /// Reason
    pub reason: Option<String>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Ban member request
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct MemberBanRequest {
    /// Server/hosting ID
    pub server_id: String,
    /// Member ID to ban
    pub member_id: String,
    /// Reason for banning
    pub reason: Option<String>,
    /// Ban duration in seconds (None = permanent)
    pub duration_secs: Option<u64>,
}

/// Member banned notification
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct MemberBannedNotification {
    /// Server ID
    pub server_id: String,
    /// Banned member ID
    pub member_id: String,
    /// Who banned the member
    pub banned_by: String,
    /// Reason
    pub reason: Option<String>,
    /// Ban expiry time (None = permanent)
    pub expires_at: Option<DateTime<Utc>>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}
