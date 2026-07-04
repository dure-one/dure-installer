# MVVM Actor-Based Architecture Refactoring - Dure

**Date:** 2026-07-04  
**Status:** Approved Design  
**Goal:** Refactor Dure to use actor-based MVVM architecture with smol async runtime for all deployment modes (Desktop, CLI, Android, WASM)

## Executive Summary

Refactor Dure from direct UI-blocking I/O to a clean **actor-based MVVM architecture** where:
- UI operations are non-blocking with smooth progress updates
- Database, network, SSH, and cloud operations run on background actors
- Actors communicate via typed channels using smol async runtime
- Single coordination layer (ViewModel) replaces scattered state management
- Clear separation between UI (view), coordination (ViewModel), and business logic (actors)
- All deployment modes (GUI, CLI, WASM) share same ViewModel architecture

## Objectives

### Primary Goals
1. **Unified architecture**: Single ViewModel API for GUI, CLI, and WASM modes
2. **Non-blocking I/O**: Move all blocking operations (DB, network, SSH, GCP) to background actors
3. **Clean separation**: Actor-based MVVM with clear boundaries and testable components
4. **Cross-platform async**: Use `smol` for desktop/Android, smol components for WASM
5. **Preserve functionality**: Keep all existing features working, improve UX with progress updates

### Non-Goals
- Changing `calc/*.rs` implementation (reuse as-is)
- Changing database schema or Diesel models
- Adding new features beyond architecture refactoring
- Implementing WSS server/client (design included, implementation is stub)

## Architecture Overview

### System Layers

```
┌─────────────────────────────────────────────────────┐
│                    UI Layer                         │
│  Main Thread - egui rendering (GUI) or stdio (CLI)  │
│                                                      │
│  DureApp owns ViewModel (GUI mode)                  │
│  CLI main owns ViewModel (headless mode)            │
│  Tabs/Commands call ViewModel methods               │
└──────────────────┬──────────────────────────────────┘
                   │ Command methods
                   │ Event polling (.poll_events())
                   │ State getters (.platforms(), etc.)
┌──────────────────▼──────────────────────────────────┐
│              ViewModel Layer                        │
│  Coordination - lives on UI thread (GUI) or main    │
│  thread (CLI)                                       │
│                                                      │
│  • Holds channels to actors                         │
│  • Provides unified command API                     │
│  • Aggregates events from all actors                │
│  • Exposes transient state (progress, errors)       │
└──────────────────┬──────────────────────────────────┘
                   │ smol::channel
                   │ Commands ↓ Events ↑
┌──────────────────▼──────────────────────────────────┐
│               Actor Layer                           │
│  Background Thread - smol executor                  │
│                                                      │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐           │
│  │Platform  │ │   SSH    │ │    NS    │           │
│  │ Actor    │ │  Actor   │ │  Actor   │           │
│  └──────────┘ └──────────┘ └──────────┘           │
│                 ┌──────────┐                        │
│                 │   WSS    │                        │
│                 │  Actor   │  (stub)                │
│                 └──────────┘                        │
│                                                      │
│  Each actor: async command loop + DB access         │
└──────────────────┬──────────────────────────────────┘
                   │ Function calls (reuse existing)
┌──────────────────▼──────────────────────────────────┐
│            Business Logic + Data Layer              │
│  (Unchanged - reused by actors)                     │
│                                                      │
│  calc/*.rs - business logic (gcp, ssh, dns, etc.)   │
│  storage/ - Diesel models & schema                  │
│  Database - SQLite/PostgreSQL (durable state)       │
└─────────────────────────────────────────────────────┘
```

### Platform-Specific Runtime

**Desktop/Android:**
- Full `smol` runtime (`smol::block_on`, `smol::spawn`)
- Native async I/O (`async-io`, `smol::Timer`)
- Multi-threaded executor

**WASM:**
- `async-executor` + `async-task` (smol components)
- `gloo-timers` for timeouts (replaces `smol::Timer`)
- `web-sys::fetch` for HTTP (replaces native I/O)
- Web Workers for background execution

**CLI Headless:**
- Same as Desktop but `ViewModel::new_headless()` (no egui Context)
- `smol::block_on` in main() to await operations synchronously

### Message Flow Example

