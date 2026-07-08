# AsyncAPI Message Library Refactor Design

**Date:** 2026-07-08  
**Author:** Claude Sonnet 4.5  
**Status:** Approved  

## Overview

Refactor the `crates/asyncapi-gen` crate to eliminate code duplication by splitting it into a focused message library and a documentation generation tool. Currently, message types are defined in `crates/asyncapi-gen/src/messages/*.rs` and duplicated in `mobile/src/site/messages/*.rs`, requiring manual synchronization.

## Goals

1. **Eliminate duplication** - Single source of truth for message type definitions
2. **Clean separation** - Library for types, binary for doc generation
3. **Improve maintainability** - Changes to message types happen in one place
4. **Enable reuse** - Message types can be used by any Rust project
5. **Preserve functionality** - Doc generation continues to work identically

## Non-Goals

- Client code generation in the mobile app (not needed)
- Runtime documentation generation (only manual generation via binary)
- Publishing to crates.io (internal use only)

## Current State

### Problems

1. **Code duplication**: Message types exist in two locations:
   - `crates/asyncapi-gen/src/messages/*.rs` (10 files)
   - `mobile/src/site/messages/*.rs` (10 identical files)

2. **Synchronization burden**: Any change requires updating both locations

3. **Workspace confusion**: `asyncapi-gen` is excluded from workspace but defines its own workspace

4. **Redundant AsyncAPI specs**: Both `crates/asyncapi-gen/src/asyncapi_spec.rs` and `mobile/src/asyncapi_spec.rs` define the same thing

### Current Structure

```
crates/asyncapi-gen/          # Excluded from workspace
├── Cargo.toml                # Has [workspace] declaration
├── src/
│   ├── lib.rs
│   ├── main.rs               # Doc generator binary
│   ├── asyncapi_spec.rs      # DureApi definition
│   └── messages/             # Message type definitions (10 files)
│       ├── mod.rs
│       ├── auth.rs
│       ├── channel.rs
│       └── ...

mobile/src/
├── asyncapi_spec.rs          # Duplicate DureApi definition
└── site/messages/            # Duplicate message definitions (10 files)
    ├── mod.rs
    ├── auth.rs
    ├── channel.rs
    └── ...
```

### Current Usage

- `mobile/src/wss/server/handlers/*.rs` - Import from `crate::site::messages`
- `mobile/src/asyncapi_spec.rs` - Defines `DureApi` with tests
- `crates/asyncapi-gen/src/main.rs` - Generates `docs/asyncapi.{json,yaml}`

## Proposed Architecture

Split `crates/asyncapi-gen` into two focused crates:

### 1. `crates/dure-messages` (Library)

**Purpose:** Pure message type definitions

**Responsibilities:**
- Define all WebSocket message types
- Provide `ClientMessage` and `ServerMessage` enums
- Export types for use by any Rust project
- Zero binary/codegen code

**Dependencies:** Minimal
- `serde` - Serialization
- `schemars` - JSON Schema generation
- `asyncapi-rust` - AsyncAPI message trait
- `chrono` - Date/time types

### 2. `crates/dure-asyncapi-gen` (Binary Tool)

**Purpose:** AsyncAPI documentation generator

**Responsibilities:**
- Define `DureApi` struct with AsyncAPI attributes
- Generate `docs/asyncapi.{json,yaml}` files
- Schema reference fixing and post-processing
- Standalone CLI tool

**Dependencies:**
- `dure-messages` - Message type definitions
- `asyncapi-rust` - Spec generation
- `serde_json`, `serde_yaml` - Serialization
- `anyhow` - Error handling

### Dependency Flow

```
mobile (app) ──────┐
                   ├──> dure-messages (lib)
dure-asyncapi-gen  │
    (bin tool)  ───┘
```

**Key insight:** Both consumers depend on the library, but neither knows about the other. Clean separation of concerns.

## Detailed Design

### Crate Structure

#### New: `crates/dure-messages/`

```
crates/dure-messages/
├── Cargo.toml                 # Library-only, no binary
├── src/
│   ├── lib.rs                 # Public API with re-exports
│   ├── auth.rs               # Authentication messages
│   ├── channel.rs            # Channel management messages
│   ├── hosting.rs            # Hosting operation messages
│   ├── member.rs             # Member management messages
│   ├── message.rs            # Chat message types
│   ├── order.rs              # Order management messages
│   ├── payment.rs            # Payment integration messages
│   ├── product.rs            # Product catalog messages
│   └── review.rs             # Review system messages
```

