# Actor Implementation Plan

## Overview

Implement 3 new actors following the established pattern used by platform and ssh actors:
1. **MycartActor**: E-commerce store management (products, orders, shipments)
2. **LogsActor**: Operation logging and audit trails
3. **DnsActor**: DNS and domain management

## Architecture Pattern

Each actor follows this structure:

```
mobile/src/viewmodel/{actor_name}/
├── actor.rs         # Main actor implementation
├── commands.rs      # Command enum (UI → Actor)
├── events.rs        # Event enum (Actor → UI)
├── mod.rs          # Public exports
└── drawer_types.rs  # Optional: Drawer UI state (if needed)
```

## Actor 1: MycartActor

### Purpose
Manage e-commerce operations: products, orders, promotions, shipments, customers.

### Module Structure
```
mobile/src/viewmodel/mycart/
├── actor.rs
├── commands.rs
├── events.rs
└── mod.rs
```

### Commands (commands.rs)

```rust
#[derive(Debug, Clone)]
pub enum MycartCommand {
    // Product Operations
    ListProducts {
        store_id: String,
        limit: usize,
        offset: usize,
    },
    CreateProduct {
        store_id: String,
        name: String,
        price: i64,
        description: Option<String>,
        category: Option<String>,
        stock: i32,
    },
    UpdateProduct {
        product_id: String,
        name: Option<String>,
        price: Option<i64>,
        description: Option<String>,
        stock: Option<i32>,
    },
    DeleteProduct {
        product_id: String,
    },
    
    // Order Operations
    ListOrders {
        store_id: String,
        status: Option<OrderStatus>,
        limit: usize,
        offset: usize,
    },
    GetOrder {
        order_id: String,
    },
    UpdateOrderStatus {
        order_id: String,
        status: OrderStatus,
    },
    CancelOrder {
        order_id: String,
        reason: String,
    },
    
    // Shipment Operations
    CreateShipment {
        order_id: String,
        tracking_number: String,
        carrier: String,
    },
    UpdateShipment {
        shipment_id: String,
        status: ShipmentStatus,
        tracking_number: Option<String>,
    },
    
    // Promotion Operations
    ListPromotions {
        store_id: String,
        active_only: bool,
    },
    CreatePromotion {
        store_id: String,
        code: String,
        discount_percent: f32,
        valid_from: i64,
        valid_until: i64,
    },
    
    // Customer Operations
    ListCustomers {
        store_id: String,
        limit: usize,
    },
    GetCustomerOrders {
        customer_id: String,
        limit: usize,
    },
    
    // Statistics
    GetStoreStats {
        store_id: String,
        period: StatsPeriod, // Today, Week, Month, Year
    },
}

#[derive(Debug, Clone)]
pub enum OrderStatus {
    Pending,
    Confirmed,
    Shipped,
    Delivered,
    Cancelled,
}

#[derive(Debug, Clone)]
pub enum ShipmentStatus {
    Preparing,
    Shipped,
    InTransit,
    Delivered,
}

#[derive(Debug, Clone)]
pub enum StatsPeriod {
    Today,
    Week,
    Month,
    Year,
}
```

### Events (events.rs)

```rust
#[derive(Debug, Clone)]
pub struct Product {
    pub id: String,
    pub store_id: String,
    pub name: String,
    pub price: i64,
    pub description: Option<String>,
    pub category: Option<String>,
    pub stock: i32,
    pub created_at: i64,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub id: String,
    pub store_id: String,
    pub customer_id: String,
    pub customer_name: String,
    pub status: OrderStatus,
    pub total_amount: i64,
    pub items: Vec<OrderItem>,
    pub created_at: i64,
}

#[derive(Debug, Clone)]
pub struct OrderItem {
    pub product_id: String,
    pub product_name: String,
    pub quantity: i32,
    pub price: i64,
}

#[derive(Debug, Clone)]
pub struct StoreStats {
    pub total_orders: i64,
    pub total_revenue: i64,
    pub pending_orders: i64,
    pub total_products: i64,
    pub low_stock_products: i64,
}

#[derive(Debug, Clone)]
pub enum MycartEvent {
    // Product Events
    ProductsListed {
        store_id: String,
        products: Vec<Product>,
        total: usize,
    },
    ProductCreated {
        product: Product,
    },
    ProductUpdated {
        product: Product,
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
        status: OrderStatus,
    },
    OrderCancelled {
        order_id: String,
    },
    
    // Shipment Events
    ShipmentCreated {
        shipment_id: String,
        order_id: String,
        tracking_number: String,
    },
    ShipmentUpdated {
        shipment_id: String,
        status: ShipmentStatus,
    },
    
    // Statistics Events
    StoreStatsRetrieved {
        store_id: String,
        stats: StoreStats,
    },
    
    // Error Events
    Error {
        operation: String,
        error: String,
    },
}
```

### Actor Implementation (actor.rs)

