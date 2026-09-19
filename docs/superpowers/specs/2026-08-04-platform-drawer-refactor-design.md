# Platform Tab Drawer Refactor - Design Specification

**Date:** 2026-08-04  
**Status:** Approved  
**Implementation:** New feature branch, direct replacement (no feature flag)

## Problem Statement

The platform tab's table drawer has two critical issues:

1. **Information retrieval/refresh timing:** Data is loaded on startup and refreshed after operation button events, but updates don't propagate well to the UI. Users don't see operation progress or updated states.

2. **Dynamic content display:** The Steps column and drawer contents are dynamic but don't clearly show state changes. Status is displayed as text without visual indicators.

**Goal:** Refactor the drawer to use event-based updates with SVG emoji visual indicators, responsive compact grid layout, and reusable components for future tab improvements.

## Requirements

### Functional Requirements
- **FR1:** Steps column shows emoji progress bar: `✅ → ✅ → ⏳ → ⚪ → ⚪`
- **FR2:** Drawer displays compact grid layout (key-value pairs, 2-3 columns)
- **FR3:** Operation buttons trigger immediate visual feedback (⏳ → ✅/✗)
- **FR4:** SSH actions via dropdown menu (Copy Command | Copy Key | Copy IP)
- **FR5:** Grid auto-reflows responsively (3 col → 2 col → 1 col)
- **FR6:** Event-based updates (no polling, update on ViewModel events)

### Non-Functional Requirements
- **NFR1:** Components reusable across other tabs (ssh, ns, site)
- **NFR2:** SVG emoji with Unicode fallback
- **NFR3:** No breaking changes to ViewModel API
- **NFR4:** Works on all platforms (Desktop, Android, WASM)

## Architecture Overview

### New Module Structure

```
mobile/src/
├── ui_components/          # NEW: Reusable UI components
│   ├── mod.rs              # Module exports
│   ├── status_grid.rs      # Compact grid layout component
│   ├── emoji_progress.rs   # Emoji progress bar (✅→⏳→⚪)
│   ├── action_menu.rs      # Dropdown action menu
│   └── emoji_loader.rs     # SVG emoji asset loader
│
├── ui_tabs/
│   ├── platform.rs         # Refactored to use new components
│   ├── ssh.rs              # (Future: can adopt same components)
│   ├── ns.rs               # (Future: can adopt same components)
│   └── site.rs             # (Future: can adopt same components)
│
└── assets/
    └── emoji/              # NEW: SVG emoji assets
        ├── checkmark.svg   # ✅
        ├── progress.svg    # ⏳
        ├── circle.svg      # ⚪
        ├── cross.svg       # ✗
        ├── email.svg       # 📧
        ├── project.svg     # 📁
        ├── vm.svg          # 💻
        ├── firewall.svg    # 🔥
        ├── key.svg         # 🔑
        └── terminal.svg    # 📋
```

### State Management Changes

Add operation state tracking to `PlatformRow`:

```rust
/// Operation state for visual feedback with timestamps for auto-clear
pub enum OperationState {
    Idle,
    InProgress { 
        operation: String,      // "Updating firewall..."
        started_at: i64,        // Unix timestamp for timeout detection
    },
    Completed { 
        operation: String,      // "firewall"
        completed_at: i64,      // Auto-clear after 3 seconds
    },
    Failed { 
        operation: String,      // "firewall"
        error: String,          // Error message for tooltip
        failed_at: i64,         // Auto-clear after 10 seconds
    },
}

/// Platform row data for data table
#[derive(Clone, Debug)]
struct PlatformRow {
    // ... existing fields ...
    
    // NEW: Track operation state for UI feedback
    operation_state: OperationState,
}
```

## Component Designs

### 1. StatusGrid Component

**Purpose:** Responsive key-value grid that auto-reflows based on available width.

**Location:** `mobile/src/ui_components/status_grid.rs`

**API:**
```rust
pub struct StatusGrid {
    items: Vec<StatusGridItem>,
    min_column_width: f32,  // Default: 200.0
}

pub struct StatusGridItem {
    emoji: SvgEmoji,        // Left icon
    label: String,          // "Email:", "VM:", etc.
    value: String,          // Actual value
    state: Option<ItemState>, // None | InProgress | Success | Error | Warning
}

pub enum ItemState {
    InProgress,  // Shows ⏳ (spinner/hourglass)
    Success,     // Shows ✅ (checkmark, green)
    Error,       // Shows ✗ (cross, red)
    Warning,     // Shows ⚠ (warning triangle, yellow/orange)
}

impl StatusGrid {
    pub fn new() -> Self;
    pub fn with_min_column_width(width: f32) -> Self;
    pub fn add_item(&mut self, emoji: SvgEmoji, label: impl Into<String>, 
                    value: impl Into<String>, state: Option<ItemState>);
    pub fn show(&self, ui: &mut egui::Ui);
}
```

