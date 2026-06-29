# Platform Tab Data Table Refactoring Design

## Goal

Replace MaterialSpreadsheet in `mobile/src/ui_tabs/platform.rs` with egui-material3's Data Table with Drawer to provide clearer visualization of GCP platform hierarchy and streamline VM management actions.

## Overview

**Current state:** Platform tab uses MaterialSpreadsheet with hierarchical rows (Account → Project → VM) where each entity is a separate row. File is 1951 lines with complex dual-tracking of metadata rows and display rows.

**Target state:** Single row per GCP platform with drawer containing hierarchical details. Action buttons integrated into table rows. Cleaner data model with one-to-one mapping between platform configs and table rows.

**Scope:** Rendering layer only. All existing dialogs (Add Platform, Delete Platform, GCP Wizard, Select Project, Billing, Delete VM) preserved unchanged.

## Requirements

### Table Structure
- **Columns:** Platform | Type | Steps | Actions
- **Rows:** One row per GCP platform (from `CloudPlatformConfig`)
- **Drawer:** Hierarchical display of email → project → VM details
- **Default state:** All drawers open initially, user can toggle, state persists in-memory

### Column Details

1. **Platform** - Platform name from config (e.g., "my-gcp-platform")
2. **Type** - Always "GCP" for now
3. **Steps** - Connection progress with status indicators:
   - Format: `✓ GCP Connected → ✗ Project Created → ✗ VM Created → ✗ SSH Connected`
   - Dynamic based on actual state (OAuth token, project selection, VM existence, external IP)
4. **Actions** - Inline buttons:
   - "Update Firewall" (enabled if project selected)
   - "Select Project" (enabled if OAuth connected)
   - "Delete VM" (enabled if VM exists)
   - "Regen VM" (enabled if VM exists)
   - "Restart VM" (enabled if VM exists)
   - "Delete Platform" (always enabled)
   - "Refresh" (always enabled)

### Drawer Content

**Format:** Plain text display (not interactive), showing hierarchical structure:

```
user@gmail.com (3 projects total)
  └─ project-id (selected)
     └─ VM: dure-vm-1234567890
        • Firewall: ✓ Whitelisted (1.2.3.4)
        • SSH: ✓ Ready
```

**Partial state handling:**
- No OAuth: "Not connected"
- No project: "user@gmail.com (N projects total) \n  └─ No project selected"
- No VM: "... \n     └─ No VM created"

### Constraints

- Platform has at most one selected project
- Selected project has at most one VM
- Action buttons operate on the single VM in the platform
- No multi-selection needed (remove checkbox column from reference example)

## Architecture

### Three-Layer Design

1. **Data Layer** - `PlatformRow` struct holds all data for one table row
2. **Rendering Layer** - `data_table()` builder constructs table from rows
3. **Interaction Layer** - Button handlers trigger operations, dialogs preserved

### Key Simplification

**Remove dual tracking:**
- Old: `self.rows: Vec<[String; 3]>` (metadata) + `data_rows: Vec<Vec<String>>` (display)
- New: `self.rows: Vec<PlatformRow>` (single source of truth)

**Benefits:**
- Simpler mental model (one row = one platform)
- No synchronization issues
- Easier to maintain

## Data Model

### PlatformRow Struct

```rust
#[derive(Clone, Debug)]
struct PlatformRow {
    // Identity
    platform_name: String,        // Internal platform name from config
    platform_type: String,        // "GCP"
    
    // Connection state flags (for Steps column)
    gcp_connected: bool,          // Has OAuth access token
    project_selected: bool,       // Has gcp_selected_project_id
    vm_created: bool,             // vms.len() > 0
    ssh_ready: bool,              // VM has external_ip.is_some()
    
    // Drawer content data
    email: Option<String>,        // Connected Google account
    total_project_count: usize,   // Fetched from GCP API
    selected_project_id: Option<String>,
    vm_name: Option<String>,      // First VM name
    firewall_status: String,      // "✓ Whitelisted (IP)" or "✗ Not whitelisted"
    ssh_status: String,           // "✓ Ready" or "? No external IP"
    
    // Action button state
    has_vm: bool,                 // Enable/disable VM operation buttons
    vm_zone: Option<String>,      // For VM operations (delete, restart, regen)
}
```

