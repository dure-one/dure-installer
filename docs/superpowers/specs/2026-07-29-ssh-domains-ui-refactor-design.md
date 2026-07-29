# SSH and Domains Tab UI Refactor Design

**Date:** 2026-07-29  
**Author:** Claude Sonnet 4.5  
**Status:** Approved  

## Overview

This design addresses two UI improvements in the Dure desktop application:

1. **SSH Tab**: Add width flexibility to match platform tab's adaptive column sizing
2. **Domains Tab**: Redesign from stair-like dual-spreadsheet layout to drawer-based nested tables

Both changes follow the established pattern from `platform.rs` to maintain UI consistency across tabs.

## Problem Statement

### SSH Tab (Current)
The SSH hosts table uses hardcoded column widths (`200.0`, `150.0`, etc.) that don't adapt to window size. When the window is resized, columns either overflow or waste space, unlike the platform tab which scales proportionally.

### Domains Tab (Current)
The nameserver management UI uses two stacked `MaterialSpreadsheet` instances:
- Top spreadsheet: domains list
- Bottom spreadsheet: records for selected domain

This "stair-like" layout is inconsistent with platform/SSH tabs and requires two separate table components to manage related data.

## Goals

1. **SSH Tab**: Make columns width-flexible using adaptive ratio scaling
2. **Domains Tab**: Consolidate to single table with drawer-based record view
3. **Consistency**: Both tabs follow platform tab's UI patterns
4. **TDD**: Test-driven development with comprehensive test coverage

## Non-Goals

- Refactoring platform tab or creating new shared components
- Changing functionality (operations, data flow, state management logic)
- Supporting mobile/WASM (desktop-only feature)

## Design

### Architecture Overview

**Scope:**
Two independent UI refactors in `mobile/src/ui_tabs/`:
1. `ssh.rs` - Add width flexibility to `render_table()` method
2. `ns.rs` - Replace `MaterialSpreadsheet` logic with `data_table()` + drawer

**Files Modified:**
- `mobile/src/ui_tabs/ssh.rs` - Update `render_table()` method
- `mobile/src/ui_tabs/ns.rs` - Replace dual-spreadsheet with drawer-based layout

**Pattern Source:**
Both changes follow `platform.rs` (lines 1059-1249):
- Width ratio calculation: `let width_ratio = available_width / base_width`
- Drawer state: Persisted via `DataTableState` in egui data storage
- Column definition: `.column(name, base_width * width_ratio, resizable)`

**Data Flow (Domains Tab):**
```
Config File (YAML)
    ↓
NsTab.load_data() → domain_rows (Vec<DomainRowData>)
    ↓
Main table renders domains
    ↓
User expands drawer → render_drawer_content()
    ↓
Nested data_table() shows records for that domain
    ↓
User clicks operation → Update temp data → ViewModel → Config
```

**Independence:**
SSH and domains changes are completely independent - no shared code. Can implement/test separately, merge together.

---

### SSH Tab: Width Flexibility

#### Current Implementation
```rust
// mobile/src/ui_tabs/ssh.rs line 966
let mut table = data_table()
    .id(table_id)
    .allow_selection(false)
    .allow_drawer(true)
    .column("Host (Port)", 200.0, false)      // Fixed widths
    .column("Platform", 150.0, false)
    .column("Status", 300.0, false)
    .column("Operations", 350.0, false);
```

#### Proposed Implementation
```rust
fn render_table(&mut self, ui: &mut egui::Ui, vm: Option<&mut crate::viewmodel::ViewModel>) {
    // Calculate width ratio for responsive columns
    let available_width = ui.available_width();
    let base_width = 200.0 + 150.0 + 300.0 + 350.0; // Total of base widths = 1000.0
    let width_ratio = calculate_width_ratio(available_width, base_width);
    
    let table_id = egui::Id::new("ssh_table");
    
    // ... existing drawer state initialization ...
    
    // Build table with scaled widths
    let mut table = data_table()
        .id(table_id)
        .allow_selection(false)
        .allow_drawer(true)
        .column("Host (Port)", 200.0 * width_ratio, false)
        .column("Platform", 150.0 * width_ratio, false)
        .column("Status", 300.0 * width_ratio, false)
        .column("Operations", 350.0 * width_ratio, false);
    
    // ... rest of implementation unchanged ...
}

/// Calculate width ratio with clamping to prevent extreme scaling
fn calculate_width_ratio(available_width: f32, base_width: f32) -> f32 {
    (available_width / base_width).max(0.5).min(2.0)
}
```