**Usage Example:**
```rust
fn render_drawer_content(ui: &mut egui::Ui, row: &PlatformRow) {
    let mut grid = StatusGrid::new();
    
    // Connection info
    grid.add_item(SvgEmoji::Email, "Email", 
        row.email.as_deref().unwrap_or("—"), None);
    grid.add_item(SvgEmoji::Project, "Project", &row.project_id, None);
    
    // Refresh staleness
    if let Some(last_refresh) = row.last_refresh_time {
        let elapsed = chrono::Utc::now().timestamp() - last_refresh;
        let (time_str, state) = if elapsed < 60 {
            ("just now", None)
        } else if elapsed < 3600 {
            (format!("{} min ago", elapsed / 60), None)
        } else if elapsed < 86400 {
            (format!("{} hours ago", elapsed / 3600), Some(ItemState::Warning))
        } else {
            (format!("{} days ago", elapsed / 86400), Some(ItemState::Warning))
        };
        grid.add_item(SvgEmoji::Clock, "Refreshed", time_str, state);
    }
    
    // VM details
    if let Some(vm_name) = &row.vm_name {
        grid.add_item(SvgEmoji::VM, "VM", vm_name, None);
        grid.add_item(SvgEmoji::Network, "IP", 
            row.vm_external_ip.as_deref().unwrap_or("⚠ No external IP"),
            if row.vm_external_ip.is_none() { 
                Some(ItemState::Warning) 
            } else { 
                None 
            });
        
        // Firewall status (with operation state)
        let (firewall_value, firewall_state) = match &row.operation_state {
            OperationState::InProgress { operation } 
                if operation.contains("firewall") => {
                ("Updating...", Some(ItemState::InProgress))
            }
            OperationState::Failed { operation, error } 
                if operation.contains("firewall") => {
                (error.as_str(), Some(ItemState::Error))
            }
            _ => (row.firewall_status.as_str(), None)
        };
        grid.add_item(SvgEmoji::Firewall, "Firewall", firewall_value, firewall_state);
        
        // SSH status
        grid.add_item(SvgEmoji::Key, "SSH", &row.ssh_status, None);
    } else {
        grid.add_item(SvgEmoji::VM, "VM", "— No VM created", None);
    }
    
    grid.show(ui);
    
    // SSH action menu (if available)
    if row.vm_external_ip.is_some() && row.ssh_private_key.is_some() {
        ui.add_space(8.0);
        render_ssh_actions(ui, row);
    }
}
```

**Responsive Behavior:**
- Calculate available width
- Determine columns: `max(1, min(3, available_width / min_column_width))`
- Use `egui::Grid` with calculated columns
- Items flow left-to-right, top-to-bottom

### 2. EmojiProgressBar Component

**Purpose:** Visual progress indicator showing completion state of sequential steps.

**Location:** `mobile/src/ui_components/emoji_progress.rs`

**API:**
```rust
pub struct EmojiProgressBar {
    steps: Vec<ProgressStep>,
    compact: bool,  // true = inline, false = stacked
}

pub struct ProgressStep {
    label: String,           // "OAuth", "Project", "VM", "Firewall", "SSH"
    state: ProgressState,
}

pub enum ProgressState {
    Completed,   // ✅
    InProgress,  // ⏳
    Pending,     // ⚪
    Failed,      // ✗
}

impl EmojiProgressBar {
    pub fn new() -> Self;
    pub fn add_step(&mut self, label: impl Into<String>, state: ProgressState);
    pub fn from_platform_row(row: &PlatformRow) -> Self;
    pub fn compact(mut self, compact: bool) -> Self;
    pub fn show(&self, ui: &mut egui::Ui) -> egui::Response;
}
```

**Implementation of `from_platform_row`:**
```rust
impl EmojiProgressBar {
    pub fn from_platform_row(row: &PlatformRow) -> Self {
        let mut bar = Self::new();
        
        // Step 1: OAuth
        bar.add_step("OAuth", if row.gcp_connected {
            ProgressState::Completed
        } else {
            ProgressState::Pending
        });
        
        // Step 2: Project
        bar.add_step("Project", if row.project_selected {
            ProgressState::Completed
        } else if row.gcp_connected {
            ProgressState::Pending
        } else {
            ProgressState::Pending
        });
        
        // Step 3: VM
        bar.add_step("VM", if row.vm_created {
            ProgressState::Completed
        } else if row.project_selected {
            // Check if VM operation is in progress
            if matches!(row.operation_state, 
                OperationState::InProgress { operation } 
                if operation.contains("VM") || operation.contains("vm")) {
                ProgressState::InProgress
            } else {
                ProgressState::Pending
            }
        } else {
            ProgressState::Pending
        });
        
        // Step 4: Firewall
        bar.add_step("Firewall", if row.firewall_updated {
            ProgressState::Completed
        } else if row.vm_created {
            // Check if firewall operation is in progress
            if matches!(row.operation_state, 
                OperationState::InProgress { operation } 
                if operation.contains("firewall")) {
                ProgressState::InProgress
            } else {
                ProgressState::Pending
            }
        } else {
            ProgressState::Pending
        });
        
        // Step 5: SSH
        bar.add_step("SSH", if row.ssh_ready {
            ProgressState::Completed
        } else if row.vm_created {
            ProgressState::Pending
        } else {
            ProgressState::Pending
        });
        
        bar
    }
}
```