### PlatformTab State Changes

**Remove:**
- `spreadsheet: Option<MaterialSpreadsheet>`
- Old row tracking arrays
- `drawer_expanded: HashSet<usize>` (not needed - data_table manages internally)

**Keep:**
- `rows: Vec<PlatformRow>` (repurposed)
- All dialog state fields (unchanged)

**Add:**
- None

## Data Flow

### Load Sequence

1. **`load_rows()` called** (on tab open, refresh, wizard close)
   
2. **Load config:**
   ```rust
   let (app_config, _) = load_config()?;
   ```

3. **Build rows:**
   ```rust
   for platform in app_config.platforms.iter() {
       if platform.platform_type != "gcp" { continue; }
       
       let row = PlatformRow {
           platform_name: platform.name.clone(),
           platform_type: "GCP".to_string(),
           
           // Compute state flags
           gcp_connected: platform.gcp_oauth_access_token.is_some(),
           project_selected: platform.gcp_selected_project_id.is_some(),
           vm_created: !platform.vms.is_empty(),
           ssh_ready: platform.vms.first()
               .and_then(|vm| vm.external_ip.as_ref())
               .is_some(),
           
           // Extract drawer data
           email: platform.gcp_connected_email.clone(),
           total_project_count: fetch_project_count(platform),
           selected_project_id: platform.gcp_selected_project_id.clone(),
           vm_name: platform.vms.first().map(|vm| vm.name.clone()),
           firewall_status: compute_firewall_status(platform),
           ssh_status: compute_ssh_status(platform),
           
           // Action state
           has_vm: !platform.vms.is_empty(),
           vm_zone: platform.vms.first().map(|vm| vm.zone.clone()),
       };
       
       self.rows.push(row);
   }
   ```

4. **Drawer state initialization:**
   - Drawer state is automatically managed by data_table widget via egui persistence
   - On first load, all drawers default to closed (egui default behavior)
   - To open all drawers by default, we need to initialize the DataTableState on first load
   - See State Management section for details

### Rendering Flow

1. **`ui()` called each frame**

2. **Build data_table:**
   ```rust
   let mut table = data_table()
       .id(Id::new("platform_table"))
       .allow_selection(false)
       .allow_drawer(true)
       .column("Platform", 200.0, false)
       .column("Type", 100.0, false)
       .column("Steps", 400.0, false)
       .column("Actions", 500.0, false);  // Wider for 7 buttons
   
   for (idx, platform_row) in self.rows.iter().enumerate() {
       table = table.row(|row| {
           row.cell(&platform_row.platform_name)
              .cell(&platform_row.platform_type)
              .cell(&format_steps(platform_row))
              .cell_ui(|ui| render_action_buttons(ui, idx, platform_row))
              .drawer(|ui| render_drawer_content(ui, platform_row))
       });
   }
   
   table.show(ui);
   ```

3. **Render dialogs** (unchanged)

### Action Flow

1. **Button clicked in Actions column**

2. **Handler stores context:**
   ```rust
   // Example: Delete VM clicked
   self.show_delete_vm_dialog = true;
   self.delete_vm_platform = platform_row.platform_name.clone();
   self.delete_vm_list = vec![(
       platform_row.vm_name.clone().unwrap(),
       platform_row.vm_zone.clone().unwrap(),
       "RUNNING".to_string(),
   )];
   ```

3. **Existing dialog renders** (next frame)

4. **On dialog confirm, operation executes**

5. **Reload triggered:**
   ```rust
   self.loaded = false;  // Triggers load_rows() next frame
   ```

## Component Details

### Helper Functions

#### format_steps()

```rust
fn format_steps(row: &PlatformRow) -> String {
    let gcp = if row.gcp_connected { "✓" } else { "✗" };
    let proj = if row.project_selected { "✓" } else { "✗" };
    let vm = if row.vm_created { "✓" } else { "✗" };
    let ssh = if row.ssh_ready { "✓" } else { "✗" };
    
    format!(
        "{} GCP Connected → {} Project Created → {} VM Created → {} SSH Connected",
        gcp, proj, vm, ssh
    )
}
```

