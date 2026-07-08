# AsyncAPI Message Library Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Eliminate code duplication by splitting asyncapi-gen into a focused message library (dure-messages) and documentation generation tool (dure-asyncapi-gen).

**Architecture:** Two-crate design where both mobile app and asyncapi-gen tool depend on a shared dure-messages library. Message types defined once, imported by both consumers.

**Tech Stack:** Rust (nightly), serde, schemars, asyncapi-rust, chrono

## Global Constraints

- Rust edition: 2024
- All workspace crates use `version.workspace = true`, `edition.workspace = true`
- No external crate publishing (publish = false)
- AsyncAPI documentation output must remain identical to baseline
- All existing tests must continue passing
- Build must succeed for all platforms (desktop, headless, WASM, Android)
- Follow TDD: write test → verify failure → implement → verify success → commit

---

### Task 1: Establish Baseline and TDD Infrastructure

**Files:**
- Create: `crates/dure-messages/tests/integration_test.rs`
- Create: `/tmp/test-baseline.txt` (baseline test results)
- Create: `/tmp/asyncapi-baseline.json` (baseline AsyncAPI output)

**Interfaces:**
- Consumes: None (starting point)
- Produces: Baseline files for comparison, failing integration test

- [ ] **Step 1: Save baseline test results**

```bash
cargo test --workspace > /tmp/test-baseline.txt 2>&1
echo "Baseline test results saved to /tmp/test-baseline.txt"
cat /tmp/test-baseline.txt | tail -20
```

Expected: Test summary shows current passing/failing state

- [ ] **Step 2: Save baseline AsyncAPI documentation**

```bash
cd crates/asyncapi-gen
cargo run > /dev/null 2>&1
cd ../..
cp docs/asyncapi.json /tmp/asyncapi-baseline.json
echo "Baseline AsyncAPI saved to /tmp/asyncapi-baseline.json"
wc -l /tmp/asyncapi-baseline.json
```

Expected: JSON file copied, shows line count

- [ ] **Step 3: Create dure-messages crate structure (will fail)**

```bash
mkdir -p crates/dure-messages/tests
mkdir -p crates/dure-messages/src
```

Expected: Directories created

- [ ] **Step 4: Write integration test (will fail initially)**

Create `crates/dure-messages/tests/integration_test.rs`:

```rust
//! Integration tests for message serialization and validation

use dure_messages::*;

#[test]
fn test_client_message_serialization() {
    // Test that ClientMessage can serialize/deserialize
    let msg = ClientMessage::AuthLogin(AuthLoginRequest {
        server_id: "test-server".to_string(),
        device_id: "test-device".to_string(),
        public_key: "test-pubkey".to_string(),
        device_info: None,
    });
    
    let json = serde_json::to_string(&msg).expect("should serialize");
    let deserialized: ClientMessage = serde_json::from_str(&json).expect("should deserialize");
    
    // Round-trip should succeed
    assert!(matches!(deserialized, ClientMessage::AuthLogin(_)));
}

#[test]
fn test_server_message_serialization() {
    // Test that ServerMessage can serialize/deserialize
    let msg = ServerMessage::AuthResponse(AuthResponse {
        success: true,
        session_id: Some("session-123".to_string()),
        device_id: Some("device-456".to_string()),
        error: None,
    });
    
    let json = serde_json::to_string(&msg).expect("should serialize");
    let deserialized: ServerMessage = serde_json::from_str(&json).expect("should deserialize");
    
    assert!(matches!(deserialized, ServerMessage::AuthResponse(_)));
}

#[test]
fn test_message_types_are_send_sync() {
    // Ensure types can be used in async contexts
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ClientMessage>();
    assert_send_sync::<ServerMessage>();
    assert_send_sync::<ErrorResponse>();
}

#[test]
fn test_json_schema_generation() {
    use schemars::schema_for;
    
    // Verify schemars integration works
    let schema = schema_for!(ClientMessage);
    assert!(schema.schema.metadata.is_some());
    
    let schema = schema_for!(ServerMessage);
    assert!(schema.schema.metadata.is_some());
}
```