```rust
pub struct MycartActor {
    command_rx: Receiver<MycartCommand>,
    event_tx: Sender<ViewModelEvent>,
}

impl MycartActor {
    pub fn new(
        command_rx: Receiver<MycartCommand>,
        event_tx: Sender<ViewModelEvent>,
    ) -> Self {
        Self { command_rx, event_tx }
    }
    
    pub async fn run(mut self) {
        dure_info!("MycartActor started");
        
        loop {
            match self.command_rx.recv().await {
                Ok(cmd) => {
                    if let Err(e) = self.handle_command(cmd).await {
                        dure_error!("MycartActor command error: {}", e);
                    }
                }
                Err(_) => {
                    dure_info!("MycartActor channel closed, shutting down");
                    break;
                }
            }
        }
    }
    
    async fn handle_command(&mut self, cmd: MycartCommand) -> anyhow::Result<()> {
        match cmd {
            MycartCommand::ListProducts { store_id, limit, offset } => {
                self.list_products(store_id, limit, offset).await
            }
            MycartCommand::CreateProduct { store_id, name, price, description, category, stock } => {
                self.create_product(store_id, name, price, description, category, stock).await
            }
            // ... other commands
        }
    }
    
    async fn list_products(
        &mut self,
        store_id: String,
        limit: usize,
        offset: usize,
    ) -> anyhow::Result<()> {
        // Use runtime::unblock for database access
        let (products, total) = runtime::unblock({
            move || -> anyhow::Result<(Vec<Product>, usize)> {
                use crate::calc::db;
                let mut conn = db::establish_connection();
                
                // Query products from database
                let products = crate::storage::models::product::list_products(
                    &mut conn,
                    &store_id,
                    limit,
                    offset,
                )?;
                
                let total = crate::storage::models::product::count_products(
                    &mut conn,
                    &store_id,
                )?;
                
                Ok((products, total))
            }
        })
        .await?;
        
        self.send_event(MycartEvent::ProductsListed {
            store_id,
            products,
            total,
        })
        .await;
        
        Ok(())
    }
    
    async fn send_event(&self, event: MycartEvent) {
        let _ = self.event_tx.send(ViewModelEvent::Mycart(event)).await;
    }
}
```

### ViewModel Integration (viewmodel/mod.rs)

```rust
pub struct ViewModel {
    // ... existing fields
    mycart_tx: Sender<mycart::MycartCommand>,
}

impl ViewModel {
    pub fn new() -> Self {
        // ... existing code
        
        let (mycart_tx, mycart_rx) = async_channel::unbounded();
        let mycart_actor = mycart::MycartActor::new(mycart_rx, event_tx.clone());
        
        smol::spawn(mycart_actor.run()).detach();
        
        Self {
            // ... existing fields
            mycart_tx,
        }
    }
    
    // Public API
    pub fn list_products(
        &self,
        store_id: String,
        limit: usize,
        offset: usize,
    ) -> anyhow::Result<()> {
        self.mycart_tx
            .send_blocking(mycart::MycartCommand::ListProducts {
                store_id,
                limit,
                offset,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }
    
    pub fn create_product(
        &self,
        store_id: String,
        name: String,
        price: i64,
        description: Option<String>,
        category: Option<String>,
        stock: i32,
    ) -> anyhow::Result<()> {
        self.mycart_tx
            .send_blocking(mycart::MycartCommand::CreateProduct {
                store_id,
                name,
                price,
                description,
                category,
                stock,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }
}
```

### UI Integration (ui_tabs/products.rs or similar)

```rust
// In event handling loop
match event {
    ViewModelEvent::Mycart(mycart_event) => {
        match mycart_event {
            MycartEvent::ProductsListed { products, total, .. } => {
                self.products = products;
                self.total_products = total;
                self.loading = false;
            }
            MycartEvent::ProductCreated { product } => {
                self.products.push(product);
                self.show_create_dialog = false;
            }
            // ... other events
        }
    }
}

// In UI render
if ui.button("Load Products").clicked() {
    if let Some(vm) = vm {
        let _ = vm.list_products(
            self.current_store_id.clone(),
            50,  // limit
            0,   // offset
        );
        self.loading = true;
    }
}
```

---

## Actor 2: LogsActor

### Purpose
Centralized operation logging, audit trails, and log querying.

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

## Actor 3: DnsActor

### Purpose
DNS and domain management across multiple providers (Cloudflare, GCP DNS, DuckDNS, Porkbun).

### Module Structure
```
mobile/src/viewmodel/dns/
├── actor.rs
├── commands.rs
├── events.rs
└── mod.rs
```

### Commands (commands.rs)

