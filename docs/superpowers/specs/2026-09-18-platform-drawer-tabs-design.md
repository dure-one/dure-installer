# Platform Drawer Tabs with Operation Logging - Design Specification

**Date:** 2026-09-18  
**Author:** Claude Sonnet 4.5  
**Status:** Design Approved  
**Classification:** Architectural

## Overview

This spec defines a comprehensive UI improvement for the Platform tab in the Dure installer application. The changes include:

1. **Tabbed Drawer UI**: Add Status/Logs/Operations tabs to the platform drawer
2. **Operation Logging**: SQLite-backed audit trail for all external API operations (GCP, Cloudflare, Supabase)
3. **Filtered Logs**: Project-specific stdout log filtering in the Logs tab
4. **Compact Table Layout**: 30px row height with dynamic window-filling behavior
5. **Full MVVM Architecture**: Clean separation of concerns with DrawerViewModel

## Goals

1. **Auditability**: Track all infrastructure operations (VM creation, firewall updates, DNS changes) in persistent storage
2. **Debuggability**: Filter application logs by project for easier troubleshooting
3. **Usability**: Compact, responsive table layout that maximizes screen real estate
4. **Maintainability**: MVVM pattern ensures testable, extensible architecture
5. **Consistency**: Follow existing ViewModel patterns in the codebase

## Design Decisions

### Question 1: rust-skills Integration Strategy
**Decision:** Full MVVM Pattern (Approach 2)  
**Rationale:** 
- Follows existing ViewModel pattern in codebase (consistency)
- Clean separation of concerns (UI → ViewModel → Repository)
- Highly testable (mock repository in tests)
- Easy to extend (add new tabs, change logging logic)
- Long-term maintainability over quick implementation

### Question 2: Tab Structure
**Decision:** Keep current drawer content in "Status" tab, add "Logs" and "Operations" sibling tabs  
**Rationale:** Preserves existing functionality while adding new features without disruption

### Question 3: Operation Logging Scope
**Decision:** Log anything that passes through external systems (GCP, Cloudflare, Supabase, etc.)  
**Rationale:** Creates audit trail for infrastructure changes, debugging cloud provider issues

### Question 4: Log Filtering
**Decision:** Filter stdout logs by selected project_id  
**Rationale:** Focused debugging - only show relevant logs for the platform being viewed

### Question 5: Row Height and Table Layout
**Decision:** 
- Exactly 30px row height (hard requirement)
- Fill both vertical and horizontal space with small margins (8px left/right, 4px top)
- Table header scrolls with content (not sticky)
- Action buttons stay above table and scroll away

**Rationale:** Maximizes data density while maintaining readability

### Question 6: Drawer Tab Style
**Decision:** Use `tabs_secondary()` for compact design  
**Rationale:** Compact full-width underline indicator fits drawer context better than primary tabs

### Question 7: Database Schema
**Decision:** Approved schema with 9 columns (id, project_id, operation_type, external_system, status, started_at, completed_at, error_message, details)  
**Rationale:** Captures all necessary information for audit trail without over-engineering

## Architecture

### Component Structure

```
┌─────────────────────────────────────────────────────────────┐
│ UI Layer (platform.rs)                                      │
│  ├─ PlatformTab::ui() - main table                         │
│  ├─ render_drawer_content() - tabbed drawer                │
│  └─ Tab content renderers (status/logs/operations)         │
└─────────────────────────────────────────────────────────────┘
                          │
                          ↓ (user interactions)
┌─────────────────────────────────────────────────────────────┐
│ ViewModel Layer (viewmodel/drawer.rs)                       │
│  ├─ DrawerViewModel - manages drawer state                 │
│  ├─ DrawerEvent - events from repository                   │
│  └─ DrawerCommand - commands from UI                       │
└─────────────────────────────────────────────────────────────┘
                          │
                          ↓ (repository calls)
┌─────────────────────────────────────────────────────────────┐
│ Repository Layer (repositories/operation_log.rs)            │
│  └─ OperationLogRepository - CRUD operations               │
└─────────────────────────────────────────────────────────────┘
                          │
                          ↓ (Diesel ORM)
┌─────────────────────────────────────────────────────────────┐
│ Database Layer (SQLite)                                     │
│  └─ operation_logs table                                   │
└─────────────────────────────────────────────────────────────┘
```

### Data Flow - Operation Logging

```
GCP API call happens
    → ViewModel emits PlatformEvent (e.g., VMCreated)
    → Main ViewModel dispatches to DrawerViewModel  
    → DrawerViewModel::handle_platform_event()
    → OperationLogRepository::create_log()
    → SQLite insert
    → DrawerViewModel emits DrawerEvent::OperationLogged
    → UI updates Operations tab
```

