# Actor Implementation Plan (Revised)

## Overview

Implement 3 actors following platform/ssh pattern. All tabs (platform, ssh, mycart, dns) use LogsActor for unified operation logging.

**Key Changes:**
- LogsActor used by all 4 tabs (platform, ssh, mycart, dns)
- DnsActor only uses Cloudflare API (api/cloudflare/*), extract from ui_tabs/ssh
- MycartActor uses remote API (https://mycart.dure.app/api/_/*) + SSH log fetching
- All tabs get drawer UI like platform (logs + operations tabs)

## Architecture Pattern

Each actor follows this structure:

```
mobile/src/viewmodel/{actor_name}/
├── actor.rs         # Main actor implementation
├── commands.rs      # Command enum (UI → Actor)
├── events.rs        # Event enum (Actor → UI)
├── drawer_types.rs  # Drawer UI state (DrawerTab, DrawerState)
└── mod.rs          # Public exports
```

**All actors** (platform, ssh, mycart, dns) include `drawer_types.rs` for unified drawer UI pattern.

## Actor 1: MycartActor

### Purpose
Manage remote mycart backend (https://mycart.dure.app/api/_/*) + fetch logs via SSH.

**Dual Mode**:
1. **Remote API**: CRUD operations via REST API (swagger.json)
2. **SSH Logs**: Fetch application logs from remote VM (like ssh actor)

### Module Structure
```
mobile/src/viewmodel/mycart/
├── actor.rs
├── commands.rs
├── events.rs
├── drawer_types.rs  # DrawerTab, DrawerState (operations + logs)
└── mod.rs
```

### Commands (commands.rs)

```rust
#[derive(Debug, Clone)]
pub enum MycartCommand {
    // Store Health
    CheckStoreHealth {
        store_id: String,
    },
    
    // Product Operations (GET /api/_/products)
    ListProducts {
        store_id: String,
        limit: Option<usize>,
        offset: Option<usize>,
    },
    GetProduct {
        product_id: String,
    },
    CreateProduct {
        store_id: String,
        name: String,
        price: i64,
        description: Option<String>,
    },
    UpdateProduct {
        product_id: String,
        name: Option<String>,
        price: Option<i64>,
    },
    DeleteProduct {
        product_id: String,
    },
    
    // Order Operations (GET /api/_/orders)
    ListOrders {
        store_id: String,
        status: Option<String>,
        limit: Option<usize>,
    },
    GetOrder {
        order_id: String,
    },
    UpdateOrderStatus {
        order_id: String,
        status: String, // "pending", "confirmed", "shipped", "delivered"
    },
    
    // SSH Log Operations (like ssh actor)
    FetchLogs {
        store_id: String,
        vm_name: String,
        log_type: LogType, // Application, Nginx, System
        lines: usize,
    },
    StreamLogs {
        store_id: String,
        vm_name: String,
        log_type: LogType,
    },
    StopLogStream {
        store_id: String,
    },
}

#[derive(Debug, Clone)]
pub enum LogType {
    Application,  // /var/log/mycart/app.log
    Nginx,        // /var/log/nginx/access.log
    System,       // /var/log/syslog
}
```

### Events (events.rs)

```rust
#[derive(Debug, Clone)]
pub struct StoreHealth {
    pub store_id: String,
    pub status: String, // "healthy", "degraded", "down"
    pub uptime_seconds: u64,
    pub version: String,
}

#[derive(Debug, Clone)]
pub struct Product {
    pub id: String,
    pub store_id: String,
    pub name: String,
    pub price: i64,
    pub description: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub id: String,
    pub store_id: String,
    pub customer_name: String,
    pub status: String,
    pub total_amount: i64,
    pub created_at: i64,
}

#[derive(Debug, Clone)]
pub enum MycartEvent {
    // Health
    StoreHealthChecked {
        store_id: String,
        health: StoreHealth,
    },
    
    // Product Events
    ProductsListed {
        store_id: String,
        products: Vec<Product>,
        total: usize,
    },
    ProductRetrieved {
        product: Product,
    },
    ProductCreated {
        product: Product,
    },
    ProductUpdated {
        product_id: String,
    },
    ProductDeleted {
        product_id: String,
    },
    
    // Order Events
    OrdersListed {
        store_id: String,
        orders: Vec<Order>,
        total: usize,
    },
    OrderRetrieved {
        order: Order,
    },
    OrderStatusUpdated {
        order_id: String,
        status: String,
    },
    
    // Log Events (like ssh actor)
    LogsFetched {
        store_id: String,
        log_type: LogType,
        lines: Vec<String>,
    },
    LogLine {
        store_id: String,
        line: String,
    },
    LogStreamStopped {
        store_id: String,
    },
    
    // Errors
    Error {
        operation: String,
        error: String,
    },
}
```

### Actor Implementation Notes

**Remote API Client**:
- Use `ureq` for HTTP requests (already in dependencies)
- Base URL: `https://dure.co/api/_/` (user baseurl)
- Auth: Bearer token from platform config
- Endpoints defined in swagger.json (reference only)
- Use `runtime::unblock` for HTTP calls

**SSH Log Fetching** (like ssh actor):
- Reuse SSH connection logic from `crate::calc::ssh`
- Commands: `tail -n 100 /var/log/mycart/app.log`
- Stream: `tail -f /var/log/mycart/app.log`
- Parse structured logs (JSON format expected)

**Example Implementation**:
```rust
async fn list_products(&mut self, store_id: String, limit: Option<usize>) -> anyhow::Result<()> {
    let products = runtime::unblock(move || -> anyhow::Result<Vec<Product>> {
        let url = format!("https://mycart.dure.app/api/_/products?store_id={}", store_id);
        let response: ApiResponse<Vec<Product>> = ureq::get(&url)
            .call()?
            .into_json()?;
        Ok(response.data)
    }).await?;
    
    self.send_event(MycartEvent::ProductsListed {
        store_id,
        total: products.len(),
        products,
    }).await;
    
    Ok(())
}

async fn fetch_logs(&mut self, store_id: String, vm_name: String, log_type: LogType) -> anyhow::Result<()> {
    let lines = runtime::unblock(move || -> anyhow::Result<Vec<String>> {
        use crate::calc::ssh;
        let log_path = match log_type {
            LogType::Application => "/var/log/mycart/app.log",
            LogType::Nginx => "/var/log/nginx/access.log",
            LogType::System => "/var/log/syslog",
        };
        
        let cmd = format!("tail -n 100 {}", log_path);
        let output = ssh::execute_remote(&vm_name, &cmd)?;
        Ok(output.lines().map(|s| s.to_string()).collect())
    }).await?;
    
    self.send_event(MycartEvent::LogsFetched {
        store_id,
        log_type,
        lines,
    }).await;
    
    Ok(())
}
```

---

## Actor 2: LogsActor (Shared by All Tabs)

### Purpose
Centralized operation logging used by **all 4 tabs** (platform, ssh, mycart, dns). Provides audit trails and log querying.

**Usage**: Every actor logs operations via LogsActor. Each tab displays its own logs via shared LogsActor instance.

### Module Structure
```
mobile/src/viewmodel/logs/
├── actor.rs
├── commands.rs
├── events.rs
└── mod.rs
```

### Commands (commands.rs)

```rust
#[derive(Debug, Clone)]
pub enum LogsCommand {
    // Create log entry
    CreateLog {
        project_id: String,
        operation_type: String,
        operation_subtype: String,
        status: LogStatus,
        metadata: Option<String>, // JSON metadata
    },
    
    // Update log entry
    UpdateLog {
        log_id: i64,
        status: LogStatus,
        error_message: Option<String>,
        metadata: Option<String>,
    },
    
    // Query logs
    ListLogs {
        project_id: Option<String>,
        operation_type: Option<String>,
        status: Option<LogStatus>,
        limit: usize,
        offset: usize,
    },
    
    GetLog {
        log_id: i64,
    },
    
    // Statistics
    GetLogStats {
        project_id: String,
        period: StatsPeriod,
    },
    
    // Cleanup
    PurgeLogs {
        older_than_days: i64,
    },
}

#[derive(Debug, Clone)]
pub enum LogStatus {
    Running,
    Success,
    Failed,
    Cancelled,
}
```

### Events (events.rs)

```rust
#[derive(Debug, Clone)]
pub struct OperationLog {
    pub id: i64,
    pub project_id: String,
    pub operation_type: String,
    pub operation_subtype: String,
    pub status: String,
    pub error_message: Option<String>,
    pub metadata: Option<String>,
    pub started_at: i64,
    pub completed_at: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct LogStats {
    pub total_operations: i64,
    pub successful_operations: i64,
    pub failed_operations: i64,
    pub running_operations: i64,
}

#[derive(Debug, Clone)]
pub enum LogsEvent {
    LogCreated {
        log_id: i64,
        project_id: String,
        operation_type: String,
    },
    
    LogUpdated {
        log_id: i64,
        status: LogStatus,
    },
    
    LogsListed {
        logs: Vec<OperationLog>,
        total: usize,
    },
    
    LogRetrieved {
        log: OperationLog,
    },
    
    LogStatsRetrieved {
        project_id: String,
        stats: LogStats,
    },
    
    LogsPurged {
        count: i64,
    },
    
    Error {
        operation: String,
        error: String,
    },
}
```

### Actor Implementation Notes

- Use `runtime::unblock` for all database operations
- Support pagination for large log sets
- Implement efficient querying with indexes
- Support JSON metadata for flexible logging
- Automatic timestamp management

---

## Actor 3: DnsActor (Cloudflare Only)

### Purpose
Cloudflare DNS management. Extract existing commands from `ui_dlg/dns_wizard.rs`.

**Source**: Operations from DNS wizard + `api/cloudflare/dns.rs` client.

### Module Structure
```
mobile/src/viewmodel/dns/
├── actor.rs
├── commands.rs
├── events.rs
├── drawer_types.rs  # DrawerTab, DrawerState (operations + logs)
└── mod.rs
```

### Commands (Extract from ui_dlg/dns_wizard.rs)

```rust
#[derive(Debug, Clone)]
pub enum DnsCommand {
    // Zone (Domain) Operations
    ListZones,
    
    FindZone {
        domain: String,
    },
    
    CreateZone {
        domain: String,
    },
    
    // DNS Record Operations
    ListRecords {
        zone_id: String,
    },
    
    FindRecord {
        zone_id: String,
        record_type: RecordType,
        name: String,
    },
    
    CreateRecord {
        zone_id: String,
        record_type: RecordType,
        name: String,
        content: String,
        ttl: Option<u32>,
        proxied: Option<bool>,
    },
    
    UpdateRecord {
        zone_id: String,
        record_id: String,
        content: String,
        ttl: Option<u32>,
        proxied: Option<bool>,
    },
    
    DeleteRecord {
        zone_id: String,
        record_id: String,
    },
    
    UpsertRecord {
        zone_id: String,
        record_type: RecordType,
        name: String,
        content: String,
        ttl: Option<u32>,
        proxied: Option<bool>,
    },
    
    // SRV Record Operations
    CreateSrvRecord {
        zone_id: String,
        name: String,
        priority: u16,
        weight: u16,
        port: u16,
        target: String,
    },
    
    UpsertSrvRecord {
        zone_id: String,
        name: String,
        priority: u16,
        weight: u16,
        port: u16,
        target: String,
    },
    
    // Preset Operations (from wizard)
    CreateDefaultXmppRecords {
        zone_id: String,
        domain: String,
        vm_ip: String,
    },
    
    VerifyDefaultXmppRecords {
        zone_id: String,
        domain: String,
        vm_ip: String,
    },
}

#[derive(Debug, Clone)]
pub enum RecordType {
    A,
    AAAA,
    CNAME,
    MX,
    TXT,
    NS,
    SRV,
}
```

### Events (events.rs)

```rust
#[derive(Debug, Clone)]
pub struct Zone {
    pub id: String,
    pub name: String,
    pub name_servers: Vec<String>,
    pub account_id: String,
    pub account_name: String,
}

#[derive(Debug, Clone)]
pub struct DnsRecord {
    pub id: String,
    pub name: String,
    pub record_type: String,
    pub content: String,
    pub ttl: u32,
    pub proxied: bool,
}

#[derive(Debug, Clone)]
pub struct SrvRecord {
    pub id: String,
    pub name: String,
    pub priority: u16,
    pub weight: u16,
    pub port: u16,
    pub target: String,
}

#[derive(Debug, Clone)]
pub enum DnsEvent {
    // Zone Events
    ZonesListed {
        zones: Vec<Zone>,
    },
    
    ZoneFound {
        zone: Option<Zone>,
    },
    
    ZoneCreated {
        zone: Zone,
    },
    
    // Record Events
    RecordsListed {
        zone_id: String,
        records: Vec<DnsRecord>,
    },
    
    RecordFound {
        zone_id: String,
        record: Option<DnsRecord>,
    },
    
    RecordCreated {
        zone_id: String,
        record: DnsRecord,
    },
    
    RecordUpdated {
        zone_id: String,
        record_id: String,
    },
    
    RecordDeleted {
        zone_id: String,
        record_id: String,
    },
    
    RecordUpserted {
        zone_id: String,
        record: DnsRecord,
    },
    
    // SRV Record Events
    SrvRecordCreated {
        zone_id: String,
        record: SrvRecord,
    },
    
    SrvRecordUpserted {
        zone_id: String,
        record: SrvRecord,
    },
    
    // Preset Operations
    DefaultXmppRecordsCreated {
        zone_id: String,
        domain: String,
    },
    
    DefaultXmppRecordsVerified {
        zone_id: String,
        domain: String,
        verified: bool,
    },
    
    Error {
        operation: String,
        error: String,
    },
}
```

### Actor Implementation Notes

**Source Extraction**:
- Extract from `ui_dlg/dns_wizard.rs` (OAuth, zones, records)
- Use `api/cloudflare/dns.rs` client (CloudflareClient)
- No multi-provider abstraction needed (Cloudflare-only)

**Operations**:
- Use `runtime::unblock` for all API calls
- Support Cloudflare-specific features (proxied flag, SRV records)
- Preset operations for XMPP default records
- OAuth token management (access + refresh tokens)

**Example Implementation**:
```rust
async fn list_zones(&mut self) -> anyhow::Result<()> {
    let token = self.cloudflare_token.clone();
    let zones = runtime::unblock(move || -> anyhow::Result<Vec<Zone>> {
        use crate::api::cloudflare::CloudflareClient;
        let client = CloudflareClient::new(token);
        let cf_zones = client.list_zones()?;
        
        // Convert to event Zone type
        Ok(cf_zones.into_iter().map(|z| Zone {
            id: z.id,
            name: z.name,
            name_servers: z.name_servers,
            account_id: z.account.id,
            account_name: z.account.name,
        }).collect())
    }).await?;
    
    self.send_event(DnsEvent::ZonesListed { zones }).await;
    Ok(())
}

async fn create_default_xmpp_records(&mut self, zone_id: String, domain: String, vm_ip: String) -> anyhow::Result<()> {
    let token = self.cloudflare_token.clone();
    runtime::unblock(move || -> anyhow::Result<()> {
        use crate::api::cloudflare::CloudflareClient;
        let client = CloudflareClient::new(token);
        client.create_default_xmpp_records(&zone_id, &domain, &vm_ip)
    }).await?;
    
    self.send_event(DnsEvent::DefaultXmppRecordsCreated { zone_id, domain }).await;
    Ok(())
}
```

**OAuth Integration**:
- Store access_token and refresh_token
- Handle token expiration (401/403 errors)
- Trigger re-auth when token expires
- Use `api/cloudflare/oauth.rs` for OAuth flow

---

## Drawer UI Pattern (All Tabs)

### Purpose
Unified drawer UI like platform tab. All tabs (platform, ssh, mycart, dns) show operations + logs.

**Platform tab already has this**: See `ui_tabs/platform.rs` for reference.

### Structure

Each tab UI has:
1. **Main content**: Tab-specific operations (VM list, SSH hosts, products, domains)
2. **Drawer panel**: Operations tab + Logs tab
3. **Drawer state**: Track open/closed, active tab

### Drawer Types (drawer_types.rs)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawerTab {
    Operations,
    Logs,
}

#[derive(Debug, Clone)]
pub struct DrawerState {
    pub open: bool,
    pub active_tab: DrawerTab,
    pub width: f32,
}

impl Default for DrawerState {
    fn default() -> Self {
        Self {
            open: false,
            active_tab: DrawerTab::Operations,
            width: 400.0,
        }
    }
}
```

### UI Integration

**Example** (ssh tab, similar for mycart/dns):

```rust
// ui_tabs/ssh.rs
pub struct SshTab {
    // Existing fields
    drawer_state: DrawerState,
    logs: Vec<String>,
}

impl SshTab {
    pub fn render(&mut self, ui: &mut egui::Ui, vm: Option<&ViewModel>) {
        egui::SidePanel::right("ssh_drawer")
            .show_separator_line(true)
            .resizable(true)
            .default_width(self.drawer_state.width)
            .show_animated(ui.ctx(), self.drawer_state.open, |ui| {
                self.render_drawer(ui, vm);
            });
        
        egui::CentralPanel::default().show_inside(ui, |ui| {
            self.render_main_content(ui, vm);
        });
    }
    
    fn render_drawer(&mut self, ui: &mut egui::Ui, vm: Option<&ViewModel>) {
        ui.horizontal(|ui| {
            if ui.selectable_label(self.drawer_state.active_tab == DrawerTab::Operations, "Operations").clicked() {
                self.drawer_state.active_tab = DrawerTab::Operations;
            }
            if ui.selectable_label(self.drawer_state.active_tab == DrawerTab::Logs, "Logs").clicked() {
                self.drawer_state.active_tab = DrawerTab::Logs;
            }
        });
        
        ui.separator();
        
        match self.drawer_state.active_tab {
            DrawerTab::Operations => self.render_operations_tab(ui, vm),
            DrawerTab::Logs => self.render_logs_tab(ui, vm),
        }
    }
    
    fn render_operations_tab(&mut self, ui: &mut egui::Ui, vm: Option<&ViewModel>) {
        // Tab-specific operations (SSH: host mgmt, connections)
    }
    
    fn render_logs_tab(&mut self, ui: &mut egui::Ui, vm: Option<&ViewModel>) {
        // Show logs from LogsActor
        egui::ScrollArea::vertical().show(ui, |ui| {
            for log_line in &self.logs {
                ui.label(log_line);
            }
        });
    }
}
```

### Event Handling

Each tab listens to both actor events and LogsActor events:

```rust
// In event loop
match event {
    ViewModelEvent::Ssh(ssh_event) => {
        // Handle SSH-specific events
    }
    ViewModelEvent::Logs(logs_event) => {
        // Handle logs (shared by all tabs)
        match logs_event {
            LogsEvent::LogsListed { logs, .. } => {
                self.logs = logs.iter().map(|l| format!("{}: {}", l.operation_type, l.status)).collect();
            }
        }
    }
}
```

### Tab-Specific Implementations

| Tab | Operations Tab Shows | Logs Tab Shows |
|-----|---------------------|----------------|
| **Platform** | VM operations, firewall updates, billing | Platform operation logs (via LogsActor) |
| **SSH** | Host list, connections, file transfers | SSH operation logs (via LogsActor) |
| **Mycart** | Products, orders, store health | Mycart operation logs + remote logs (via LogsActor + SSH) |
| **DNS** | Domains, records, TXT challenges | DNS operation logs (via LogsActor) |

---

## Implementation Order

### Phase 1: LogsActor (Shared Foundation)
**Effort**: 2-3 days

**Why first**: 
- Used by all 4 tabs (platform, ssh, mycart, dns)
- Only database operations (simplest)
- No external API dependencies
- Foundation for other actors

**Steps**:
1. Create module structure (`viewmodel/logs/`)
2. Define commands and events
3. Implement actor with database operations
4. Add ViewModel integration
5. Update platform tab to use LogsActor (replace direct DB calls)
6. Add drawer UI to platform tab (operations + logs tabs)

**Success**: Platform tab fully uses LogsActor, drawer UI working.

### Phase 2: DnsActor (Cloudflare Only)
**Effort**: 2-3 days

**Why second**:
- Extract existing DNS logic from `ui_dlg/dns_wizard.rs`
- Cloudflare-only (no multi-provider complexity)
- Independent from mycart
- Uses LogsActor for logging

**Steps**:
1. Create module structure (`viewmodel/dns/`)
2. Define commands and events
3. Extract DNS operations from `ui_dlg/dns_wizard.rs`:
   - OAuth flow (via `api/cloudflare/oauth.rs`)
   - Zone management (list, create, find)
   - Record management (CRUD, upsert)
   - Preset operations (XMPP defaults)
4. Implement actor using `api/cloudflare/dns.rs` client
5. Create new DNS tab with drawer UI
6. Integrate LogsActor for DNS operation logging
7. Migrate wizard UI to use DnsActor (optional refactor)

**Success**: DNS tab working with Cloudflare API, logs via LogsActor.

### Phase 3: MycartActor (Remote API + SSH Logs)
**Effort**: 3-4 days

**Why third**:
- Dual mode (remote API + SSH logs)
- Remote API client needed (ureq)
- SSH log fetching like ssh actor
- Uses LogsActor for operation logging

**Steps**:
1. Create module structure (`viewmodel/mycart/`)
2. Define commands and events
3. Implement remote API client (https://mycart.dure.app/api/_/*)
4. Implement SSH log fetching
5. Create mycart tab with drawer UI
6. Integrate LogsActor for operation logging
7. Add operations tab (products, orders, health)
8. Add logs tab (remote logs via SSH)

**Success**: Mycart tab working with remote API, logs via SSH + LogsActor.

### Phase 4: SSH Tab Refactoring
**Effort**: 1-2 days

**Why last**:
- SSH actor already exists
- Add drawer UI like other tabs
- Use LogsActor for SSH operation logging
- Maintain existing SSH functionality

**Steps**:
1. Add `drawer_types.rs` to `viewmodel/ssh/`
2. Refactor ssh tab UI to use drawer pattern
3. Add operations tab (host mgmt, connections)
4. Add logs tab (SSH operation logs via LogsActor)
5. Integrate LogsActor for SSH logging

**Success**: SSH tab matches platform/dns/mycart pattern with drawer UI.

---

## Common Patterns

### Database Operations (All Actors)

```rust
async fn operation(&mut self, params: Type) -> anyhow::Result<()> {
    let result = runtime::unblock({
        move || -> anyhow::Result<ReturnType> {
            use crate::calc::db;
            let mut conn = db::establish_connection();
            
            // Database operations here
            crate::storage::models::entity::operation(&mut conn, params)
        }
    })
    .await?;
    
    self.send_event(Event::OperationCompleted { result }).await;
    Ok(())
}
```

### API Operations (DnsActor)

```rust
async fn operation(&mut self, params: Type) -> anyhow::Result<()> {
    let result = runtime::unblock({
        move || -> anyhow::Result<ReturnType> {
            let client = crate::api::dns::provider::Client::new(credentials);
            client.operation(params)
        }
    })
    .await?;
    
    self.send_event(Event::OperationCompleted { result }).await;
    Ok(())
}
```

### Error Handling

```rust
async fn handle_command(&mut self, cmd: Command) -> anyhow::Result<()> {
    let operation = match &cmd {
        Command::Operation1 { .. } => "operation1",
        Command::Operation2 { .. } => "operation2",
    };
    
    let result = match cmd {
        Command::Operation1 { params } => {
            self.operation1(params).await
        }
        // ...
    };
    
    if let Err(e) = result {
        dure_error!("{} failed: {}", operation, e);
        self.send_event(Event::Error {
            operation: operation.to_string(),
            error: e.to_string(),
        })
        .await;
    }
    
    Ok(())
}
```

### Progress Reporting

```rust
async fn long_operation(&mut self) -> anyhow::Result<()> {
    self.send_progress("operation", 0.0, "Starting...").await;
    
    // Step 1
    do_step1()?;
    self.send_progress("operation", 0.33, "Step 1 complete").await;
    
    // Step 2
    do_step2()?;
    self.send_progress("operation", 0.66, "Step 2 complete").await;
    
    // Step 3
    do_step3()?;
    self.send_progress("operation", 1.0, "Complete").await;
    
    Ok(())
}

async fn send_progress(&self, operation: &str, progress: f32, status: &str) {
    let _ = self.event_tx.send(ViewModelEvent::Actor(Event::Progress {
        operation: operation.to_string(),
        progress,
        status: status.to_string(),
    })).await;
}
```

---

## Testing Strategy

### Unit Tests (actor.rs)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_command_handling() {
        smol::block_on(async {
            let (cmd_tx, cmd_rx) = async_channel::unbounded();
            let (evt_tx, evt_rx) = async_channel::unbounded();
            
            let actor = Actor::new(cmd_rx, evt_tx);
            smol::spawn(actor.run()).detach();
            
            // Send command
            cmd_tx.send(Command::Operation { ... }).await.unwrap();
            
            // Receive event
            let event = evt_rx.recv().await.unwrap();
            assert!(matches!(event, Event::OperationCompleted { .. }));
        });
    }
}
```

### Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    // Test full flow: UI → ViewModel → Actor → Database → Event → UI
    #[test]
    fn test_full_flow() {
        let vm = ViewModel::new();
        
        // Send command via public API
        vm.operation(params).unwrap();
        
        // Poll events
        let events = vm.poll_events();
        assert!(events.iter().any(|e| matches!(e, Event::Success { .. })));
    }
}
```

---

## Migration Strategy

### For Existing Code

1. **Identify violations**: Grep for direct `calc::db`, `api::dns`, etc. in UI
2. **Create actor**: Implement actor module
3. **Add commands**: One command per existing operation
4. **Replace calls**: Update UI to use `vm.operation()` instead of direct calls
5. **Handle events**: Add event handlers in UI
6. **Test**: Verify functionality preserved
7. **Remove old code**: Delete unused direct access functions

### Example Migration

**Before** (UI directly accessing database):
```rust
// In ui_tabs/products.rs
let products = {
    let mut conn = crate::calc::db::establish_connection();
    crate::storage::models::product::list_products(&mut conn, store_id, 50, 0)
}.unwrap();
self.products = products;
```

**After** (Using actor):
```rust
// In ui_tabs/products.rs
if let Some(vm) = vm {
    let _ = vm.list_products(store_id.clone(), 50, 0);
    self.loading = true;
}

// In event handling
match event {
    ViewModelEvent::Mycart(MycartEvent::ProductsListed { products, .. }) => {
        self.products = products;
        self.loading = false;
    }
}
```

---

## Checklist per Actor

- [ ] Create module directory and files
- [ ] Define all commands in commands.rs
- [ ] Define all events in events.rs
- [ ] Implement Actor struct with run loop
- [ ] Implement all command handlers
- [ ] Add to ViewModel (channels, spawn, public API)
- [ ] Add event enum variant to ViewModelEvent
- [ ] Update UI to use actor (replace direct calls)
- [ ] Add event handlers in UI
- [ ] Write unit tests
- [ ] Write integration tests
- [ ] Update documentation
- [ ] Verify no direct calc/api calls remain in UI

---

## Success Criteria

1. **Zero direct access**: No `calc::db`, `api::dns`, or similar calls in `ui_tabs/` or `ui_dlg/`
2. **Event-driven**: All UI updates happen via events
3. **Non-blocking**: No `runtime::unblock` in UI render loops
4. **Testable**: Each actor has unit and integration tests
5. **Maintainable**: Clear separation of concerns
6. **Performance**: No regression in UI responsiveness

---

## Estimated Timeline

- **Phase 1 (LogsActor)**: 2-3 days
- **Phase 2 (DnsActor)**: 2-3 days  
- **Phase 3 (MycartActor)**: 3-4 days
- **Phase 4 (SSH Tab Refactor)**: 1-2 days
- **Total**: 8-12 days (2-2.5 weeks)

---

## Additional Notes

### Why LogsActor is Shared

- **Cross-cutting concern**: All tabs (platform, ssh, mycart, dns) need operation logging
- **Reusability**: Single LogsActor instance used by all actors
- **Simplicity**: One source of truth for logs
- **UI consistency**: All tabs show logs in same format

### Why Drawer UI Pattern

- **Consistency**: All tabs look similar (platform already has this)
- **Context switching**: Operations + logs in one place
- **Efficiency**: No tab switching to view logs
- **Platform tab as reference**: See `ui_tabs/platform.rs` for implementation

### DnsActor Simplification

**Cloudflare-only approach**:
- No multi-provider abstraction needed
- Extract existing logic from `ui_dlg/dns_wizard.rs`
- Use `api/cloudflare/dns.rs` client directly
- Simpler implementation, faster development
- Operations already proven in wizard (OAuth, zones, records, XMPP presets)

### MycartActor Dual Mode

**Remote API**:
- Backend at https://mycart.dure.app
- Use swagger.json for API reference
- Only `/api/_/*` endpoints (admin API)

**SSH Logs**:
- Fetch logs from remote VM like ssh actor
- Reuse existing SSH connection logic
- Support streaming logs

### Database Models

**For LogsActor** (already exists):
- `storage/models/opslog.rs`

**For DnsActor**:
- Uses Cloudflare API directly (no local DB models needed)
- May cache domain list in memory

**For MycartActor**:
- Uses remote API (no local DB models needed)
- Logs stored via LogsActor

---

## Quick Reference

| Actor | Purpose | Key Technology | Tab UI |
|-------|---------|---------------|--------|
| **LogsActor** | Operation logging for all tabs | SQLite via runtime::unblock | Used by all 4 tabs |
| **DnsActor** | Cloudflare DNS management | api/cloudflare/* | DNS tab with drawer |
| **MycartActor** | Remote API + SSH logs | ureq (HTTP) + russh (SSH) | Mycart tab with drawer |
| **SshActor** | Existing, add drawer UI | russh | SSH tab with drawer |

**Drawer UI Pattern** (all tabs):
- Operations tab: Tab-specific operations
- Logs tab: LogsActor logs for this tab
- Reference: `ui_tabs/platform.rs`

**Implementation Priority**:
1. LogsActor → Foundation for all tabs
2. DnsActor → Extract from ssh tab, Cloudflare-only
3. MycartActor → Remote API + SSH logs
4. SSH Tab → Add drawer UI pattern