#### Column Width Ratios
- **Host (Port)**: 200px base (20%)
- **Platform**: 150px base (15%)
- **Status**: 300px base (30%)
- **Operations**: 350px base (35%) - largest for button wrapping

#### Constraints
- Width ratio clamped to `0.5..2.0` range
- Prevents extreme squishing (< 50% of base) or stretching (> 200% of base)
- Base widths total 1000px (optimal for typical desktop layouts)

---

### Domains Tab: Drawer-Based Redesign

#### Data Structure Changes

**Remove:**
```rust
domain_spreadsheet: Option<MaterialSpreadsheet>
record_spreadsheet: Option<MaterialSpreadsheet>
selected_domain: Option<(String, String)>
```

**Add:**
```rust
#[derive(Clone, Debug)]
struct DomainRowData {
    domain: String,
    provider: String,              // Internal identifier (e.g., "cloudflare", "gcloud:email")
    provider_display: String,      // Display name (e.g., "Cloudflare", "Google Cloud (email)")
    records: Vec<DnsRecord>,       // All records for this domain
}

// In NsTab struct:
domain_rows: Vec<DomainRowData>
```

#### Main Table Structure

**Columns:**
- **Domain** (300px base, 40%)
- **Provider** (200px base, 27%)
- **Operations** (250px base, 33%)

**Implementation:**
```rust
fn render_domains_table(&mut self, ui: &mut egui::Ui, vm: Option<&mut ViewModel>) {
    // Width ratio calculation
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
    
    // Build table
    let mut table = data_table()
        .id(table_id)
        .allow_selection(false)
        .allow_drawer(true)
        .column("Domain", 300.0 * width_ratio, false)
        .column("Provider", 200.0 * width_ratio, false)
        .column("Operations", 250.0 * width_ratio, false);
    
    for (idx, domain_row) in self.domain_rows.iter().enumerate() {
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
    self.process_action_triggers(ui, vm);
}
```

#### Drawer Content (Nested Records Table)

```rust
fn render_domain_drawer(ui: &mut egui::Ui, domain: &DomainRowData, idx: usize) {
    ui.heading(format!("Records for {}", domain.domain));
    ui.add_space(8.0);
    
    if domain.records.is_empty() {
        // Empty state: show message
        ui.label(
            egui::RichText::new("No records yet")
                .color(ui.visuals().weak_text_color())
        );
        ui.add_space(4.0);
    } else {
        // Nested data_table for records
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
                     use egui_material3::MaterialButton;
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

#### Operations Column

**Domain-Level Operations (Main Table):**
```rust
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

**Record-Level Operations (Nested Table):**
- Delete icon (🗑) button per record row
- Inline action using temp data pattern
- Triggers delete via ViewModel → Config update

#### State Management

**Drawer State:**
- Managed by `DataTableState` (auto-persisted by egui)
- Keyed by table ID: `egui::Id::new("ns_domains_table")`
- Persists across tab switches

**Data Loading:**
- `load_data()` builds `domain_rows` from config
- Each `DomainRowData` includes all records upfront (no lazy loading)
- Drawer expansion just renders existing data

**Action Triggers:**
- Operations use temp data pattern (same as platform/SSH tabs)
- `process_action_triggers()` polls temp data and dispatches to ViewModel
- ViewModel events update config → reload data

**Migration Path:**
1. Keep existing `load_data()` logic for config parsing
2. Replace spreadsheet rendering with `render_domains_table()`
3. Remove `load_records()` method (no longer needed)
4. Remove `selected_domain` tracking (drawer state replaces it)