### File Structure

```
mobile/src/
├── viewmodel/
│   ├── mod.rs                  # Add drawer module export
│   ├── drawer.rs               # NEW: DrawerViewModel
│   └── platform.rs             # Existing, no changes
├── repositories/
│   ├── mod.rs                  # NEW: Module root
│   └── operation_log.rs        # NEW: OperationLogRepository
├── storage/
│   ├── diesel_schema.rs        # Add operation_logs table
│   └── models.rs               # Add OperationLog model
├── ui_tabs/
│   └── platform.rs             # Modify: add tabs, integrate DrawerViewModel
└── calc/
    └── db.rs                   # Add database connection helper
```

## Database Schema

### Migration SQL

**Up Migration:**

```sql
-- Operation logs table for tracking external API operations
CREATE TABLE operation_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id TEXT NOT NULL,           -- Platform/project identifier
    operation_type TEXT NOT NULL,       -- "vm_create", "firewall_update", "dns_update"
    external_system TEXT NOT NULL,      -- "GCP", "Cloudflare", "Supabase", "DuckDNS"
    status TEXT NOT NULL,               -- "started", "completed", "failed"
    started_at INTEGER NOT NULL,        -- Unix timestamp (seconds)
    completed_at INTEGER,               -- Unix timestamp (null if in progress)
    error_message TEXT,                 -- Null if successful, error details if failed
    details TEXT                        -- JSON metadata (VM name, IP, zone, etc.)
);

-- Index for fast project-specific queries (drawer filtering)
CREATE INDEX idx_operation_logs_project_id ON operation_logs(project_id);

-- Index for chronological queries (recent operations first)
CREATE INDEX idx_operation_logs_started_at ON operation_logs(started_at DESC);

-- Composite index for project + time queries
CREATE INDEX idx_operation_logs_project_time ON operation_logs(project_id, started_at DESC);
```

**Down Migration:**

```sql
DROP INDEX IF EXISTS idx_operation_logs_project_time;
DROP INDEX IF EXISTS idx_operation_logs_started_at;
DROP INDEX IF EXISTS idx_operation_logs_project_id;
DROP TABLE IF EXISTS operation_logs;
```

### Diesel Schema

```rust
diesel::table! {
    operation_logs (id) {
        id -> Integer,
        project_id -> Text,
        operation_type -> Text,
        external_system -> Text,
        status -> Text,
        started_at -> BigInt,
        completed_at -> Nullable<BigInt>,
        error_message -> Nullable<Text>,
        details -> Nullable<Text>,
    }
}
```

### Rust Models

```rust
/// Operation log entry for external API operations
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::storage::diesel_schema::operation_logs)]
pub struct OperationLog {
    pub id: i32,
    pub project_id: String,
    pub operation_type: String,
    pub external_system: String,
    pub status: String,
    pub started_at: i64,
    pub completed_at: Option<i64>,
    pub error_message: Option<String>,
    pub details: Option<String>,
}

/// New operation log (for inserts)
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = crate::storage::diesel_schema::operation_logs)]
pub struct NewOperationLog {
    pub project_id: String,
    pub operation_type: String,
    pub external_system: String,
    pub status: String,
    pub started_at: i64,
    pub completed_at: Option<i64>,
    pub error_message: Option<String>,
    pub details: Option<String>,
}
```

## Repository Layer

### OperationLogRepository

**Key Methods:**

- `create(log: NewOperationLog) -> Result<OperationLog>` - Create new operation log
- `get_by_project(project_id: &str, limit: i64) -> Result<Vec<OperationLog>>` - Get logs for specific project
- `get_all(limit: i64) -> Result<Vec<OperationLog>>` - Get all logs (newest first)
- `get_by_id(log_id: i32) -> Result<OperationLog>` - Get specific log
- `mark_completed(log_id: i32) -> Result<()>` - Update status to completed
- `mark_failed(log_id: i32, error: &str) -> Result<()>` - Update status to failed
- `delete_older_than(days: i64) -> Result<usize>` - Retention policy cleanup
- `get_status_counts(project_id: &str) -> Result<StatusCounts>` - Count operations by status

**Error Handling:**

All methods use `anyhow::Result` with context for proper error propagation. Database connection failures, query failures, and data inconsistencies are all handled with descriptive error messages.

## ViewModel Layer

### DrawerViewModel

**State Management:**

