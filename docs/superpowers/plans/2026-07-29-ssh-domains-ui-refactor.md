# SSH and Domains Tab UI Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add width flexibility to SSH tab and redesign Domains tab from stair-like dual-spreadsheet to drawer-based nested tables for UI consistency.

**Architecture:** Two independent UI refactors following platform.rs patterns: (1) SSH tab adds adaptive width ratio calculation to column definitions, (2) Domains tab replaces MaterialSpreadsheet with data_table() + drawer containing nested records table.

**Tech Stack:** Rust (nightly), egui 0.33, eframe 0.33, egui-material3, Diesel 2.3 (SQLite)

## Global Constraints

- Rust nightly toolchain required
- Desktop-only feature (not mobile/WASM)
- Follow TDD: write test first, verify failure, implement, verify pass, commit
- No new clippy warnings
- Width ratio clamped to 0.5..2.0 range
- All operations use temp data pattern for consistency
- Preserve existing functionality (no behavior changes)

---

## Task 1: SSH Tab - Add Width Flexibility

**Files:**
- Modify: `mobile/src/ui_tabs/ssh.rs:952-999` (render_table method)
- Modify: `mobile/src/ui_tabs/ssh.rs:2894` (add test module after existing tests)

**Interfaces:**
- Consumes: `ui.available_width()` (f32) from egui
- Produces: `calculate_width_ratio(available_width: f32, base_width: f32) -> f32`

- [ ] **Step 1: Write failing tests for width ratio calculation**

Add at end of `mobile/src/ui_tabs/ssh.rs`:

```rust
#[cfg(test)]
mod ssh_width_tests {
    use super::*;
    
    #[test]
    fn test_width_ratio_normal_window() {
        let ratio = calculate_width_ratio(1000.0, 1000.0);
        assert_eq!(ratio, 1.0);
    }
    
    #[test]
    fn test_width_ratio_narrow_window() {
        let ratio = calculate_width_ratio(400.0, 1000.0);
        assert_eq!(ratio, 0.5);
    }
    
    #[test]
    fn test_width_ratio_wide_window() {
        let ratio = calculate_width_ratio(2500.0, 1000.0);
        assert_eq!(ratio, 2.0);
    }
    
    #[test]
    fn test_width_ratio_zero_base() {
        let ratio = calculate_width_ratio(1000.0, 0.0);
        assert_eq!(ratio, 2.0);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cd mobile
cargo test ssh_width_tests --lib -- --nocapture
```

Expected output: `error[E0425]: cannot find function 'calculate_width_ratio' in this scope`

- [ ] **Step 3: Implement calculate_width_ratio function**

Add before `impl SshTab` block (around line 336):

```rust
/// Calculate width ratio with clamping to prevent extreme scaling
fn calculate_width_ratio(available_width: f32, base_width: f32) -> f32 {
    (available_width / base_width).max(0.5).min(2.0)
}
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cd mobile
cargo test ssh_width_tests --lib -- --nocapture
```

Expected output: `test result: ok. 4 passed; 0 failed`

- [ ] **Step 5: Update render_table to use width ratios**

In `mobile/src/ui_tabs/ssh.rs`, find the `render_table` method (around line 952) and modify:

```rust
fn render_table(&mut self, ui: &mut egui::Ui, vm: Option<&mut crate::viewmodel::ViewModel>) {
    use egui_material3::data_table;

    // Calculate width ratio for responsive columns
    let available_width = ui.available_width();
    let base_width = 200.0 + 150.0 + 300.0 + 350.0; // 1000.0 total
    let width_ratio = calculate_width_ratio(available_width, base_width);

    let table_id = egui::Id::new("ssh_table");

    // Initialize drawer state (all closed by default)
    use egui_material3::datatable::DataTableState;
    let state: DataTableState = ui.data_mut(|d| {
        d.get_persisted::<DataTableState>(table_id)
            .unwrap_or_default()
    });
    ui.data_mut(|d| d.insert_persisted(table_id, state));

    // Build table with scaled widths
    let mut table = data_table()
        .id(table_id)
        .allow_selection(false)
        .allow_drawer(true)
        .column("Host (Port)", 200.0 * width_ratio, false)
        .column("Platform", 150.0 * width_ratio, false)
        .column("Status", 300.0 * width_ratio, false)
        .column("Operations", 350.0 * width_ratio, false);

    // ... rest of function unchanged (for loop building rows) ...
```

- [ ] **Step 6: Verify code compiles**

```bash
cd mobile
cargo check --lib
```