```rust
#[derive(Debug, Clone)]
pub enum DnsCommand {
    // Domain Operations
    ListDomains {
        provider: DnsProvider,
    },
    
    AddDomain {
        provider: DnsProvider,
        domain: String,
    },
    
    DeleteDomain {
        provider: DnsProvider,
        domain: String,
    },
    
    // Record Operations
    ListRecords {
        provider: DnsProvider,
        domain: String,
    },
    
    CreateRecord {
        provider: DnsProvider,
        domain: String,
        record_type: RecordType,
        name: String,
        content: String,
        ttl: u32,
    },
    
    UpdateRecord {
        provider: DnsProvider,
        domain: String,
        record_id: String,
        content: String,
        ttl: Option<u32>,
    },
    
    DeleteRecord {
        provider: DnsProvider,
        domain: String,
        record_id: String,
    },
    
    // Verification
    VerifyDomain {
        provider: DnsProvider,
        domain: String,
    },
    
    // TXT Record (for challenges)
    CreateTxtRecord {
        provider: DnsProvider,
        domain: String,
        name: String,
        content: String,
    },
}

#[derive(Debug, Clone)]
pub enum DnsProvider {
    Cloudflare { api_token: String },
    GcpDns { project_id: String },
    DuckDns { token: String },
    Porkbun { api_key: String, secret: String },
}

#[derive(Debug, Clone)]
pub enum RecordType {
    A,
    AAAA,
    CNAME,
    MX,
    TXT,
    NS,
}
```

### Events (events.rs)

```rust
#[derive(Debug, Clone)]
pub struct Domain {
    pub name: String,
    pub provider: String,
    pub status: String,
    pub created_at: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct DnsRecord {
    pub id: String,
    pub domain: String,
    pub record_type: String,
    pub name: String,
    pub content: String,
    pub ttl: u32,
    pub proxied: Option<bool>, // Cloudflare specific
}

#[derive(Debug, Clone)]
pub enum DnsEvent {
    // Domain Events
    DomainsListed {
        provider: String,
        domains: Vec<Domain>,
    },
    
    DomainAdded {
        provider: String,
        domain: String,
    },
    
    DomainDeleted {
        provider: String,
        domain: String,
    },
    
    DomainVerified {
        domain: String,
        verified: bool,
    },
    
    // Record Events
    RecordsListed {
        domain: String,
        records: Vec<DnsRecord>,
    },
    
    RecordCreated {
        domain: String,
        record: DnsRecord,
    },
    
    RecordUpdated {
        domain: String,
        record_id: String,
    },
    
    RecordDeleted {
        domain: String,
        record_id: String,
    },
    
    // TXT Record (for challenges)
    TxtRecordCreated {
        domain: String,
        name: String,
        content: String,
    },
    
    Error {
        operation: String,
        error: String,
    },
}
```

### Actor Implementation Notes

- Abstract provider-specific APIs behind common interface
- Use `crate::api::dns::*` modules for provider implementations
- Support async DNS verification
- Handle provider-specific features (Cloudflare proxy, etc.)
- Cache domain lists for performance

---

## Implementation Order

### Phase 1: LogsActor (Simplest)
**Effort**: 1-2 days

**Why first**: 
- Simplest actor (only database operations)
- No external API dependencies
- Useful for other actors' logging

**Steps**:
1. Create module structure
2. Define commands and events
3. Implement actor with database operations
4. Add ViewModel integration
5. Update UI to use logs actor (replace direct DB calls)

### Phase 2: DnsActor (Medium)
**Effort**: 2-3 days

**Why second**:
- Existing `calc::dns` and `api::dns` code to refactor
- Multiple provider support adds complexity
- Independent from mycart

**Steps**:
1. Create module structure
2. Define commands and events
3. Implement actor with provider abstraction
4. Refactor existing DNS UI to use actor
5. Test with each provider (Cloudflare, GCP, DuckDNS, Porkbun)

### Phase 3: MycartActor (Complex)
**Effort**: 3-4 days

**Why last**:
- Most complex domain logic (products, orders, shipments)
- Multiple related entities
- May need new database models
- Largest UI integration

**Steps**:
1. Create module structure
2. Define commands and events
3. Implement actor with all e-commerce operations
4. Create/update database models if needed
5. Update store management UI
6. Add order processing UI
7. Add shipment tracking UI

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

- **LogsActor**: 1-2 days
- **DnsActor**: 2-3 days  
- **MycartActor**: 3-4 days
- **Total**: 6-9 days (1.5-2 weeks)

---

## Additional Notes

### Why Not Merge LogsActor with PlatformActor?

- **Separation of concerns**: Logs are cross-cutting (used by all actors)
- **Reusability**: Other actors can log operations
- **Simplicity**: Smaller, focused actors are easier to maintain

### Provider Abstraction (DnsActor)

Create trait for DNS providers:

```rust
#[async_trait]
pub trait DnsProvider {
    async fn list_domains(&self) -> anyhow::Result<Vec<Domain>>;
    async fn create_record(&self, domain: &str, record: RecordParams) -> anyhow::Result<DnsRecord>;
    // ... other operations
}

pub struct CloudflareProvider { api_token: String }
pub struct GcpDnsProvider { project_id: String, access_token: String }
pub struct DuckDnsProvider { token: String }
pub struct PorkbunProvider { api_key: String, secret: String }

// Implement trait for each provider
```

### Database Models

Ensure these exist or create them:

**For LogsActor** (already exists):
- `storage/models/opslog.rs`

**For MycartActor** (may need creation):
- `storage/models/product.rs`
- `storage/models/order.rs`
- `storage/models/shipment.rs`
- `storage/models/promotion.rs`
- `storage/models/customer.rs`

**For DnsActor** (may need creation):
- `storage/models/dns.rs` (domain and record configs)