```
User clicks "Create VM" (GUI) or runs CLI command
  ↓
UI/CLI calls: vm.create_vm(platform, vm_name, zone, type)
  ↓
ViewModel sends: PlatformCommand::CreateVM { ... }
  ↓ (via channel)
PlatformActor receives command
  ↓
Actor calls: smol::unblock(|| calc::gcp_rest::create_vm(...))
  ↓
Actor sends: ViewModelEvent::Platform(PlatformEvent::Progress { ... })
  ↓ (via channel)
ViewModel.poll_events() receives event
  ↓
ViewModel updates internal state
  ↓
UI renders updated state / CLI prints progress
```

## Actor Domains

### 1. PlatformActor

**Responsibilities:**
- GCP OAuth authentication flow
- Project listing and selection
- VM creation, deletion, restart
- Firewall rule management (IP whitelisting)
- Billing data fetching
- SSH key generation and storage (via keyring)

**Commands:**
```rust
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

**Events:**
```rust
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

---

### 2. SshActor

**Responsibilities:**
- SSH host management (add, delete, list)
- Docker container operations (pull, run, stop, remove)
- nftables port management (open, close)
- Ansible role deployment
- System hardening (Jangbi integration)
- Dure WSS service deployment

**Commands:**
```rust
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
        ports: Vec<(u16, u16)>,  // (host_port, container_port)
        env: Vec<(String, String)>,
    },
    DockerStop { host_name: String, container_name: String },
    DockerRemove { host_name: String, container_name: String },
    DockerList { host_name: String },
    
    // Port Management (nftables)
    PortOpen { host_name: String, port: u16, protocol: String },
    PortClose { host_name: String, port: u16, protocol: String },
    PortList { host_name: String },
    
    // Ansible Operations
    AnsibleInstallRole { host_name: String, role_name: String },
    AnsibleRunPlaybook { host_name: String, playbook_path: String },
    
    // System Hardening
    HardenSystem { host_name: String },
    
    // Dure WSS Deployment
    DeployDureWss { 
        host_name: String, 
        domain: String,
        acme_email: String,
    },
}
```

**Events:**
```rust
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

---

### 3. NsActor

**Responsibilities:**
- DNS nameserver management (Cloudflare, GCP Cloud DNS, DuckDNS, Porkbun)
- Add/delete domains
- Add/delete DNS records (A, AAAA, TXT, CNAME, MX)
- DNS verification for site-to-site auth

**Commands:**
```rust
pub enum NsCommand {
    // Provider Management
    AddProvider { 
        name: String, 
        provider_type: String,  // "cloudflare", "gcp", "duckdns", "porkbun"
        credentials: ProviderCredentials,
    },
    DeleteProvider { name: String },
    ListProviders,
    
    // Domain Management
    AddDomain { provider_name: String, domain: String },
    DeleteDomain { provider_name: String, domain: String },
    ListDomains { provider_name: String },
    
    // DNS Record Operations
    AddRecord { 
        provider_name: String, 
        domain: String,
        record_type: String,  // "A", "AAAA", "TXT", "CNAME", "MX"
        name: String,
        value: String,
        ttl: u32,
    },
    DeleteRecord { 
        provider_name: String, 
        domain: String,
        record_id: String,
    },
    ListRecords { provider_name: String, domain: String },
    
    // Site-to-Site Auth
    PublishSiteKey { 
        provider_name: String, 
        domain: String,
        public_key: String,  // ed25519 public key
    },
    VerifySiteKey { domain: String },
}
```

**Events:**
```rust
pub enum NsEvent {
    // Provider Events
    ProviderAdded { name: String, provider_type: String },
    ProviderDeleted { name: String },
    ProvidersListed { providers: Vec<DnsProviderInfo> },
    
    // Domain Events
    DomainAdded { provider_name: String, domain: String },
    DomainDeleted { provider_name: String, domain: String },
    DomainsListed { provider_name: String, domains: Vec<String> },
    
    // Record Events
    RecordAdded { 
        provider_name: String, 
        domain: String,
        record_id: String,
        record_type: String,
    },
    RecordDeleted { provider_name: String, domain: String, record_id: String },
    RecordsListed { 
        provider_name: String, 
        domain: String,
        records: Vec<DnsRecord>,
    },
    
    // Auth Events
    SiteKeyPublished { domain: String, txt_record: String },
    SiteKeyVerified { domain: String, valid: bool, public_key: Option<String> },
    
