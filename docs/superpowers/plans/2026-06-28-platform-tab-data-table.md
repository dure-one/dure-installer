# Platform Tab Data Table Refactoring Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace MaterialSpreadsheet in platform tab with egui-material3 data_table with drawer for clearer GCP platform visualization.

**Architecture:** Three-layer refactoring - update data model (PlatformRow struct), implement helper functions for data transformation, replace rendering layer (MaterialSpreadsheet → data_table).

**Tech Stack:** Rust 1.81+, egui 0.33, egui-material3, diesel 2.2

## Global Constraints

- Rust edition 2021, minimum version 1.81
- Preserve all existing dialog functionality (Add Platform, Delete Platform, GCP Wizard, Select Project, Billing, Delete VM)
- Platform has at most one selected project with at most one VM
- No multi-row selection needed
- All drawers open by default, state persists via egui

---

## File Structure

This refactoring modifies a single file:

**Modify:** `mobile/src/ui_tabs/platform.rs` (1951 lines)
- Currently uses MaterialSpreadsheet with dual row tracking
- Will use data_table with single PlatformRow vec
- Helper functions added for data transformation and rendering
- PlatformAction enum added for button click handling

**No new files created** - this is an in-place refactoring.

---

### Task 1: Update PlatformRow Struct

**Files:**
- Modify: `mobile/src/ui_tabs/platform.rs:14-26`

**Interfaces:**
- Consumes: `CloudPlatformConfig` from `crate::config`
- Produces: `PlatformRow` struct with fields matching spec

- [ ] **Step 1: Update PlatformRow struct fields**

Replace the existing incomplete PlatformRow with the complete structure from the spec:

```rust
/// Platform row data for data table
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

File: `mobile/src/ui_tabs/platform.rs`
Location: Replace lines 14-26

- [ ] **Step 2: Verify compilation**

Run: `cargo check --manifest-path mobile/Cargo.toml`
Expected: Compilation errors about missing fields in PlatformRow construction (expected - will fix in later tasks)

- [ ] **Step 3: Commit**

```bash
git add mobile/src/ui_tabs/platform.rs
git commit -m "refactor: update PlatformRow struct to match data table spec

Add complete field set for single-row-per-platform model:
- Connection state flags (gcp_connected, project_selected, vm_created, ssh_ready)
- Drawer content data (email, total_project_count, project_id, vm details)
- Action button state (has_vm, vm_zone)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 2: Add PlatformAction Enum

**Files:**
- Modify: `mobile/src/ui_tabs/platform.rs:27` (insert after PlatformRow)

**Interfaces:**
- Consumes: User button clicks in data_table cells
- Produces: `PlatformAction` enum for deferred action handling

- [ ] **Step 1: Add PlatformAction enum**

Add this enum right after the PlatformRow struct definition:

```rust
/// Actions that can be triggered from platform table rows
#[derive(Debug, Clone)]
enum PlatformAction {
    UpdateFirewall(String),  // platform_name
    SelectProject(String),   // platform_name
    DeleteVM {
        platform_name: String,
        vm_name: String,
        vm_zone: String,
    },
    RegenVM(String),         // platform_name
    RestartVM(String),       // platform_name
    DeletePlatform(String),  // platform_name
    Refresh,                 // Refresh table data
}
```

File: `mobile/src/ui_tabs/platform.rs`
Location: Insert after PlatformRow struct (around line 46)

- [ ] **Step 2: Verify compilation**

Run: `cargo check --manifest-path mobile/Cargo.toml`
Expected: Same errors as before (enum added but not used yet)

- [ ] **Step 3: Commit**

```bash
git add mobile/src/ui_tabs/platform.rs
git commit -m "feat: add PlatformAction enum for deferred button handling

Enables action button clicks in data_table cell_ui closures to store
pending actions and process them after table.show() returns.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 3: Add Helper Functions

**Files:**
- Modify: `mobile/src/ui_tabs/platform.rs` (add functions before `impl PlatformTab`)

**Interfaces:**
- Consumes: `CloudPlatformConfig`, `PlatformRow`
- Produces: Helper functions for data transformation and rendering

- [ ] **Step 1: Add format_steps() helper**

Insert this function before `impl PlatformTab`:

```rust
/// Format connection progress steps with status indicators
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

- [ ] **Step 2: Add compute_firewall_status() helper**