#### render_action_buttons()

```rust
fn render_action_buttons(
    ui: &mut egui::Ui,
    row_idx: usize,
    row: &PlatformRow,
    tab_state: &mut PlatformTab,  // Need mutable access to tab
) {
    ui.horizontal(|ui| {
        // Update Firewall
        let firewall_btn = MaterialButton::text("Update Firewall")
            .enabled(row.project_selected);
        if ui.add(firewall_btn).clicked() {
            tab_state.trigger_firewall_update(&row.platform_name);
        }
        
        // Select Project
        let select_project_btn = MaterialButton::text("Select Project")
            .enabled(row.gcp_connected);
        if ui.add(select_project_btn).clicked() {
            tab_state.show_select_project_dialog(row.platform_name.clone());
        }
        
        // Delete VM
        let delete_btn = MaterialButton::text("Delete VM")
            .enabled(row.has_vm);
        if ui.add(delete_btn).clicked() {
            tab_state.show_delete_vm_confirmation(
                row.platform_name.clone(),
                row.vm_name.clone().unwrap(),
                row.vm_zone.clone().unwrap(),
            );
        }
        
        // Regen VM
        let regen_btn = MaterialButton::text("Regen VM")
            .enabled(row.has_vm);
        if ui.add(regen_btn).clicked() {
            tab_state.trigger_vm_regenerate(&row.platform_name);
        }
        
        // Restart VM
        let restart_btn = MaterialButton::text("Restart VM")
            .enabled(row.has_vm);
        if ui.add(restart_btn).clicked() {
            tab_state.trigger_vm_restart(&row.platform_name);
        }
        
        // Delete Platform
        if ui.add(MaterialButton::text("Delete Platform")).clicked() {
            tab_state.show_delete_platform_confirmation(row.platform_name.clone());
        }
        
        // Refresh
        if ui.add(MaterialButton::text("Refresh")).clicked() {
            tab_state.loaded = false;
        }
    });
}
```

**Challenge:** `.cell_ui()` closure doesn't have mutable access to tab state. Need to either:
- Store clicked action in local state, process after table.show()
- Use message passing pattern (events)
- Restructure to handle clicks outside the builder

**Solution:** Use closure captures to store pending actions, process after table.show():

```rust
#[derive(Debug, Clone)]
enum PlatformAction {
    UpdateFirewall(String),  // platform_name
    SelectProject(String),
    DeleteVM { platform_name: String, vm_name: String, vm_zone: String },
    RegenVM(String),
    RestartVM(String),
    DeletePlatform(String),
    Refresh,
}

// In ui():
let mut pending_action: Option<PlatformAction> = None;

// In cell_ui closure:
if ui.add(delete_btn).clicked() {
    pending_action = Some(PlatformAction::DeleteVM {
        platform_name: row.platform_name.clone(),
        vm_name: row.vm_name.clone().unwrap(),
        vm_zone: row.vm_zone.clone().unwrap(),
    });
}

// After table.show():
if let Some(action) = pending_action {
    match action {
        PlatformAction::DeleteVM { platform_name, vm_name, vm_zone } => {
            self.show_delete_vm_confirmation(platform_name, vm_name, vm_zone);
        }
        PlatformAction::UpdateFirewall(platform_name) => {
            self.trigger_firewall_update(&platform_name);
        }
        PlatformAction::SelectProject(platform_name) => {
            self.show_select_project_dialog(platform_name);
        }
        PlatformAction::RegenVM(platform_name) => {
            self.trigger_vm_regenerate(&platform_name);
        }
        PlatformAction::RestartVM(platform_name) => {
            self.trigger_vm_restart(&platform_name);
        }
        PlatformAction::DeletePlatform(platform_name) => {
            self.show_delete_platform_confirmation(platform_name);
        }
        PlatformAction::Refresh => {
            self.loaded = false;
        }
    }
}
```

#### render_drawer_content()