- [ ] **Step 5: Verify test fails (crate doesn't exist yet)**

```bash
cd crates/dure-messages
cargo test 2>&1 | head -20
```

Expected: Error about missing Cargo.toml or missing crate

- [ ] **Step 6: Commit baseline files**

```bash
git add crates/dure-messages/tests/integration_test.rs
git commit -m "test: add failing integration tests for dure-messages crate

Tests verify message serialization, Send+Sync traits, and JSON schema
generation. Will pass once dure-messages library is created.

Related to asyncapi-gen refactor.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

Expected: Commit succeeds with test file

---

### Task 2: Create dure-messages Library Crate

**Files:**
- Create: `crates/dure-messages/Cargo.toml`
- Create: `crates/dure-messages/src/lib.rs`
- Copy: `crates/asyncapi-gen/src/messages/*.rs` → `crates/dure-messages/src/`
- Copy: `mobile/src/site/messages/mod.rs` → extract enums to `crates/dure-messages/src/lib.rs`

**Interfaces:**
- Consumes: Message type definitions from asyncapi-gen and mobile
- Produces: `dure_messages` crate exporting `ClientMessage`, `ServerMessage`, `ErrorResponse`, and all message types

- [ ] **Step 1: Write Cargo.toml**

Create `crates/dure-messages/Cargo.toml`:

```toml
[package]
name = "dure-messages"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
publish = false

[dependencies]
# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# JSON Schema generation
schemars = { version = "1.1", features = ["derive", "chrono04"] }

# AsyncAPI message trait
asyncapi-rust = "0.2"

# Date/time types
chrono = { version = "0.4", features = ["serde"] }
```

- [ ] **Step 2: Copy message type files**

```bash
cp crates/asyncapi-gen/src/messages/auth.rs crates/dure-messages/src/
cp crates/asyncapi-gen/src/messages/channel.rs crates/dure-messages/src/
cp crates/asyncapi-gen/src/messages/hosting.rs crates/dure-messages/src/
cp crates/asyncapi-gen/src/messages/member.rs crates/dure-messages/src/
cp crates/asyncapi-gen/src/messages/message.rs crates/dure-messages/src/
cp crates/asyncapi-gen/src/messages/order.rs crates/dure-messages/src/
cp crates/asyncapi-gen/src/messages/payment.rs crates/dure-messages/src/
cp crates/asyncapi-gen/src/messages/product.rs crates/dure-messages/src/
cp crates/asyncapi-gen/src/messages/review.rs crates/dure-messages/src/
echo "Copied 9 message type files"
```

Expected: 9 files copied

- [ ] **Step 3: Create lib.rs with ClientMessage and ServerMessage enums**

Read the enum definitions from `mobile/src/site/messages/mod.rs`:

```bash
head -220 mobile/src/site/messages/mod.rs | tail -160
```

Create `crates/dure-messages/src/lib.rs`:

```rust
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
    #[serde(rename = "auth.login")]
    AuthLogin(AuthLoginRequest),
    #[serde(rename = "auth.logout")]
    AuthLogout(AuthLogoutRequest),

    // WebAuthn Operations
    #[serde(rename = "webauthn.signup.begin")]
    WebAuthnSignupBegin(WebAuthnSignupBeginRequest),
    #[serde(rename = "webauthn.signup.finish")]
    WebAuthnSignupFinish(WebAuthnSignupFinishRequest),
    #[serde(rename = "webauthn.signin.begin")]
    WebAuthnSigninBegin(WebAuthnSigninBeginRequest),
    #[serde(rename = "webauthn.signin.finish")]
    WebAuthnSigninFinish(WebAuthnSigninFinishRequest),

    // Hosting Operations
    #[serde(rename = "hosting.init")]
    HostingInit(HostingInitRequest),
    #[serde(rename = "hosting.show")]
    HostingShow(HostingShowRequest),
    #[serde(rename = "hosting.select")]
    HostingSelect(HostingSelectRequest),
    #[serde(rename = "hosting.list")]
    HostingList(HostingListRequest),
    #[serde(rename = "hosting.close")]
    HostingClose(HostingCloseRequest),

    // Member Operations
    #[serde(rename = "member.list")]
    MemberList(MemberListRequest),
    #[serde(rename = "member.info")]
    MemberInfo(MemberInfoRequest),
    #[serde(rename = "member.kick")]
    MemberKick(MemberKickRequest),
    #[serde(rename = "member.ban")]
    MemberBan(MemberBanRequest),

    // Channel Operations
    #[serde(rename = "channel.list")]
    ChannelList(ChannelListRequest),
    #[serde(rename = "channel.info")]
    ChannelInfo(ChannelInfoRequest),
    #[serde(rename = "channel.create")]
    ChannelCreate(ChannelCreateRequest),
    #[serde(rename = "channel.edit")]
    ChannelEdit(ChannelEditRequest),
    #[serde(rename = "channel.delete")]
    ChannelDelete(ChannelDeleteRequest),

    // Message Operations
    #[serde(rename = "message.send")]
    MessageSend(MessageSendRequest),
    #[serde(rename = "message.list")]
    MessageList(MessageListRequest),
    #[serde(rename = "message.edit")]
    MessageEdit(MessageEditRequest),
    #[serde(rename = "message.delete")]
    MessageDelete(MessageDeleteRequest),
    #[serde(rename = "message.reply")]
    MessageReply(MessageReplyRequest),

    // Product Operations
    #[serde(rename = "product.create")]
    ProductCreate(ProductCreateRequest),
    #[serde(rename = "product.list")]
    ProductList(ProductListRequest),
    #[serde(rename = "product.modify")]
    ProductModify(ProductModifyRequest),
    #[serde(rename = "product.delete")]
    ProductDelete(ProductDeleteRequest),

    // Order Operations
    #[serde(rename = "order.create")]
    OrderCreate(OrderCreateRequest),
    #[serde(rename = "order.list")]
    OrderList(OrderListRequest),

    // Payment Operations
    #[serde(rename = "payment.create")]
    PaymentCreate(PaymentCreateRequest),
    #[serde(rename = "payment.verify")]
    PaymentVerify(PaymentVerifyRequest),
    #[serde(rename = "payment.list")]
    PaymentList(PaymentListRequest),

    // Review Operations
    #[serde(rename = "review.create")]
    ReviewCreate(ReviewCreateRequest),
    #[serde(rename = "review.list")]
    ReviewList(ReviewListRequest),
}

/// All server-to-client messages
#[derive(Serialize, Deserialize, JsonSchema, ToAsyncApiMessage)]
#[serde(tag = "type")]
pub enum ServerMessage {
    // Authentication Responses
    #[serde(rename = "auth.response")]
    AuthResponse(AuthResponse),
    #[serde(rename = "auth.logout.response")]
    AuthLogoutResponse(AuthLogoutResponse),

    // WebAuthn Responses
    #[serde(rename = "webauthn.signup.begin.response")]
    WebAuthnSignupBeginResponse(WebAuthnSignupBeginResponse),
    #[serde(rename = "webauthn.signup.finish.response")]
    WebAuthnSignupFinishResponse(WebAuthnSignupFinishResponse),
    #[serde(rename = "webauthn.signin.begin.response")]
    WebAuthnSigninBeginResponse(WebAuthnSigninBeginResponse),
    #[serde(rename = "webauthn.signin.finish.response")]
    WebAuthnSigninFinishResponse(WebAuthnSigninFinishResponse),

    // Hosting Responses
    #[serde(rename = "hosting.init.response")]
    HostingInitResponse(HostingInitResponse),
    #[serde(rename = "hosting.show.response")]
    HostingShowResponse(HostingShowResponse),
    #[serde(rename = "hosting.select.response")]
    HostingSelectResponse(HostingSelectResponse),
    #[serde(rename = "hosting.list.response")]
    HostingListResponse(HostingListResponse),

    // Member Responses
    #[serde(rename = "member.list.response")]
    MemberListResponse(MemberListResponse),
    #[serde(rename = "member.info.response")]
    MemberInfoResponse(MemberInfoResponse),
    #[serde(rename = "member.kicked")]
    MemberKicked(MemberKickedNotification),
    #[serde(rename = "member.banned")]
    MemberBanned(MemberBannedNotification),

    // Channel Responses
    #[serde(rename = "channel.list.response")]
    ChannelListResponse(ChannelListResponse),
    #[serde(rename = "channel.info.response")]
    ChannelInfoResponse(ChannelInfoResponse),
    #[serde(rename = "channel.created")]
    ChannelCreated(ChannelCreatedNotification),
    #[serde(rename = "channel.edited")]
    ChannelEdited(ChannelEditedNotification),
    #[serde(rename = "channel.deleted")]
    ChannelDeleted(ChannelDeletedNotification),

    // Message Responses
    #[serde(rename = "message.sent")]
    MessageSent(MessageSentResponse),
    #[serde(rename = "message.list.response")]
    MessageListResponse(MessageListResponse),
    #[serde(rename = "message.received")]
    MessageReceived(MessageReceivedNotification),
    #[serde(rename = "message.edited")]
    MessageEdited(MessageEditedNotification),
    #[serde(rename = "message.deleted")]
    MessageDeleted(MessageDeletedNotification),

    // Product Responses
    #[serde(rename = "product.created")]
    ProductCreated(ProductCreatedResponse),
    #[serde(rename = "product.list.response")]
    ProductListResponse(ProductListResponse),
    #[serde(rename = "product.modified")]
    ProductModified(ProductModifiedNotification),
    #[serde(rename = "product.deleted")]
    ProductDeleted(ProductDeletedNotification),

    // Order Responses
    #[serde(rename = "order.created")]
    OrderCreated(OrderCreatedResponse),
    #[serde(rename = "order.list.response")]
    OrderListResponse(OrderListResponse),
    #[serde(rename = "order.status.update")]
    OrderStatusUpdate(OrderStatusUpdateNotification),

    // Payment Responses
    #[serde(rename = "payment.created")]
    PaymentCreated(PaymentCreatedResponse),
    #[serde(rename = "payment.verified")]
    PaymentVerified(PaymentVerifiedResponse),
    #[serde(rename = "payment.list.response")]
    PaymentListResponse(PaymentListResponse),

    // Review Responses
    #[serde(rename = "review.created")]
    ReviewCreated(ReviewCreatedResponse),
    #[serde(rename = "review.list.response")]
    ReviewListResponse(ReviewListResponse),

    // Error Response
    #[serde(rename = "error")]
    Error(ErrorResponse),

    // Server Notifications
    #[serde(rename = "server.ping")]
    ServerPing(ServerPingMessage),
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
```

- [ ] **Step 4: Verify compilation**

```bash
cd crates/dure-messages
cargo check
```

Expected: Compiles successfully with no warnings

- [ ] **Step 5: Run integration tests (should pass now)**

```bash
cargo test
```

Expected: All 4 tests pass (serialization, Send+Sync, JSON schema)

- [ ] **Step 6: Commit dure-messages library**

```bash
git add crates/dure-messages/
git commit -m "feat: create dure-messages library crate

Add new library crate for WebSocket message type definitions:
- Message type modules (auth, channel, hosting, member, message, order,
  payment, product, review)
- ClientMessage and ServerMessage enums with serde discriminator
- ErrorResponse and common types
- Integration tests for serialization and traits

This eliminates duplication between asyncapi-gen and mobile crates.

Tests: 4 passed

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

Expected: Commit succeeds

---

### Task 3: Refactor dure-asyncapi-gen Crate

**Files:**
- Modify: `crates/asyncapi-gen/Cargo.toml` → `crates/dure-asyncapi-gen/Cargo.toml`
- Move: `mobile/src/asyncapi_spec.rs` → `crates/dure-asyncapi-gen/src/asyncapi_spec.rs`
- Modify: `crates/dure-asyncapi-gen/src/lib.rs`
- Modify: `crates/dure-asyncapi-gen/src/main.rs`
- Delete: `crates/dure-asyncapi-gen/src/messages/` (entire directory)

**Interfaces:**
- Consumes: `dure_messages::{ClientMessage, ServerMessage}` from Task 2
- Produces: `dure_asyncapi_gen::asyncapi_spec::DureApi` struct, AsyncAPI doc generator binary

- [ ] **Step 1: Rename crate directory**

```bash
mv crates/asyncapi-gen crates/dure-asyncapi-gen
echo "Crate renamed to dure-asyncapi-gen"
```

Expected: Directory renamed

- [ ] **Step 2: Update Cargo.toml**

Edit `crates/dure-asyncapi-gen/Cargo.toml`:

```toml
[package]
name = "dure-asyncapi-gen"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
publish = false

[[bin]]
name = "dure-asyncapi-gen"
path = "src/main.rs"

[dependencies]
# Message types
dure-messages = { path = "../dure-messages" }

# AsyncAPI spec generation
asyncapi-rust = "0.2"
schemars = { version = "1.1", features = ["derive", "chrono04"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"

# Error handling
anyhow = "1.0"
```

Remove the `[workspace]` section entirely.

- [ ] **Step 3: Move asyncapi_spec.rs from mobile**

```bash
cp mobile/src/asyncapi_spec.rs crates/dure-asyncapi-gen/src/asyncapi_spec.rs
echo "Copied asyncapi_spec.rs"
```

Expected: File copied

- [ ] **Step 4: Update import in asyncapi_spec.rs**

Edit `crates/dure-asyncapi-gen/src/asyncapi_spec.rs`:

Find line:
```rust
use crate::site::messages::{ClientMessage, ServerMessage};
```

Replace with:
```rust
use dure_messages::{ClientMessage, ServerMessage};
```

- [ ] **Step 5: Update lib.rs to export asyncapi_spec**

Edit `crates/dure-asyncapi-gen/src/lib.rs`:

```rust
//! Dure AsyncAPI specification generator
//!
//! This crate contains the AsyncAPI specification for the Dure distributed
//! e-commerce platform and a binary tool to generate documentation.

pub mod asyncapi_spec;

pub use asyncapi_spec::DureApi;
```

- [ ] **Step 6: Update main.rs import**

Edit `crates/dure-asyncapi-gen/src/main.rs`:

Find line:
```rust
use dure_asyncapi_gen::DureApi;
```

Replace with:
```rust
use dure_asyncapi_gen::asyncapi_spec::DureApi;
```

- [ ] **Step 7: Delete old messages directory**

```bash
rm -rf crates/dure-asyncapi-gen/src/messages
echo "Deleted old messages directory"
```

Expected: Directory removed

- [ ] **Step 8: Verify compilation**

```bash
cd crates/dure-asyncapi-gen
cargo check
```

Expected: Compiles successfully

- [ ] **Step 9: Test documentation generation**

```bash
cargo run
```

Expected: Generates `docs/asyncapi.json` and `docs/asyncapi.yaml` with success messages

- [ ] **Step 10: Verify generated docs match baseline**

```bash
diff /tmp/asyncapi-baseline.json ../../docs/asyncapi.json
```

Expected: No differences (files are identical)

- [ ] **Step 11: Commit refactored asyncapi-gen**

```bash
git add crates/dure-asyncapi-gen/
git commit -m "refactor: convert asyncapi-gen to use dure-messages library

Changes:
- Rename crate from asyncapi-gen to dure-asyncapi-gen
- Remove standalone [workspace] declaration
- Add dependency on dure-messages library
- Move asyncapi_spec.rs from mobile to this crate
- Update imports to use dure_messages::
- Delete duplicate messages/ directory
- Update lib.rs to export asyncapi_spec module

Documentation generation verified: output matches baseline.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

Expected: Commit succeeds

---

### Task 4: Update Workspace Configuration

**Files:**
- Modify: `Cargo.toml` (workspace root)

**Interfaces:**
- Consumes: `dure-messages` and `dure-asyncapi-gen` crates from Tasks 2 and 3
- Produces: Updated workspace with both crates as members

- [ ] **Step 1: Update workspace members**

Edit `Cargo.toml` at workspace root:

Find the `[workspace]` section with `members = [...]` and `exclude = [...]`:

```toml
[workspace]
resolver = "2"
members = [
    "mobile",
    "crates/dure-messages",
    "crates/dure-asyncapi-gen",
    "crates/darkhttpd-sys",
    "crates/winhttpd-sys",
    "crates/go-webauthn-client",
    "crates/windows-installer",
]
exclude = [
    "crates/go-webauthn",
]
```

Changes:
- Add `"crates/dure-messages"` to members
- Add `"crates/dure-asyncapi-gen"` to members  
- Remove `"crates/asyncapi-gen"` from exclude list (it's now in members)

- [ ] **Step 2: Verify workspace configuration**

```bash
cargo metadata --format-version 1 | jq '.workspace_members' | grep -E "dure-messages|dure-asyncapi-gen"
```

Expected: Shows both crates as workspace members:
```
"dure-messages 0.0.1 ..."
"dure-asyncapi-gen 0.0.1 ..."
```

- [ ] **Step 3: Test workspace build**

```bash
cargo check --workspace
```

Expected: All workspace crates compile successfully

- [ ] **Step 4: Commit workspace configuration**

```bash
git add Cargo.toml
git commit -m "build: add dure-messages and dure-asyncapi-gen to workspace

Update workspace configuration:
- Add crates/dure-messages to members
- Add crates/dure-asyncapi-gen to members
- Remove crates/asyncapi-gen from exclude (now renamed and included)

Workspace check: passed

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

Expected: Commit succeeds

---

### Task 5: Update Mobile Crate

**Files:**
- Modify: `mobile/Cargo.toml`
- Modify: `mobile/src/wss/server/handlers/auth.rs`
- Modify: `mobile/src/wss/server/handlers/mod.rs`
- Modify: `mobile/src/site/mod.rs`
- Delete: `mobile/src/site/messages/` (entire directory)
- Delete: `mobile/src/asyncapi_spec.rs`

**Interfaces:**
- Consumes: `dure_messages::*` from Task 2
- Produces: Mobile crate using library imports instead of local message definitions

- [ ] **Step 1: Add dure-messages dependency**

Edit `mobile/Cargo.toml`, add to `[dependencies]` section:

```toml
# WebSocket message types
dure-messages = { path = "../crates/dure-messages" }
```

- [ ] **Step 2: Remove unused dependencies**

Edit `mobile/Cargo.toml`, find and remove these lines:

```toml
asyncapi-rust = "0.2"
schemars = { version = "1.2", features = ["derive", "chrono04"] }
```

Note: Only remove if they appear in the dependencies. Keep `serde_yaml` if it's used elsewhere.

- [ ] **Step 3: Find all files importing site::messages**

```bash
grep -r "use crate::site::messages" mobile/src/ | cut -d: -f1 | sort -u
```

Expected: Shows files that need import updates:
```
mobile/src/asyncapi_spec.rs
mobile/src/wss/server/handlers/auth.rs
mobile/src/wss/server/handlers/mod.rs
```

- [ ] **Step 4: Update import in wss/server/handlers/auth.rs**

Edit `mobile/src/wss/server/handlers/auth.rs`:

Find:
```rust
use crate::site::messages::{
    AuthLoginRequest, AuthResponse, DeviceInfo,
    WebAuthnSigninBeginRequest, WebAuthnSigninBeginResponse,
    WebAuthnSigninFinishRequest, WebAuthnSigninFinishResponse,
    WebAuthnSignupBeginRequest, WebAuthnSignupBeginResponse,
    WebAuthnSignupFinishRequest, WebAuthnSignupFinishResponse,
};
```

Replace with:
```rust
use dure_messages::{
    AuthLoginRequest, AuthResponse, DeviceInfo,
    WebAuthnSigninBeginRequest, WebAuthnSigninBeginResponse,
    WebAuthnSigninFinishRequest, WebAuthnSigninFinishResponse,
    WebAuthnSignupBeginRequest, WebAuthnSignupBeginResponse,
    WebAuthnSignupFinishRequest, WebAuthnSignupFinishResponse,
};
```

- [ ] **Step 5: Update import in wss/server/handlers/mod.rs**

Edit `mobile/src/wss/server/handlers/mod.rs`:

Find:
```rust
use crate::site::messages::{ClientMessage, ErrorResponse, ServerMessage};
```

Replace with:
```rust
use dure_messages::{ClientMessage, ErrorResponse, ServerMessage};
```

- [ ] **Step 6: Delete mobile/src/asyncapi_spec.rs**

```bash
rm mobile/src/asyncapi_spec.rs
echo "Deleted mobile/src/asyncapi_spec.rs (moved to dure-asyncapi-gen)"
```

Expected: File removed

- [ ] **Step 7: Delete mobile/src/site/messages/ directory**

```bash
rm -rf mobile/src/site/messages
echo "Deleted mobile/src/site/messages/ (duplicates removed)"
```

Expected: Directory and all 10 message files removed

- [ ] **Step 8: Update mobile/src/site/mod.rs**

Edit `mobile/src/site/mod.rs`:

Find and remove line:
```rust
pub mod messages;
```

If the file only contains this line, leave it empty or with just module-level docs.

- [ ] **Step 9: Verify no remaining site::messages imports**

```bash
grep -r "site::messages" mobile/src/
```

Expected: No results (all references removed)

- [ ] **Step 10: Verify mobile crate compiles**

```bash
cd mobile
cargo check
```

Expected: Compiles successfully with no errors

- [ ] **Step 11: Run mobile tests**

```bash
cargo test
```

Expected: All tests pass (same as baseline)

- [ ] **Step 12: Commit mobile crate changes**

```bash
git add mobile/
git commit -m "refactor: migrate mobile crate to use dure-messages library

Changes:
- Add dure-messages dependency
- Remove asyncapi-rust and schemars (only used in deleted files)
- Update all imports: crate::site::messages -> dure_messages
- Delete mobile/src/site/messages/ (10 files, duplicates removed)
- Delete mobile/src/asyncapi_spec.rs (moved to dure-asyncapi-gen)
- Update site/mod.rs to remove messages module

Verified: cargo check passes, tests pass

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

Expected: Commit succeeds

---

### Task 6: Final Verification and Cleanup

**Files:**
- None (verification only)

**Interfaces:**
- Consumes: All completed tasks
- Produces: Verified refactor with no regressions

- [ ] **Step 1: Build entire workspace**

```bash
cargo build --workspace
```

Expected: All crates build successfully with no warnings

- [ ] **Step 2: Run all workspace tests**

```bash
cargo test --workspace > /tmp/test-final.txt 2>&1
cat /tmp/test-final.txt | tail -20
```

Expected: All tests pass

- [ ] **Step 3: Compare test results with baseline**

```bash
diff <(grep "test result:" /tmp/test-baseline.txt) <(grep "test result:" /tmp/test-final.txt) || echo "Test counts may differ due to new dure-messages tests"
```

Expected: Same number of passing tests (or more, due to new integration tests)

- [ ] **Step 4: Generate AsyncAPI documentation**

```bash
cargo run -p dure-asyncapi-gen
```

Expected: Generates docs successfully with output:
```
🚀 Generating AsyncAPI specification for Dure WebSocket API...
✓ Generated docs/asyncapi.json
✓ Generated docs/asyncapi.yaml
```

- [ ] **Step 5: Verify AsyncAPI output matches baseline**

```bash
diff /tmp/asyncapi-baseline.json docs/asyncapi.json
echo "Diff result: $?"
```

Expected: Exit code 0 (files are identical)

- [ ] **Step 6: Verify no stale imports remain**

```bash
grep -r "site::messages" . --include="*.rs" 2>/dev/null | grep -v target | grep -v ".git"
```

Expected: No results (all old imports replaced)

- [ ] **Step 7: Test platform-specific builds**

```bash
cargo build --bin dure-desktop
cargo build --bin dure-desktop --no-default-features
```

Expected: Both builds succeed (desktop GUI and headless)

- [ ] **Step 8: Verify crate structure**

```bash
echo "=== dure-messages files ==="
find crates/dure-messages/src -name "*.rs" | wc -l
echo "Expected: 10 (lib.rs + 9 message modules)"

echo "=== No duplicate messages in mobile ==="
test ! -d mobile/src/site/messages && echo "✓ Directory deleted" || echo "✗ Still exists"

echo "=== No duplicate asyncapi_spec in mobile ==="
test ! -f mobile/src/asyncapi_spec.rs && echo "✓ File deleted" || echo "✗ Still exists"
```

Expected: All checks pass

- [ ] **Step 9: Final commit**

```bash
git add -A
git status
# Verify no unexpected changes remain
git commit -m "chore: complete asyncapi-gen refactor verification

All verification steps passed:
- Workspace builds successfully
- All tests pass (baseline maintained)
- AsyncAPI documentation output identical to baseline
- No stale imports remain
- Platform-specific builds succeed
- Code duplication eliminated

Summary:
- Created: crates/dure-messages (10 files)
- Refactored: crates/dure-asyncapi-gen
- Updated: mobile crate imports
- Deleted: mobile/src/site/messages/ (10 files)
- Deleted: mobile/src/asyncapi_spec.rs

Single source of truth established for message types.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

Expected: Commit succeeds

- [ ] **Step 10: Display summary**

```bash
echo "
========================================
  AsyncAPI Message Library Refactor
========================================

✅ Refactor Complete

Summary:
--------
• Created dure-messages library (10 message modules)
• Refactored dure-asyncapi-gen to use library
• Updated mobile crate to import from library
• Eliminated all code duplication
• All tests passing
• AsyncAPI documentation output verified identical

Metrics:
--------
• Files deleted: 11 (mobile/src/site/messages/* + asyncapi_spec.rs)
• Files created: 11 (crates/dure-messages/src/*)
• Crates added to workspace: 2
• Code duplication: 100% eliminated

Next Steps:
-----------
• Run 'cargo run -p dure-asyncapi-gen' to regenerate docs
• Future message changes only need edits in crates/dure-messages
• Both mobile and asyncapi-gen automatically use updated types
"
```

Expected: Success summary displayed

---

## Self-Review Checklist

**Spec Coverage:**
- ✅ Task 1: Baseline establishment (Spec: Testing Strategy)
- ✅ Task 2: dure-messages library creation (Spec: Step 1)
- ✅ Task 3: dure-asyncapi-gen refactor (Spec: Step 2)
- ✅ Task 4: Workspace configuration (Spec: Step 3)
- ✅ Task 5: Mobile crate updates (Spec: Step 4)
- ✅ Task 6: Final verification (Spec: Step 5)

**Placeholder Scan:**
- ✅ No TBD, TODO, or "implement later"
- ✅ All code blocks complete
- ✅ All commands show expected output
- ✅ All file paths are absolute and exact

**Type Consistency:**
- ✅ `ClientMessage` and `ServerMessage` used consistently across tasks
- ✅ `dure_messages::` import path consistent
- ✅ `dure-messages` (crate name) vs `dure_messages` (import path) correct
- ✅ File paths match between tasks (e.g., asyncapi_spec.rs movement)

**Execution Flow:**
- ✅ Each task builds on previous tasks (proper dependency order)
- ✅ TDD pattern followed: test → fail → implement → pass → commit
- ✅ Integration tests written before implementation (Task 1)
- ✅ Commits are atomic and well-described
- ✅ Verification steps included in each task