```rust
/// Compute firewall whitelist status for a platform
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

- [ ] **Step 3: Add compute_ssh_status() helper**

```rust
/// Compute SSH readiness status for a platform's VM
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

- [ ] **Step 4: Add fetch_project_count() helper**

```rust
/// Fetch total project count from GCP API
fn fetch_project_count(platform: &CloudPlatformConfig) -> usize {
    if let Some(access_token) = &platform.gcp_oauth_access_token {
        use crate::calc::gcp_rest::GcpRestClient;
        let client = GcpRestClient::new(access_token.clone());
        
        match client.list_projects(None) {
            Ok(list) => list.projects.len(),
            Err(e) => {
                eprintln!("Failed to fetch project count: {}", e);
                0
            }
        }
    } else {
        0
    }
}
```

- [ ] **Step 5: Add render_drawer_content() helper**

```rust
/// Render drawer content showing platform hierarchy
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

File: `mobile/src/ui_tabs/platform.rs`
Location: Insert all helpers before `impl PlatformTab` block (around line 190-200)

- [ ] **Step 6: Verify compilation**

Run: `cargo check --manifest-path mobile/Cargo.toml`
Expected: May show unused function warnings (expected - will use in next tasks)

- [ ] **Step 7: Commit**

```bash
git add mobile/src/ui_tabs/platform.rs
git commit -m "feat: add helper functions for data table rendering

Add five helpers for data transformation and display:
- format_steps: Build progress indicator string
- compute_firewall_status: Check IP whitelist status
- compute_ssh_status: Check VM SSH readiness
- fetch_project_count: Query GCP API for project count
- render_drawer_content: Display hierarchical platform details

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 4: Rewrite load_rows() Function

**Files:**
- Modify: `mobile/src/ui_tabs/platform.rs` (find and replace load_rows method in impl PlatformTab)

**Interfaces:**
- Consumes: `AppConfig` from config file, GCP REST API
- Produces: `Vec<PlatformRow>` with one row per GCP platform

- [ ] **Step 1: Find current load_rows() location**

Run: `grep -n "fn load_rows" mobile/src/ui_tabs/platform.rs`
Expected: Shows line number (likely around 459)

- [ ] **Step 2: Replace load_rows() implementation**

Replace the entire `fn load_rows(&mut self)` method body with:

```rust
fn load_rows(&mut self) {
    self.rows.clear();
    self.load_error = None;

    #[cfg(not(target_arch = "wasm32"))]
    {
        match load_config() {
            Ok((app_config, _)) => {
                eprintln!("DEBUG: Building rows for {} platforms", app_config.platforms.len());

                for platform in app_config.platforms.iter() {
                    // Only show GCP platforms for now
                    if platform.platform_type != "gcp" {
                        eprintln!("DEBUG: Skipping non-GCP platform: {}", platform.name);
                        continue;
                    }

                    eprintln!("DEBUG: Processing GCP platform: {}, selected_project: {:?}, vm_count: {}",
                        platform.name, platform.gcp_selected_project_id, platform.vms.len());

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
                        
                        // Action button state
                        has_vm: !platform.vms.is_empty(),
                        vm_zone: platform.vms.first().map(|vm| vm.zone.clone()),
                    };

                    eprintln!("DEBUG: Created row: {} - connected:{} project:{} vm:{}",
                        row.platform_name, row.gcp_connected, row.project_selected, row.vm_created);

                    self.rows.push(row);
                }

                self.loaded = true;
                eprintln!("DEBUG: Loaded {} platform rows", self.rows.len());
            }
            Err(e) => {
                self.load_error = Some(format!("Failed to load config: {}", e));
                eprintln!("DEBUG: Config load error: {}", e);
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    {
        self.load_error = Some("WASM platform not supported".to_string());
    }
}
```

File: `mobile/src/ui_tabs/platform.rs`
Location: Replace existing `fn load_rows(&mut self)` method body

- [ ] **Step 3: Verify compilation**

Run: `cargo check --manifest-path mobile/Cargo.toml`
Expected: Should compile now that PlatformRow construction is complete

- [ ] **Step 4: Commit**