---

## Testing Strategy

### Unit Tests

#### SSH Tab Tests
**File:** `mobile/src/ui_tabs/ssh.rs` test module

```rust
#[cfg(test)]
mod ssh_width_tests {
    use super::*;
    
    #[test]
    fn test_width_ratio_normal_window() {
        // 1000px window with 1000px base = 1.0 ratio
        let ratio = calculate_width_ratio(1000.0, 1000.0);
        assert_eq!(ratio, 1.0);
    }
    
    #[test]
    fn test_width_ratio_narrow_window() {
        // 400px window with 1000px base = 0.4, clamped to 0.5
        let ratio = calculate_width_ratio(400.0, 1000.0);
        assert_eq!(ratio, 0.5);
    }
    
    #[test]
    fn test_width_ratio_wide_window() {
        // 2500px window with 1000px base = 2.5, clamped to 2.0
        let ratio = calculate_width_ratio(2500.0, 1000.0);
        assert_eq!(ratio, 2.0);
    }
    
    #[test]
    fn test_width_ratio_zero_base() {
        // Edge case: zero base width should not panic
        let ratio = calculate_width_ratio(1000.0, 0.0);
        assert_eq!(ratio, 2.0); // f32::INFINITY clamped to 2.0
    }
}
```

#### Domains Tab Tests
**File:** `mobile/src/ui_tabs/ns.rs` test module

```rust
#[cfg(test)]
mod ns_table_tests {
    use super::*;
    use crate::calc::ns::{NsConfig, DnsRecord, RecordType};
    
    #[test]
    fn test_domain_row_data_from_empty_config() {
        let config = NsConfig::default();
        let rows = build_domain_rows(&config);
        assert_eq!(rows.len(), 0);
    }
    
    #[test]
    fn test_domain_with_no_records() {
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
        assert_eq!(rows[0].records.len(), 0);
    }
    
    #[test]
    fn test_domain_with_multiple_records() {
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
    
    #[test]
    fn test_gcp_provider_display_format() {
        let mut config = NsConfig::default();
        
        // Add GCP account
        let oauth = crate::api::gcp::oauth::OAuthResult {
            access_token: "test_token".to_string(),
            refresh_token: Some("refresh".to_string()),
            expires_at: chrono::Utc::now().timestamp() + 3600,
        };
        
        // Simulate GCP domain addition (provider format: "gcloud:email@example.com")
        config.add_domain(
            "gcloud:user@gmail.com".to_string(),
            "example.com".to_string(),
            oauth.access_token.clone()
        ).unwrap();
        
        let rows = build_domain_rows(&config);
        assert_eq!(rows[0].provider, "gcloud:user@gmail.com");
        assert_eq!(rows[0].provider_display, "Google Cloud (user@gmail.com)");
    }
}
```

**Helper Function to Add:**
```rust
/// Build domain rows from config (extracted for testing)
fn build_domain_rows(config: &NsConfig) -> Vec<DomainRowData> {
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
```

### Integration Tests

**Manual Testing Checklist:**

**SSH Tab:**
- [ ] Resize window from 800px to 1920px - columns scale proportionally
- [ ] Very narrow window (< 500px) - verify text wraps, no horizontal scroll
- [ ] Wide window (> 2000px) - columns don't exceed 2x base width
- [ ] Drawer opens/closes correctly (no regression)
- [ ] Operations buttons still clickable at all window sizes
- [ ] Text in Status column wraps properly when narrow

**Domains Tab:**
- [ ] Main table shows domains with correct Provider formatting
- [ ] Expand drawer - nested records table appears
- [ ] Empty domain (no records) - shows "No records yet" message
- [ ] Delete record (🗑 icon) - confirmation dialog appears
- [ ] Add Record button - opens dialog with domain pre-filled
- [ ] Nameservers button - opens nameserver comparison dialog
- [ ] Delete Domain button - removes domain and updates table
- [ ] Drawer state persists when switching tabs and returning
- [ ] Multiple drawers can be open simultaneously
- [ ] Nested records table scrolls if > 10 records