**Usage in Steps Column:**
```rust
// In table column rendering
.cell_widget(|ui| {
    let progress = EmojiProgressBar::from_platform_row(&row)
        .compact(true);
    progress.show(ui);
})
```

### 3. ActionMenu Component

**Purpose:** Dropdown menu for multiple related actions.

**Location:** `mobile/src/ui_components/action_menu.rs`

**API:**
```rust
pub struct ActionMenu {
    label: String,
    icon: Option<SvgEmoji>,
    actions: Vec<String>,  // Action labels
}

impl ActionMenu {
    pub fn new(label: impl Into<String>) -> Self;
    pub fn with_icon(mut self, icon: SvgEmoji) -> Self;
    pub fn add_action(&mut self, label: impl Into<String>);
    
    /// Show menu and return index of clicked action (if any)
    pub fn show(&mut self, ui: &mut egui::Ui) -> Option<usize>;
}
```

**Usage for SSH Actions:**
```rust
fn render_ssh_actions(ui: &mut egui::Ui, row: &PlatformRow) {
    let Some(external_ip) = &row.vm_external_ip else { return };
    let Some(private_key) = &row.ssh_private_key else { return };
    
    let ssh_command = format!(
        "K=$(mktemp) && cat > $K <<'EOF'\n{}\nEOF\nchmod 600 $K && ssh -i $K root@{} && rm $K",
        private_key.trim(),
        external_ip
    );
    
    let mut menu = ActionMenu::new("📋 SSH")
        .with_icon(SvgEmoji::Terminal);
    
    menu.add_action("Copy SSH Command");
    menu.add_action("Copy Private Key");
    menu.add_action("Copy IP Address");
    
    // Show menu and handle selected action
    if let Some(action_idx) = menu.show(ui) {
        let text_to_copy = match action_idx {
            0 => &ssh_command,
            1 => private_key,
            2 => external_ip,
            _ => return,
        };
        
        // Copy to clipboard (use egui's output for cross-platform support)
        ui.output_mut(|o| o.copied_text = text_to_copy.to_string());
    }
}
```

### 4. EmojiCache / SVG Loader

**Purpose:** Load and cache SVG emoji assets for rendering.

**Location:** `mobile/src/ui_components/emoji_loader.rs`

**API:**
```rust
pub enum SvgEmoji {
    Checkmark,    // ✅
    Progress,     // ⏳
    Circle,       // ⚪
    Cross,        // ✗
    Email,        // 📧
    Project,      // 📁
    VM,           // 💻
    Firewall,     // 🔥
    Key,          // 🔑
    Terminal,     // 📋
    Network,      // 🌐
    Clock,        // 🕐
    Warning,      // ⚠
    // ... more as needed
}

impl SvgEmoji {
    /// Get Unicode fallback character
    pub fn to_unicode(&self) -> &'static str;
    
    /// Get SVG file path
    fn svg_path(&self) -> &'static str;
}

pub struct EmojiCache {
    textures: HashMap<SvgEmoji, egui::TextureHandle>,
    ctx: egui::Context,
}

impl EmojiCache {
    pub fn new(ctx: egui::Context) -> Self;
    
    /// Load SVG from assets, render to texture (call once per emoji)
    pub fn load(&mut self, emoji: SvgEmoji);
    
    /// Load all standard emoji
    pub fn load_all(&mut self);
    
    /// Get texture for rendering
    pub fn get(&self, emoji: SvgEmoji) -> Option<&egui::TextureHandle>;
    
    /// Render emoji at specified size (with fallback)
    pub fn show(&self, ui: &mut egui::Ui, emoji: SvgEmoji, size: f32);
}
```

**Asset Loading Strategy:**
- **Desktop/Android:** Embed SVGs in binary via `include_bytes!()` macro
- **WASM:** Bundle SVGs in assets, load via HTTP on first use
- **Fallback:** Use Unicode emoji if SVG fails to load