```rust
pub struct DrawerState {
    pub project_id: String,
    pub selected_tab: usize,  // 0 = Status, 1 = Logs, 2 = Operations
    pub operation_logs: Vec<OperationLog>,
    pub filtered_logs: Vec<String>,  // Filtered stdout logs
    pub loading: bool,
}
```

**Commands (UI → ViewModel):**

```rust
pub enum DrawerCommand {
    SwitchTab { project_id: String, tab_index: usize },
    LoadOperationLogs { project_id: String, limit: i64 },
    LoadStdoutLogs { project_id: String },
    Refresh { project_id: String },
}
```

**Events (ViewModel → UI):**

```rust
pub enum DrawerEvent {
    TabSwitched { project_id: String, tab_index: usize },
    OperationLogsLoaded { project_id: String, logs: Vec<OperationLog> },
    StdoutLogsLoaded { project_id: String, logs: Vec<String> },
    OperationLogged { project_id: String, log: OperationLog },
    Error { project_id: String, error: String },
}
```

**Platform Event Integration:**

DrawerViewModel listens to `PlatformEvent` and logs operations to database:

- `VMCreated` → Log "vm_create" to GCP
- `VMDeleted` → Log "vm_delete" to GCP
- `VMRestarted` → Log "vm_restart" to GCP
- `VMRegenerated` → Log "vm_regenerate" to GCP
- `FirewallUpdated` → Log "firewall_update" to GCP
- `ProjectSelected` → Log "project_select" to GCP
- `BillingFetched` → Log "billing_fetch" to GCP
- `Error` → Log failed operation with error message

**Async Runtime:**

Uses `smol` executor with async-channel for command/event processing. Background command processor runs in detached task.

## UI Layer

### Table Layout Changes

**Row Height:** Exactly 30px (hard requirement)

```rust
let mut table = data_table::DataTableBuilder::new()
    .columns(columns)
    .striped(true)
    .resizable(false)
    .min_scrolled_height(0.0)  // Allow table to shrink
    .max_scroll_height(f32::INFINITY)  // Allow table to grow
    .row_height(30.0)  // ⭐ Compact 30px rows
    .header_height(40.0)  // Slightly taller header for readability
    ;
```

**Fill Window Layout:**

```rust
// Calculate available height (window height - top bar - padding)
let available_height = ui.available_height();

// Add small margins
ui.add_space(4.0);

// Vertical layout with small horizontal margins
ui.horizontal(|ui| {
    ui.add_space(8.0);  // Left margin
    
    ui.vertical(|ui| {
        // Action buttons (these will scroll away)
        self.render_action_buttons(ui, vm.as_deref_mut());
        
        ui.add_space(4.0);
        
        // Table in scroll area
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])  // Don't auto-shrink
            .max_height(available_height - 60.0)  // Reserve space for buttons
            .show(ui, |ui| {
                // Set table to fill available width
                ui.set_min_width(ui.available_width());
                table.show(ui);
            });
    });
    
    ui.add_space(8.0);  // Right margin
});
```

### Drawer Tabs

**Tab Rendering:**

```rust
fn render_drawer_content(
    ui: &mut egui::Ui,
    row: &PlatformRow,
    drawer_vm: Option<&DrawerViewModel>,
    drawer_state: Option<&mut DrawerState>,
) {
    ui.add_space(4.0);
    
    // Get or create drawer state
    let mut local_state = drawer_state
        .map(|s| s.clone())
        .unwrap_or_else(|| DrawerState::new(row.project_id.clone()));
    
    let mut tab_index = local_state.selected_tab;
    
    // Compact secondary tabs
    ui.add(
        egui_material3::tabs_secondary(&mut tab_index)
            .id_salt(format!("drawer_tabs_{}", row.project_id))
            .tab("Status")
            .tab("Logs")
            .tab("Operations")
    );
    
    ui.add_space(8.0);
    ui.separator();
    ui.add_space(8.0);
    
    // Render tab content
    egui::ScrollArea::vertical()
        .max_height(400.0)
        .show(ui, |ui| {
            match tab_index {
                0 => render_status_tab(ui, row),
                1 => render_logs_tab(ui, &local_state),
                2 => render_operations_tab(ui, &local_state),
                _ => {}
            }
        });
}
```

### Tab Content

**Status Tab:**
- Shows all existing drawer content (connection info, project details, VM status, firewall status, SSH menu)
- No changes to existing functionality

**Logs Tab:**
- Filters stdout logs by `project_id`
- Color-codes by log level: ERROR (red), WARN (amber), INFO (blue)
- Shows line numbers
- Scrollable when many entries

