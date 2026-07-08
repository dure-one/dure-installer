//! Dure WebSocket protocol message types
//!
//! This crate defines all message types used in the Dure distributed
//! e-commerce platform's WebSocket communication protocol.

pub mod auth;
pub mod channel;
pub mod hosting;
pub mod member;
pub mod message;
pub mod order;
pub mod payment;
pub mod product;
pub mod review;

// Re-export all message types for convenience
pub use auth::*;
pub use channel::*;
pub use hosting::*;
pub use member::*;
pub use message::*;
pub use order::*;
pub use payment::*;
pub use product::*;
pub use review::*;

use asyncapi_rust::{ToAsyncApiMessage, schemars::JsonSchema};
use serde::{Deserialize, Serialize};

/// All client-to-server messages
#[derive(Serialize, Deserialize, JsonSchema, ToAsyncApiMessage)]
#[serde(tag = "type")]
pub enum ClientMessage {
    // Authentication
    /// Authenticate with device credentials
    #[serde(rename = "auth.login")]
    AuthLogin(AuthLoginRequest),

    /// Logout from server
    #[serde(rename = "auth.logout")]
    AuthLogout(AuthLogoutRequest),

    // WebAuthn Operations
    /// Begin WebAuthn registration
    #[serde(rename = "webauthn.signup.begin")]
    WebAuthnSignupBegin(WebAuthnSignupBeginRequest),

    /// Finish WebAuthn registration
    #[serde(rename = "webauthn.signup.finish")]
    WebAuthnSignupFinish(WebAuthnSignupFinishRequest),

    /// Begin WebAuthn authentication
    #[serde(rename = "webauthn.signin.begin")]
    WebAuthnSigninBegin(WebAuthnSigninBeginRequest),

    /// Finish WebAuthn authentication
    #[serde(rename = "webauthn.signin.finish")]
    WebAuthnSigninFinish(WebAuthnSigninFinishRequest),

    // Hosting Operations
    /// Initialize a hosting configuration
    #[serde(rename = "hosting.init")]
    HostingInit(HostingInitRequest),

    /// Show hosting details
    #[serde(rename = "hosting.show")]
    HostingShow(HostingShowRequest),

    /// Select hosting for operations
    #[serde(rename = "hosting.select")]
    HostingSelect(HostingSelectRequest),

    /// List all hostings
    #[serde(rename = "hosting.list")]
    HostingList(HostingListRequest),

    /// Close a hosting
    #[serde(rename = "hosting.close")]
    HostingClose(HostingCloseRequest),

    // Member Operations
    /// List members in a server
    #[serde(rename = "member.list")]
    MemberList(MemberListRequest),

    /// Get member information
    #[serde(rename = "member.info")]
    MemberInfo(MemberInfoRequest),

    /// Kick a member
    #[serde(rename = "member.kick")]
    MemberKick(MemberKickRequest),

    /// Ban a member
    #[serde(rename = "member.ban")]
    MemberBan(MemberBanRequest),

    // Channel Operations
    /// List all channels
    #[serde(rename = "channel.list")]
    ChannelList(ChannelListRequest),

    /// Get channel information
    #[serde(rename = "channel.info")]
    ChannelInfo(ChannelInfoRequest),

    /// Create a new channel
    #[serde(rename = "channel.create")]
    ChannelCreate(ChannelCreateRequest),

    /// Edit channel settings
    #[serde(rename = "channel.edit")]
    ChannelEdit(ChannelEditRequest),

    /// Delete a channel
    #[serde(rename = "channel.delete")]
    ChannelDelete(ChannelDeleteRequest),

    // Message Operations
    /// Send a message to a channel
    #[serde(rename = "message.send")]
    MessageSend(MessageSendRequest),

    /// List messages in a channel
    #[serde(rename = "message.list")]
    MessageList(MessageListRequest),

    /// Edit a message
    #[serde(rename = "message.edit")]
    MessageEdit(MessageEditRequest),