```bash
git add mobile/src/ui_tabs/platform.rs
git commit -m "refactor: rewrite load_rows to build PlatformRow structs

Replace dual-tracking row system with single Vec<PlatformRow>.
Each platform config becomes one row with computed state flags
and pre-fetched drawer data.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 5: Replace MaterialSpreadsheet with data_table

**Files:**
- Modify: `mobile/src/ui_tabs/platform.rs` (rewrite ui() method table rendering section)

**Interfaces:**
- Consumes: `Vec<PlatformRow>` from load_rows()
- Produces: Rendered data_table widget with drawers

- [ ] **Step 1: Find the table rendering section in ui()**

Run: `grep -n "MaterialSpreadsheet\|spreadsheet.show" mobile/src/ui_tabs/platform.rs | head -5`
Expected: Shows lines where MaterialSpreadsheet is used (likely around 400-405)

- [ ] **Step 2: Replace table rendering code**

Find the section that builds and shows the MaterialSpreadsheet (typically in `ui()` method around lines 380-407).

Replace from the line after "ui.add_space(8.0);" (after the Refresh button section) up to but NOT including the "// Add platform dialog" comment with:

```rust
        // Table rendering
        if !self.loaded {
            self.load_rows();
        }

        if let Some(error) = &self.load_error {
            ui.colored_label(egui::Color32::from_rgb(255, 0, 0), format!("Error: {}", error));
        } else if self.rows.is_empty() {
            ui.label("No platforms configured. Click 'Add Platform' to get started.");
        } else {
            // Store pending action from button clicks
            let mut pending_action: Option<PlatformAction> = None;

            // Build data table
            let table_id = egui::Id::new("platform_table");
            
            // Initialize drawer state (all open by default on first load)
            use egui_material3::datatable::DataTableState;
            let state: DataTableState = ui.data_mut(|d| {
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
            
            // Store state back
            ui.data_mut(|d| d.insert_persisted(table_id, state));

            let mut table = data_table()
                .id(table_id)
                .allow_selection(false)
                .allow_drawer(true)
                .column("Platform", 200.0, false)
                .column("Type", 100.0, false)
                .column("Steps", 400.0, false)
                .column("Actions", 500.0, false);

            for row in self.rows.iter() {
                let row_clone = row.clone();
                let row_clone2 = row.clone();
                
                table = table.row(|r| {
                    r.cell(&row_clone.platform_name)
                     .cell(&row_clone.platform_type)
                     .cell(&format_steps(&row_clone))
                     .cell_ui(|ui| {
                         ui.horizontal(|ui| {
                             // Update Firewall
                             let firewall_btn = MaterialButton::text("Update Firewall")
                                 .enabled(row_clone.project_selected);
                             if ui.add(firewall_btn).clicked() {
                                 pending_action = Some(PlatformAction::UpdateFirewall(
                                     row_clone.platform_name.clone()
                                 ));
                             }
                             
                             // Select Project
                             let select_project_btn = MaterialButton::text("Select Project")
                                 .enabled(row_clone.gcp_connected);
                             if ui.add(select_project_btn).clicked() {
                                 pending_action = Some(PlatformAction::SelectProject(
                                     row_clone.platform_name.clone()
                                 ));
                             }
                             
                             // Delete VM
                             let delete_vm_btn = MaterialButton::text("Delete VM")
                                 .enabled(row_clone.has_vm);
                             if ui.add(delete_vm_btn).clicked() {
                                 pending_action = Some(PlatformAction::DeleteVM {
                                     platform_name: row_clone.platform_name.clone(),
                                     vm_name: row_clone.vm_name.clone().unwrap_or_default(),
                                     vm_zone: row_clone.vm_zone.clone().unwrap_or_default(),
                                 });
                             }
                             
                             // Regen VM
                             let regen_vm_btn = MaterialButton::text("Regen VM")
                                 .enabled(row_clone.has_vm);
                             if ui.add(regen_vm_btn).clicked() {
                                 pending_action = Some(PlatformAction::RegenVM(
                                     row_clone.platform_name.clone()
                                 ));
                             }
                             
                             // Restart VM
                             let restart_vm_btn = MaterialButton::text("Restart VM")
                                 .enabled(row_clone.has_vm);
                             if ui.add(restart_vm_btn).clicked() {
                                 pending_action = Some(PlatformAction::RestartVM(
                                     row_clone.platform_name.clone()
                                 ));
                             }
                             
                             // Delete Platform
                             if ui.add(MaterialButton::text("Delete Platform")).clicked() {
                                 pending_action = Some(PlatformAction::DeletePlatform(
                                     row_clone.platform_name.clone()
                                 ));
                             }
                             
                             // Refresh
                             if ui.add(MaterialButton::text("Refresh")).clicked() {
                                 pending_action = Some(PlatformAction::Refresh);
                             }
                         });
                     })
                     .drawer(|ui| {
                         render_drawer_content(ui, &row_clone2);
                     })
                });
            }

            table.show(ui);

            // Process pending action after table rendering
            if let Some(action) = pending_action {
                match action {
                    PlatformAction::UpdateFirewall(platform_name) => {
                        // Find platform and trigger firewall update
                        if let Ok((app_config, config_path)) = load_config() {
                            if let Some(platform) = app_config.platforms.iter()
                                .find(|p| p.name == platform_name)
                            {
                                if let Some(project_id) = &platform.gcp_selected_project_id {
                                    if let Some(access_token) = &platform.gcp_oauth_access_token {
                                        use crate::calc::gcp_rest::{GcpRestClient, get_current_ip};
                                        let client = GcpRestClient::new(access_token.clone());
                                        match get_current_ip() {
                                            Ok(current_ip) => {
                                                match client.add_ip_to_whitelist(project_id, &current_ip) {
                                                    Ok(_) => {
                                                        eprintln!("✓ IP {} whitelisted in project {}", current_ip, project_id);
                                                        self.loaded = false; // Refresh
                                                    }
                                                    Err(e) => {
                                                        eprintln!("Failed to whitelist IP: {}", e);
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                eprintln!("Failed to get current IP: {}", e);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    PlatformAction::SelectProject(platform_name) => {
                        self.show_select_project_dialog(platform_name);
                    }
                    PlatformAction::DeleteVM { platform_name, vm_name, vm_zone } => {
                        self.show_delete_vm_confirmation(platform_name, vm_name, vm_zone);
                    }
                    PlatformAction::RegenVM(platform_name) => {
                        // Trigger VM regeneration
                        if let Ok((mut app_config, config_path)) = load_config() {
                            if let Some(platform) = app_config.platforms.iter_mut()
                                .find(|p| p.name == platform_name)
                            {
                                if let Some(access_token) = &platform.gcp_oauth_access_token {
                                    if let Some(zone) = platform.vms.first().map(|vm| vm.zone.clone()) {
                                        use crate::calc::gcp_rest::GcpRestClient;
                                        use crate::calc::hosting_gcp;
                                        let client = GcpRestClient::new(access_token.clone());
                                        match hosting_gcp::regenerate_vm(&client, platform, &zone) {
                                            Ok(msg) => {
                                                eprintln!("✓ {}", msg);
                                                // Save config
                                                if let Err(e) = app_config.save(&config_path) {
                                                    eprintln!("Failed to save config: {}", e);
                                                }
                                                self.loaded = false; // Refresh
                                            }
                                            Err(e) => {
                                                eprintln!("Failed to regenerate VM: {}", e);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    PlatformAction::RestartVM(platform_name) => {
                        // Trigger VM restart
                        if let Ok((app_config, _)) = load_config() {
                            if let Some(platform) = app_config.platforms.iter()
                                .find(|p| p.name == platform_name)
                            {
                                if let Some(access_token) = &platform.gcp_oauth_access_token {
                                    if let Some(vm) = platform.vms.first() {
                                        use crate::calc::gcp_rest::GcpRestClient;
                                        use crate::calc::hosting_gcp;
                                        let client = GcpRestClient::new(access_token.clone());
                                        match hosting_gcp::restart_vm(&client, vm) {
                                            Ok(msg) => {
                                                eprintln!("✓ {}", msg);
                                                self.loaded = false; // Refresh
                                            }
                                            Err(e) => {
                                                eprintln!("Failed to restart VM: {}", e);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    PlatformAction::DeletePlatform(platform_name) => {
                        self.show_delete_platform_confirmation(platform_name);
                    }
                    PlatformAction::Refresh => {
                        self.loaded = false;
                    }
                }
            }
        }
```

File: `mobile/src/ui_tabs/platform.rs`
Location: Replace MaterialSpreadsheet rendering section in `ui()` method

- [ ] **Step 3: Remove old imports**

Find and remove these lines at the top of the file:
- Any import of `MaterialSpreadsheet` or `text_column` from `egui_material3::spreadsheet`

The imports should already have `data_table` and `MaterialButton` from Task 1.

- [ ] **Step 4: Verify compilation**

Run: `cargo check --manifest-path mobile/Cargo.toml`
Expected: Should compile successfully

- [ ] **Step 5: Commit**

```bash
git add mobile/src/ui_tabs/platform.rs
git commit -m "refactor: replace MaterialSpreadsheet with data_table

Replace hierarchical spreadsheet with single-row data table:
- One row per platform with drawer content
- 7 action buttons per row with enable/disable logic
- PlatformAction enum for deferred button handling
- Drawer state initialization (all open by default)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 6: Clean Up PlatformTab State

**Files:**
- Modify: `mobile/src/ui_tabs/platform.rs` (PlatformTab struct definition and Default impl)

**Interfaces:**
- Consumes: None (internal cleanup)
- Produces: Simplified PlatformTab state structure

- [ ] **Step 1: Remove obsolete fields from PlatformTab struct**

Find the PlatformTab struct definition (around lines 28-129) and remove these fields:
- `row_selection: Vec<bool>`
- `drawer_expanded: HashSet<usize>`

Keep all other fields (dialog state, wizard state, etc).

- [ ] **Step 2: Update Default implementation**

Find `impl Default for PlatformTab` (around lines 131-174) and remove initialization of:
- `row_selection: Vec::new()`
- `drawer_expanded: HashSet::new()`

- [ ] **Step 3: Remove obsolete helper methods**

Search for and remove these methods if they exist:
- `format_vm_details`
- `format_platform_details`
- Any methods that reference `data_rows` or the old spreadsheet structure

Run: `grep -n "fn format_vm_details\|fn format_platform_details" mobile/src/ui_tabs/platform.rs`

If found, delete those method implementations.

- [ ] **Step 4: Remove unused imports**

At the top of the file, remove:
- `use std::collections::HashSet;` (if no other code uses HashSet)

Check: `grep -n "HashSet" mobile/src/ui_tabs/platform.rs`
If only the removed field used it, delete the import.

- [ ] **Step 5: Verify compilation**

Run: `cargo check --manifest-path mobile/Cargo.toml`
Expected: Clean compilation with no warnings

If there are unused import warnings, remove those imports.

- [ ] **Step 6: Commit**

```bash
git add mobile/src/ui_tabs/platform.rs
git commit -m "refactor: remove obsolete PlatformTab state fields

Remove dual-tracking artifacts:
- row_selection Vec (no multi-selection needed)
- drawer_expanded HashSet (managed by data_table internally)
- format_vm_details/format_platform_details helpers

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 7: Update Top-Level Button Handlers

**Files:**
- Modify: `mobile/src/ui_tabs/platform.rs` (button section in ui() method)

**Interfaces:**
- Consumes: User button clicks
- Produces: Simplified top-level button handlers

- [ ] **Step 1: Find button section**

Run: `grep -n "Add Platform\|Delete Platform\|Select Project" mobile/src/ui_tabs/platform.rs | head -10`
Expected: Shows button locations (likely around 207-257)

- [ ] **Step 2: Update button logic**

Since we removed row selection, update the button handlers to work without selection:

Find the section with "Delete Platform" button (around line 214-229) and replace with:

```rust
            // Delete Platform button - now handled via Actions column in table
            // Keep button but disable with message
            let delete_platform_button = MaterialButton::outlined("Delete Platform")
                .enabled(false);
            ui.add(delete_platform_button);
            if ui.add(egui::Label::new("Use 'Delete Platform' in table Actions").sense(egui::Sense::hover())).hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Help);
            }
```

Find "Select Project" button section (around line 234-257) and replace with:

```rust
            // Select Project button - now handled via Actions column in table
            #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
            {
                let select_project_button = MaterialButton::outlined("Select Project")
                    .enabled(false);
                ui.add(select_project_button);
            }
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check --manifest-path mobile/Cargo.toml`
Expected: Clean compilation

- [ ] **Step 4: Test button UI**

Run: `cargo run --manifest-path mobile/Cargo.toml`
Expected: App launches, platform tab shows disabled Delete/Select buttons with help text

- [ ] **Step 5: Commit**

```bash
git add mobile/src/ui_tabs/platform.rs
git commit -m "refactor: disable top-level platform-specific buttons

Delete Platform and Select Project now handled via Actions column
in table rows. Top-level buttons disabled with helper text.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 8: Manual Testing - Fresh Platform

**Files:**
- Test: Manual testing of platform tab UI

**Interfaces:**
- Consumes: Running application
- Produces: Verified behavior for fresh platform scenario

- [ ] **Step 1: Clear test data (optional)**

If you want to test from scratch:

```bash
# Backup current config
cp ~/.config/dure/config.yml ~/.config/dure/config.yml.backup

# Start with empty platforms
# Edit config to remove all platforms or rename file temporarily
```

- [ ] **Step 2: Launch application**

Run: `cargo run --manifest-path mobile/Cargo.toml`
Expected: Application window opens

- [ ] **Step 3: Navigate to Platform tab**

Click on "Platform" tab
Expected: Shows "No platforms configured. Click 'Add Platform' to get started."

- [ ] **Step 4: Test Add Platform flow**

1. Click "Add Platform" button
2. Enter platform name: "test-gcp"
3. Keep Type as "GCP"
4. Click "Connect to Google Cloud"
5. Complete OAuth in browser
Expected: OAuth succeeds, shows "✓ Connected as: your-email@gmail.com"

6. Click "Save" or "Add"
Expected: Dialog closes, table shows one row

- [ ] **Step 5: Verify row display**

Check table row shows:
- Platform: "test-gcp"
- Type: "GCP"
- Steps: "✓ GCP Connected → ✗ Project Created → ✗ VM Created → ✗ SSH Connected"
- Actions: 7 buttons visible

- [ ] **Step 6: Verify drawer content**

Click drawer arrow (>) to expand (or verify it's already expanded)
Expected: Drawer shows:
```
your-email@gmail.com (N projects total)
  └─ No project selected
```

- [ ] **Step 7: Test disabled buttons**

Click "Delete VM" button
Expected: Button is disabled (grayed out), no action occurs

Click "Update Firewall" button
Expected: Button is disabled, no action occurs

- [ ] **Step 8: Document results**

Create note of any issues found:
```bash
echo "Fresh platform test: PASS" > /tmp/platform-test-1.txt
# Or document issues
```

---

### Task 9: Manual Testing - Project Selection

**Files:**
- Test: Manual testing of project selection and drawer update

**Interfaces:**
- Consumes: Platform with OAuth connected
- Produces: Verified project selection flow

- [ ] **Step 1: From previous test state, click "Select Project" button**

In the Actions column, click "Select Project"
Expected: Dialog opens showing list of GCP projects

- [ ] **Step 2: Select a project**

Choose a project from the list, click "Select" or "OK"
Expected: Dialog closes

- [ ] **Step 3: Verify table updates**

Wait for table to refresh (automatic via `self.loaded = false`)
Expected: Steps column updates to "✓ GCP Connected → ✓ Project Created → ✗ VM Created → ✗ SSH Connected"

- [ ] **Step 4: Verify drawer updates**

Check drawer content (should be open)
Expected: Shows:
```
your-email@gmail.com (N projects total)
  └─ your-project-id (selected)
     └─ No VM created
```

- [ ] **Step 5: Verify button enable states**

Check "Update Firewall" button - should be ENABLED
Check "Delete VM" button - should be DISABLED (no VM yet)
Check "Select Project" button - should be ENABLED

- [ ] **Step 6: Test firewall update**

Click "Update Firewall" button
Expected: Firewall whitelist updated, console shows success message

Refresh table (click "Refresh" button in Actions)
Expected: Drawer firewall status updates to "✓ Whitelisted (your-IP)"

- [ ] **Step 7: Document results**

```bash
echo "Project selection test: PASS" >> /tmp/platform-test-1.txt
```

---

### Task 10: Manual Testing - VM Creation

**Files:**
- Test: Manual testing of VM creation flow and complete state

**Interfaces:**
- Consumes: Platform with project selected
- Produces: Verified VM creation and full drawer display

- [ ] **Step 1: Click "Add VM" button (top-level)**

Expected: GCP Wizard dialog opens

- [ ] **Step 2: Complete VM creation wizard**

Follow wizard steps:
1. Select zone (e.g., "us-central1-a")
2. Confirm settings
3. Wait for VM creation
Expected: Wizard shows progress, then success message, closes automatically

- [ ] **Step 3: Wait for table refresh**

Table should refresh automatically after wizard closes
Expected: Row updates within 2-3 seconds

- [ ] **Step 4: Verify Steps column**

Expected: "✓ GCP Connected → ✓ Project Created → ✓ VM Created → ✓ SSH Connected"
(All green checkmarks if VM has external IP)

- [ ] **Step 5: Verify drawer content**

Expected: Shows complete hierarchy:
```
your-email@gmail.com (N projects total)
  └─ your-project-id (selected)
     └─ VM: dure-vm-1234567890
        • Firewall: ✓ Whitelisted (your-IP)
        • SSH: ✓ Ready
```

- [ ] **Step 6: Verify all action buttons enabled**

All buttons should be enabled:
- Update Firewall: ENABLED
- Select Project: ENABLED
- Delete VM: ENABLED
- Regen VM: ENABLED
- Restart VM: ENABLED
- Delete Platform: ENABLED
- Refresh: ENABLED

- [ ] **Step 7: Test VM operations**

Test "Restart VM":
1. Click button
2. Wait for operation (console shows progress)
3. Click "Refresh" button
Expected: VM restarts, status remains same

- [ ] **Step 8: Document results**

```bash
echo "VM creation test: PASS" >> /tmp/platform-test-1.txt
```

---

### Task 11: Manual Testing - Drawer Toggle

**Files:**
- Test: Manual testing of drawer expand/collapse persistence

**Interfaces:**
- Consumes: Platform row with drawer open
- Produces: Verified drawer state persistence

- [ ] **Step 1: Verify drawer is open**

Look for drawer content visible below the platform row
Expected: Hierarchy visible

- [ ] **Step 2: Click collapse arrow (v)**

Click the arrow icon on the left side of the row
Expected: Drawer collapses, content hidden, arrow changes to (>)

- [ ] **Step 3: Click expand arrow (>)**

Click the arrow again
Expected: Drawer expands, content visible again

- [ ] **Step 4: Collapse drawer again**

Close the drawer

- [ ] **Step 5: Refresh table**

Click "Refresh" button in Actions column
Expected: Table reloads, drawer REMAINS COLLAPSED (state persisted)

- [ ] **Step 6: Restart application**

Close and reopen the application, navigate to Platform tab
Expected: Drawer is OPEN again (state persisted in-memory only, reset on app restart)

- [ ] **Step 7: Document results**

```bash
echo "Drawer toggle test: PASS" >> /tmp/platform-test-1.txt
```

---

### Task 12: Manual Testing - Multiple Platforms

**Files:**
- Test: Manual testing with multiple platform rows

**Interfaces:**
- Consumes: Existing platform
- Produces: Verified multi-row display and independent drawer states

- [ ] **Step 1: Add second platform**

Click "Add Platform", create "test-gcp-2", complete OAuth
Expected: Two rows in table

- [ ] **Step 2: Verify independent row data**

Each row should show its own:
- Platform name
- Steps (may differ if one has project/VM, other doesn't)
- Drawer content (different emails or same email but different states)

- [ ] **Step 3: Test independent drawer states**

1. Expand first platform's drawer
2. Collapse second platform's drawer
Expected: Drawers toggle independently

- [ ] **Step 4: Test independent action buttons**

Click "Refresh" on first platform row
Expected: Only first platform reloads, second platform unchanged

- [ ] **Step 5: Add VM to second platform**

Use "Select Project" on second platform, then "Add VM"
Expected: Second platform gets VM, first platform unchanged

- [ ] **Step 6: Verify both rows show correctly**

Both rows should now show complete states with independent data

- [ ] **Step 7: Document results**

```bash
echo "Multiple platforms test: PASS" >> /tmp/platform-test-1.txt
cat /tmp/platform-test-1.txt
```

Expected output:
```
Fresh platform test: PASS
Project selection test: PASS
VM creation test: PASS
Drawer toggle test: PASS
Multiple platforms test: PASS
```

---

### Task 13: Manual Testing - Delete Operations

**Files:**
- Test: Manual testing of delete VM and delete platform operations

**Interfaces:**
- Consumes: Platform with VM
- Produces: Verified delete operations and table updates

- [ ] **Step 1: Test Delete VM**

On a platform with a VM, click "Delete VM" button
Expected: Delete VM confirmation dialog opens

- [ ] **Step 2: Confirm deletion**

In dialog, select the VM, click "Delete"
Expected: Dialog closes, VM deletion starts, table refreshes

- [ ] **Step 3: Verify row updates after VM deletion**

Expected: 
- Steps: "✓ GCP Connected → ✓ Project Created → ✗ VM Created → ✗ SSH Connected"
- Drawer: Shows "└─ No VM created"
- VM action buttons: DISABLED

- [ ] **Step 4: Test Delete Platform**

Click "Delete Platform" button in Actions column
Expected: Delete Platform confirmation dialog opens

- [ ] **Step 5: Confirm platform deletion**

In dialog, confirm deletion
Expected: Dialog closes, platform removed from config, table refreshes

- [ ] **Step 6: Verify row removed**

Expected: Row disappears from table

If this was the last platform:
Expected: Table shows "No platforms configured. Click 'Add Platform' to get started."

- [ ] **Step 7: Document results**

```bash
echo "Delete operations test: PASS" >> /tmp/platform-test-1.txt
```

---

### Task 14: Final Cleanup and Commit

**Files:**
- Modify: `mobile/src/ui_tabs/platform.rs`

**Interfaces:**
- Consumes: All previous changes
- Produces: Clean, final state

- [ ] **Step 1: Remove all debug eprintln statements**

Search for debug output:
Run: `grep -n "eprintln.*DEBUG" mobile/src/ui_tabs/platform.rs`

Remove or comment out DEBUG eprintln lines (but keep error logging)

- [ ] **Step 2: Final compilation check**

Run: `cargo check --manifest-path mobile/Cargo.toml`
Expected: No errors, no warnings

If warnings exist, fix them.

- [ ] **Step 3: Run clippy**

Run: `cargo clippy --manifest-path mobile/Cargo.toml -- -D warnings`
Expected: No clippy warnings

Fix any issues found.

- [ ] **Step 4: Format code**

Run: `cargo fmt --manifest-path mobile/Cargo.toml`
Expected: Code formatted according to Rust style

- [ ] **Step 5: Verify file size reduction**

Run: `wc -l mobile/src/ui_tabs/platform.rs`
Expected: Fewer lines than original 1951 (likely 1400-1600)

- [ ] **Step 6: Final test run**

Run: `cargo run --manifest-path mobile/Cargo.toml`

Navigate to Platform tab, verify:
- Table displays correctly
- Drawers work
- Buttons work
- All dialogs open correctly

- [ ] **Step 7: Commit final cleanup**

```bash
git add mobile/src/ui_tabs/platform.rs
git commit -m "refactor: final cleanup of platform tab refactoring

Remove debug output, format code, fix clippy warnings.
Platform tab now uses data_table with drawer instead of MaterialSpreadsheet.

Refactoring summary:
- Reduced file complexity by removing dual-tracking system
- Single PlatformRow per platform with computed state
- Action buttons integrated into table rows
- Drawer shows hierarchical platform details
- All existing dialogs preserved

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Self-Review Checklist

**Spec Coverage:**
- ✓ PlatformRow struct matches spec (Task 1)
- ✓ PlatformAction enum for button handling (Task 2)
- ✓ Helper functions implemented (Task 3)
- ✓ load_rows() rewritten (Task 4)
- ✓ data_table rendering replaces MaterialSpreadsheet (Task 5)
- ✓ State cleanup (Task 6)
- ✓ Button handlers updated (Task 7)
- ✓ Manual testing covers all scenarios (Tasks 8-13)

**Placeholder Scan:**
- ✓ No TBD/TODO markers
- ✓ All code blocks complete
- ✓ All commands have expected output

**Type Consistency:**
- ✓ PlatformRow fields match usage across all tasks
- ✓ PlatformAction enum variants match button handlers
- ✓ Helper function signatures consistent

**Missing Spec Requirements:**
- None - all requirements covered

## Notes

This plan uses manual testing instead of automated tests because:
1. egui UI testing is complex and not well-established in Rust ecosystem
2. Platform tab heavily depends on external services (GCP API, OAuth)
3. Dialog interactions are visual and state-dependent
4. Manual testing provides better validation for UI/UX correctness

The refactoring is substantial but focused - a single file transformation with clear before/after states. Each task is independently committable and verifiable.