**SVG → Texture Pipeline:**
```
SVG bytes → resvg::Tree::from_data() 
         → resvg::render() to RGBA buffer
         → egui::ColorImage::from_rgba_unmultiplied()
         → ctx.load_texture()
         → egui::TextureHandle
```

**Dependency:** Add `resvg = "0.42"` to `Cargo.toml`

## Event-Based Data Flow

### Current Flow (Problem)

```
User clicks "Firewall" button
  → Event sent to ViewModel  
  → Async operation starts in background
  → ??? UI doesn't show progress ???
  → Operation completes (seconds later)
  → Event fired: FirewallUpdated
  → self.loaded = false (triggers full config reload)
  → load_rows() rebuilds entire rows array
  → UI finally updates (flicker, scroll position lost)
```

**Issues:**
- No immediate feedback when button clicked
- Full reload is expensive and disruptive
- User doesn't know if button click registered
- Progress invisible during async operation

### New Flow (Solution)

```
User clicks "Firewall" button
  → Immediately: row.operation_state = InProgress("Updating firewall")
  → UI updates instantly: firewall status shows ⏳ in grid
  → Event sent to ViewModel
  → Async operation runs in background
  → Event fired: FirewallUpdated { project_id, whitelisted_ip }
  → row.operation_state = Completed("firewall")
  → row.firewall_status = "✅ Whitelisted (X.X.X.X)"  
  → row.firewall_updated = true (progress bar advances: ⚪ → ✅)
  → UI updates incrementally (no reload, no flicker)
```

**Benefits:**
- Immediate visual feedback (⏳ appears instantly)
- No full reload (incremental updates only)
- Clear progress indication
- Scroll position preserved
- Lower CPU/memory usage

### Implementation Details

#### 1. Optimistic Update on Button Click

```rust
// In PlatformTab::ui() after button click detection
if let Some(project_id) = ui.data(|d| 
    d.get_temp::<String>(egui::Id::new("platform_action_update_firewall"))
) {
    // NEW: Immediately update operation state (optimistic)
    if let Some(row) = self.rows.iter_mut()
        .find(|r| r.project_id == project_id) 
    {
        row.operation_state = OperationState::InProgress {
            operation: "Updating firewall".to_string()
        };
    }
    
    // Then send command to ViewModel
    self.update_firewall(project_id, vm.as_deref_mut());
    
    ui.data_mut(|d| {
        d.remove::<String>(egui::Id::new("platform_action_update_firewall"))
    });
}
```

Apply same pattern to all operations:
- `platform_action_restart_vm` → "Restarting VM"
- `platform_action_delete_vm` → "Deleting VM"
- `platform_action_add_vm` → "Creating VM"
- `platform_action_scan_vms` → "Scanning VMs"
- `platform_action_refresh` → "Refreshing status"

#### 2. Update State When Event Arrives

```rust
// In PlatformTab::ui() event processing
match event {
    ViewModelEvent::Platform(PlatformEvent::FirewallUpdated { 
        project_id, 
        whitelisted_ip 
    }) => {
        if let Some(row) = self.rows.iter_mut()
            .find(|r| r.project_id == project_id) 
        {
            row.operation_state = OperationState::Completed {
                operation: "firewall".to_string()
            };
            row.firewall_status = format!("✅ Whitelisted ({})", whitelisted_ip);
            row.firewall_updated = true;
            
            // Auto-clear Completed state after 3 seconds
            // (return to Idle so ✅ doesn't stay forever)
            ui.ctx().request_repaint_after(std::time::Duration::from_secs(3));
        }
        // Note: No self.loaded = false! No full reload needed!
    }
    
    ViewModelEvent::Platform(PlatformEvent::VMRestarted { 
        project_id, 
        vm_name 
    }) => {
        if let Some(row) = self.rows.iter_mut()
            .find(|r| r.project_id == project_id) 
        {
            row.operation_state = OperationState::Completed {
                operation: "restart".to_string()
            };
            // Optionally update VM status if event includes it
        }
    }
    
    // ... similar for other events
}
```

#### 3. Add New OperationFailed Event

**ViewModel side** (`mobile/src/viewmodel/platform/events.rs`):
```rust
pub enum PlatformEvent {
    // ... existing events ...
    
    /// Operation failed with error
    OperationFailed {
        project_id: String,
        operation: String,    // "firewall", "restart", etc.
        error: String,
    },
}
```

**UI side** (handle failures):
```rust
ViewModelEvent::Platform(PlatformEvent::OperationFailed { 
    project_id, 
    operation, 
    error 
}) => {
    if let Some(row) = self.rows.iter_mut()
        .find(|r| r.project_id == project_id) 
    {
        row.operation_state = OperationState::Failed {
            operation: operation.clone(),
            error: error.clone(),
        };
        
        // Show error in UI for 10 seconds, then return to Idle
        ui.ctx().request_repaint_after(std::time::Duration::from_secs(10));
    }
}
```