**`lib.rs` structure:**
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
    // ... (moved from mobile/src/site/messages/mod.rs)
}

/// All server-to-client messages
#[derive(Serialize, Deserialize, JsonSchema, ToAsyncApiMessage)]
#[serde(tag = "type")]
pub enum ServerMessage {
    // ... (moved from mobile/src/site/messages/mod.rs)
}

/// Generic error response
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
    pub request_id: Option<String>,
    pub details: Option<serde_json::Value>,
}

// ... (other common types)
```

#### Refactored: `crates/dure-asyncapi-gen/`

```
crates/dure-asyncapi-gen/
├── Cargo.toml                 # Binary with dure-messages dependency
├── src/
│   ├── main.rs               # Doc generation (unchanged logic)
│   └── asyncapi_spec.rs      # DureApi struct (moved from mobile)
```

**`asyncapi_spec.rs` changes:**
```rust
// BEFORE:
use crate::site::messages::{ClientMessage, ServerMessage};

// AFTER:
use dure_messages::{ClientMessage, ServerMessage};

// Rest of DureApi definition stays the same
```

**`main.rs` changes:**
```rust
// BEFORE:
use dure_asyncapi_gen::DureApi;

// AFTER:
use dure_asyncapi_gen::asyncapi_spec::DureApi;

// Rest of main() stays the same
```

### Cargo Configuration

#### `crates/dure-messages/Cargo.toml`

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

#### `crates/dure-asyncapi-gen/Cargo.toml`

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

#### Workspace `Cargo.toml`

```toml
[workspace]
resolver = "2"
members = [
    "mobile",
    "crates/dure-messages",        # ✅ NEW
    "crates/dure-asyncapi-gen",    # ✅ ADDED (was excluded)
    "crates/darkhttpd-sys",
    "crates/winhttpd-sys",
    "crates/go-webauthn-client",
    "crates/windows-installer",
]
exclude = [
    "crates/go-webauthn",          # ✅ asyncapi-gen removed from exclude
]

# ... rest unchanged
```

#### `mobile/Cargo.toml`

```toml
[dependencies]
# ... existing dependencies ...

# WebSocket message types
dure-messages = { path = "../crates/dure-messages" }