**Edge Cases:**
1. **Zero state**: No domains configured → empty table with "Add Nameserver Provider" prompt
2. **GCP domains**: Provider displays as "Google Cloud (email@example.com)"
3. **Very long domain names**: Verify text wrapping/ellipsis (e.g., "subdomain.example.verylongdomainname.co.uk")
4. **50+ records**: Verify nested table scrolls, no performance issues
5. **Rapid drawer toggle**: Click drawer icon quickly 10x - no UI glitches
6. **Window resize while drawer open**: Nested table adjusts width correctly

### Performance Tests

**Domains Tab (Large Dataset):**
- Load 100 domains with 20 records each (2000 total records)
- Verify table renders in < 200ms
- Verify drawer expansion is instant (< 50ms)
- Verify smooth scrolling (60 FPS)

---

## Implementation Plan

### Phase 1: SSH Tab (Quick Win)
1. Extract `calculate_width_ratio()` helper function
2. Write unit tests for width ratio calculation
3. Update `render_table()` to use width ratios
4. Manual testing at various window sizes
5. Commit: "feat(ui): add width flexibility to SSH tab"

### Phase 2: Domains Tab (Main Refactor)
1. Add `DomainRowData` struct
2. Write `build_domain_rows()` helper + unit tests
3. Write `render_domains_table()` function
4. Write `render_domain_drawer()` with nested table
5. Write `render_domain_operations()` function
6. Update `ui()` method to call new rendering functions
7. Remove old spreadsheet code
8. Manual testing per checklist
9. Commit: "feat(ui): redesign domains tab with drawer-based layout"

### Phase 3: Polish
1. Add loading states (spinner while fetching domains)
2. Add error states (display API errors inline)
3. Accessibility review (keyboard navigation, screen reader labels)
4. Commit: "polish(ui): improve domains tab UX"

---

## Migration & Rollback

### Data Migration
**None required** - both changes are UI-only. Config file format remains unchanged.

### Rollback Plan
If bugs are discovered post-merge:
1. **SSH Tab**: Revert to hardcoded widths (one-line change)
2. **Domains Tab**: Revert to dual-spreadsheet layout (restore deleted code from git history)

Both rollbacks are low-risk since they're isolated UI changes.

---

## Success Metrics

**User Experience:**
- SSH tab columns adapt to window size (no horizontal scroll at any width)
- Domains tab drawer workflow is intuitive (< 2 clicks to delete a record)
- UI consistency across all three tabs (platform, SSH, domains)

**Code Quality:**
- 100% test coverage for width ratio calculations
- 80%+ test coverage for domain row data transformations
- No new clippy warnings
- No regressions in existing functionality

**Performance:**
- No measurable performance degradation (< 5ms rendering difference)
- Smooth 60 FPS scrolling in both tabs

---

## Open Questions

None - all design decisions approved during brainstorming.

---

## Appendix

### Reference Code Locations

**Platform Tab (Reference Implementation):**
- Width ratio calculation: `mobile/src/ui_tabs/platform.rs` lines 1043-1048
- Table builder: lines 1059-1066
- Drawer rendering: lines 1241-1243

**SSH Tab (Current):**
- Table rendering: `mobile/src/ui_tabs/ssh.rs` lines 952-999
- Operations rendering: lines 2689-2727

**Domains Tab (Current):**
- Spreadsheet initialization: `mobile/src/ui_tabs/ns.rs` lines 146-178
- Spreadsheet rendering: lines 1026-1029
- Load records: lines 1224-1274

### Related Issues

None yet - this is the initial design.

---

## Glossary

- **Width Ratio**: Multiplier applied to base column widths to scale proportionally with window size
- **Drawer**: Expandable row content that shows additional details when toggled
- **Nested Table**: Secondary data_table rendered inside a drawer
- **Temp Data**: egui's temporary storage mechanism for inter-frame communication
- **MaterialSpreadsheet**: egui-material3 component (being replaced with data_table for consistency)