#### 4. Auto-Clear Operation State

After showing Completed/Failed for a few seconds, return to Idle:

```rust
// In PlatformTab::ui(), after rendering
for row in &mut self.rows {
    match &row.operation_state {
        OperationState::Completed { .. } => {
            // Clear after 3 seconds (assume ctx.request_repaint_after was called)
            // This is handled by storing a timestamp and checking elapsed time
            // For simplicity, clear immediately on next frame after timeout
            row.operation_state = OperationState::Idle;
        }
        OperationState::Failed { .. } => {
            // Clear after 10 seconds
            row.operation_state = OperationState::Idle;
        }
        _ => {}
    }
}
```

Better approach: Add timestamp to operation state:
```rust
pub enum OperationState {
    Idle,
    InProgress { operation: String, started_at: i64 },
    Completed { operation: String, completed_at: i64 },
    Failed { operation: String, error: String, failed_at: i64 },
}

// Then in ui():
let now = chrono::Utc::now().timestamp();
for row in &mut self.rows {
    match &row.operation_state {
        OperationState::Completed { completed_at, .. } 
            if now - completed_at > 3 => {
            row.operation_state = OperationState::Idle;
        }
        OperationState::Failed { failed_at, .. } 
            if now - failed_at > 10 => {
            row.operation_state = OperationState::Idle;
        }
        _ => {}
    }
}
```

## Error Handling & Edge Cases

### 1. Operation Failures

**Scenario:** Network error, insufficient permissions, resource not found, etc.

**Handling:**
- ViewModel actor catches error, fires `OperationFailed` event
- UI displays ✗ with error message in grid
- Tooltip shows full error details
- Auto-clear after 10 seconds

**Example:**
```rust
// In StatusGrid rendering when operation_state is Failed
let (value, state, tooltip) = match &row.operation_state {
    OperationState::Failed { operation, error } 
        if operation.contains("firewall") => {
        ("✗ Failed", Some(ItemState::Error), Some(error.clone()))
    }
    _ => (row.firewall_status.as_str(), None, None)
};

let mut item = grid.add_item(SvgEmoji::Firewall, "Firewall", value, state);
if let Some(tooltip_text) = tooltip {
    item = item.on_hover_text(tooltip_text);
}
```

### 2. SVG Loading Failures

**Scenario:** SVG file missing, parse error, platform doesn't support resvg, etc.

**Handling:**
- Fallback to Unicode emoji (✅, ⏳, ⚪, ✗, etc.)
- Log warning on first failure
- Don't retry failed loads (cache failure state)

**Implementation:**
```rust
impl EmojiCache {
    pub fn show(&self, ui: &mut egui::Ui, emoji: SvgEmoji, size: f32) {
        match self.get(emoji) {
            Some(texture) => {
                // Render SVG texture
                ui.image(texture, egui::vec2(size, size));
            }
            None => {
                // Fallback to Unicode emoji
                let unicode = emoji.to_unicode();
                ui.label(egui::RichText::new(unicode).size(size));
            }
        }
    }
}
```

### 3. Stale Data Handling

**Problem:** Cached status becomes stale if user doesn't manually refresh.

**Solution:** Visual staleness indicator in grid
```rust
if let Some(last_refresh) = row.last_refresh_time {
    let elapsed = chrono::Utc::now().timestamp() - last_refresh;
    
    let (time_str, state) = if elapsed < 60 {
        ("just now", None)
    } else if elapsed < 3600 {
        (format!("{} min ago", elapsed / 60), None)
    } else if elapsed < 86400 {
        (format!("{} hours ago", elapsed / 3600), Some(ItemState::Warning))
    } else {
        (format!("{} days ago", elapsed / 86400), Some(ItemState::Warning))
    };
    
    grid.add_item(SvgEmoji::Clock, "Refreshed", time_str, state);
}
```

### 4. Missing Data

**Scenario:** VM has no external IP, SSH key not in keyring, project not selected, etc.

**Handling:**
- Use em dash `—` for truly missing fields (e.g., no email)
- Use warning text for expected-but-missing fields (e.g., "⚠ No external IP")
- Use `ItemState::Warning` to highlight issues