**Operations Tab:**
- Displays operation history from SQLite
- Groups operations by status (✅ completed, ❌ failed, 🔄 in progress)
- Shows timestamp, operation type, external system
- Displays operation details (VM name, IP, zone, etc.)
- Shows error messages for failed operations
- Scrollable when many entries

## Data Flow

### Complete End-to-End Flow

```
┌─────────────────────────────────────────────────────────────┐
│ User clicks "Update Firewall" button                        │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│ UI: PlatformTab stores action in egui temp data             │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│ UI: PlatformTab sends command to main ViewModel             │
│     vm.update_firewall(project_id)                          │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│ ViewModel: Calls GCP API via gcp/compute.rs                 │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│ GCP API: Returns success/failure                            │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│ ViewModel: Emits PlatformEvent::FirewallUpdated             │
└─────────────────────────────────────────────────────────────┘
                          ↓
        ┌─────────────────┴─────────────────┐
        ↓                                   ↓
┌──────────────────────┐      ┌────────────────────────────┐
│ UI: PlatformTab      │      │ DrawerViewModel            │
│ Updates row state    │      │ handle_platform_event()    │
└──────────────────────┘      └────────────────────────────┘
                                            ↓
                              ┌────────────────────────────┐
                              │ OperationLogRepository     │
                              │ create(NewOperationLog)    │
                              └────────────────────────────┘
                                            ↓
                              ┌────────────────────────────┐
                              │ SQLite: INSERT INTO        │
                              │ operation_logs             │
                              └────────────────────────────┘
                                            ↓
                              ┌────────────────────────────┐
                              │ DrawerViewModel emits      │
                              │ DrawerEvent::OperationLogged│
                              └────────────────────────────┘
                                            ↓
                              ┌────────────────────────────┐
                              │ UI: PlatformTab polls      │
                              │ drawer_vm.poll_events()    │
                              └────────────────────────────┘
                                            ↓
                              ┌────────────────────────────┐
                              │ UI: Refreshes Operations   │
                              │ tab if currently open      │
                              └────────────────────────────┘
```

## Testing Strategy

### Unit Tests

**Repository Tests** (`repositories/operation_log.rs`):
- Create and retrieve operation logs
- Filter logs by project_id
- Mark operations as completed/failed
- Delete old logs (retention policy)
- Status count aggregation

**ViewModel Tests** (`viewmodel/drawer.rs`):
- Command processing (SwitchTab, LoadOperationLogs, LoadStdoutLogs, Refresh)
- Event emission (TabSwitched, OperationLogsLoaded, etc.)
- Platform event handling (VMCreated, FirewallUpdated, etc.)
- State management (drawer states keyed by project_id)

### Integration Tests

**UI Tests** (`ui_tabs/platform.rs`):
- Drawer tab switching
- Drawer state initialization
- Tab content rendering
- Event polling and state updates

### Manual Testing Checklist

**UI Testing:**
- [ ] Table fills window vertically (resize window to confirm)
- [ ] Table fills window horizontally (resize window to confirm)
- [ ] Rows are exactly 30px tall (measure with screenshot)
- [ ] Small margins around table (8px left/right, 4px top)
- [ ] Action buttons scroll away when scrolling down
- [ ] Table header scrolls with content (not sticky)

**Drawer Testing:**
- [ ] Drawer opens when clicking row
- [ ] Three tabs visible: Status, Logs, Operations
- [ ] Tabs use compact secondary style (full-width underline)
- [ ] Status tab shows all original drawer content
- [ ] Tab switching works (click each tab)

**Logs Tab Testing:**
- [ ] Shows "No logs found" when no filtered logs
- [ ] Filters logs by project_id correctly
- [ ] Color-codes ERROR (red), WARN (amber), INFO (blue)
- [ ] Shows line numbers
- [ ] Scrolls when many log entries

**Operations Tab Testing:**
- [ ] Shows "No operations recorded" initially
- [ ] Logs VM create operation to database
- [ ] Logs VM delete operation to database
- [ ] Logs firewall update operation to database
- [ ] Shows ✅ for completed operations
- [ ] Shows ❌ for failed operations
- [ ] Shows 🔄 for in-progress operations
- [ ] Displays timestamp correctly
- [ ] Shows operation details (VM name, IP, etc.)
- [ ] Shows error message for failed operations
- [ ] Operations persist across app restarts