    // Progress & Errors
    Progress { operation: String, progress: f32, status: String },
    Error { operation: String, error: String },
}
```

---

### 4. WssActor (Stub Implementation)

**Responsibilities:**
- WebSocket client connections (to partner stores)
- WebSocket server management (own store WSS service)
- Message routing between UI and WSS connections
- Connection health monitoring
- Reconnection logic

**Commands:**
```rust
pub enum WssCommand {
    // Client Operations
    ConnectClient { url: String, auth_token: Option<String> },
    DisconnectClient { connection_id: String },
    SendMessage { connection_id: String, message: Vec<u8> },
    
    // Server Operations (if running WSS service locally)
    StartServer { bind_address: String, port: u16, tls_cert_path: String },
    StopServer,
    BroadcastMessage { message: Vec<u8> },
    
    // Connection Management
    ListConnections,
    PingConnection { connection_id: String },
}
```

**Events:**
```rust
pub enum WssEvent {
    // Client Events
    ClientConnected { connection_id: String, url: String },
    ClientDisconnected { connection_id: String, reason: String },
    MessageReceived { connection_id: String, message: Vec<u8> },
    
    // Server Events
    ServerStarted { bind_address: String, port: u16 },
    ServerStopped,
    ClientConnectedToServer { client_id: String, remote_addr: String },
    ClientDisconnectedFromServer { client_id: String },
    
    // Connection Health
    ConnectionsListed { connections: Vec<WssConnectionInfo> },
    PongReceived { connection_id: String, latency_ms: u64 },
    
    // Errors
    Error { operation: String, error: String },
}
```

**Implementation Note:** WssActor is designed but implemented as a no-op stub. All commands log warnings and send "not implemented" errors. Full implementation will be added when WSS features are built.

## ViewModel Structure

### Core ViewModel

```rust
pub struct ViewModel {
    // Actor communication channels
    platform_tx: smol::channel::Sender<PlatformCommand>,
    ssh_tx: smol::channel::Sender<SshCommand>,
    ns_tx: smol::channel::Sender<NsCommand>,
    wss_tx: smol::channel::Sender<WssCommand>,
    
    // Unified event receiver (all actors send here)
    event_rx: smol::channel::Receiver<ViewModelEvent>,
    
    // Transient state (progress, errors, UI state)
    state: ViewModelState,
    
    // Runtime handle (None in WASM - uses different executor)
    runtime_handle: Option<RuntimeHandle>,
    
    // Optional egui context (for texture loading, UI requests)
    #[cfg(feature = "gui")]
    egui_ctx: Option<egui::Context>,
}

// Platform-specific runtime handle
enum RuntimeHandle {
    #[cfg(not(target_arch = "wasm32"))]
    Native(std::thread::JoinHandle<()>),
    
    #[cfg(target_arch = "wasm32")]
    Wasm(WasmExecutorHandle),
}

pub struct ViewModelState {
    // Active operations (operation_id -> progress)
    pub active_operations: HashMap<String, OperationProgress>,
    
    // Recent errors (last 50, for error log UI)
    pub recent_errors: VecDeque<ErrorRecord>,
    
    // WSS connection state
    pub wss_connections: HashMap<String, WssConnectionInfo>,
    
    // Platform transient state
    pub platform_oauth_in_progress: HashMap<String, OAuthProgress>,
    
    // SSH operation state
    pub ssh_operations: HashMap<String, SshOperationStatus>,
    
    // Textures (loaded by actors, stored here for UI)
    #[cfg(feature = "gui")]
    pub textures: HashMap<String, egui::TextureHandle>,
}

pub struct OperationProgress {
    pub operation: String,
    pub progress: f32,  // 0.0 to 1.0
    pub status: String,
    pub started_at: std::time::Instant,
}

pub struct ErrorRecord {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub operation: String,
    pub error: String,
    pub actor: String,  // "platform", "ssh", "ns", "wss"
}

// Unified event type
pub enum ViewModelEvent {
    Platform(PlatformEvent),
    Ssh(SshEvent),
    Ns(NsEvent),
    Wss(WssEvent),
}
```

### Initialization (Platform-Specific)

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
                // Create actors
                let platform_actor = PlatformActor::new(platform_rx, event_tx.clone());
                let ssh_actor = SshActor::new(ssh_rx, event_tx.clone());
                let ns_actor = NsActor::new(ns_rx, event_tx.clone());
                let wss_actor = WssActor::new(wss_rx, event_tx.clone());
                
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
        // Same as new() but without egui::Context
        // ... (implementation omitted for brevity)
    }
    
    /// Create ViewModel for WASM mode
    #[cfg(target_arch = "wasm32")]
    pub fn new_wasm() -> Self {
        // Use async-executor in Web Worker
        // ... (implementation omitted for brevity)
    }
}
```