**Example:**
```rust
grid.add_item(
    SvgEmoji::Email,
    "Email",
    row.email.as_deref().unwrap_or("—"),  // Em dash for not connected
    None
);

grid.add_item(
    SvgEmoji::Network,
    "IP",
    row.vm_external_ip.as_deref().unwrap_or("⚠ No external IP"),
    if row.vm_external_ip.is_none() { 
        Some(ItemState::Warning) 
    } else { 
        None 
    }
);

// SSH actions only shown if both IP and key exist
if row.vm_external_ip.is_some() && row.ssh_private_key.is_some() {
    render_ssh_actions(ui, row);
} else if row.vm_external_ip.is_some() && row.ssh_keyring_domain.is_some() {
    ui.colored_label(
        egui::Color32::from_rgb(255, 152, 0),
        "⚠ SSH key not found in keyring"
    );
}
```

### 5. Concurrent Operations

**Problem:** User clicks multiple buttons rapidly (Firewall → Restart → Delete).

**Solution:** Disable all operation buttons when any operation is in progress.

**Implementation:**
```rust
// In button rendering (Operations column)
let operation_in_progress = matches!(
    row.operation_state, 
    OperationState::InProgress { .. }
);

ui.add_enabled_ui(!operation_in_progress, |ui| {
    if ui.add(MaterialButton::outlined("Restart").small()).clicked() {
        // ... trigger restart
    }
    
    if ui.add(MaterialButton::outlined("Firewall").small()).clicked() {
        // ... trigger firewall update
    }
    
    // ... other buttons
});

// Exception: Refresh button is always enabled (safe to spam)
if ui.add(MaterialButton::outlined("Refresh").small()).clicked() {
    // ... trigger refresh
}
```

## Testing Strategy

### Unit Tests

```rust
// ui_components/status_grid.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_calculate_columns() {
        let grid = StatusGrid::new();
        assert_eq!(grid.calculate_columns(700.0), 3);
        assert_eq!(grid.calculate_columns(450.0), 2);
        assert_eq!(grid.calculate_columns(250.0), 1);
    }
    
    #[test]
    fn test_item_state_rendering() {
        // Verify correct emoji/color for each ItemState
    }
}

// ui_components/emoji_progress.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_progress_from_platform_row_all_completed() {
        let row = PlatformRow {
            gcp_connected: true,
            project_selected: true,
            vm_created: true,
            firewall_updated: true,
            ssh_ready: true,
            operation_state: OperationState::Idle,
            // ... other fields
        };
        
        let progress = EmojiProgressBar::from_platform_row(&row);
        assert_eq!(progress.steps.len(), 5);
        assert!(progress.steps.iter().all(|s| 
            matches!(s.state, ProgressState::Completed)
        ));
    }
    
    #[test]
    fn test_progress_with_operation_in_progress() {
        let row = PlatformRow {
            gcp_connected: true,
            project_selected: true,
            vm_created: false,
            firewall_updated: false,
            ssh_ready: false,
            operation_state: OperationState::InProgress {
                operation: "Creating VM".to_string(),
            },
            // ... other fields
        };
        
        let progress = EmojiProgressBar::from_platform_row(&row);
        // VM step should be InProgress
        assert!(matches!(progress.steps[2].state, ProgressState::InProgress));
    }
}

// ui_components/emoji_loader.rs
#[cfg(test)]
mod tests {
    #[test]
    fn test_unicode_fallback() {
        assert_eq!(SvgEmoji::Checkmark.to_unicode(), "✅");
        assert_eq!(SvgEmoji::Progress.to_unicode(), "⏳");
        assert_eq!(SvgEmoji::Circle.to_unicode(), "⚪");
    }
}
```

### Integration Tests

```rust
// Test event flow: button click → optimistic update → event → final state
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[test]
    fn test_firewall_update_flow() {
        let mut tab = PlatformTab::default();
        
        // Setup: Add a platform row
        tab.rows.push(PlatformRow {
            project_id: "test-project".to_string(),
            firewall_updated: false,
            firewall_status: "✗ Not whitelisted".to_string(),
            operation_state: OperationState::Idle,
            // ... other fields
        });
        
        // 1. Simulate button click (optimistic update)
        tab.rows[0].operation_state = OperationState::InProgress {
            operation: "Updating firewall".to_string(),
        };
        assert!(matches!(tab.rows[0].operation_state, 
            OperationState::InProgress { .. }));
        
        // 2. Simulate successful event
        tab.rows[0].operation_state = OperationState::Completed {
            operation: "firewall".to_string(),
        };
        tab.rows[0].firewall_status = "✅ Whitelisted (1.2.3.4)".to_string();
        tab.rows[0].firewall_updated = true;
        
        // 3. Verify final state
        assert_eq!(tab.rows[0].firewall_status, "✅ Whitelisted (1.2.3.4)");
        assert!(tab.rows[0].firewall_updated);
    }
    
    #[test]
    fn test_operation_failure_flow() {
        let mut tab = PlatformTab::default();
        tab.rows.push(PlatformRow::default());
        
        // Simulate failure
        tab.rows[0].operation_state = OperationState::Failed {
            operation: "restart".to_string(),
            error: "VM not found".to_string(),
        };
        
        assert!(matches!(tab.rows[0].operation_state, 
            OperationState::Failed { .. }));
    }
}
```