```rust
fn render_drawer_content(ui: &mut egui::Ui, row: &PlatformRow) {
    ui.add_space(8.0);
    
    // Level 1: Email + project count
    if let Some(email) = &row.email {
        ui.label(format!("{} ({} projects total)", email, row.total_project_count));
    } else {
        ui.label("Not connected");
    }
    
    // Level 2: Selected project
    if let Some(project_id) = &row.selected_project_id {
        ui.label(format!("  └─ {} (selected)", project_id));
        
        // Level 3: VM details
        if let Some(vm_name) = &row.vm_name {
            ui.label(format!("     └─ VM: {}", vm_name));
            ui.label(format!("        • Firewall: {}", row.firewall_status));
            ui.label(format!("        • SSH: {}", row.ssh_status));
        } else {
            ui.label("     └─ No VM created");
        }
    } else {
        ui.label("  └─ No project selected");
    }
}
```

#### compute_firewall_status()

```rust
fn compute_firewall_status(platform: &CloudPlatformConfig) -> String {
    if let Some(project_id) = &platform.gcp_selected_project_id {
        if let Some(access_token) = &platform.gcp_oauth_access_token {
            use crate::calc::gcp_rest::{GcpRestClient, get_current_ip};
            
            let client = GcpRestClient::new(access_token.clone());
            
            match get_current_ip() {
                Ok(current_ip) => {
                    match client.check_ip_whitelisted(project_id, &current_ip) {
                        Ok(true) => format!("✓ Whitelisted ({})", current_ip),
                        Ok(false) => "✗ Not whitelisted".to_string(),
                        Err(_) => "? Status unknown".to_string(),
                    }
                }
                Err(_) => "? Failed to get IP".to_string(),
            }
        } else {
            "Not connected".to_string()
        }
    } else {
        "No project".to_string()
    }
}
```

#### compute_ssh_status()

```rust
fn compute_ssh_status(platform: &CloudPlatformConfig) -> String {
    if let Some(vm) = platform.vms.first() {
        if vm.external_ip.is_some() {
            "✓ Ready".to_string()
        } else {
            "? No external IP".to_string()
        }
    } else {
        "No VM".to_string()
    }
}
```

#### fetch_project_count()

```rust
fn fetch_project_count(platform: &CloudPlatformConfig) -> usize {
    if let Some(access_token) = &platform.gcp_oauth_access_token {
        use crate::calc::gcp_rest::GcpRestClient;
        let client = GcpRestClient::new(access_token.clone());
        
        match client.list_projects(None) {
            Ok(list) => list.projects.len(),
            Err(_) => 0,
        }
    } else {
        0
    }
}
```

## State Management

### Drawer Expansion State

**Automatic management:** The data_table widget automatically manages drawer state via egui's persistence system. The state is stored in `DataTableState.drawer_open_rows: HashSet<usize>` and persisted by egui across frames.

**Our responsibility:** To make all drawers open by default (as per requirements), we need to initialize the state on first load:

```rust
// In ui(), before building the table:
let table_id = Id::new("platform_table");

// Get or initialize state with all drawers open
let mut state: DataTableState = ui.data_mut(|d| {
    let existing = d.get_persisted::<DataTableState>(table_id);
    match existing {
        Some(state) => state,
        None => {
            // First load - initialize with all drawers open
            let mut state = DataTableState::default();
            state.drawer_open_rows = (0..self.rows.len()).collect();
            state
        }
    }
});

// Store back (data_table will manage from here)
ui.data_mut(|d| d.insert_persisted(table_id, state));
```

**After initialization:** The data_table widget handles all drawer toggle interactions and persists state automatically. We don't need to track drawer state in PlatformTab.

### Button Enable/Disable State

**Computed dynamically from PlatformRow:**
- Update Firewall: `row.project_selected`
- VM operations: `row.has_vm`
- Refresh: always enabled

No separate state tracking needed.

## Dialog Preservation

### Unchanged Dialogs

All existing dialog methods preserved:

1. **render_add_dialog()** - OAuth flow, platform creation
2. **render_delete_platform_dialog()** - Platform deletion confirmation
3. **render_select_project_dialog()** - GCP project selection
4. **render_delete_vm_dialog()** - VM deletion confirmation
5. **render_billing_dialog()** - Billing data display
6. **GCP Wizard (gcp_wizard)** - VM creation flow