Expected output: `Finished 'dev' profile [unoptimized + debuginfo]` with no errors

- [ ] **Step 7: Commit SSH tab width flexibility**

```bash
git add mobile/src/ui_tabs/ssh.rs
git commit -m "$(cat <<'EOF'
feat(ui): add width flexibility to SSH tab

Add adaptive width ratio system to SSH hosts table columns.
Columns now scale proportionally with window width (0.5x-2.0x range).

- Add calculate_width_ratio() helper with clamping
- Update render_table() to apply width ratios to columns
- Add unit tests for width ratio calculation

Base widths: Host(200) + Platform(150) + Status(300) + Operations(350) = 1000px

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: Domains Tab - Add DomainRowData Structure

**Files:**
- Modify: `mobile/src/ui_tabs/ns.rs:15-221` (NsTab struct and impl)

**Interfaces:**
- Consumes: `NsConfig` from `crate::calc::ns`
- Produces:
  - `struct DomainRowData { domain: String, provider: String, provider_display: String, records: Vec<DnsRecord> }`
  - `fn build_domain_rows(config: &NsConfig) -> Vec<DomainRowData>`

- [ ] **Step 1: Write failing tests for build_domain_rows**

Add at end of `mobile/src/ui_tabs/ns.rs` (after line 3710):

```rust
#[cfg(test)]
mod ns_table_tests {
    use super::*;
    
    #[test]
    fn test_domain_row_data_from_empty_config() {
        #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
        {
            use crate::calc::ns::NsConfig;
            let config = NsConfig::default();
            let rows = build_domain_rows(&config);
            assert_eq!(rows.len(), 0);
        }
    }
    