# ❌ REMOVE these (only used in files being deleted):
# asyncapi-rust = "0.2"  
# schemars = { version = "1.2", features = ["derive", "chrono04"] }
```

**Note:** Both `asyncapi-rust` and `schemars` are only used in `mobile/src/asyncapi_spec.rs` and `mobile/src/site/messages/*`, which are being deleted. They can be safely removed from mobile's dependencies.

### Import Changes

All files in `mobile/src/` that currently import from `crate::site::messages` will change:

**Find pattern:** `use crate::site::messages`  
**Replace with:** `use dure_messages`

**Examples:**

```rust
// mobile/src/wss/server/handlers/auth.rs
// BEFORE:
use crate::site::messages::{
    AuthLoginRequest, AuthResponse, DeviceInfo,
    WebAuthnSigninBeginRequest, WebAuthnSigninBeginResponse,
};

// AFTER:
use dure_messages::{
    AuthLoginRequest, AuthResponse, DeviceInfo,
    WebAuthnSigninBeginRequest, WebAuthnSigninBeginResponse,
};
```

```rust
// mobile/src/wss/server/handlers/mod.rs
// BEFORE:
use crate::site::messages::{ClientMessage, ErrorResponse, ServerMessage};

// AFTER:
use dure_messages::{ClientMessage, ErrorResponse, ServerMessage};
```

### Files to Delete

**Complete removal:**
1. `mobile/src/site/messages/` - Entire directory (10 files)
2. `mobile/src/asyncapi_spec.rs` - Moved to asyncapi-gen
3. Update `mobile/src/site/mod.rs` if it references the messages module

## Migration Plan

### Step 1: Create `dure-messages` library

**Actions:**
1. Create directory: `crates/dure-messages/`
2. Create `Cargo.toml` with library configuration
3. Copy all files from `crates/asyncapi-gen/src/messages/*.rs` → `crates/dure-messages/src/`
4. Move `ClientMessage`, `ServerMessage`, and common types from `mobile/src/site/messages/mod.rs` → `crates/dure-messages/src/lib.rs`
5. Create `lib.rs` with module declarations and public re-exports
6. Run `cargo check -p dure-messages` to verify compilation

**Verification:**
```bash
cargo check -p dure-messages
# Should compile successfully with no warnings
```

### Step 2: Refactor `dure-asyncapi-gen`

**Actions:**
1. Remove `[workspace]` declaration from `crates/dure-asyncapi-gen/Cargo.toml`
2. Add `version.workspace = true`, `edition.workspace = true`, etc.
3. Add `dure-messages = { path = "../dure-messages" }` dependency
4. Move `mobile/src/asyncapi_spec.rs` → `crates/dure-asyncapi-gen/src/asyncapi_spec.rs`
5. Update imports in `asyncapi_spec.rs`: `crate::site::messages` → `dure_messages`
6. Update `main.rs` to import `DureApi` from `asyncapi_spec` module
7. Delete `crates/dure-asyncapi-gen/src/messages/` directory
8. Update `lib.rs` to export `asyncapi_spec` module
9. Run `cargo check -p dure-asyncapi-gen` to verify compilation

**Verification:**
```bash
cargo check -p dure-asyncapi-gen
cargo run -p dure-asyncapi-gen
# Should generate docs/asyncapi.{json,yaml} successfully
```

### Step 3: Update workspace configuration

**Actions:**
1. Edit workspace `Cargo.toml`:
   - Add `"crates/dure-messages"` to members
   - Add `"crates/dure-asyncapi-gen"` to members
   - Remove `"crates/asyncapi-gen"` from exclude list
2. Run `cargo metadata --format-version 1 | jq '.workspace_members'` to verify

**Verification:**
```bash
cargo metadata --format-version 1 | jq '.workspace_members' | grep -E "dure-messages|dure-asyncapi-gen"
# Should show both crates
```

### Step 4: Update mobile crate

**Actions:**
1. Add `dure-messages = { path = "../crates/dure-messages" }` to `mobile/Cargo.toml`
2. Remove `asyncapi-rust = "0.2"` and `schemars = { ... }` dependencies from `mobile/Cargo.toml` (only used in files being deleted)
3. Find all imports of `crate::site::messages` in `mobile/src/`:
   ```bash
   grep -r "use crate::site::messages" mobile/src/
   ```
4. Replace each occurrence with `use dure_messages`
5. Delete `mobile/src/site/messages/` directory
6. Delete `mobile/src/asyncapi_spec.rs`
7. Update `mobile/src/site/mod.rs` - remove `pub mod messages;` if present
8. Run `cargo check -p dure` to verify compilation

**Verification:**
```bash
cargo check -p dure
# Should compile successfully

# Verify no remaining references
grep -r "site::messages" mobile/src/
# Should return nothing
```

### Step 5: Final verification

**Actions:**
1. Build entire workspace: `cargo build --workspace`
2. Run all tests: `cargo test --workspace`
3. Generate AsyncAPI docs: `cargo run -p dure-asyncapi-gen`
4. Compare generated `docs/asyncapi.json` with baseline (should be identical)
5. Build mobile for all targets:
   ```bash
   cargo build --bin dure-desktop
   cargo build --bin dure-desktop --no-default-features
   ```
6. Search for any remaining old imports:
   ```bash
   grep -r "site::messages" . --include="*.rs"
   ```

**Success criteria:**
- ✅ All crates compile without warnings
- ✅ All tests pass with same results as baseline
- ✅ Generated AsyncAPI docs are identical to baseline
- ✅ No references to `site::messages` remain
- ✅ Mobile builds successfully for all platform targets

## Testing Strategy

### Pre-migration Baseline

```bash
# Save baseline test results
cargo test --workspace > /tmp/test-baseline.txt 2>&1

# Save baseline AsyncAPI output
cargo run -p dure-asyncapi-gen
cp docs/asyncapi.json /tmp/asyncapi-baseline.json
```

### Test-Driven Development

Create `crates/dure-messages/tests/integration_test.rs`:

```rust
//! Integration tests for message serialization and validation

use dure_messages::*;
use serde_json;

#[test]
fn test_client_message_serialization() {
    // Test each ClientMessage variant can serialize/deserialize
    let msg = ClientMessage::AuthLogin(AuthLoginRequest {
        server_id: "test".to_string(),
        device_id: "device1".to_string(),
        public_key: "key".to_string(),
        device_info: None,
    });
    
    let json = serde_json::to_string(&msg).expect("serialize");
    let deserialized: ClientMessage = serde_json::from_str(&json).expect("deserialize");
    
    // Round-trip should succeed
    assert!(matches!(deserialized, ClientMessage::AuthLogin(_)));
}

#[test]
fn test_server_message_serialization() {
    // Similar test for ServerMessage
}

#[test]
fn test_message_types_are_send_sync() {
    // Ensure types can be used in async contexts
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ClientMessage>();
    assert_send_sync::<ServerMessage>();
}

#[test]
fn test_json_schema_generation() {
    use schemars::schema_for;
    
    let schema = schema_for!(ClientMessage);
    assert!(!schema.schema.metadata.is_none());
}
```

This test will:
1. **Fail initially** (crate doesn't exist yet)
2. **Pass after Step 1** (library created)
3. **Continue passing** through all migration steps

### Post-migration Verification

```bash
# Compare test results
cargo test --workspace > /tmp/test-final.txt 2>&1
diff /tmp/test-baseline.txt /tmp/test-final.txt
# Should show no test behavior changes (only passing test additions)

# Compare AsyncAPI output
cargo run -p dure-asyncapi-gen
diff /tmp/asyncapi-baseline.json docs/asyncapi.json
# Should be identical

# Verify no old imports remain
grep -r "site::messages" . --include="*.rs"
# Should return nothing
```

## Error Handling

### Potential Issues

1. **Circular dependencies**: If mobile tries to depend on asyncapi-gen instead of messages
   - **Prevention:** Clear naming (`dure-messages` vs `dure-asyncapi-gen`)
   - **Detection:** `cargo check` will fail with circular dependency error

2. **Missing re-exports**: Types not exported from `lib.rs`
   - **Prevention:** Comprehensive `pub use` statements in `lib.rs`
   - **Detection:** Compilation errors when importing types

3. **Platform-specific compilation issues**: WASM/Android builds fail
   - **Prevention:** Test builds for all platforms
   - **Detection:** CI/CD failures (if configured)

4. **Stale imports**: Missed some `site::messages` references
   - **Prevention:** Use global search-and-replace, then grep verification
   - **Detection:** Compilation errors

### Rollback Plan

If migration fails at any step:

1. **Step 1-3 failure**: Delete new crates, restore workspace Cargo.toml from git
2. **Step 4 failure**: Restore `mobile/src/site/messages/` and `mobile/src/asyncapi_spec.rs` from git
3. **Step 5 failure**: `git reset --hard HEAD` to restore entire worktree

All changes should be committed in small, atomic commits per step for easy rollback.

## Future Enhancements

### Potential Improvements (Out of Scope)

1. **Publish to crates.io**: If message types need to be used by external projects
2. **Versioning strategy**: Semantic versioning for message protocol changes
3. **Client code generation**: TypeScript/Python clients from AsyncAPI spec
4. **Runtime validation**: JSON Schema validation of incoming WebSocket messages
5. **Message versioning**: Support multiple protocol versions simultaneously

These are not required for the current refactor but become easier with the new structure.

## Success Metrics

**Quantitative:**
- Zero files in `mobile/src/site/messages/` (deleted)
- Single `dure-messages` crate contains all 10 message modules
- Zero grep results for `site::messages` in codebase
- 100% test pass rate maintained (baseline vs final)
- Identical AsyncAPI documentation output

**Qualitative:**
- Message type changes require editing only one location
- Clear separation between library and tooling concerns
- Documentation generation remains a standalone tool
- Mobile crate has cleaner dependency graph

## Timeline Estimate

- **Step 1:** Create dure-messages library - ~15 minutes
- **Step 2:** Refactor dure-asyncapi-gen - ~10 minutes  
- **Step 3:** Update workspace config - ~5 minutes
- **Step 4:** Update mobile crate - ~20 minutes
- **Step 5:** Final verification - ~10 minutes

**Total estimated time:** ~60 minutes (1 hour)

**With TDD and testing:** ~90 minutes (1.5 hours)

## Approval

- [x] Architecture reviewed and approved
- [x] Crate structure reviewed and approved
- [x] Dependencies reviewed and approved
- [x] Import strategy reviewed and approved
- [x] Migration plan reviewed and approved
- [x] Testing strategy reviewed and approved

**Approved by:** User  
**Date:** 2026-07-08  
**Ready for implementation:** Yes

---

**Next Steps:** Invoke `superpowers:writing-plans` skill to create detailed implementation plan with tasks.
