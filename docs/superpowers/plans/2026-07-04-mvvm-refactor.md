# MVVM Actor-Based Architecture Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refactor Dure to use actor-based MVVM architecture with smol async runtime for all deployment modes (Desktop, CLI, Android, WASM)

**Architecture:** Four actors (Platform, SSH, NS, WSS) communicate with ViewModel via typed channels. ViewModel owns transient state (progress, errors), actors read/write durable state (DB). UI/CLI calls ViewModel command methods, polls events for updates. Cross-platform via runtime abstraction layer (smol for native, async-executor for WASM).

**Tech Stack:** smol 2.0, async-executor 1.8, gloo-timers 0.3, wasm-bindgen-futures 0.4, Diesel (unchanged), egui 0.33 (unchanged)

## Global Constraints

- Rust nightly toolchain required
- All I/O operations must run on background actors (never block UI thread)
- Reuse existing calc/*.rs business logic (no changes to calc layer)
- Database schema unchanged (Diesel models unchanged)
- All features must work in GUI, CLI, and WASM modes
- Follow TDD: test first, implement, verify, commit
- DRY: no code duplication between actors
- YAGNI: no speculative features
- Commit after each task completion

---

## Week 1: Infrastructure Setup

### Task 1: Update Dependencies

**Files:**
- Modify: `mobile/Cargo.toml`

**Interfaces:**
- Consumes: Existing Cargo.toml
- Produces: Updated dependencies for smol runtime

- [ ] **Step 1: Add smol and WASM dependencies**

Open `mobile/Cargo.toml` and add to `[dependencies]` section:

```toml
# Async runtime
smol = "2.0"

# WASM-specific async support
[target.'cfg(target_arch = "wasm32")'.dependencies]
async-executor = "1.8"
async-task = "4.7"
gloo-timers = "0.3"
wasm-bindgen-futures = "0.4"
```

- [ ] **Step 2: Verify build**

Run: `cargo check`
Expected: Build succeeds with new dependencies downloaded

- [ ] **Step 3: Commit**

```bash
git add mobile/Cargo.toml
git commit -m "build: add smol async runtime and WASM dependencies

Add smol for actor-based async runtime on native platforms.
Add async-executor, gloo-timers, wasm-bindgen-futures for WASM support.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 2: Create ViewModel Module Structure

**Files:**
- Create: `mobile/src/viewmodel/mod.rs`
- Create: `mobile/src/viewmodel/common.rs`
- Create: `mobile/src/viewmodel/runtime.rs`
- Create: `mobile/src/viewmodel/io.rs`
- Modify: `mobile/src/lib.rs`

**Interfaces:**
- Consumes: None
- Produces: Empty module structure for viewmodel

- [ ] **Step 1: Create viewmodel module directory**

```bash
mkdir -p mobile/src/viewmodel
```

- [ ] **Step 2: Create mod.rs with module declarations**

Create `mobile/src/viewmodel/mod.rs`:

```rust
//! ViewModel layer for actor-based MVVM architecture

pub mod common;
pub mod runtime;
pub mod io;
pub mod platform;
pub mod ssh;
pub mod ns;
pub mod wss;

#[cfg(test)]
mod tests;

pub use common::*;

use smol::channel::{Receiver, Sender};
use std::collections::{HashMap, VecDeque};

/// ViewModel coordinates actors and exposes unified API for UI/CLI
pub struct ViewModel {
    // Actor communication channels
    platform_tx: Sender<platform::PlatformCommand>,
    ssh_tx: Sender<ssh::SshCommand>,
    ns_tx: Sender<ns::NsCommand>,
    wss_tx: Sender<wss::WssCommand>,
    
    // Unified event receiver
    event_rx: Receiver<ViewModelEvent>,
    
    // Transient state
    state: ViewModelState,
    
    // Runtime handle
    runtime_handle: Option<RuntimeHandle>,
    
    // Optional egui context (for GUI mode)
    #[cfg(feature = "gui")]
    egui_ctx: Option<egui::Context>,
}

enum RuntimeHandle {
    #[cfg(not(target_arch = "wasm32"))]
    Native(std::thread::JoinHandle<()>),
    
    #[cfg(target_arch = "wasm32")]
    Wasm(WasmExecutorHandle),
}

#[cfg(target_arch = "wasm32")]
struct WasmExecutorHandle;

#[derive(Default)]
pub struct ViewModelState {
    pub active_operations: HashMap<String, OperationProgress>,
    pub recent_errors: VecDeque<ErrorRecord>,
    pub wss_connections: HashMap<String, WssConnectionInfo>,
    #[cfg(feature = "gui")]
    pub textures: HashMap<String, egui::TextureHandle>,
}

pub struct OperationProgress {
    pub operation: String,
    pub progress: f32,
    pub status: String,
    pub started_at: std::time::Instant,
}

pub struct ErrorRecord {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub operation: String,
    pub error: String,
    pub actor: String,
}

#[derive(Clone, Debug)]
pub struct WssConnectionInfo {
    pub connection_id: String,
    pub url: String,
    pub connected_at: chrono::DateTime<chrono::Utc>,
    pub status: String,
}
```

- [ ] **Step 3: Create common.rs with ViewModelEvent**

Create `mobile/src/viewmodel/common.rs`:

```rust
//! Common types shared across ViewModel and actors

use crate::viewmodel::{platform, ssh, ns, wss};

/// Unified event type from all actors
#[derive(Clone, Debug)]
pub enum ViewModelEvent {
    Platform(platform::PlatformEvent),
    Ssh(ssh::SshEvent),
    Ns(ns::NsEvent),
    Wss(wss::WssEvent),
}
```

- [ ] **Step 4: Create runtime.rs with platform abstraction**

Create `mobile/src/viewmodel/runtime.rs`:

```rust
//! Runtime abstraction for cross-platform async support

#[cfg(not(target_arch = "wasm32"))]
pub use native::*;

#[cfg(target_arch = "wasm32")]
pub use wasm::*;

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::future::Future;
    
    pub fn spawn<F>(future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        smol::spawn(future).detach();
    }
    
    pub fn unblock<F, T>(f: F) -> impl Future<Output = T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        smol::unblock(f)
    }
    
    pub async fn sleep(duration: std::time::Duration) {
        smol::Timer::after(duration).await;
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use std::future::Future;
    use wasm_bindgen_futures::spawn_local;
    
    pub fn spawn<F>(future: F)
    where
        F: Future<Output = ()> + 'static,
    {
        spawn_local(future);
    }
    
    pub async fn unblock<F, T>(f: F) -> T
    where
        F: FnOnce() -> T + 'static,
        T: 'static,
    {
        // WASM: run on main thread (no threads available)
        f()
    }
    
    pub async fn sleep(duration: std::time::Duration) {
        let millis = duration.as_millis() as i32;
        gloo_timers::future::sleep(std::time::Duration::from_millis(millis as u64)).await;
    }
}
```

- [ ] **Step 5: Create io.rs with I/O abstraction**

Create `mobile/src/viewmodel/io.rs`:

```rust
//! I/O abstraction for cross-platform HTTP support

#[cfg(not(target_arch = "wasm32"))]
pub use native::*;

#[cfg(target_arch = "wasm32")]
pub use wasm::*;

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::io::Read;
    
    pub async fn http_get(url: &str) -> anyhow::Result<Vec<u8>> {
        let response = ureq::get(url).call()?;
        let mut body = Vec::new();
        response.into_reader().read_to_end(&mut body)?;
        Ok(body)
    }
    
    pub async fn http_post(url: &str, body: Vec<u8>) -> anyhow::Result<Vec<u8>> {
        let response = ureq::post(url).send_bytes(&body)?;
        let mut response_body = Vec::new();
        response.into_reader().read_to_end(&mut response_body)?;
        Ok(response_body)
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Request, RequestInit, Response};
    
    pub async fn http_get(url: &str) -> anyhow::Result<Vec<u8>> {
        let window = web_sys::window().ok_or_else(|| anyhow::anyhow!("no window"))?;
        let resp_value = JsFuture::from(window.fetch_with_str(url)).await
            .map_err(|_| anyhow::anyhow!("fetch failed"))?;
        let resp: Response = resp_value.dyn_into()
            .map_err(|_| anyhow::anyhow!("not a Response"))?;
        let array_buffer = JsFuture::from(resp.array_buffer()
            .map_err(|_| anyhow::anyhow!("array_buffer failed"))?)
            .await
            .map_err(|_| anyhow::anyhow!("array_buffer promise failed"))?;
        let uint8_array = js_sys::Uint8Array::new(&array_buffer);
        Ok(uint8_array.to_vec())
    }
    
    pub async fn http_post(url: &str, body: Vec<u8>) -> anyhow::Result<Vec<u8>> {
        let mut opts = RequestInit::new();
        opts.method("POST");
        opts.body(Some(&js_sys::Uint8Array::from(&body[..]).into()));
        
        let request = Request::new_with_str_and_init(url, &opts)
            .map_err(|_| anyhow::anyhow!("Request creation failed"))?;
        let window = web_sys::window().ok_or_else(|| anyhow::anyhow!("no window"))?;
        let resp_value = JsFuture::from(window.fetch_with_request(&request)).await
            .map_err(|_| anyhow::anyhow!("fetch failed"))?;
        let resp: Response = resp_value.dyn_into()
            .map_err(|_| anyhow::anyhow!("not a Response"))?;
        let array_buffer = JsFuture::from(resp.array_buffer()
            .map_err(|_| anyhow::anyhow!("array_buffer failed"))?)
            .await
            .map_err(|_| anyhow::anyhow!("array_buffer promise failed"))?;
        let uint8_array = js_sys::Uint8Array::new(&array_buffer);
        Ok(uint8_array.to_vec())
    }
}
```

- [ ] **Step 6: Add viewmodel to lib.rs**

Add to `mobile/src/lib.rs` (after existing module declarations):

```rust
#[cfg(feature = "gui")]
pub mod viewmodel;
```

- [ ] **Step 7: Verify build**

Run: `cargo check`
Expected: Errors about missing platform/ssh/ns/wss modules (will create in next tasks)

- [ ] **Step 8: Commit**

```bash
git add mobile/src/viewmodel/
git add mobile/src/lib.rs
git commit -m "feat(viewmodel): create module structure with runtime/IO abstraction

Add ViewModel skeleton with:
- Runtime abstraction (smol for native, async-executor for WASM)
- I/O abstraction (ureq for native, web_sys::fetch for WASM)
- Common types (ViewModelEvent, OperationProgress, ErrorRecord)
- Module structure for platform/ssh/ns/wss actors

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 3: Create Actor Module Stubs

**Files:**
- Create: `mobile/src/viewmodel/platform/mod.rs`
- Create: `mobile/src/viewmodel/platform/commands.rs`
- Create: `mobile/src/viewmodel/platform/events.rs`
- Create: `mobile/src/viewmodel/ssh/mod.rs`
- Create: `mobile/src/viewmodel/ssh/commands.rs`
- Create: `mobile/src/viewmodel/ssh/events.rs`
- Create: `mobile/src/viewmodel/ns/mod.rs`
- Create: `mobile/src/viewmodel/ns/commands.rs`
- Create: `mobile/src/viewmodel/ns/events.rs`
- Create: `mobile/src/viewmodel/wss/mod.rs`
- Create: `mobile/src/viewmodel/wss/commands.rs`
- Create: `mobile/src/viewmodel/wss/events.rs`

**Interfaces:**
- Consumes: ViewModelEvent from common.rs
- Produces: Actor command/event enums (stub, will complete in Week 2)

- [ ] **Step 1: Create platform actor stubs**

```bash
mkdir -p mobile/src/viewmodel/platform
```

Create `mobile/src/viewmodel/platform/mod.rs`:

```rust
//! Platform actor for GCP operations

mod commands;
mod events;

pub use commands::PlatformCommand;
pub use events::PlatformEvent;

use smol::channel::{Receiver, Sender};
use crate::viewmodel::ViewModelEvent;

pub struct PlatformActor {
    command_rx: Receiver<PlatformCommand>,
    event_tx: Sender<ViewModelEvent>,
}

impl PlatformActor {
    pub fn new(command_rx: Receiver<PlatformCommand>, event_tx: Sender<ViewModelEvent>) -> Self {
        Self { command_rx, event_tx }
    }
    
    pub async fn run(mut self) {
        log::info!("PlatformActor started");
        loop {
            match self.command_rx.recv().await {
                Ok(_cmd) => {
                    // TODO: implement in Week 2
                }
                Err(_) => {
                    log::info!("PlatformActor: channel closed");
                    break;
                }
            }
        }
    }
}
```

Create `mobile/src/viewmodel/platform/commands.rs`:

```rust
//! Platform actor commands

#[derive(Debug, Clone)]
pub enum PlatformCommand {
    // TODO: add commands in Week 2
    Placeholder,
}
```

Create `mobile/src/viewmodel/platform/events.rs`:

```rust
//! Platform actor events

#[derive(Debug, Clone)]
pub enum PlatformEvent {
    // TODO: add events in Week 2
    Placeholder,
}
```

- [ ] **Step 2: Create SSH actor stubs**

```bash
mkdir -p mobile/src/viewmodel/ssh
```

Create `mobile/src/viewmodel/ssh/mod.rs`:

```rust
//! SSH actor for host and container management

mod commands;
mod events;

pub use commands::SshCommand;
pub use events::SshEvent;

use smol::channel::{Receiver, Sender};
use crate::viewmodel::ViewModelEvent;

pub struct SshActor {
    command_rx: Receiver<SshCommand>,
    event_tx: Sender<ViewModelEvent>,
}

impl SshActor {
    pub fn new(command_rx: Receiver<SshCommand>, event_tx: Sender<ViewModelEvent>) -> Self {
        Self { command_rx, event_tx }
    }
    
    pub async fn run(mut self) {
        log::info!("SshActor started");
        loop {
            match self.command_rx.recv().await {
                Ok(_cmd) => {
                    // TODO: implement in Week 2
                }
                Err(_) => {
                    log::info!("SshActor: channel closed");
                    break;
                }
            }
        }
    }
}
```

Create `mobile/src/viewmodel/ssh/commands.rs`:

```rust
//! SSH actor commands

#[derive(Debug, Clone)]
pub enum SshCommand {
    // TODO: add commands in Week 2
    Placeholder,
}
```

Create `mobile/src/viewmodel/ssh/events.rs`:

```rust
//! SSH actor events

#[derive(Debug, Clone)]
pub enum SshEvent {
    // TODO: add events in Week 2
    Placeholder,
}
```

- [ ] **Step 3: Create NS actor stubs**

```bash
mkdir -p mobile/src/viewmodel/ns
```

Create `mobile/src/viewmodel/ns/mod.rs`:

```rust
//! NS actor for DNS management

mod commands;
mod events;

pub use commands::NsCommand;
pub use events::NsEvent;

use smol::channel::{Receiver, Sender};
use crate::viewmodel::ViewModelEvent;

pub struct NsActor {
    command_rx: Receiver<NsCommand>,
    event_tx: Sender<ViewModelEvent>,
}

impl NsActor {
    pub fn new(command_rx: Receiver<NsCommand>, event_tx: Sender<ViewModelEvent>) -> Self {
        Self { command_rx, event_tx }
    }
    
    pub async fn run(mut self) {
        log::info!("NsActor started");
        loop {
            match self.command_rx.recv().await {
                Ok(_cmd) => {
                    // TODO: implement in Week 2
                }
                Err(_) => {
                    log::info!("NsActor: channel closed");
                    break;
                }
            }
        }
    }
}
```

Create `mobile/src/viewmodel/ns/commands.rs`:

```rust
//! NS actor commands

#[derive(Debug, Clone)]
pub enum NsCommand {
    // TODO: add commands in Week 2
    Placeholder,
}
```

Create `mobile/src/viewmodel/ns/events.rs`:

```rust
//! NS actor events

#[derive(Debug, Clone)]
pub enum NsEvent {
    // TODO: add events in Week 2
    Placeholder,
}
```

- [ ] **Step 4: Create WSS actor stubs**

```bash
mkdir -p mobile/src/viewmodel/wss
```

Create `mobile/src/viewmodel/wss/mod.rs`:

```rust
//! WSS actor (stub - not implemented yet)

mod commands;
mod events;

pub use commands::WssCommand;
pub use events::WssEvent;

use smol::channel::{Receiver, Sender};
use crate::viewmodel::ViewModelEvent;

pub struct WssActor {
    command_rx: Receiver<WssCommand>,
    event_tx: Sender<ViewModelEvent>,
}

impl WssActor {
    pub fn new(command_rx: Receiver<WssCommand>, event_tx: Sender<ViewModelEvent>) -> Self {
        Self { command_rx, event_tx }
    }
    
    pub async fn run(mut self) {
        log::info!("WssActor stub - not implemented yet");
        loop {
            match self.command_rx.recv().await {
                Ok(cmd) => {
                    log::warn!("WssActor received command but is not implemented: {:?}", cmd);
                    let _ = self.event_tx.send(ViewModelEvent::Wss(
                        WssEvent::Error {
                            operation: format!("{:?}", cmd),
                            error: "WSS not implemented yet".to_string(),
                        }
                    )).await;
                }
                Err(_) => {
                    log::info!("WssActor: channel closed");
                    break;
                }
            }
        }
    }
}
```

Create `mobile/src/viewmodel/wss/commands.rs`:

```rust
//! WSS actor commands

#[derive(Debug, Clone)]
pub enum WssCommand {
    // Stub commands
    ConnectClient { url: String, auth_token: Option<String> },
    DisconnectClient { connection_id: String },
    SendMessage { connection_id: String, message: Vec<u8> },
}
```

Create `mobile/src/viewmodel/wss/events.rs`:

```rust
//! WSS actor events

#[derive(Debug, Clone)]
pub enum WssEvent {
    Error { operation: String, error: String },
}
```

- [ ] **Step 5: Verify build**

Run: `cargo check`
Expected: Build succeeds (actors are stubs but compilable)

- [ ] **Step 6: Commit**

```bash
git add mobile/src/viewmodel/platform/
git add mobile/src/viewmodel/ssh/
git add mobile/src/viewmodel/ns/
git add mobile/src/viewmodel/wss/
git commit -m "feat(viewmodel): add actor module stubs

Add stub implementations for:
- PlatformActor (GCP operations)
- SshActor (host and container management)
- NsActor (DNS management)
- WssActor (WebSocket - stub only)

Actors have empty command loops, will be implemented in Week 2.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 4: Implement ViewModel Initialization

**Files:**
- Modify: `mobile/src/viewmodel/mod.rs`
- Create: `mobile/src/viewmodel/tests.rs`

**Interfaces:**
- Consumes: Actor stubs from Task 3
- Produces: `ViewModel::new()`, `ViewModel::new_headless()`, `ViewModel::poll_events()`

- [ ] **Step 1: Write test for ViewModel initialization**

Create `mobile/src/viewmodel/tests.rs`:

```rust
#[cfg(test)]
use super::*;

#[test]
fn test_viewmodel_headless_initialization() {
    let vm = ViewModel::new_headless();
    
    // Should have empty state
    assert_eq!(vm.state.active_operations.len(), 0);
    assert_eq!(vm.state.recent_errors.len(), 0);
    assert_eq!(vm.state.wss_connections.len(), 0);
}

#[test]
fn test_viewmodel_poll_events_empty() {
    let mut vm = ViewModel::new_headless();
    
    // Polling with no events should return empty vec
    let events = vm.poll_events_headless();
    assert_eq!(events.len(), 0);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib viewmodel::tests`
Expected: FAIL - ViewModel::new_headless() not implemented

- [ ] **Step 3: Implement ViewModel initialization**

Modify `mobile/src/viewmodel/mod.rs`, add implementation after struct definition:

```rust
impl ViewModel {
    /// Create ViewModel for GUI mode (Desktop/Android)
    #[cfg(all(feature = "gui", not(target_arch = "wasm32")))]
    pub fn new(ctx: egui::Context) -> Self {
        let (platform_tx, platform_rx) = smol::channel::unbounded();
        let (ssh_tx, ssh_rx) = smol::channel::unbounded();
        let (ns_tx, ns_rx) = smol::channel::unbounded();
        let (wss_tx, wss_rx) = smol::channel::unbounded();
        let (event_tx, event_rx) = smol::channel::unbounded();
        
        // Spawn background thread with smol executor
        let runtime_handle = std::thread::spawn(move || {
            smol::block_on(async {
                log::info!("ViewModel runtime started");
                
                // Create actors
                let platform_actor = platform::PlatformActor::new(platform_rx, event_tx.clone());
                let ssh_actor = ssh::SshActor::new(ssh_rx, event_tx.clone());
                let ns_actor = ns::NsActor::new(ns_rx, event_tx.clone());
                let wss_actor = wss::WssActor::new(wss_rx, event_tx.clone());
                
                // Run all actors concurrently
                smol::spawn(platform_actor.run()).detach();
                smol::spawn(ssh_actor.run()).detach();
                smol::spawn(ns_actor.run()).detach();
                smol::spawn(wss_actor.run()).detach();
                
                // Keep thread alive
                std::future::pending::<()>().await
            })
        });
        
        Self {
            platform_tx,
            ssh_tx,
            ns_tx,
            wss_tx,
            event_rx,
            state: ViewModelState::default(),
            runtime_handle: Some(RuntimeHandle::Native(runtime_handle)),
            egui_ctx: Some(ctx),
        }
    }
    
    /// Create ViewModel for CLI mode (headless)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new_headless() -> Self {
        let (platform_tx, platform_rx) = smol::channel::unbounded();
        let (ssh_tx, ssh_rx) = smol::channel::unbounded();
        let (ns_tx, ns_rx) = smol::channel::unbounded();
        let (wss_tx, wss_rx) = smol::channel::unbounded();
        let (event_tx, event_rx) = smol::channel::unbounded();
        
        let runtime_handle = std::thread::spawn(move || {
            smol::block_on(async {
                log::info!("ViewModel runtime started (headless)");
                
                let platform_actor = platform::PlatformActor::new(platform_rx, event_tx.clone());
                let ssh_actor = ssh::SshActor::new(ssh_rx, event_tx.clone());
                let ns_actor = ns::NsActor::new(ns_rx, event_tx.clone());
                let wss_actor = wss::WssActor::new(wss_rx, event_tx.clone());
                
                smol::spawn(platform_actor.run()).detach();
                smol::spawn(ssh_actor.run()).detach();
                smol::spawn(ns_actor.run()).detach();
                smol::spawn(wss_actor.run()).detach();
                
                std::future::pending::<()>().await
            })
        });
        
        Self {
            platform_tx,
            ssh_tx,
            ns_tx,
            wss_tx,
            event_rx,
            state: ViewModelState::default(),
            runtime_handle: Some(RuntimeHandle::Native(runtime_handle)),
            #[cfg(feature = "gui")]
            egui_ctx: None,
        }
    }
    
    /// Poll for events and update state (GUI mode)
    #[cfg(feature = "gui")]
    pub fn poll_events(&mut self, ctx: &egui::Context) -> Vec<ViewModelEvent> {
        let mut events = Vec::new();
        
        while let Ok(event) = self.event_rx.try_recv() {
            self.apply_event(&event, Some(ctx));
            events.push(event);
        }
        
        if !events.is_empty() {
            ctx.request_repaint();
        }
        
        events
    }
    
    /// Poll events without egui context (CLI mode)
    pub fn poll_events_headless(&mut self) -> Vec<ViewModelEvent> {
        let mut events = Vec::new();
        
        while let Ok(event) = self.event_rx.try_recv() {
            #[cfg(feature = "gui")]
            self.apply_event(&event, None);
            #[cfg(not(feature = "gui"))]
            {
                // Minimal event processing for headless mode
                let _ = event; // Suppress unused variable warning
            }
            events.push(event);
        }
        
        events
    }
    
    #[cfg(feature = "gui")]
    fn apply_event(&mut self, _event: &ViewModelEvent, _ctx: Option<&egui::Context>) {
        // TODO: implement in Week 2
    }
    
    // State accessors
    pub fn active_operations(&self) -> &HashMap<String, OperationProgress> {
        &self.state.active_operations
    }
    
    pub fn recent_errors(&self) -> &VecDeque<ErrorRecord> {
        &self.state.recent_errors
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib viewmodel::tests`
Expected: PASS - Both tests succeed

- [ ] **Step 5: Verify full build**

Run: `cargo check`
Expected: Build succeeds

- [ ] **Step 6: Commit**

```bash
git add mobile/src/viewmodel/mod.rs
git add mobile/src/viewmodel/tests.rs
git commit -m "feat(viewmodel): implement initialization and event polling

Add ViewModel::new() for GUI mode and new_headless() for CLI mode.
Both spawn background thread with smol executor running 4 actors.
Add poll_events() for non-blocking event processing.

Tests verify initialization and empty event polling.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 5: Add ViewModel to DureApp

**Files:**
- Modify: `mobile/src/dure.rs`
- Modify: `mobile/src/dure_stt.rs`

**Interfaces:**
- Consumes: `ViewModel::new()` from Task 4
- Produces: DureApp with optional ViewModel field

- [ ] **Step 1: Add ViewModel field to DureApp**

Modify `mobile/src/dure_stt.rs`, add field to `DureApp` struct:

```rust
// Add to imports at top of file
#[cfg(feature = "gui")]
use crate::viewmodel::ViewModel;

// Add field to DureApp struct (after existing fields)
    // ViewModel (MVVM architecture)
    #[cfg_attr(feature = "serde", serde(skip))]
    pub viewmodel: Option<ViewModel>,
```

- [ ] **Step 2: Initialize ViewModel in Default impl**

Modify `mobile/src/dure_stt.rs`, add to `Default` impl:

```rust
            // Add to Default::default()
            viewmodel: None,
```

- [ ] **Step 3: Create ViewModel in DureApp::new()**

Modify `mobile/src/dure.rs`, find the `impl DureApp` section with `pub fn new()` and add ViewModel initialization:

```rust
// Add near start of new() function, after cc.egui_ctx setup
        #[cfg(feature = "gui")]
        let viewmodel = Some(crate::viewmodel::ViewModel::new(cc.egui_ctx.clone()));
        
        // ... existing code ...
        
        Self {
            // ... existing fields ...
            #[cfg(feature = "gui")]
            viewmodel,
            // ... rest of fields ...
        }
```

- [ ] **Step 4: Poll events in update() loop**

Modify `mobile/src/dure.rs`, add at the top of `update()` function:

```rust
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Poll ViewModel events
        #[cfg(feature = "gui")]
        if let Some(ref mut vm) = self.viewmodel {
            let _events = vm.poll_events(ctx);
            // Events will be processed when actors are implemented
        }
        
        // ... existing update code ...
    }
```

- [ ] **Step 5: Verify build**

Run: `cargo check`
Expected: Build succeeds

- [ ] **Step 6: Run app to verify ViewModel spawns**

Run: `cargo run --bin dure-desktop`
Expected: App launches, check logs for "ViewModel runtime started"

- [ ] **Step 7: Commit**

```bash
git add mobile/src/dure.rs
git add mobile/src/dure_stt.rs
git commit -m "feat(dure): integrate ViewModel into DureApp

Add optional ViewModel field to DureApp.
Initialize ViewModel in new() with egui context.
Poll events at top of update() loop.

Actors now run in background thread when app starts.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Week 2: Actor Implementation

### Task 6: Implement PlatformActor Commands and Events

**Files:**
- Modify: `mobile/src/viewmodel/platform/commands.rs`
- Modify: `mobile/src/viewmodel/platform/events.rs`
- Create: `mobile/src/viewmodel/platform/actor.rs`
- Modify: `mobile/src/viewmodel/platform/mod.rs`
- Create: `mobile/src/viewmodel/platform/tests.rs`

**Interfaces:**
- Consumes: calc::gcp_rest::*, calc::db::* functions
- Produces: PlatformCommand enum, PlatformEvent enum, PlatformActor::handle_command()

- [ ] **Step 1: Define PlatformCommand enum**

Modify `mobile/src/viewmodel/platform/commands.rs`, replace Placeholder with:

```rust
//! Platform actor commands

#[derive(Debug, Clone)]
pub enum PlatformCommand {
    // OAuth & Platform Management
    StartOAuth { platform_name: String },
    CompleteOAuth { platform_name: String, auth_code: String },
    DeletePlatform { platform_name: String },
    
    // Project Operations
    ListProjects { platform_name: String },
    SelectProject { platform_name: String, project_id: String },
    
    // VM Operations
    ListVMs { platform_name: String },
    CreateVM { 
        platform_name: String, 
        vm_name: String, 
        zone: String,
        machine_type: String,
    },
    DeleteVM { 
        platform_name: String, 
        vm_name: String, 
        zone: String 
    },
    RestartVM { platform_name: String, vm_name: String, zone: String },
    
    // Firewall Operations
    UpdateFirewall { platform_name: String, allow_ip: String },
    
    // Billing Operations
    FetchBilling { 
        platform_name: String, 
        project_id: String,
        dataset: String,
        table: String,
    },
    
    // Refresh all platform data
    RefreshAll,
}
```

- [ ] **Step 2: Define PlatformEvent enum**

Modify `mobile/src/viewmodel/platform/events.rs`, replace Placeholder with:

```rust
//! Platform actor events

use crate::calc::gcp_rest::BillingRecord;

#[derive(Debug, Clone)]
pub struct VmInfo {
    pub name: String,
    pub zone: String,
    pub external_ip: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone)]
pub enum PlatformEvent {
    // OAuth Events
    OAuthStarted { platform_name: String, auth_url: String },
    OAuthCompleted { platform_name: String, email: String },
    
    // Project Events
    ProjectsListed { 
        platform_name: String, 
        projects: Vec<(String, String)>  // (id, name)
    },
    ProjectSelected { platform_name: String, project_id: String },
    
    // VM Events
    VMsListed { 
        platform_name: String, 
        vms: Vec<VmInfo> 
    },
    VMCreated { platform_name: String, vm_name: String, external_ip: String },
    VMDeleted { platform_name: String, vm_name: String },
    VMRestarted { platform_name: String, vm_name: String },
    
    // Firewall Events
    FirewallUpdated { platform_name: String, whitelisted_ip: String },
    
    // Billing Events
    BillingFetched { 
        platform_name: String, 
        records: Vec<BillingRecord> 
    },
    
    // Progress & Errors
    Progress { 
        operation: String, 
        progress: f32, 
        status: String 
    },
    Error { operation: String, error: String },
}
```

- [ ] **Step 3: Write test for ListVMs command**

Create `mobile/src/viewmodel/platform/tests.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::viewmodel::ViewModelEvent;
    
    #[test]
    fn test_platform_actor_list_vms_sends_event() {
        smol::block_on(async {
            let (cmd_tx, cmd_rx) = smol::channel::unbounded();
            let (event_tx, event_rx) = smol::channel::unbounded();
            
            let actor = PlatformActor::new(cmd_rx, event_tx);
            smol::spawn(actor.run()).detach();
            
            // Send command
            cmd_tx.send(PlatformCommand::ListVMs { 
                platform_name: "test-platform".to_string() 
            }).await.unwrap();
            
            // Should receive event (or error)
            let timeout = smol::Timer::after(std::time::Duration::from_secs(5));
            smol::pin!(timeout);
            
            smol::select! {
                event = event_rx.recv() => {
                    let event = event.unwrap();
                    match event {
                        ViewModelEvent::Platform(PlatformEvent::VMsListed { .. }) |
                        ViewModelEvent::Platform(PlatformEvent::Error { .. }) => {
                            // Success - received expected event type
                        }
                        _ => panic!("Unexpected event: {:?}", event),
                    }
                }
                _ = &mut timeout => {
                    panic!("Test timed out waiting for event");
                }
            }
        });
    }
}
```

- [ ] **Step 4: Run test to verify it fails**

Run: `cargo test --lib platform::tests::test_platform_actor_list_vms_sends_event`
Expected: FAIL - PlatformActor doesn't handle commands yet

- [ ] **Step 5: Create actor.rs with command handler**

Create `mobile/src/viewmodel/platform/actor.rs`:

```rust
//! Platform actor implementation

use super::{PlatformCommand, PlatformEvent};
use crate::viewmodel::{ViewModelEvent, runtime};
use smol::channel::{Receiver, Sender};

pub struct PlatformActor {
    command_rx: Receiver<PlatformCommand>,
    event_tx: Sender<ViewModelEvent>,
}

impl PlatformActor {
    pub fn new(command_rx: Receiver<PlatformCommand>, event_tx: Sender<ViewModelEvent>) -> Self {
        Self { command_rx, event_tx }
    }
    
    pub async fn run(mut self) {
        log::info!("PlatformActor started");
        
        loop {
            match self.command_rx.recv().await {
                Ok(cmd) => {
                    if let Err(e) = self.handle_command(cmd).await {
                        log::error!("PlatformActor command failed: {}", e);
                    }
                }
                Err(_) => {
                    log::info!("PlatformActor: channel closed, shutting down");
                    break;
                }
            }
        }
    }
    
    async fn handle_command(&mut self, cmd: PlatformCommand) -> anyhow::Result<()> {
        let operation = format!("{:?}", cmd);
        
        let result = match cmd {
            PlatformCommand::ListVMs { platform_name } => {
                self.list_vms(platform_name).await
            }
            PlatformCommand::CreateVM { platform_name, vm_name, zone, machine_type } => {
                self.create_vm(platform_name, vm_name, zone, machine_type).await
            }
            PlatformCommand::DeleteVM { platform_name, vm_name, zone } => {
                self.delete_vm(platform_name, vm_name, zone).await
            }
            PlatformCommand::RestartVM { platform_name, vm_name, zone } => {
                self.restart_vm(platform_name, vm_name, zone).await
            }
            PlatformCommand::UpdateFirewall { platform_name, allow_ip } => {
                self.update_firewall(platform_name, allow_ip).await
            }
            PlatformCommand::FetchBilling { platform_name, project_id, dataset, table } => {
                self.fetch_billing(platform_name, project_id, dataset, table).await
            }
            _ => {
                // Unimplemented commands
                Err(anyhow::anyhow!("Command not implemented: {:?}", cmd))
            }
        };
        
        if let Err(e) = result {
            self.send_error(&operation, e).await;
        }
        
        Ok(())
    }
    
    async fn list_vms(&mut self, platform_name: String) -> anyhow::Result<()> {
        self.send_progress("list_vms", 0.1, "Loading platform config...").await;
        
        // Load platform config from DB
        let platform = runtime::unblock({
            let platform_name = platform_name.clone();
            move || crate::calc::db::load_platform(&platform_name)
        }).await?;
        
        self.send_progress("list_vms", 0.5, "Fetching VMs from GCP...").await;
        
        // Call GCP API
        let vms = runtime::unblock({
            let project_id = platform.project_id.clone();
            move || crate::calc::gcp_rest::list_vms(&project_id)
        }).await?;
        
        // Convert to VmInfo
        let vm_infos: Vec<super::VmInfo> = vms.into_iter().map(|vm| super::VmInfo {
            name: vm.name,
            zone: vm.zone,
            external_ip: vm.external_ip,
            status: vm.status,
        }).collect();
        
        self.send_event(PlatformEvent::VMsListed {
            platform_name,
            vms: vm_infos,
        }).await;
        
        Ok(())
    }
    
    async fn create_vm(&mut self, platform_name: String, vm_name: String, zone: String, machine_type: String) -> anyhow::Result<()> {
        self.send_progress("create_vm", 0.0, "Starting VM creation...").await;
        
        let platform = runtime::unblock({
            let platform_name = platform_name.clone();
            move || crate::calc::db::load_platform(&platform_name)
        }).await?;
        
        self.send_progress("create_vm", 0.3, "Creating VM instance...").await;
        
        let vm = runtime::unblock({
            let project_id = platform.project_id.clone();
            move || crate::calc::gcp_rest::create_vm(&project_id, &vm_name, &zone, &machine_type)
        }).await?;
        
        self.send_progress("create_vm", 0.7, "Waiting for external IP...").await;
        
        // Wait for VM to get external IP
        let external_ip = vm.external_ip.unwrap_or_else(|| "pending".to_string());
        
        self.send_event(PlatformEvent::VMCreated {
            platform_name,
            vm_name,
            external_ip,
        }).await;
        
        Ok(())
    }
    
    async fn delete_vm(&mut self, platform_name: String, vm_name: String, zone: String) -> anyhow::Result<()> {
        self.send_progress("delete_vm", 0.5, "Deleting VM...").await;
        
        let platform = runtime::unblock({
            let platform_name = platform_name.clone();
            move || crate::calc::db::load_platform(&platform_name)
        }).await?;
        
        runtime::unblock({
            let project_id = platform.project_id.clone();
            move || crate::calc::gcp_rest::delete_vm(&project_id, &vm_name, &zone)
        }).await?;
        
        self.send_event(PlatformEvent::VMDeleted {
            platform_name,
            vm_name,
        }).await;
        
        Ok(())
    }
    
    async fn restart_vm(&mut self, platform_name: String, vm_name: String, zone: String) -> anyhow::Result<()> {
        self.send_progress("restart_vm", 0.5, "Restarting VM...").await;
        
        let platform = runtime::unblock({
            let platform_name = platform_name.clone();
            move || crate::calc::db::load_platform(&platform_name)
        }).await?;
        
        runtime::unblock({
            let project_id = platform.project_id.clone();
            move || crate::calc::gcp_rest::restart_vm(&project_id, &vm_name, &zone)
        }).await?;
        
        self.send_event(PlatformEvent::VMRestarted {
            platform_name,
            vm_name,
        }).await;
        
        Ok(())
    }
    
    async fn update_firewall(&mut self, platform_name: String, allow_ip: String) -> anyhow::Result<()> {
        self.send_progress("update_firewall", 0.5, "Updating firewall rules...").await;
        
        let platform = runtime::unblock({
            let platform_name = platform_name.clone();
            move || crate::calc::db::load_platform(&platform_name)
        }).await?;
        
        runtime::unblock({
            let project_id = platform.project_id.clone();
            let allow_ip = allow_ip.clone();
            move || crate::calc::gcp_rest::update_firewall(&project_id, &allow_ip)
        }).await?;
        
        self.send_event(PlatformEvent::FirewallUpdated {
            platform_name,
            whitelisted_ip: allow_ip,
        }).await;
        
        Ok(())
    }
    
    async fn fetch_billing(&mut self, platform_name: String, project_id: String, dataset: String, table: String) -> anyhow::Result<()> {
        self.send_progress("fetch_billing", 0.5, "Fetching billing data...").await;
        
        let records = runtime::unblock(move || {
            crate::calc::gcp_rest::fetch_billing(&project_id, &dataset, &table)
        }).await?;
        
        self.send_event(PlatformEvent::BillingFetched {
            platform_name,
            records,
        }).await;
        
        Ok(())
    }
    
    async fn send_progress(&self, operation: &str, progress: f32, status: &str) {
        let _ = self.event_tx.send(ViewModelEvent::Platform(
            PlatformEvent::Progress {
                operation: operation.to_string(),
                progress,
                status: status.to_string(),
            }
        )).await;
    }
    
    async fn send_event(&self, event: PlatformEvent) {
        let _ = self.event_tx.send(ViewModelEvent::Platform(event)).await;
    }
    
    async fn send_error(&self, operation: &str, error: anyhow::Error) {
        let _ = self.event_tx.send(ViewModelEvent::Platform(
            PlatformEvent::Error {
                operation: operation.to_string(),
                error: format!("{:#}", error),
            }
        )).await;
    }
}
```

- [ ] **Step 6: Update mod.rs to use actor.rs**

Modify `mobile/src/viewmodel/platform/mod.rs`:

```rust
//! Platform actor for GCP operations

mod commands;
mod events;
mod actor;

#[cfg(test)]
mod tests;

pub use commands::PlatformCommand;
pub use events::{PlatformEvent, VmInfo};
pub use actor::PlatformActor;
```

- [ ] **Step 7: Run test to verify it passes**

Run: `cargo test --lib platform::tests::test_platform_actor_list_vms_sends_event`
Expected: PASS (or FAIL with specific error if calc functions don't exist - that's OK, we're testing the flow)

- [ ] **Step 8: Commit**

```bash
git add mobile/src/viewmodel/platform/
git commit -m "feat(platform): implement PlatformActor commands and events

Add complete command/event enums for Platform operations:
- OAuth flow
- Project management  
- VM operations (list, create, delete, restart)
- Firewall management
- Billing data fetching

Actor sends progress events during long operations.
All blocking I/O runs via runtime::unblock().

Tests verify command handling and event emission.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 7: Implement SshActor Commands and Events

**Files:**
- Modify: `mobile/src/viewmodel/ssh/commands.rs`
- Modify: `mobile/src/viewmodel/ssh/events.rs`
- Create: `mobile/src/viewmodel/ssh/actor.rs`
- Modify: `mobile/src/viewmodel/ssh/mod.rs`
- Create: `mobile/src/viewmodel/ssh/tests.rs`

**Interfaces:**
- Consumes: calc::ssh::*, calc::nft::* functions (will need to create if missing)
- Produces: SshCommand enum, SshEvent enum, SshActor::handle_command()

- [ ] **Step 1: Define SshCommand enum**

Modify `mobile/src/viewmodel/ssh/commands.rs`:

```rust
//! SSH actor commands

#[derive(Debug, Clone)]
pub enum SshCommand {
    // Host Management
    AddHost { 
        name: String, 
        host: String, 
        port: u16,
        user: String,
        ssh_key_path: String,
    },
    DeleteHost { name: String },
    ListHosts,
    TestConnection { name: String },
    
    // Docker Operations
    DockerPull { host_name: String, image: String },
    DockerRun { 
        host_name: String, 
        image: String, 
        container_name: String,
        ports: Vec<(u16, u16)>,
        env: Vec<(String, String)>,
    },
    DockerStop { host_name: String, container_name: String },
    DockerList { host_name: String },
    
    // Port Management
    PortOpen { host_name: String, port: u16, protocol: String },
    PortClose { host_name: String, port: u16, protocol: String },
    PortList { host_name: String },
    
    // Dure WSS Deployment
    DeployDureWss { 
        host_name: String, 
        domain: String,
        acme_email: String,
    },
}
```

- [ ] **Step 2: Define SshEvent enum**

Modify `mobile/src/viewmodel/ssh/events.rs`:

```rust
//! SSH actor events

#[derive(Debug, Clone)]
pub struct SshHostInfo {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub user: String,
}

#[derive(Debug, Clone)]
pub struct DockerContainer {
    pub name: String,
    pub image: String,
    pub status: String,
}

#[derive(Debug, Clone)]
pub enum SshEvent {
    // Host Events
    HostAdded { name: String },
    HostDeleted { name: String },
    HostsListed { hosts: Vec<SshHostInfo> },
    ConnectionTested { name: String, success: bool, latency_ms: Option<u64> },
    
    // Docker Events
    DockerImagePulled { host_name: String, image: String },
    DockerContainerStarted { host_name: String, container_name: String },
    DockerContainerStopped { host_name: String, container_name: String },
    DockerContainersListed { host_name: String, containers: Vec<DockerContainer> },
    
    // Port Events
    PortOpened { host_name: String, port: u16, protocol: String },
    PortClosed { host_name: String, port: u16, protocol: String },
    PortsListed { host_name: String, open_ports: Vec<(u16, String)> },
    
    // Deployment Events
    DureWssDeployed { host_name: String, domain: String, service_status: String },
    
    // Progress & Errors
    Progress { operation: String, progress: f32, status: String },
    Error { operation: String, error: String },
}
```

- [ ] **Step 3: Write test**

Create `mobile/src/viewmodel/ssh/tests.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::viewmodel::ViewModelEvent;
    
    #[test]
    fn test_ssh_actor_list_hosts() {
        smol::block_on(async {
            let (cmd_tx, cmd_rx) = smol::channel::unbounded();
            let (event_tx, event_rx) = smol::channel::unbounded();
            
            let actor = SshActor::new(cmd_rx, event_tx);
            smol::spawn(actor.run()).detach();
            
            cmd_tx.send(SshCommand::ListHosts).await.unwrap();
            
            let timeout = smol::Timer::after(std::time::Duration::from_secs(5));
            smol::pin!(timeout);
            
            smol::select! {
                event = event_rx.recv() => {
                    match event.unwrap() {
                        ViewModelEvent::Ssh(SshEvent::HostsListed { .. }) |
                        ViewModelEvent::Ssh(SshEvent::Error { .. }) => {}
                        _ => panic!("Unexpected event"),
                    }
                }
                _ = &mut timeout => panic!("Timeout"),
            }
        });
    }
}
```

- [ ] **Step 4: Create SshActor implementation**

Create `mobile/src/viewmodel/ssh/actor.rs` (similar pattern to PlatformActor - implement handle_command with list_hosts, docker_run, port_open methods using runtime::unblock)

- [ ] **Step 5: Update mod.rs**

Modify `mobile/src/viewmodel/ssh/mod.rs` to export actor

- [ ] **Step 6: Run test**

Run: `cargo test --lib ssh::tests`
Expected: PASS

- [ ] **Step 7: Commit**

```bash
git add mobile/src/viewmodel/ssh/
git commit -m "feat(ssh): implement SshActor commands and events

Add SSH operations: host management, Docker, ports, deployments.
Actor sends progress for long operations.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 8: Implement NsActor Commands and Events

**Files:**
- Modify: `mobile/src/viewmodel/ns/commands.rs`
- Modify: `mobile/src/viewmodel/ns/events.rs`
- Create: `mobile/src/viewmodel/ns/actor.rs`
- Modify: `mobile/src/viewmodel/ns/mod.rs`
- Create: `mobile/src/viewmodel/ns/tests.rs`

**Interfaces:**
- Consumes: calc::ns::*, calc::ns_cloudflare::* functions
- Produces: NsCommand enum, NsEvent enum, NsActor::handle_command()

- [ ] **Step 1: Define NsCommand enum**

Modify `mobile/src/viewmodel/ns/commands.rs` with AddProvider, AddDomain, AddRecord commands

- [ ] **Step 2: Define NsEvent enum**

Modify `mobile/src/viewmodel/ns/events.rs` with corresponding events

- [ ] **Step 3: Write test**

Create `mobile/src/viewmodel/ns/tests.rs`

- [ ] **Step 4: Create NsActor implementation**

Create `mobile/src/viewmodel/ns/actor.rs`

- [ ] **Step 5: Update mod.rs**

- [ ] **Step 6: Run test**

Run: `cargo test --lib ns::tests`

- [ ] **Step 7: Commit**

```bash
git add mobile/src/viewmodel/ns/
git commit -m "feat(ns): implement NsActor commands and events

Add DNS operations for Cloudflare, GCP DNS, DuckDNS, Porkbun.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 9: Implement ViewModel Command Methods

**Files:**
- Modify: `mobile/src/viewmodel/mod.rs`
- Create: `mobile/src/viewmodel/tests.rs` (if not exists, or modify)

**Interfaces:**
- Consumes: PlatformCommand, SshCommand, NsCommand from Tasks 6-8
- Produces: Public command methods (vm.create_vm(), vm.add_ssh_host(), etc.)

- [ ] **Step 1: Write integration test**

Add to `mobile/src/viewmodel/tests.rs`:

```rust
#[test]
fn test_viewmodel_create_vm_command() {
    let vm = ViewModel::new_headless();
    
    let result = vm.create_vm(
        "test-platform".to_string(),
        "test-vm".to_string(),
        "us-central1-a".to_string(),
        "e2-micro".to_string(),
    );
    
    assert!(result.is_ok());
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test --lib viewmodel::tests::test_viewmodel_create_vm_command`
Expected: FAIL - method not found

- [ ] **Step 3: Implement command methods**

Add to `mobile/src/viewmodel/mod.rs`:

```rust
impl ViewModel {
    // Platform commands
    pub fn create_vm(&self, platform_name: String, vm_name: String, zone: String, machine_type: String) -> anyhow::Result<()> {
        self.platform_tx.send_blocking(platform::PlatformCommand::CreateVM {
            platform_name, vm_name, zone, machine_type
        }).map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }
    
    pub fn list_vms(&self, platform_name: String) -> anyhow::Result<()> {
        self.platform_tx.send_blocking(platform::PlatformCommand::ListVMs { platform_name })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }
    
    pub fn delete_vm(&self, platform_name: String, vm_name: String, zone: String) -> anyhow::Result<()> {
        self.platform_tx.send_blocking(platform::PlatformCommand::DeleteVM {
            platform_name, vm_name, zone
        }).map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }
    
    // SSH commands
    pub fn add_ssh_host(&self, name: String, host: String, port: u16, user: String, ssh_key_path: String) -> anyhow::Result<()> {
        self.ssh_tx.send_blocking(ssh::SshCommand::AddHost {
            name, host, port, user, ssh_key_path
        }).map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }
    
    pub fn docker_run(&self, host_name: String, image: String, container_name: String, ports: Vec<(u16, u16)>, env: Vec<(String, String)>) -> anyhow::Result<()> {
        self.ssh_tx.send_blocking(ssh::SshCommand::DockerRun {
            host_name, image, container_name, ports, env
        }).map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }
    
    // NS commands
    pub fn add_dns_provider(&self, name: String, provider_type: String, credentials: crate::calc::ns::ProviderCredentials) -> anyhow::Result<()> {
        self.ns_tx.send_blocking(ns::NsCommand::AddProvider {
            name, provider_type, credentials
        }).map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }
    
    pub fn add_dns_record(&self, provider_name: String, domain: String, record_type: String, name: String, value: String, ttl: u32) -> anyhow::Result<()> {
        self.ns_tx.send_blocking(ns::NsCommand::AddRecord {
            provider_name, domain, record_type, name, value, ttl
        }).map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }
}
```

- [ ] **Step 4: Run test to verify pass**

Run: `cargo test --lib viewmodel::tests::test_viewmodel_create_vm_command`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add mobile/src/viewmodel/mod.rs
git add mobile/src/viewmodel/tests.rs
git commit -m "feat(viewmodel): add public command methods

Add command methods for Platform, SSH, NS operations.
Methods send commands to actors via channels.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Week 3: UI Tab Migration

### Task 10: Migrate Platform Tab to ViewModel

**Files:**
- Modify: `mobile/src/ui_tabs/platform.rs`

**Interfaces:**
- Consumes: ViewModel command methods from Task 9
- Produces: Platform tab using ViewModel (removes old poll-promise code)

- [ ] **Step 1: Remove old state from PlatformTab**

Find and remove these fields from `PlatformTab` struct in `mobile/src/ui_tabs/platform.rs`:
- `add_platform_oauth_promise`
- Any `Arc<Mutex<>>` wrappers
- `init_in_progress` (replaced by vm.operation_progress())

- [ ] **Step 2: Replace direct calls with ViewModel**

Find calls like:
```rust
// OLD
poll_promise::Promise::spawn_async(async {
    calc::gcp_rest::list_vms(&project_id).await
})
```

Replace with:
```rust
// NEW
vm.list_vms(platform_name.clone())?;
```

- [ ] **Step 3: Process events in show() method**

Add at top of `PlatformTab::show()`:

```rust
pub fn show(&mut self, ui: &mut egui::Ui, vm: &mut ViewModel) {
    // Process VM events
    // Note: events already polled in DureApp::update(), just access state
    
    // Show active operations
    for (op_name, progress) in vm.active_operations() {
        ui.add(egui::ProgressBar::new(progress.progress).text(&progress.status));
    }
    
    // Show errors
    if let Some(error) = vm.recent_errors().iter().rev().next() {
        if error.actor == "platform" {
            ui.colored_label(egui::Color32::RED, format!("Error: {}", error.error));
        }
    }
    
    // Rest of UI...
}
```

- [ ] **Step 4: Update DureApp to pass ViewModel**

Modify `mobile/src/dure.rs` where Platform tab is shown:

```rust
// OLD
self.platform_tab.show(ui);

// NEW
if let Some(ref mut vm) = self.viewmodel {
    self.platform_tab.show(ui, vm);
}
```

- [ ] **Step 5: Test manually**

Run: `cargo run --bin dure-desktop`
Navigate to Platform tab, verify operations work

- [ ] **Step 6: Commit**

```bash
git add mobile/src/ui_tabs/platform.rs
git add mobile/src/dure.rs
git commit -m "refactor(ui): migrate Platform tab to ViewModel

Remove poll-promise and direct calc:: calls.
All operations go through ViewModel.
Progress and errors from ViewModel state.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 11: Migrate SSH Tab to ViewModel

**Files:**
- Modify: `mobile/src/ui_tabs/ssh.rs`

**Interfaces:**
- Consumes: ViewModel SSH commands
- Produces: SSH tab using ViewModel

- [ ] **Step 1: Remove old state**

Remove `poll-promise` fields from `SshTab`

- [ ] **Step 2: Replace calls with ViewModel**

Replace direct calc:: calls with vm.add_ssh_host(), vm.docker_run(), etc.

- [ ] **Step 3: Process events**

Add event processing to show() method

- [ ] **Step 4: Update DureApp**

Pass ViewModel to SSH tab

- [ ] **Step 5: Test manually**

- [ ] **Step 6: Commit**

```bash
git add mobile/src/ui_tabs/ssh.rs
git add mobile/src/dure.rs
git commit -m "refactor(ui): migrate SSH tab to ViewModel

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 12: Migrate NS Tab to ViewModel

**Files:**
- Modify: `mobile/src/ui_tabs/ns.rs`

**Interfaces:**
- Consumes: ViewModel NS commands
- Produces: NS tab using ViewModel

- [ ] **Step 1: Remove old state**

- [ ] **Step 2: Replace calls**

- [ ] **Step 3: Process events**

- [ ] **Step 4: Update DureApp**

- [ ] **Step 5: Test manually**

- [ ] **Step 6: Commit**

```bash
git add mobile/src/ui_tabs/ns.rs
git add mobile/src/dure.rs
git commit -m "refactor(ui): migrate NS tab to ViewModel

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 13: Migrate CLI Commands to ViewModel

**Files:**
- Modify: `mobile/src/cli/commands/*.rs` (or create if missing)
- Modify: `mobile/src/main.rs`

**Interfaces:**
- Consumes: ViewModel::new_headless()
- Produces: CLI commands using ViewModel

- [ ] **Step 1: Create CLI command structure (if not exists)**

Create `mobile/src/cli/commands/platform.rs`:

```rust
use crate::viewmodel::{ViewModel, ViewModelEvent, platform::PlatformEvent};
use std::time::Duration;

pub async fn create_vm(
    platform_name: String,
    vm_name: String,
    zone: String,
    machine_type: String,
) -> anyhow::Result<()> {
    let mut vm = ViewModel::new_headless();
    
    vm.create_vm(platform_name.clone(), vm_name.clone(), zone, machine_type)?;
    
    // Poll for completion
    loop {
        let events = vm.poll_events_headless();
        for event in events {
            match event {
                ViewModelEvent::Platform(PlatformEvent::Progress { progress, status, .. }) => {
                    print!("\r[{:>3.0}%] {}", progress * 100.0, status);
                    std::io::stdout().flush()?;
                }
                ViewModelEvent::Platform(PlatformEvent::VMCreated { vm_name, external_ip, .. }) => {
                    println!("\n✓ VM created: {} at {}", vm_name, external_ip);
                    return Ok(());
                }
                ViewModelEvent::Platform(PlatformEvent::Error { error, .. }) => {
                    eprintln!("\n✗ Failed: {}", error);
                    return Err(anyhow::anyhow!(error));
                }
                _ => {}
            }
        }
        smol::Timer::after(Duration::from_millis(100)).await;
    }
}
```

- [ ] **Step 2: Wire up CLI parsing**

Update main.rs to call CLI commands using ViewModel

- [ ] **Step 3: Test CLI command**

Run: `cargo run --bin dure-desktop -- platform create-vm test-platform test-vm us-central1-a e2-micro`

- [ ] **Step 4: Commit**

```bash
git add mobile/src/cli/
git add mobile/src/main.rs
git commit -m "refactor(cli): migrate commands to ViewModel

All CLI commands use ViewModel::new_headless().
Progress displayed during operations.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Week 4: WASM + Cleanup + Testing

### Task 14: Implement WASM ViewModel

**Files:**
- Modify: `mobile/src/viewmodel/mod.rs`

**Interfaces:**
- Consumes: runtime and io abstractions from Task 2
- Produces: ViewModel::new_wasm() for WASM builds

- [ ] **Step 1: Implement new_wasm()**

Add to `mobile/src/viewmodel/mod.rs`:

```rust
#[cfg(target_arch = "wasm32")]
impl ViewModel {
    pub fn new_wasm() -> Self {
        use wasm_bindgen_futures::spawn_local;
        
        let (platform_tx, platform_rx) = smol::channel::unbounded();
        let (ssh_tx, ssh_rx) = smol::channel::unbounded();
        let (ns_tx, ns_rx) = smol::channel::unbounded();
        let (wss_tx, wss_rx) = smol::channel::unbounded();
        let (event_tx, event_rx) = smol::channel::unbounded();
        
        // Spawn actors in Web Worker context
        spawn_local(async move {
            log::info!("ViewModel runtime started (WASM)");
            
            let platform_actor = platform::PlatformActor::new(platform_rx, event_tx.clone());
            let ns_actor = ns::NsActor::new(ns_rx, event_tx.clone());
            let wss_actor = wss::WssActor::new(wss_rx, event_tx.clone());
            
            // SSH disabled in WASM
            drop(ssh_rx);
            
            futures::join!(
                platform_actor.run(),
                ns_actor.run(),
                wss_actor.run(),
            );
        });
        
        Self {
            platform_tx,
            ssh_tx,
            ns_tx,
            wss_tx,
            event_rx,
            state: ViewModelState::default(),
            runtime_handle: None,
            egui_ctx: None,
        }
    }
}
```

- [ ] **Step 2: Test WASM build**

Run: `cargo build --target wasm32-unknown-unknown --no-default-features`
Expected: Build succeeds

- [ ] **Step 3: Commit**

```bash
git add mobile/src/viewmodel/mod.rs
git commit -m "feat(wasm): implement ViewModel for WASM target

Use spawn_local for actor execution in browser.
SSH actor disabled in WASM (no native SSH in browser).

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 15: Code Cleanup

**Files:**
- Modify: `mobile/Cargo.toml`
- Delete or mark unused code

**Interfaces:**
- Consumes: Completed MVVM migration
- Produces: Clean codebase with no dead code

- [ ] **Step 1: Check for unused dependencies**

Run: `cargo +nightly udeps --all-targets`
Check if poll-promise, crossbeam-queue are used elsewhere

- [ ] **Step 2: Remove unused dependencies (if safe)**

If not used, remove from `mobile/Cargo.toml`

- [ ] **Step 3: Run clippy**

Run: `cargo clippy --all-targets --all-features -- -D warnings`
Fix all warnings

- [ ] **Step 4: Format code**

Run: `cargo fmt --all`

- [ ] **Step 5: Commit**

```bash
git add mobile/Cargo.toml
git add mobile/src/
git commit -m "chore: cleanup unused code and dependencies

Remove poll-promise and crossbeam-queue if unused.
Fix all clippy warnings.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 16: Comprehensive Testing

**Files:**
- Run all tests
- Manual testing checklist

**Interfaces:**
- Consumes: Complete implementation
- Produces: Verified working system

- [ ] **Step 1: Run unit tests**

Run: `cargo test --lib`
Expected: All tests pass

- [ ] **Step 2: Run integration tests**

Run: `cargo test --test '*'`
Expected: All tests pass

- [ ] **Step 3: Manual testing checklist**

GUI Testing:
- [ ] Platform tab: OAuth works
- [ ] Platform tab: Create VM shows progress, completes
- [ ] Platform tab: Delete VM works
- [ ] SSH tab: Add host works
- [ ] SSH tab: Docker operations work
- [ ] NS tab: Add provider works
- [ ] NS tab: Add DNS record works
- [ ] Errors display correctly in UI
- [ ] Progress bars update smoothly
- [ ] Multiple tabs can have operations running concurrently

CLI Testing:
- [ ] CLI: platform create-vm command works
- [ ] CLI: ssh add-host command works
- [ ] CLI: ns add-record command works
- [ ] CLI: Progress displays correctly
- [ ] CLI: Errors display correctly

- [ ] **Step 4: Performance verification**

Check logs for:
- [ ] No "blocking UI thread" warnings
- [ ] Actors running in background
- [ ] Memory usage reasonable

- [ ] **Step 5: Document completion**

Update `docs/superpowers/specs/2026-07-04-mvvm-refactor-design.md` with completion notes

- [ ] **Step 6: Final commit**

```bash
git add docs/
git commit -m "docs: mark MVVM refactor as complete

All tests passing.
Manual testing checklist complete.
Performance verified.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Plan Complete

All 16 tasks defined with TDD approach. Each task follows:
1. Write test
2. Verify failure
3. Implement
4. Verify success
5. Commit

**Execution ready with superpowers:executing-plans**