### Manual Testing Checklist

- [ ] **Responsive layout**
  - [ ] Desktop (>600px): 3 column grid
  - [ ] Tablet (400-600px): 2 column grid
  - [ ] Mobile (<400px): 1 column grid
  
- [ ] **SVG emoji rendering**
  - [ ] Desktop: SVGs load and render correctly
  - [ ] Android: SVGs load and render correctly
  - [ ] WASM: SVGs load and render correctly
  - [ ] Fallback: Unicode emoji shows when SVG unavailable
  
- [ ] **Operation progress indicators**
  - [ ] Firewall: Click → ⏳ → ✅
  - [ ] Restart VM: Click → ⏳ → ✅
  - [ ] Delete VM: Click → ⏳ → (row removed)
  - [ ] Add VM: Click → ⏳ → ✅ (new VM appears)
  - [ ] Scan VMs: Click → ⏳ → ✅ (VMs imported)
  
- [ ] **Operation failures**
  - [ ] Network error: ⏳ → ✗ with tooltip
  - [ ] Permission error: ⏳ → ✗ with tooltip
  - [ ] Auto-clear after 10 seconds
  
- [ ] **SSH dropdown menu**
  - [ ] "Copy SSH Command" copies full bash script
  - [ ] "Copy Private Key" copies key only
  - [ ] "Copy IP Address" copies IP only
  - [ ] Menu disabled when no VM or no key
  
- [ ] **Emoji progress bar (Steps column)**
  - [ ] Shows ⚪⚪⚪⚪⚪ for new platform
  - [ ] Shows ✅✅⚪⚪⚪ after OAuth + Project selected
  - [ ] Shows ✅✅✅⚪⚪ after VM created
  - [ ] Shows ✅✅✅✅⚪ after Firewall updated
  - [ ] Shows ✅✅✅✅✅ after SSH ready
  - [ ] Shows ⏳ on active step during operation
  
- [ ] **Stale data warning**
  - [ ] Shows "Refreshed: X min ago" normally
  - [ ] Shows warning state after 1+ hour
  - [ ] Warning clears after manual refresh
  
- [ ] **Concurrent operations**
  - [ ] Buttons disabled during operation
  - [ ] Refresh button always enabled
  - [ ] Can't click Restart while Firewall updating
  
- [ ] **Missing data handling**
  - [ ] Shows "—" for no email
  - [ ] Shows "⚠ No external IP" for VM without IP
  - [ ] Shows "⚠ SSH key not found" when keyring missing
  - [ ] SSH menu hidden when no IP or no key

## Implementation Phases

### Phase 1: Foundation (Week 1)

**Goal:** Set up infrastructure for new components.

**Tasks:**
- [ ] Create `mobile/src/ui_components/` module
- [ ] Create `mobile/assets/emoji/` directory
- [ ] Add SVG assets (checkmark, progress, circle, cross, etc.)
- [ ] Implement `emoji_loader.rs` with Unicode fallback
- [ ] Add `OperationState` enum to `platform.rs`
- [ ] Add `operation_state` field to `PlatformRow`
- [ ] Update `PlatformRow` initialization to set `Idle`
- [ ] Add dependency: `resvg = "0.42"` to `Cargo.toml`

**Testing:**
- [ ] Verify SVG assets load on all platforms
- [ ] Verify Unicode fallback works
- [ ] Compile without errors

### Phase 2: Components (Week 2)

**Goal:** Implement reusable UI components.

**Tasks:**
- [ ] Implement `StatusGrid` component
  - [ ] Responsive column calculation
  - [ ] Item rendering with emoji + label + value
  - [ ] State indicators (InProgress/Success/Error/Warning)
  - [ ] Unit tests
- [ ] Implement `EmojiProgressBar` component
  - [ ] Step rendering with state emoji
  - [ ] `from_platform_row()` logic
  - [ ] Compact vs. stacked modes
  - [ ] Unit tests
- [ ] Implement `ActionMenu` component
  - [ ] Dropdown menu UI
  - [ ] Action callbacks
  - [ ] Icon support
- [ ] Export components from `ui_components/mod.rs`

**Testing:**
- [ ] Unit tests pass
- [ ] Components compile without errors
- [ ] Manual test in isolated example (optional)

### Phase 3: Platform Tab Integration (Week 3)

**Goal:** Refactor platform tab to use new components.

**Tasks:**
- [ ] Refactor `render_drawer_content()` function
  - [ ] Replace text hierarchy with `StatusGrid`
  - [ ] Add responsive grid layout
  - [ ] Add SSH `ActionMenu`
  - [ ] Handle missing data gracefully