**Data Flow Testing:**
- [ ] Create VM → Operation logged → Appears in Operations tab
- [ ] Delete VM → Operation logged → Appears in Operations tab
- [ ] Update firewall → Operation logged → Appears in Operations tab
- [ ] Filter logs by project A → Only project A logs shown
- [ ] Switch to project B drawer → Different logs shown

## Migration Path

### Step 1: Database Migration

```bash
# Create migration
cd mobile
diesel migration generate create_operation_logs

# Edit up.sql and down.sql (see Database Schema section)

# Run migration
diesel migration run

# Verify schema
diesel print-schema > src/storage/diesel_schema_new.rs
# Manually merge into diesel_schema.rs
```

### Step 2: Code Integration (Incremental)

**Phase 1: Add models and repository (no UI changes)**
- Add `models.rs` with `OperationLog` and `NewOperationLog`
- Add `repositories/operation_log.rs` with `OperationLogRepository`
- Test repository independently with unit tests

**Phase 2: Add DrawerViewModel (no logging yet)**
- Add `viewmodel/drawer.rs` with `DrawerViewModel`
- Implement command/event processing
- Test ViewModel commands/events with unit tests

**Phase 3: Add tabs to drawer (no operation logging yet)**
- Modify `render_drawer_content()` to use tabs
- Add `render_status_tab()`, `render_logs_tab()`, `render_operations_tab()`
- Test tab switching manually

**Phase 4: Wire up operation logging**
- Connect `DrawerViewModel` to `PlatformEvent` in `PlatformTab::ui()`
- Test end-to-end flow: operation → event → log → UI

**Phase 5: Table layout changes**
- Apply 30px row height to `DataTableBuilder`
- Apply fill-window layout with margins
- Test responsiveness with different window sizes

### Step 3: Rollback Plan

If anything breaks, DrawerViewModel can be disabled without breaking existing UI:

```rust
impl Default for PlatformTab {
    fn default() -> Self {
        Self {
            // ... existing fields ...
            drawer_vm: None,  // ← Disable DrawerViewModel
            drawer_states: std::collections::HashMap::new(),
        }
    }
}

// Drawer will still work with Status tab only (existing content)
```

## Performance Considerations

### Database Indexing

Three indexes ensure fast queries:
1. `idx_operation_logs_project_id` - Fast project filtering
2. `idx_operation_logs_started_at` - Fast chronological queries
3. `idx_operation_logs_project_time` - Composite index for drawer queries

### Memory Management

- Drawer states are cached in `HashMap<String, DrawerState>` in `PlatformTab`
- Operation logs loaded on-demand (limit 100 per project)
- Stdout logs filtered in-memory from existing `LOG_BUFFER`

### Async Processing

- All repository operations run in `smol` async tasks
- Commands processed in background thread (non-blocking UI)
- Events polled each frame (`poll_events()` uses `try_recv()`)

## Security Considerations

### SQL Injection

- All queries use Diesel ORM with parameterized queries
- No raw SQL string interpolation
- Repository layer validates input types

### Data Retention

- Repository provides `delete_older_than(days)` method for retention policy
- Recommended: Delete logs older than 90 days
- Can be run as periodic background task

### Sensitive Data

- Operation details stored as JSON strings (not parsed)
- Error messages may contain sensitive info (stack traces, IPs)
- Consider encrypting `details` and `error_message` columns for production

## Open Questions

None - all design decisions validated with user.

## Success Criteria

1. ✅ Database schema created with migrations
2. ✅ Repository layer implements CRUD operations
3. ✅ DrawerViewModel handles commands and events
4. ✅ UI integrates tabs with Status/Logs/Operations content
5. ✅ Table layout uses 30px rows and fills window
6. ✅ Operation logging works end-to-end
7. ✅ All manual tests pass
8. ✅ Unit tests cover repository and ViewModel

## Next Steps

1. Invoke `writing-plans` skill to create implementation plan
2. Create Diesel migration files
3. Implement repository layer with unit tests
4. Implement DrawerViewModel with unit tests
5. Integrate UI layer with tabs
6. Apply table layout changes
7. Wire up operation logging
8. Manual testing and iteration
9. Code review and merge

## References

- Existing ViewModel pattern: `mobile/src/viewmodel/platform.rs`
- egui-material3 tabs: `reference/egui-material3/examples/stories/tabs_window.rs`
- Diesel schema: `mobile/src/storage/diesel_schema.rs`
- Log capture: `mobile/src/log_capture.rs`
- PlatformRow structure: `mobile/src/ui_tabs/platform.rs` lines 18-67
- Existing drawer: `mobile/src/ui_tabs/platform.rs` lines 737-856