    #[test]
    fn test_domain_with_no_records() {
        #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
        {
            use crate::calc::ns::NsConfig;
            let mut config = NsConfig::default();
            config.add_domain(
                "cloudflare".to_string(),
                "example.com".to_string(),
                "test_token".to_string()
            ).unwrap();
            
            let rows = build_domain_rows(&config);
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].domain, "example.com");
            assert_eq!(rows[0].provider, "cloudflare");
            assert_eq!(rows[0].provider_display, "Cloudflare");
            assert_eq!(rows[0].records.len(), 0);
        }
    }
    
    #[test]
    fn test_domain_with_multiple_records() {
        #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
        {
            use crate::calc::ns::{NsConfig, DnsRecord, RecordType};
            let mut config = NsConfig::default();
            config.add_domain(
                "cloudflare".to_string(),
                "example.com".to_string(),
                "test_token".to_string()
            ).unwrap();
            
            let domain_entry = config.get_domain_mut("cloudflare", "example.com").unwrap();
            domain_entry.records.push(DnsRecord {
                record_type: RecordType::A,
                name: "www".to_string(),
                value: "1.2.3.4".to_string(),
                ttl: Some(300),
            });
            domain_entry.records.push(DnsRecord {
                record_type: RecordType::TXT,
                name: "_dmarc".to_string(),
                value: "v=DMARC1; p=none;".to_string(),
                ttl: Some(3600),
            });
            
            let rows = build_domain_rows(&config);
            assert_eq!(rows[0].records.len(), 2);
            assert_eq!(rows[0].records[0].name, "www");
            assert_eq!(rows[0].records[1].name, "_dmarc");
        }
    }
    
    #[test]
    fn test_gcp_provider_display_format() {
        #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
        {
            use crate::calc::ns::NsConfig;
            let mut config = NsConfig::default();
            
            config.add_domain(
                "gcloud:user@gmail.com".to_string(),
                "example.com".to_string(),
                "test_token".to_string()
            ).unwrap();
            
            let rows = build_domain_rows(&config);
            assert_eq!(rows[0].provider, "gcloud:user@gmail.com");
            assert_eq!(rows[0].provider_display, "Google Cloud (user@gmail.com)");
        }
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cd mobile
cargo test ns_table_tests --lib -- --nocapture
```

Expected output: `error[E0425]: cannot find function 'build_domain_rows' in this scope`

- [ ] **Step 3: Add DomainRowData struct**

Add after `NsTab` struct definition (around line 221):

```rust
/// Display data for domain table row + drawer
#[derive(Clone, Debug)]
struct DomainRowData {
    domain: String,
    provider: String,              // Internal identifier (e.g., "cloudflare", "gcloud:email")
    provider_display: String,      // Display name (e.g., "Cloudflare", "Google Cloud (email)")
    records: Vec<crate::calc::ns::DnsRecord>,
}
```

- [ ] **Step 4: Implement build_domain_rows function**

Add after DomainRowData struct:

```rust
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
fn build_domain_rows(config: &crate::calc::ns::NsConfig) -> Vec<DomainRowData> {
    let mut rows = Vec::new();
    
    for (provider, domains) in &config.providers {
        for (domain_name, domain_entry) in domains {
            let provider_display = if provider.starts_with("gcloud:") {
                let email = &provider[7..];
                format!("Google Cloud ({})", email)
            } else {
                match provider.as_str() {
                    "cloudflare" => "Cloudflare".to_string(),
                    "porkbun" => "Porkbun".to_string(),
                    "duckdns" => "DuckDNS".to_string(),
                    _ => provider.clone(),
                }
            };
            
            rows.push(DomainRowData {
                domain: domain_name.clone(),
                provider: provider.clone(),
                provider_display,
                records: domain_entry.records.clone(),
            });
        }
    }
    
    rows.sort_by(|a, b| a.domain.cmp(&b.domain));
    rows
}

#[cfg(any(target_os = "android", target_arch = "wasm32"))]
fn build_domain_rows(_config: &crate::calc::ns::NsConfig) -> Vec<DomainRowData> {
    Vec::new()
}
```

- [ ] **Step 5: Run tests to verify they pass**

```bash
cd mobile
cargo test ns_table_tests --lib -- --nocapture
```

Expected output: `test result: ok. 4 passed; 0 failed`

- [ ] **Step 6: Add domain_rows field to NsTab struct**

In NsTab struct (around line 20), add:

```rust
/// Display data for domain table
#[cfg_attr(feature = "serde", serde(skip))]
domain_rows: Vec<DomainRowData>,
```

And in `Default for NsTab` impl (around line 180), add:

```rust
domain_rows: Vec::new(),
```

- [ ] **Step 7: Verify code compiles**

```bash
cd mobile
cargo check --lib
```

Expected output: `Finished 'dev' profile [unoptimized + debuginfo]` with no errors

- [ ] **Step 8: Commit DomainRowData structure**

```bash
git add mobile/src/ui_tabs/ns.rs
git commit -m "$(cat <<'EOF'
feat(ui): add DomainRowData structure for domains tab

Add DomainRowData struct to prepare for drawer-based redesign.
Build domain rows from config with provider display formatting.

- Add DomainRowData struct with domain, provider, records fields
- Add build_domain_rows() helper to transform NsConfig → Vec<DomainRowData>
- Add unit tests for empty config, domains without/with records, GCP formatting
- Add domain_rows field to NsTab struct

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: Domains Tab - Implement Main Table with Drawer

**Files:**
- Modify: `mobile/src/ui_tabs/ns.rs` (add render functions, update load_data)

**Interfaces:**
- Consumes: `DomainRowData` from Task 2, `ui: &mut egui::Ui`, `vm: Option<&mut ViewModel>`
- Produces:
  - `fn render_domains_table(&mut self, ui: &mut egui::Ui, vm: Option<&mut ViewModel>)`
  - `fn render_domain_drawer(ui: &mut egui::Ui, domain: &DomainRowData, idx: usize)`
  - `fn render_domain_operations(ui: &mut egui::Ui, domain: &DomainRowData, idx: usize)`

- [ ] **Step 1: Add calculate_width_ratio helper**

Add after `build_domain_rows` function:

```rust
/// Calculate width ratio with clamping to prevent extreme scaling
fn calculate_width_ratio(available_width: f32, base_width: f32) -> f32 {
    (available_width / base_width).max(0.5).min(2.0)
}
```

- [ ] **Step 2: Implement render_domain_operations**

Add after `calculate_width_ratio`:

```rust
/// Render operations buttons for domain row
fn render_domain_operations(ui: &mut egui::Ui, domain: &DomainRowData, idx: usize) {
    use egui_material3::MaterialButton;
    
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        ui.style_mut().spacing.button_padding = egui::vec2(6.0, 2.0);
        
        // Add Record
        if ui.add(MaterialButton::outlined("Add Record").small())
            .on_hover_text("Add DNS record to this domain")
            .clicked()
        {
            ui.data_mut(|d| d.insert_temp(
                egui::Id::new(format!("add_record_{}", idx)),
                (domain.provider.clone(), domain.domain.clone())
            ));
        }
        
        // Nameservers
        if ui.add(MaterialButton::outlined("Nameservers").small())
            .on_hover_text("View nameserver configuration")
            .clicked()
        {
            ui.data_mut(|d| d.insert_temp(
                egui::Id::new(format!("view_ns_{}", idx)),
                (domain.provider.clone(), domain.domain.clone())
            ));
        }
        
        // Delete Domain
        if ui.add(MaterialButton::outlined("Delete").small())
            .on_hover_text("Delete domain")
            .clicked()
        {
            ui.data_mut(|d| d.insert_temp(
                egui::Id::new(format!("delete_domain_{}", idx)),
                (domain.provider.clone(), domain.domain.clone())
            ));
        }
    });
}
```

- [ ] **Step 3: Implement render_domain_drawer with nested records table**

Add after `render_domain_operations`:

```rust
/// Render drawer content with nested records table
fn render_domain_drawer(ui: &mut egui::Ui, domain: &DomainRowData, idx: usize) {
    use egui_material3::{data_table, MaterialButton};
    
    ui.heading(format!("Records for {}", domain.domain));
    ui.add_space(8.0);
    
    if domain.records.is_empty() {
        ui.label(
            egui::RichText::new("No records yet")
                .color(ui.visuals().weak_text_color())
        );
        ui.add_space(4.0);
    } else {
        let mut records_table = data_table()
            .id(egui::Id::new(format!("records_table_{}", idx)))
            .allow_selection(false)
            .allow_drawer(false)
            .column("Name", 150.0, false)
            .column("Type", 80.0, false)
            .column("Value", 300.0, false)
            .column("Actions", 80.0, false);
        
        for (rec_idx, record) in domain.records.iter().enumerate() {
            let record_for_delete = record.clone();
            let domain_for_delete = domain.clone();
            
            records_table = records_table.row(move |r| {
                r.cell(&record_for_delete.name)
                 .cell(&record_for_delete.record_type.as_str().to_uppercase())
                 .cell(&record_for_delete.value)
                 .widget_cell(move |ui| {
                     if ui.add(MaterialButton::outlined("🗑").small())
                         .on_hover_text("Delete record")
                         .clicked()
                     {
                         ui.data_mut(|d| d.insert_temp(
                             egui::Id::new(format!("delete_record_{}_{}", idx, rec_idx)),
                             (
                                 domain_for_delete.provider.clone(),
                                 domain_for_delete.domain.clone(),
                                 record_for_delete.name.clone(),
                                 record_for_delete.record_type.as_str().to_string()
                             )
                         ));
                     }
                 })
            });
        }
        
        records_table.show(ui);
    }
}
```

- [ ] **Step 4: Implement render_domains_table**

Add after `render_domain_drawer`:

```rust
/// Render domains table with drawer-based records
fn render_domains_table(
    domain_rows: &[DomainRowData],
    ui: &mut egui::Ui,
    _vm: Option<&mut crate::viewmodel::ViewModel>
) {
    use egui_material3::data_table;
    
    let available_width = ui.available_width();
    let base_width = 300.0 + 200.0 + 250.0; // 750px total
    let width_ratio = calculate_width_ratio(available_width, base_width);
    
    let table_id = egui::Id::new("ns_domains_table");
    
    // Initialize drawer state
    use egui_material3::datatable::DataTableState;
    let state: DataTableState = ui.data_mut(|d| {
        d.get_persisted::<DataTableState>(table_id)
            .unwrap_or_default()
    });
    ui.data_mut(|d| d.insert_persisted(table_id, state));
    
    let mut table = data_table()
        .id(table_id)
        .allow_selection(false)
        .allow_drawer(true)
        .column("Domain", 300.0 * width_ratio, false)
        .column("Provider", 200.0 * width_ratio, false)
        .column("Operations", 250.0 * width_ratio, false);
    
    for (idx, domain_row) in domain_rows.iter().enumerate() {
        let row_for_cells = domain_row.clone();
        let row_for_ops = domain_row.clone();
        let row_for_drawer = domain_row.clone();
        
        table = table.row(move |r| {
            r.cell(&row_for_cells.domain)
             .cell(&row_for_cells.provider_display)
             .widget_cell(move |ui| {
                 render_domain_operations(ui, &row_for_ops, idx);
             })
             .drawer(move |ui| {
                 render_domain_drawer(ui, &row_for_drawer, idx);
             })
        });
    }
    
    egui::ScrollArea::vertical().show(ui, |ui| {
        table.show(ui);
    });
}
```

- [ ] **Step 5: Update load_data to build domain_rows**

In `NsTab::load_data()` method (around line 1150-1222), replace the MaterialSpreadsheet building logic with:

```rust
fn load_data(&mut self) {
    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    {
        match load_ns_config() {
            Ok(config) => {
                self.domain_rows = build_domain_rows(&config);
                self.loaded = true;
            }
            Err(e) => {
                self.load_error = Some(e);
            }
        }
    }

    #[cfg(any(target_os = "android", target_arch = "wasm32"))]
    {
        self.load_error = Some("NS management not supported on this platform".to_string());
    }
}
```

- [ ] **Step 6: Update ui() method to use new render function**

In `NsTab::ui()` method, find the domain spreadsheet rendering section (around lines 1026-1090) and replace with:

```rust
// Domain table with drawer-based records
render_domains_table(&self.domain_rows, ui, vm.as_deref_mut());
```

- [ ] **Step 7: Verify code compiles**

```bash
cd mobile
cargo check --lib
```

Expected output: `Finished 'dev' profile [unoptimized + debuginfo]` with no errors

- [ ] **Step 8: Test manually - verify drawer opens with records**

```bash
cd mobile
cargo run --release
```

Manual checks:
1. Navigate to Domains tab
2. Expand a domain row - verify drawer shows records table
3. Verify empty domains show "No records yet"
4. Verify operations buttons are clickable

- [ ] **Step 9: Commit domains table implementation**

```bash
git add mobile/src/ui_tabs/ns.rs
git commit -m "$(cat <<'EOF'
feat(ui): implement drawer-based domains table

Replace dual-spreadsheet layout with single data_table + nested drawer.
Domains table now shows Domain | Provider | Operations columns.
Expanding drawer reveals nested records table with inline delete.

- Add calculate_width_ratio() for adaptive column widths
- Add render_domain_operations() for Add Record/Nameservers/Delete buttons
- Add render_domain_drawer() with nested records data_table
- Add render_domains_table() as main rendering function
- Update load_data() to build domain_rows from config
- Update ui() to use new render function

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: Domains Tab - Wire Up Action Triggers

**Files:**
- Modify: `mobile/src/ui_tabs/ns.rs` (add process_action_triggers)

**Interfaces:**
- Consumes: temp data from egui (via `ui.data()`)
- Produces: `fn process_action_triggers(&mut self, ui: &mut egui::Ui, vm: Option<&mut ViewModel>)`

- [ ] **Step 1: Implement process_action_triggers method**

Add to `impl NsTab` block (after `ui()` method):

```rust
/// Process action triggers from operations buttons
fn process_action_triggers(
    &mut self,
    ui: &mut egui::Ui,
    mut vm: Option<&mut crate::viewmodel::ViewModel>
) {
    // Process each domain row's action triggers
    for idx in 0..self.domain_rows.len() {
        // Add Record trigger
        let add_record_id = egui::Id::new(format!("add_record_{}", idx));
        if let Some((provider, domain)) = ui.data(|d| d.get_temp::<(String, String)>(add_record_id)) {
            ui.data_mut(|d| d.remove::<(String, String)>(add_record_id));
            
            // Open add record dialog with pre-filled domain
            self.show_add_record_dialog = true;
            self.add_record_name.clear();
            self.add_record_value.clear();
            
            // Store selected domain for dialog
            self.selected_domain = Some((provider, domain));
        }
        
        // View Nameservers trigger
        let view_ns_id = egui::Id::new(format!("view_ns_{}", idx));
        if let Some((provider, domain)) = ui.data(|d| d.get_temp::<(String, String)>(view_ns_id)) {
            ui.data_mut(|d| d.remove::<(String, String)>(view_ns_id));
            
            self.show_nameservers_dialog = true;
            self.ns_dialog_domain = domain.clone();
            self.ns_dialog_loading = true;
            self.load_nameservers_for_dialog(&provider, &domain);
        }
        
        // Delete Domain trigger
        let delete_domain_id = egui::Id::new(format!("delete_domain_{}", idx));
        if let Some((provider, domain)) = ui.data(|d| d.get_temp::<(String, String)>(delete_domain_id)) {
            ui.data_mut(|d| d.remove::<(String, String)>(delete_domain_id));
            
            self.execute_delete_domain(&domain, vm.as_deref_mut());
        }
        
        // Delete Record triggers (check all record indices)
        for rec_idx in 0..100 {  // Max 100 records per domain (reasonable limit)
            let delete_record_id = egui::Id::new(format!("delete_record_{}_{}", idx, rec_idx));
            if let Some((provider, domain, name, record_type)) = ui.data(|d| {
                d.get_temp::<(String, String, String, String)>(delete_record_id)
            }) {
                ui.data_mut(|d| d.remove::<(String, String, String, String)>(delete_record_id));
                
                let record_id = format!("{}:{}", name, record_type);
                
                // ViewModel-based implementation
                if let Some(ref mut vm) = vm {
                    if let Err(e) = vm.delete_dns_record(provider.clone(), domain.clone(), record_id) {
                        self.error_message = format!("Failed to delete DNS record: {}", e);
                        self.show_error_dialog = true;
                    }
                } else {
                    self.error_message = "ViewModel not available".to_string();
                    self.show_error_dialog = true;
                }
                
                break;  // Only process one delete per frame
            }
        }
    }
}
```

- [ ] **Step 2: Restore selected_domain field**

In NsTab struct, add back (we removed it in Task 2, but dialogs still need it):

```rust
// Selected domain for record operations
#[cfg_attr(feature = "serde", serde(skip))]
selected_domain: Option<(String, String)>,
```

And in Default impl:

```rust
selected_domain: None,
```

- [ ] **Step 3: Call process_action_triggers in render_domains_table**

Update `render_domains_table` signature and add call at end:

```rust
fn render_domains_table(
    ns_tab: &mut NsTab,
    ui: &mut egui::Ui,
    vm: Option<&mut crate::viewmodel::ViewModel>
) {
    use egui_material3::data_table;
    
    let available_width = ui.available_width();
    let base_width = 300.0 + 200.0 + 250.0;
    let width_ratio = calculate_width_ratio(available_width, base_width);
    
    let table_id = egui::Id::new("ns_domains_table");
    
    use egui_material3::datatable::DataTableState;
    let state: DataTableState = ui.data_mut(|d| {
        d.get_persisted::<DataTableState>(table_id)
            .unwrap_or_default()
    });
    ui.data_mut(|d| d.insert_persisted(table_id, state));
    
    let mut table = data_table()
        .id(table_id)
        .allow_selection(false)
        .allow_drawer(true)
        .column("Domain", 300.0 * width_ratio, false)
        .column("Provider", 200.0 * width_ratio, false)
        .column("Operations", 250.0 * width_ratio, false);
    
    for (idx, domain_row) in ns_tab.domain_rows.iter().enumerate() {
        let row_for_cells = domain_row.clone();
        let row_for_ops = domain_row.clone();
        let row_for_drawer = domain_row.clone();
        
        table = table.row(move |r| {
            r.cell(&row_for_cells.domain)
             .cell(&row_for_cells.provider_display)
             .widget_cell(move |ui| {
                 render_domain_operations(ui, &row_for_ops, idx);
             })
             .drawer(move |ui| {
                 render_domain_drawer(ui, &row_for_drawer, idx);
             })
        });
    }
    
    egui::ScrollArea::vertical().show(ui, |ui| {
        table.show(ui);
    });
    
    // Process action triggers
    ns_tab.process_action_triggers(ui, vm);
}
```

- [ ] **Step 4: Update ui() call site**

In `ui()` method, change the call to:

```rust
render_domains_table(self, ui, vm.as_deref_mut());
```

- [ ] **Step 5: Verify code compiles**

```bash
cd mobile
cargo check --lib
```

Expected output: `Finished 'dev' profile [unoptimized + debuginfo]` with no errors

- [ ] **Step 6: Test manually - verify operations work**

```bash
cd mobile
cargo run --release
```

Manual checks:
1. Click "Add Record" - verify dialog opens
2. Click "Nameservers" - verify dialog opens
3. Click "Delete" on domain - verify domain is deleted
4. Click 🗑 on record - verify record is deleted

- [ ] **Step 7: Commit action triggers**

```bash
git add mobile/src/ui_tabs/ns.rs
git commit -m "$(cat <<'EOF'
feat(ui): wire up domains tab action triggers

Connect operations buttons to ViewModel handlers.
Add Record, Nameservers, Delete Domain buttons trigger dialogs.
Delete record icon triggers immediate deletion via ViewModel.

- Add process_action_triggers() to handle temp data from buttons
- Restore selected_domain field for dialog coordination
- Update render_domains_table to accept ns_tab for trigger processing
- Wire up Add Record, Nameservers, Delete Domain, Delete Record actions

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 5: Domains Tab - Remove Old Code

**Files:**
- Modify: `mobile/src/ui_tabs/ns.rs` (remove MaterialSpreadsheet fields and methods)

**Interfaces:**
- Consumes: None
- Produces: Cleaned-up NsTab struct without legacy code

- [ ] **Step 1: Remove MaterialSpreadsheet fields from NsTab struct**

In NsTab struct definition, remove:

```rust
// OLD - REMOVE THESE:
domain_spreadsheet: Option<MaterialSpreadsheet>,
record_spreadsheet: Option<MaterialSpreadsheet>,
domain_rows: Vec<[String; 3]>,
record_rows: Vec<[String; 3]>,
row_selection_enabled: bool,
```

- [ ] **Step 2: Remove MaterialSpreadsheet initialization from Default impl**

In `Default for NsTab`, remove:

```rust
// OLD - REMOVE THIS ENTIRE BLOCK:
let domain_spreadsheet = {
    let columns = vec![
        text_column("Domain", 250.0),
        text_column("Provider", 120.0),
        text_column("Records", 80.0),
    ];

    MaterialSpreadsheet::new("ns_domain_spreadsheet", columns)
        .ok()
        .map(|mut s| {
            s.set_striped(true);
            s.set_row_selection_enabled(true);
            s.set_allow_selection(true);
            s
        })
};

let record_spreadsheet = {
    let columns = vec![
        text_column("Name", 150.0),
        text_column("Type", 80.0),
        text_column("Value", 300.0),
    ];

    MaterialSpreadsheet::new("ns_record_spreadsheet", columns)
        .ok()
        .map(|mut s| {
            s.set_striped(true);
            s.set_row_selection_enabled(true);
            s.set_allow_selection(true);
            s
        })
};

// And remove these field initializations:
domain_spreadsheet,
record_spreadsheet,
row_selection_enabled: true,
```

- [ ] **Step 3: Remove load_records method**

Remove the entire `load_records()` method from `impl NsTab` (around lines 1224-1274).

- [ ] **Step 4: Remove MaterialSpreadsheet imports**

At top of file, remove:

```rust
use egui_material3::spreadsheet::{MaterialSpreadsheet, text_column};
```

- [ ] **Step 5: Verify code compiles**

```bash
cd mobile
cargo check --lib
```

Expected output: `Finished 'dev' profile [unoptimized + debuginfo]` with no errors

- [ ] **Step 6: Run all tests to verify no regressions**

```bash
cd mobile
cargo test ns_table_tests --lib -- --nocapture
```

Expected output: `test result: ok. 4 passed; 0 failed`

- [ ] **Step 7: Commit cleanup**

```bash
git add mobile/src/ui_tabs/ns.rs
git commit -m "$(cat <<'EOF'
refactor(ui): remove MaterialSpreadsheet from domains tab

Remove legacy dual-spreadsheet code now replaced by drawer-based layout.

- Remove domain_spreadsheet, record_spreadsheet fields
- Remove domain_rows, record_rows String array fields
- Remove load_records() method (no longer needed)
- Remove MaterialSpreadsheet imports
- Clean up Default impl

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 6: Integration Testing and Documentation

**Files:**
- Create: `docs/testing/ui-refactor-manual-tests.md`
- Modify: None (manual testing only)

**Interfaces:**
- Consumes: Built application binary
- Produces: Test results documentation

- [ ] **Step 1: Create manual testing checklist**

```bash
cat > docs/testing/ui-refactor-manual-tests.md <<'EOF'
# SSH and Domains Tab UI Refactor - Manual Test Checklist

## SSH Tab Testing

### Width Flexibility
- [ ] Resize window from 800px to 1920px - columns scale proportionally
- [ ] Very narrow window (< 500px) - verify text wraps, no horizontal scroll
- [ ] Wide window (> 2000px) - columns don't exceed 2x base width
- [ ] Operations buttons still clickable at all window sizes
- [ ] Text in Status column wraps properly when narrow

### No Regressions
- [ ] Drawer opens/closes correctly
- [ ] Host connection test works
- [ ] Refresh button updates status
- [ ] Delete host removes entry
- [ ] Add host dialog works

## Domains Tab Testing

### Main Table
- [ ] Main table shows domains with correct Provider formatting
- [ ] Cloudflare displays as "Cloudflare"
- [ ] GCP displays as "Google Cloud (email@example.com)"
- [ ] Porkbun displays as "Porkbun"
- [ ] DuckDNS displays as "DuckDNS"

### Drawer Functionality
- [ ] Expand drawer - nested records table appears
- [ ] Empty domain (no records) - shows "No records yet" message
- [ ] Drawer state persists when switching tabs and returning
- [ ] Multiple drawers can be open simultaneously
- [ ] Nested records table scrolls if > 10 records

### Operations
- [ ] Add Record button - opens dialog with domain pre-filled
- [ ] Nameservers button - opens nameserver comparison dialog
- [ ] Delete Domain button - removes domain and updates table
- [ ] Delete record (🗑 icon) - removes record from nested table

### Edge Cases
- [ ] Zero state: No domains → empty table with helpful message
- [ ] Very long domain name (> 50 chars) - verify text wrapping
- [ ] 50+ records in domain - verify smooth scrolling
- [ ] Rapid drawer toggle (10x) - no UI glitches
- [ ] Window resize while drawer open - nested table adjusts

## Performance Testing

### Domains Tab Large Dataset
- [ ] Load 100 domains with 20 records each
- [ ] Table renders in < 200ms (subjective, should feel instant)
- [ ] Drawer expansion is instant (< 50ms)
- [ ] Smooth scrolling (60 FPS, no jank)

## Browser Compatibility (WASM)
- [ ] N/A - Desktop only feature

## Results

Date tested: ____________
Tester: ____________
All tests passed: [ ] Yes [ ] No

Issues found:
1. 
2. 
3. 
EOF
```

- [ ] **Step 2: Build release binary**

```bash
cd mobile
cargo build --release
```

Expected output: `Finished 'release' profile [optimized]`

- [ ] **Step 3: Run SSH tab tests**

```bash
./target/release/dure-desktop
```

Follow "SSH Tab Testing" checklist in `docs/testing/ui-refactor-manual-tests.md`

- [ ] **Step 4: Run Domains tab tests**

Continue with same binary, follow "Domains Tab Testing" checklist

- [ ] **Step 5: Document test results**

Fill in the Results section at bottom of `docs/testing/ui-refactor-manual-tests.md`

- [ ] **Step 6: Run unit tests one final time**

```bash
cd mobile
cargo test ssh_width_tests ns_table_tests --lib -- --nocapture
```

Expected output: `test result: ok. 8 passed; 0 failed`

- [ ] **Step 7: Run clippy to verify no new warnings**

```bash
cd mobile
cargo clippy --lib -- -D warnings
```

Expected output: `Finished 'dev' profile` with 0 warnings

- [ ] **Step 8: Commit test documentation**

```bash
git add docs/testing/ui-refactor-manual-tests.md
git commit -m "$(cat <<'EOF'
docs: add manual testing checklist for UI refactor

Add comprehensive manual test checklist for SSH and Domains tab changes.
Covers width flexibility, drawer functionality, operations, and edge cases.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Plan Self-Review

**Spec Coverage Check:**

1. ✅ SSH Tab width flexibility - Task 1 implements `calculate_width_ratio()` and updates `render_table()`
2. ✅ Domains Tab DomainRowData structure - Task 2 adds struct and `build_domain_rows()`
3. ✅ Domains Tab main table with drawer - Task 3 implements `render_domains_table()`, `render_domain_drawer()`, `render_domain_operations()`
4. ✅ Domains Tab action triggers - Task 4 wires up operations to ViewModel
5. ✅ Domains Tab cleanup - Task 5 removes MaterialSpreadsheet code
6. ✅ Testing strategy - Task 1 has SSH unit tests, Task 2 has domains unit tests, Task 6 has integration tests
7. ✅ Width ratio constraints (0.5..2.0) - Implemented in both `calculate_width_ratio()` functions
8. ✅ Empty state handling - Task 3 includes "No records yet" message in `render_domain_drawer()`
9. ✅ GCP provider formatting - Task 2 includes `build_domain_rows()` with "Google Cloud (email)" format
10. ✅ TDD approach - All tasks follow test-first pattern

**Placeholder Scan:**

- ✅ No "TBD", "TODO", "implement later", "fill in details"
- ✅ All code blocks are complete
- ✅ All file paths are exact
- ✅ All commands have expected output
- ✅ No "similar to Task N" without showing code

**Type Consistency:**

- ✅ `DomainRowData` fields match across all tasks
- ✅ `calculate_width_ratio(f32, f32) -> f32` signature consistent
- ✅ `build_domain_rows(&NsConfig) -> Vec<DomainRowData>` signature consistent
- ✅ Function names match across references (no typos like `clearLayers` vs `clearFullLayers`)

All checks passed. Plan is complete and ready for execution.