- [ ] Refactor `format_steps()` function
  - [ ] Replace text with `EmojiProgressBar`
  - [ ] Use `from_platform_row()` helper
- [ ] Update table column rendering
  - [ ] Use `.cell_widget()` for emoji progress bar
- [ ] Test on all platforms (Desktop, Android, WASM)

**Testing:**
- [ ] Visual regression: Compare old vs. new layout
- [ ] Responsive behavior on different screen sizes
- [ ] SSH menu works correctly
- [ ] No functional regressions

### Phase 4: Event Flow Refactor (Week 4)

**Goal:** Implement event-based state updates.

**Tasks:**
- [ ] Add optimistic updates for all operation buttons
  - [ ] Firewall
  - [ ] Restart VM
  - [ ] Delete VM
  - [ ] Add VM
  - [ ] Scan VMs
  - [ ] Refresh
- [ ] Update event handlers
  - [ ] `FirewallUpdated` → update row state
  - [ ] `VMRestarted` → update row state
  - [ ] `VMDeleted` → remove row
  - [ ] `VMCreated` → add row
  - [ ] `VMsScanned` → update rows
- [ ] Add `OperationFailed` event support
  - [ ] Update `PlatformEvent` enum
  - [ ] Update ViewModel actor error handling
  - [ ] Update UI event processing
- [ ] Implement auto-clear for Completed/Failed states
  - [ ] Add timestamps to `OperationState`
  - [ ] Clear after timeout in `ui()`
- [ ] Remove `self.loaded = false` full reloads
  - [ ] Replace with incremental row updates

**Testing:**
- [ ] Integration tests for event flow
- [ ] Manual testing: Click each button, verify progress
- [ ] Error injection: Simulate failures, verify ✗ display
- [ ] Performance: Verify no full reloads happening

### Phase 5: Future Expansion (Post-MVP)

**Goal:** Apply components to other tabs.

**Tasks:**
- [ ] SSH tab: Use `StatusGrid` for host details
- [ ] NS tab: Use `EmojiProgressBar` for DNS propagation
- [ ] Site tab: Use `ActionMenu` for deployment actions
- [ ] Extract shared patterns into `ui_components`
- [ ] Documentation: Component usage guide

## Rollout Strategy

**Approach:** Direct replacement on new feature branch.

**Branch:** `feature/platform-drawer-refactor`

**Steps:**
1. Create feature branch from `main`
2. Implement Phases 1-4 on feature branch
3. Manual testing on all platforms
4. Code review
5. Merge to `main` via PR
6. Monitor for issues in production

**No feature flag needed** - this is active development phase, breaking changes acceptable.

**Rollback plan:** Revert merge commit if critical issues found.

## Success Criteria

### Functional
- [ ] Emoji progress bar shows correct state in Steps column
- [ ] Drawer displays responsive grid layout (3→2→1 columns)
- [ ] Operation buttons show immediate progress (⏳ → ✅/✗)
- [ ] SSH dropdown menu copies Command/Key/IP correctly
- [ ] Event-based updates work without full reload
- [ ] All platforms supported (Desktop, Android, WASM)

### Non-Functional
- [ ] No performance regression (no full reloads)
- [ ] Components reusable across tabs
- [ ] Code coverage: >80% for new components
- [ ] No visual regressions (screenshots match design)

### User Experience
- [ ] User sees instant feedback on button clicks
- [ ] Operation progress is clear and unambiguous
- [ ] Layout is responsive and readable on all screen sizes
- [ ] Error messages are helpful and actionable
- [ ] SSH workflow is simpler than before

## Future Enhancements

**Not in scope for this refactor, but enabled by this work:**

1. **Live updates:** WebSocket connection to GCP for real-time VM status
2. **Batch operations:** Select multiple platforms, apply operation to all
3. **Operation history:** Show recent operations in a timeline
4. **Custom emoji:** User-provided SVG emoji for branding
5. **Theme support:** Dark/light mode with different emoji colors
6. **Accessibility:** Screen reader support for emoji states
7. **Animation:** Smooth transitions between states (⏳ → ✅)
8. **Keyboard shortcuts:** Ctrl+R to refresh, Ctrl+C to copy SSH

## References

- **Current Implementation:** `mobile/src/ui_tabs/platform.rs` (lines 699-803, 1068-1450)
- **ViewModel Events:** `mobile/src/viewmodel/platform/events.rs`
- **Data Models:** `mobile/src/config.rs` (CloudPlatformConfig)
- **Material3 Design:** `egui-material3` crate documentation
- **SVG Rendering:** `resvg` crate documentation

## Open Questions

None - design approved.

## Changelog

- **2026-08-04:** Initial design approved by user