### Integration Pattern

**Action buttons → State fields → Dialog render:**

```rust
// Button clicked in table
if delete_vm_clicked {
    self.show_delete_vm_dialog = true;
    self.delete_vm_platform = platform_name;
    self.delete_vm_list = vec![(vm_name, zone, status)];
}

// Dialog renders in ui()
if self.show_delete_vm_dialog {
    self.render_delete_vm_dialog(ui.ctx());
}
```

### Top-Level Buttons

**Outside table (above or below):**
- "Add Platform" - Opens add dialog
- "Delete Platform" - Needs platform selection mechanism (see below)
- "Select Project" - Needs platform selection mechanism
- "Add VM" - Opens GCP wizard
- "Estimated Billing" - Opens billing dialog
- "Refresh" - Reloads table

**Platform selection for top-level actions:**

Since we removed row selection, platform-specific top-level buttons need to move into the Actions column:

**Updated Actions column buttons:**
- "Update Firewall" (enabled if project selected)
- "Select Project" (always enabled if OAuth connected)
- "Delete VM" (enabled if VM exists)
- "Regen VM" (enabled if VM exists)
- "Restart VM" (enabled if VM exists)
- "Delete Platform" (always enabled)
- "Refresh" (always enabled)

**Top-level buttons (above table):**
- "Add Platform" - Opens add dialog
- "Add VM" - Opens GCP wizard for first GCP platform (or dropdown if multiple)
- "Estimated Billing" - Opens billing dialog
- "Refresh All" - Reloads table data

This maintains feature parity without needing row selection. The Actions column becomes wider (~500px) to accommodate more buttons.

## Error Handling

### API Failures

**GCP API calls in load_rows():**
- Project count fetch failure: Use 0, don't block loading
- Firewall status check failure: Show "? Status unknown"
- OAuth token expiry: Show "Not connected", rely on refresh flow

**Pattern:**
```rust
match client.list_projects(None) {
    Ok(list) => list.projects.len(),
    Err(e) => {
        eprintln!("Failed to fetch project count: {}", e);
        0  // Graceful degradation
    }
}
```

### Config Loading Failure

```rust
match load_config() {
    Ok((app_config, _)) => {
        // Build rows
    }
    Err(e) => {
        self.load_error = Some(format!("Failed to load config: {}", e));
        // Show error in UI instead of table
    }
}
```

### Missing Data

- No platforms: Show empty table with message
- No OAuth token: Drawer shows "Not connected"
- No project: Drawer shows "No project selected"
- No VM: Drawer shows "No VM created", VM buttons disabled

All handled gracefully via Option types and conditional display.

## Testing Approach

### Manual Testing Scenarios

1. **Fresh platform (no OAuth)**
   - Verify: Steps shows all ✗, drawer shows "Not connected"
   - Action: All buttons disabled except Refresh

2. **OAuth connected, no project**
   - Verify: Steps shows "✓ GCP Connected → ✗ ...", drawer shows email + project count
   - Action: Update Firewall disabled, VM buttons disabled

3. **Project selected, no VM**
   - Verify: Steps shows "✓ ✓ GCP Connected → Project Created → ✗ ✗"
   - Drawer: Shows project (selected), "No VM created"
   - Action: Update Firewall enabled, VM buttons disabled

4. **Fully configured (has VM)**
   - Verify: All steps ✓ (or ✓✓✗✗ if no external IP yet)
   - Drawer: Shows full hierarchy with VM details
   - Action: All buttons enabled

5. **Multiple platforms**
   - Verify: Each row independent, drawers toggle separately
   - Action: Buttons affect correct platform

6. **Drawer toggle**
   - Verify: Clicks arrow to collapse, clicks again to expand
   - State persists during refresh (in-memory only)

7. **Button operations**
   - Delete VM: Shows existing delete dialog
   - Regen VM: Triggers regenerate, reload shows new VM
   - Restart VM: Triggers restart
   - Update Firewall: Updates firewall whitelist
   - Refresh: Reloads table data

### Integration Testing

**Existing dialogs still work:**
- Add Platform → OAuth flow → Platform appears in table
- GCP Wizard → VM creation → Table updates with new VM
- Select Project → Project selection → Table shows selected project
- Delete VM → Confirmation → VM removed from table