    /// Delete a message
    #[serde(rename = "message.delete")]
    MessageDelete(MessageDeleteRequest),

    /// Reply to a message
    #[serde(rename = "message.reply")]
    MessageReply(MessageReplyRequest),

    // Product Operations
    /// Create a new product
    #[serde(rename = "product.create")]
    ProductCreate(ProductCreateRequest),

    /// List products
    #[serde(rename = "product.list")]
    ProductList(ProductListRequest),

    /// Modify a product
    #[serde(rename = "product.modify")]
    ProductModify(ProductModifyRequest),

    /// Delete a product
    #[serde(rename = "product.delete")]
    ProductDelete(ProductDeleteRequest),

    // Order Operations
    /// Create a new order
    #[serde(rename = "order.create")]
    OrderCreate(OrderCreateRequest),

    /// List orders
    #[serde(rename = "order.list")]
    OrderList(OrderListRequest),

    // Payment Operations
    /// Create a payment
    #[serde(rename = "payment.create")]
    PaymentCreate(PaymentCreateRequest),

    /// Verify payment from gateway
    #[serde(rename = "payment.verify")]
    PaymentVerify(PaymentVerifyRequest),

    /// List payments
    #[serde(rename = "payment.list")]
    PaymentList(PaymentListRequest),

    // Review Operations
    /// Create a review
    #[serde(rename = "review.create")]
    ReviewCreate(ReviewCreateRequest),

    /// List reviews
    #[serde(rename = "review.list")]
    ReviewList(ReviewListRequest),
}

/// All server-to-client messages
#[derive(Serialize, Deserialize, JsonSchema, ToAsyncApiMessage)]
#[serde(tag = "type")]
pub enum ServerMessage {
    // Authentication Responses
    /// Authentication response
    #[serde(rename = "auth.response")]
    AuthResponse(AuthResponse),

    /// Logout confirmation
    #[serde(rename = "auth.logout.response")]
    AuthLogoutResponse(AuthLogoutResponse),

    // WebAuthn Responses
    /// WebAuthn registration begin response
    #[serde(rename = "webauthn.signup.begin.response")]
    WebAuthnSignupBeginResponse(WebAuthnSignupBeginResponse),

    /// WebAuthn registration finish response
    #[serde(rename = "webauthn.signup.finish.response")]
    WebAuthnSignupFinishResponse(WebAuthnSignupFinishResponse),

    /// WebAuthn authentication begin response
    #[serde(rename = "webauthn.signin.begin.response")]
    WebAuthnSigninBeginResponse(WebAuthnSigninBeginResponse),

    /// WebAuthn authentication finish response
    #[serde(rename = "webauthn.signin.finish.response")]
    WebAuthnSigninFinishResponse(WebAuthnSigninFinishResponse),

    // Hosting Responses
    /// Hosting initialization response
    #[serde(rename = "hosting.init.response")]
    HostingInitResponse(HostingInitResponse),

    /// Hosting show response
    #[serde(rename = "hosting.show.response")]
    HostingShowResponse(HostingShowResponse),

    /// Hosting select response
    #[serde(rename = "hosting.select.response")]
    HostingSelectResponse(HostingSelectResponse),

    /// Hosting list response
    #[serde(rename = "hosting.list.response")]
    HostingListResponse(HostingListResponse),

    // Member Responses
    /// Member list response
    #[serde(rename = "member.list.response")]
    MemberListResponse(MemberListResponse),

    /// Member info response
    #[serde(rename = "member.info.response")]
    MemberInfoResponse(MemberInfoResponse),

    /// Member kicked notification
    #[serde(rename = "member.kicked")]
    MemberKicked(MemberKickedNotification),

    /// Member banned notification
    #[serde(rename = "member.banned")]
    MemberBanned(MemberBannedNotification),