### Command Methods (Public API)

```rust
impl ViewModel {
    // === Platform Commands ===
    pub fn start_oauth(&self, platform_name: String) -> anyhow::Result<()> {
        self.platform_tx.send_blocking(PlatformCommand::StartOAuth { platform_name })
            .map_err(|e| anyhow::anyhow!("Failed to send command: {}", e))
    }
    
    pub fn create_vm(&self, platform_name: String, vm_name: String, zone: String, machine_type: String) -> anyhow::Result<()> {
        self.platform_tx.send_blocking(PlatformCommand::CreateVM { 
            platform_name, vm_name, zone, machine_type 
        }).map_err(|e| anyhow::anyhow!("Failed to send command: {}", e))
    }
    
    // === SSH Commands ===
    pub fn add_ssh_host(&self, name: String, host: String, port: u16, user: String, ssh_key_path: String) -> anyhow::Result<()> {
        self.ssh_tx.send_blocking(SshCommand::AddHost { 
            name, host, port, user, ssh_key_path 
        }).map_err(|e| anyhow::anyhow!("Failed to send command: {}", e))
    }
    
    pub fn docker_run(&self, host_name: String, image: String, container_name: String, 
                      ports: Vec<(u16, u16)>, env: Vec<(String, String)>) -> anyhow::Result<()> {
        self.ssh_tx.send_blocking(SshCommand::DockerRun { 
            host_name, image, container_name, ports, env 
        }).map_err(|e| anyhow::anyhow!("Failed to send command: {}", e))
    }
    
    // === NS Commands ===
    pub fn add_dns_provider(&self, name: String, provider_type: String, credentials: ProviderCredentials) -> anyhow::Result<()> {
        self.ns_tx.send_blocking(NsCommand::AddProvider { 
            name, provider_type, credentials 
        }).map_err(|e| anyhow::anyhow!("Failed to send command: {}", e))
    }
    
    pub fn add_dns_record(&self, provider_name: String, domain: String, record_type: String, 
                          name: String, value: String, ttl: u32) -> anyhow::Result<()> {
        self.ns_tx.send_blocking(NsCommand::AddRecord { 
            provider_name, domain, record_type, name, value, ttl 
        }).map_err(|e| anyhow::anyhow!("Failed to send command: {}", e))
    }
    
    // === WSS Commands ===
    pub fn connect_wss_client(&self, url: String, auth_token: Option<String>) -> anyhow::Result<()> {
        self.wss_tx.send_blocking(WssCommand::ConnectClient { url, auth_token })
            .map_err(|e| anyhow::anyhow!("Failed to send command: {}", e))
    }
    
    pub fn send_wss_message(&self, connection_id: String, message: Vec<u8>) -> anyhow::Result<()> {
        self.wss_tx.send_blocking(WssCommand::SendMessage { connection_id, message })
            .map_err(|e| anyhow::anyhow!("Failed to send command: {}", e))
    }
}
```

### Event Processing

```rust
impl ViewModel {
    /// Poll for events and update state. Call this in update loop (GUI) or after commands (CLI).
    #[cfg(feature = "gui")]
    pub fn poll_events(&mut self, ctx: &egui::Context) -> Vec<ViewModelEvent> {
        let mut events = Vec::new();
        
        // Non-blocking receive all available events
        while let Ok(event) = self.event_rx.try_recv() {
            self.apply_event(&event, Some(ctx));
            events.push(event);
        }
        
        // Request repaint if there were events
        if !events.is_empty() {
            ctx.request_repaint();
        }
        
        events
    }
    
    /// Poll events without egui context (CLI mode)
    pub fn poll_events_headless(&mut self) -> Vec<ViewModelEvent> {
        let mut events = Vec::new();
        
        while let Ok(event) = self.event_rx.try_recv() {
            self.apply_event(&event, None);
            events.push(event);
        }
        
        events
    }
    
    /// Apply event to internal transient state
    fn apply_event(&mut self, event: &ViewModelEvent, ctx: Option<&egui::Context>) {
        match event {
            // Platform Events
            ViewModelEvent::Platform(PlatformEvent::Progress { operation, progress, status }) => {
                self.state.active_operations.insert(
                    operation.clone(),
                    OperationProgress {
                        operation: operation.clone(),
                        progress: *progress,
                        status: status.clone(),
                        started_at: std::time::Instant::now(),
                    }
                );
            }
            ViewModelEvent::Platform(PlatformEvent::Error { operation, error }) => {
                self.state.active_operations.remove(operation);
                self.state.recent_errors.push_back(ErrorRecord {
                    timestamp: chrono::Utc::now(),
                    operation: operation.clone(),
                    error: error.clone(),
                    actor: "platform".to_string(),
                });
                if self.state.recent_errors.len() > 50 {
                    self.state.recent_errors.pop_front();
                }
            }
            
            // SSH, NS, WSS events follow same pattern...
            _ => {}
        }
    }
    
    // === Read-only state accessors ===
    pub fn active_operations(&self) -> &HashMap<String, OperationProgress> {
        &self.state.active_operations
    }
    
    pub fn recent_errors(&self) -> &VecDeque<ErrorRecord> {
        &self.state.recent_errors
    }
    
    pub fn operation_progress(&self, operation: &str) -> Option<&OperationProgress> {
        self.state.active_operations.get(operation)
    }
}
```