**Refresh triggers:**
- After wizard close: `self.wizard_was_open && !wizard_is_open → self.loaded = false`
- After dialog actions: Existing handlers already set `self.loaded = false`

## Migration Strategy

### Implementation Order

1. **Add PlatformRow struct** - Define complete data model
2. **Implement helper functions** - format_steps, compute_*, render_*
3. **Refactor load_rows()** - Build Vec<PlatformRow> instead of dual arrays
4. **Replace table rendering** - Switch from MaterialSpreadsheet to data_table
5. **Update action handlers** - Adapt to new row structure
6. **Remove old code** - Delete spreadsheet field, old row arrays
7. **Test all scenarios** - Verify dialogs and actions still work

### Rollback Plan

**Preserve backup:**
```bash
cp src/ui_tabs/platform.rs src/ui_tabs/platform.rs.backup-$(date +%Y%m%d)
```

**Git branch:**
Work in feature branch, can revert if needed.

### Risk Mitigation

**Low risk items:**
- Data model change (isolated to rendering)
- Dialog preservation (no changes needed)

**Medium risk items:**
- Action button integration (closure mutable access challenge)
- Drawer state synchronization (API dependency)

**Mitigation:**
- Test action buttons first with simple logging
- Verify data_table drawer API behavior in isolation
- Incremental testing at each step

## Performance Considerations

### API Calls in load_rows()

**Current behavior:** Fetches project count and firewall status on every load.

**Optimization opportunities:**
1. Cache project count with TTL (e.g., 5 minutes)
2. Make firewall check async, show "Loading..." initially
3. Batch API calls if multiple platforms

**Decision:** Start with synchronous approach (matches current code), optimize if slow.

### Table Rendering

**data_table is efficient:**
- Renders only visible rows
- No significant change from MaterialSpreadsheet

### Memory

**Reduced memory:**
- Old: `Vec<[String; 3]>` + `Vec<Vec<String>>`
- New: `Vec<PlatformRow>` (single allocation)

## Future Enhancements

### Potential Improvements (Out of Scope)

1. **Multi-VM support** - If requirement changes to allow multiple VMs per project
2. **Firebase/Supabase platforms** - Add Type column variety
3. **Async API calls** - Non-blocking project count / firewall checks
4. **Drawer interaction** - Click project name to open Select Project dialog
5. **Action button customization** - Show/hide based on user preferences
6. **Persistent drawer state** - Save to config or egui persistence

### Extensibility

**Adding new columns:**
```rust
.column("Region", 100.0, false)
```

**Adding new action buttons:**
```rust
if ui.add(MaterialButton::text("New Action")).clicked() {
    // Handle
}
```

**Adding new drawer sections:**
```rust
ui.label("New section:");
ui.label(format!("  Data: {}", value));
```

Design supports easy extension without structural changes.

## Success Criteria

### Functional Requirements

- ✓ Table displays one row per GCP platform
- ✓ Columns: Platform | Type | Steps | Actions
- ✓ Steps show dynamic progress with ✓/✗
- ✓ Actions column has 5 buttons with correct enable/disable
- ✓ Drawer shows hierarchical email → project → VM
- ✓ Drawers open by default, user can toggle
- ✓ All existing dialogs work unchanged
- ✓ Refresh triggers reload correctly

### Non-Functional Requirements

- ✓ Code is cleaner (remove dual tracking)
- ✓ File size reduced (or at least not increased)
- ✓ Maintainability improved (clear data flow)
- ✓ Performance equivalent or better
- ✓ No regressions in existing functionality

## References

- **egui-material3 example:** `/home/wj/work/egui-material3/examples/stories/datatable_window.rs` line 865
- **Current implementation:** `/home/wj/work/dure/mobile/src/ui_tabs/platform.rs`
- **Config structure:** `/home/wj/work/dure/mobile/src/config.rs`
- **GCP operations:** `/home/wj/work/dure/mobile/src/calc/hosting_gcp.rs`
- **OAuth flow:** `/home/wj/work/dure/mobile/src/ui_dlg/platform_gcp.rs`