    // Channel Responses
    /// Channel list response
    #[serde(rename = "channel.list.response")]
    ChannelListResponse(ChannelListResponse),

    /// Channel info response
    #[serde(rename = "channel.info.response")]
    ChannelInfoResponse(ChannelInfoResponse),

    /// Channel created notification
    #[serde(rename = "channel.created")]
    ChannelCreated(ChannelCreatedNotification),

    /// Channel edited notification
    #[serde(rename = "channel.edited")]
    ChannelEdited(ChannelEditedNotification),

    /// Channel deleted notification
    #[serde(rename = "channel.deleted")]
    ChannelDeleted(ChannelDeletedNotification),

    // Message Responses
    /// Message sent confirmation
    #[serde(rename = "message.sent")]
    MessageSent(MessageSentResponse),

    /// Message list response
    #[serde(rename = "message.list.response")]
    MessageListResponse(MessageListResponse),

    /// Message received notification (broadcast)
    #[serde(rename = "message.received")]
    MessageReceived(MessageReceivedNotification),

    /// Message edited notification
    #[serde(rename = "message.edited")]
    MessageEdited(MessageEditedNotification),

    /// Message deleted notification
    #[serde(rename = "message.deleted")]
    MessageDeleted(MessageDeletedNotification),

    // Product Responses
    /// Product created response
    #[serde(rename = "product.created")]
    ProductCreated(ProductCreatedResponse),

    /// Product list response
    #[serde(rename = "product.list.response")]
    ProductListResponse(ProductListResponse),

    /// Product modified notification
    #[serde(rename = "product.modified")]
    ProductModified(ProductModifiedNotification),

    /// Product deleted notification
    #[serde(rename = "product.deleted")]
    ProductDeleted(ProductDeletedNotification),

    // Order Responses
    /// Order created response
    #[serde(rename = "order.created")]
    OrderCreated(OrderCreatedResponse),

    /// Order list response
    #[serde(rename = "order.list.response")]
    OrderListResponse(OrderListResponse),

    /// Order status update notification
    #[serde(rename = "order.status.update")]
    OrderStatusUpdate(OrderStatusUpdateNotification),

    // Payment Responses
    /// Payment created response
    #[serde(rename = "payment.created")]
    PaymentCreated(PaymentCreatedResponse),

    /// Payment verified response
    #[serde(rename = "payment.verified")]
    PaymentVerified(PaymentVerifiedResponse),

    /// Payment list response
    #[serde(rename = "payment.list.response")]
    PaymentListResponse(PaymentListResponse),

    // Review Responses
    /// Review created response
    #[serde(rename = "review.created")]
    ReviewCreated(ReviewCreatedResponse),

    /// Review list response
    #[serde(rename = "review.list.response")]
    ReviewListResponse(ReviewListResponse),

    // Error Response
    /// Generic error response
    #[serde(rename = "error")]
    Error(ErrorResponse),

    // Server Notifications
    /// Server-initiated ping
    #[serde(rename = "server.ping")]
    ServerPing(ServerPingMessage),

    /// Connection status update
    #[serde(rename = "connection.status")]
    ConnectionStatus(ConnectionStatusMessage),
}

/// Generic error response
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ErrorResponse {
    /// Error code
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Optional request ID that caused the error
    pub request_id: Option<String>,
    /// Additional error details
    pub details: Option<serde_json::Value>,
}

/// Server ping message for keepalive
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ServerPingMessage {
    /// Server timestamp
    pub timestamp: i64,
    /// Server ID
    pub server_id: String,
}

/// Connection status message
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ConnectionStatusMessage {
    /// Connection status
    pub status: ConnectionStatus,
    /// Session ID
    pub session_id: String,
    /// Message
    pub message: Option<String>,
}

/// Connection status enum
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionStatus {
    /// Connected
    Connected,
    /// Reconnecting
    Reconnecting,
    /// Disconnected
    Disconnected,
    /// Error
    Error,
}