## WASM Compatibility Layer

### Runtime Abstraction

**File: `mobile/src/viewmodel/runtime.rs`**

```rust
#[cfg(not(target_arch = "wasm32"))]
pub use native::*;

#[cfg(target_arch = "wasm32")]
pub use wasm::*;

// === Native Runtime (Desktop/Android) ===
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

// === WASM Runtime ===
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
        // WASM doesn't have threads, run on main thread
        f()
    }
    
    pub async fn sleep(duration: std::time::Duration) {
        let millis = duration.as_millis() as i32;
        gloo_timers::future::sleep(std::time::Duration::from_millis(millis as u64)).await;
    }
}
```

### I/O Abstraction

**File: `mobile/src/viewmodel/io.rs`**

```rust
#[cfg(not(target_arch = "wasm32"))]
pub use native::*;

#[cfg(target_arch = "wasm32")]
pub use wasm::*;

// === Native I/O ===
#[cfg(not(target_arch = "wasm32"))]
mod native {
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

// === WASM I/O ===
#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Request, RequestInit, Response};
    
    pub async fn http_get(url: &str) -> anyhow::Result<Vec<u8>> {
        let window = web_sys::window().ok_or_else(|| anyhow::anyhow!("no window"))?;
        let resp_value = JsFuture::from(window.fetch_with_str(url)).await?;
        let resp: Response = resp_value.dyn_into()?;
        let array_buffer = JsFuture::from(resp.array_buffer()?).await?;
        let uint8_array = js_sys::Uint8Array::new(&array_buffer);
        Ok(uint8_array.to_vec())
    }
    
    pub async fn http_post(url: &str, body: Vec<u8>) -> anyhow::Result<Vec<u8>> {
        let mut opts = RequestInit::new();
        opts.method("POST");
        opts.body(Some(&js_sys::Uint8Array::from(&body[..]).into()));
        
        let request = Request::new_with_str_and_init(url, &opts)?;
        let window = web_sys::window().ok_or_else(|| anyhow::anyhow!("no window"))?;
        let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
        let resp: Response = resp_value.dyn_into()?;
        let array_buffer = JsFuture::from(resp.array_buffer()?).await?;
        let uint8_array = js_sys::Uint8Array::new(&array_buffer);
        Ok(uint8_array.to_vec())
    }
}
```

### WASM Limitations

**What works:**
- Async/await (via wasm-bindgen-futures)
- HTTP requests (via web-sys::fetch)
- Timers (via gloo-timers)
- Channels (smol::channel works in WASM)
- JSON serialization

**What doesn't work:**
- File system access (use IndexedDB instead)
- Direct TCP/SSH (must proxy through HTTPS API)
- Native threads (use Web Workers with special setup)
- Full Diesel database (use in-memory or WASM-compatible alternative)

**WASM-specific actor implementations:**
- SshActor disabled in WASM (SSH operations not available in browser)
- PlatformActor works (GCP APIs are HTTP-based)
- NsActor works (DNS provider APIs are HTTP-based)
- WssActor stub (to be implemented with web-sys::WebSocket)

## Error Handling Strategy

### Error Flow Architecture

```
Actor Operation Fails
    ↓
Actor catches error (doesn't crash)
    ↓
Actor sends Error event to ViewModel
    ↓
ViewModel stores in recent_errors
    ↓
UI polls events, receives Error event
    ↓
UI displays error (toast/dialog/status bar)
```

### Actor-Level Error Handling

```rust
impl PlatformActor {
    async fn run(mut self) {
        log::info!("PlatformActor started");
        
        loop {
            match self.command_rx.recv().await {
                Ok(cmd) => {
                    // Handle command, catch errors
                    if let Err(e) = self.handle_command(cmd).await {
                        log::error!("Command failed: {}", e);
                        // Error already sent as event in handle_command
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
            PlatformCommand::CreateVM { platform_name, vm_name, zone, machine_type } => {
                self.create_vm(platform_name, vm_name, zone, machine_type).await
            }
            // ... other commands
        };
        
        // Convert error to event
        if let Err(e) = result {
            self.send_error(&operation, e).await;
        }
        
        Ok(())  // Actor keeps running
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

### ViewModel Error Aggregation

```rust
impl ViewModel {
    fn apply_event(&mut self, event: &ViewModelEvent, ctx: Option<&egui::Context>) {
        match event {
            // All actors use same error handling pattern
            ViewModelEvent::Platform(PlatformEvent::Error { operation, error }) |
            ViewModelEvent::Ssh(SshEvent::Error { operation, error }) |
            ViewModelEvent::Ns(NsEvent::Error { operation, error }) |
            ViewModelEvent::Wss(WssEvent::Error { operation, error }) => {
                let actor = match event {
                    ViewModelEvent::Platform(_) => "platform",
                    ViewModelEvent::Ssh(_) => "ssh",
                    ViewModelEvent::Ns(_) => "ns",
                    ViewModelEvent::Wss(_) => "wss",
                    _ => "unknown",
                };
                
                // Store in error log
                self.state.recent_errors.push_back(ErrorRecord {
                    timestamp: chrono::Utc::now(),
                    operation: operation.clone(),
                    error: error.clone(),
                    actor: actor.to_string(),
                });
                
                // Limit to last 50 errors
                if self.state.recent_errors.len() > 50 {
                    self.state.recent_errors.pop_front();
                }
                
                // Remove from active operations
                self.state.active_operations.remove(operation);
                
                // Log to console/file
                log::error!("[{}] {} failed: {}", actor, operation, error);
            }
            _ => {}
        }
    }
}
```

### UI Error Display Patterns

**Pattern 1: Toast Notifications (Non-critical errors)**
```rust
impl DureApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let events = self.viewmodel.poll_events(ctx);
        
        for event in events {
            match event {
                ViewModelEvent::Platform(PlatformEvent::Error { operation, error }) => {
                    egui::toast_info(ctx, format!("Operation failed: {}", operation));
                }
                _ => {}
            }
        }
    }
}
```

**Pattern 2: CLI Error Display**
```rust
fn main() -> anyhow::Result<()> {
    let vm = ViewModel::new_headless();
    vm.create_vm("platform".into(), "vm".into(), "zone".into(), "type".into())?;
    
    loop {
        let events = vm.poll_events_headless();
        for event in events {
            match event {
                ViewModelEvent::Platform(PlatformEvent::Error { operation, error }) => {
                    eprintln!("✗ {} failed:", operation);
                    eprintln!("  {}", error);
                    return Err(anyhow::anyhow!(error));
                }
                _ => {}
            }
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}
```

## Testing Strategy

### Three-Level Testing Approach

**Level 1: Actor Unit Tests** - Test individual actors in isolation  
**Level 2: ViewModel Integration Tests** - Test actor coordination via ViewModel  
**Level 3: End-to-End Tests** - Test full UI → ViewModel → Actor → DB flow

### Level 1: Actor Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_platform_actor_create_vm() {
        smol::block_on(async {
            let (cmd_tx, cmd_rx) = smol::channel::unbounded();
            let (event_tx, event_rx) = smol::channel::unbounded();
            
            let actor = PlatformActor::new(cmd_rx, event_tx);
            smol::spawn(actor.run()).detach();
            
            cmd_tx.send(PlatformCommand::CreateVM {
                platform_name: "test".into(),
                vm_name: "test-vm".into(),
                zone: "us-central1-a".into(),
                machine_type: "e2-micro".into(),
            }).await.unwrap();
            
            // Verify progress and completion events
            let mut vm_created = false;
            loop {
                if let Ok(event) = event_rx.recv().await {
                    if matches!(event, ViewModelEvent::Platform(PlatformEvent::VMCreated { .. })) {
                        vm_created = true;
                        break;
                    }
                }
            }
            assert!(vm_created);
        });
    }
}
```

### Level 2: ViewModel Integration Tests

```rust
#[test]
fn test_viewmodel_concurrent_operations() {
    smol::block_on(async {
        let mut vm = ViewModel::new_headless();
        
        // Start multiple operations
        vm.list_vms("platform1".into()).unwrap();
        vm.list_ssh_hosts().unwrap();
        vm.list_dns_providers().unwrap();
        
        smol::Timer::after(Duration::from_secs(2)).await;
        let events = vm.poll_events_headless();
        
        // Verify all actors responded
        assert!(events.iter().any(|e| matches!(e, ViewModelEvent::Platform(_))));
        assert!(events.iter().any(|e| matches!(e, ViewModelEvent::Ssh(_))));
        assert!(events.iter().any(|e| matches!(e, ViewModelEvent::Ns(_))));
    });
}
```

### Test Organization

```
mobile/
├── src/
│   └── viewmodel/
│       ├── platform/
│       │   └── tests.rs          # Actor unit tests
│       ├── ssh/
│       │   └── tests.rs
│       ├── ns/
│       │   └── tests.rs
│       └── tests.rs                # ViewModel integration tests
└── tests/
    ├── e2e_platform.rs             # E2E tests
    ├── e2e_ssh.rs
    └── e2e_ns.rs
```

## Migration Plan (4-Week Big Bang)

### Week 1: Infrastructure Setup

**Goal:** Complete ViewModel skeleton and actor foundations

**Tasks:**
1. Update `Cargo.toml` dependencies (add smol, async-executor, gloo-timers)
2. Create viewmodel module structure
3. Implement ViewModel skeleton with platform-specific initialization
4. Add ViewModel to DureApp (optional field initially)
5. Background thread spawning and empty actor run loops

**Success Criteria:**
- ✅ App compiles and runs
- ✅ ViewModel spawns 4 actors in background
- ✅ Unit test: ViewModel initializes without panic

---

### Week 2: Actor Implementation

**Goal:** Implement all three actors + WSS stub

**Day 1-2:** PlatformActor implementation
- OAuth, project listing, VM operations, billing
- Use `smol::unblock()` for blocking operations
- Send progress events for long operations
- Write unit tests

**Day 3-4:** SshActor implementation
- Host management, Docker operations, port management
- SSH operations via russh
- Write unit tests

**Day 5-6:** NsActor implementation
- Provider management, DNS record operations
- Provider abstraction for Cloudflare/GCP/DuckDNS/Porkbun
- Write unit tests

**Day 7:** WssActor stub + integration tests
- Implement as no-op with "not implemented" errors
- Write ViewModel integration tests

**Success Criteria:**
- ✅ All actors process commands and send events
- ✅ Unit tests pass (90%+ command coverage)
- ✅ Integration tests pass
- ✅ No blocking operations on UI thread

---

### Week 3: UI Tab Migration

**Goal:** Migrate Platform, SSH, NS tabs to use ViewModel

**Day 1-2:** Platform Tab Migration
- Remove old state (poll-promise, Arc<Mutex<>>)
- Replace with ViewModel calls
- Render from ViewModel state
- Process events in tab

**Day 3-4:** SSH Tab Migration
- Replace direct calc:: calls with ViewModel
- Remove poll-promise usage
- Process SSH events

**Day 5:** NS Tab Migration
- Replace DNS provider calls with ViewModel
- Process NS events

**Day 6-7:** CLI Migration
- Update CLI commands to use ViewModel::new_headless()
- Poll events in CLI command handlers
- Test all CLI commands

**Success Criteria:**
- ✅ All tabs functional via ViewModel
- ✅ No direct calc:: calls from UI
- ✅ All CLI commands work
- ✅ Progress bars show correctly
- ✅ Errors display correctly

---

### Week 4: WASM + Cleanup + Testing

**Day 1-2:** WASM Compatibility
- Implement ViewModel::new_wasm()
- Test WASM build compilation
- Verify runtime/io abstractions

**Day 3:** Code Cleanup
- Remove unused dependencies (poll-promise, crossbeam-queue if unused)
- Remove dead code
- Update documentation

**Day 4-5:** Comprehensive Testing
- Run all unit/integration tests
- Manual testing checklist (OAuth, VMs, SSH, Docker, DNS)
- Error handling verification
- Performance verification

**Day 6-7:** Documentation & Polish
- Update CLAUDE.md
- Write migration guide for future actors
- Add inline documentation
- Final code review and clippy fixes

**Success Criteria:**
- ✅ WASM build compiles
- ✅ All tests pass
- ✅ Manual testing complete
- ✅ No clippy warnings
- ✅ Documentation updated

## File Structure

```
mobile/
├── src/
│   ├── viewmodel/
│   │   ├── mod.rs              # ViewModel struct, init, poll_events
│   │   ├── common.rs           # ViewModelEvent, shared types
│   │   ├── runtime.rs          # Platform-specific runtime
│   │   ├── io.rs               # Platform-specific I/O
│   │   ├── platform/
│   │   │   ├── mod.rs
│   │   │   ├── actor.rs
│   │   │   ├── commands.rs
│   │   │   ├── events.rs
│   │   │   └── tests.rs
│   │   ├── ssh/
│   │   │   ├── mod.rs
│   │   │   ├── actor.rs
│   │   │   ├── commands.rs
│   │   │   ├── events.rs
│   │   │   └── tests.rs
│   │   ├── ns/
│   │   │   ├── mod.rs
│   │   │   ├── actor.rs
│   │   │   ├── commands.rs
│   │   │   ├── events.rs
│   │   │   └── tests.rs
│   │   ├── wss/
│   │   │   ├── mod.rs
│   │   │   ├── actor.rs        # Stub
│   │   │   ├── commands.rs
│   │   │   └── events.rs
│   │   └── tests.rs            # ViewModel integration tests
│   ├── dure.rs                  # DureApp owns ViewModel
│   ├── ui_tabs/
│   │   ├── platform.rs          # Uses ViewModel
│   │   ├── ssh.rs               # Uses ViewModel
│   │   └── ns.rs                # Uses ViewModel
│   ├── cli/
│   │   └── commands/            # CLI uses ViewModel::new_headless()
│   ├── calc/                    # UNCHANGED - reused by actors
│   └── storage/                 # UNCHANGED - accessed by actors
├── Cargo.toml                   # smol, async-executor added
└── tests/
    ├── e2e_platform.rs
    ├── e2e_ssh.rs
    └── e2e_ns.rs
```

## Dependencies Changes

### Add
```toml
smol = "2.0"
async-executor = "1.8"  # For WASM
async-task = "4.7"      # For WASM
gloo-timers = "0.3"     # For WASM
wasm-bindgen-futures = "0.4"  # For WASM
```

### Remove (if unused elsewhere)
```toml
# Check usage first, then remove if not needed
# poll-promise = "0.3.0"
# crossbeam-queue = "0.3"
```

## Success Criteria

**Technical:**
- ✅ All modes (GUI, CLI, WASM) use ViewModel architecture
- ✅ All I/O operations run on background actors
- ✅ UI thread never blocks
- ✅ All existing features work identically
- ✅ Progress tracking for long operations
- ✅ Proper error handling and display

**User Experience:**
- ✅ Smooth progress bars during operations
- ✅ Clear error messages on failures
- ✅ No perceived performance regression
- ✅ Concurrent operations work correctly

**Code Quality:**
- ✅ All actor logic has unit tests (90%+ coverage)
- ✅ Integration tests cover ViewModel flows
- ✅ No clippy warnings
- ✅ Public ViewModel API documented

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Actor complexity overwhelming | High | Start simple, iterate based on unit tests |
| Channel deadlocks | High | Use unbounded channels, careful error handling |
| Performance regression | Medium | Benchmark before/after if needed |
| WASM compatibility issues | Medium | Test early, use runtime/io abstractions |
| Big Bang migration too large | High | Well-defined weekly milestones with success criteria |

## Future Enhancements (Out of Scope)

- WssActor full implementation (when WSS features are built)
- Actor supervision tree (restart failed actors)
- Distributed tracing for actor messages
- Performance/load testing
- Additional UI tabs (products, orders, chat) when implemented

## References

- **Reference implementation:** `/home/wj/work/dure/reference/uad-shizuku/docs/superpowers/specs/2026-06-17-mvvm-refactor-design.md`
- **smol documentation:** https://docs.rs/smol/
- **eframe examples:** egui-based apps with async operations
- **WASM async:** https://rustwasm.github.io/wasm-bindgen/reference/js-promises-and-rust-futures.html

---

**Design approved by:** User  
**Next step:** Create implementation plan using writing-plans skill
