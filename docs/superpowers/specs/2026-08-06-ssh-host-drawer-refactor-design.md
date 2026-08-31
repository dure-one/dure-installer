# SSH Host Tab Drawer Refactor - Design Specification

**Date:** 2026-08-06  
**Status:** Approved  
**Implementation:** New feature branch, direct replacement (no feature flag)

## Problem Statement

The SSH host tab's drawer has two critical usability issues:

1. **Flat information display:** All service information (Linux status, Docker containers, Dure-WSS config) is shown in a single vertical list, making it hard to scan and navigate.

2. **No visual progress indicators:** Unlike the platform tab (which uses emoji progress bars), the SSH tab shows plain text status that doesn't clearly communicate the Linux→Docker→Dure setup progression.

3. **Limited command output visibility:** Users can't easily see detailed system information, Docker container status, or network port bindings without running separate SSH commands manually.

**Goal:** Refactor the SSH drawer to use a tab-based layout (Linux, Docker, Dure) with terminal-style output, add emoji progress indicators to the Status column, and provide per-tab refresh and install actions.

## Requirements

### Functional Requirements
- **FR1:** Status column shows emoji progress bar: `✅ → ✅ → ⚪` (Linux→Docker→Dure)
- **FR2:** Drawer displays 3 compact horizontal tabs: Linux, Docker, Dure
- **FR3:** Each tab shows terminal-style command output (monospace, scrollable)
- **FR4:** Linux tab: System info script output + Refresh button + Install Docker button
- **FR5:** Docker tab: `docker ps -a` output + Refresh button + Install Dure button (placeholder)
- **FR6:** Dure tab: `ss -nltup` output + Refresh button
- **FR7:** Tabs auto-enable based on service availability (Docker/Dure tabs disabled until installed)
- **FR8:** Auto-refresh on drawer open (show stale data, refresh in background)
- **FR9:** Per-tab refresh (clicking Refresh only refreshes that tab)
- **FR10:** Button loading states (Refresh → Refreshing..., Install → Installing...)

### Non-Functional Requirements
- **NFR1:** Reuse existing components from platform refactor (EmojiProgressBar, StatusGrid pattern)
- **NFR2:** Terminal output with monospace font and proper scrolling
- **NFR3:** Raw error messages (no silent failures, show actual stderr)
- **NFR4:** Works on all platforms (Desktop Linux, macOS, Windows)
- **NFR5:** No breaking changes to ViewModel API

## Architecture Overview

### Approach: Pragmatic Hybrid

**Philosophy:** Reuse existing platform components where appropriate, add lightweight new components for terminal output, avoid over-engineering.

**Key decisions:**
- **Tab UI:** Use egui's built-in widgets (no custom TabContainer component)
- **Terminal output:** New lightweight `TerminalOutput` component
- **Progress indicators:** Adapt existing `EmojiProgressBar` component
- **State management:** Simple boolean flags (not full `OperationState` enum like platform)
- **Event flow:** Keep existing polling-based refresh (no event-driven architecture for now)

### Module Structure

```
mobile/src/
├── ui_components/          # Reusable UI components
│   ├── mod.rs              
│   ├── terminal_output.rs  # NEW: Terminal output display
│   ├── emoji_progress.rs   # MODIFIED: Add from_ssh_row() method
│   ├── status_grid.rs      # Existing (not used in SSH drawer)
│   └── action_menu.rs      # Existing (not used in SSH drawer)
│
├── ui_tabs/
│   └── ssh.rs              # MODIFIED: Refactored drawer + Status column
│
└── calc/
    └── ssh.rs              # Existing (no changes needed)
```

## Component Designs

### 1. TerminalOutput Component (New)

**Purpose:** Display command output in a monospace scrollable area with optional loading state.

**Location:** `mobile/src/ui_components/terminal_output.rs`

**API:**
```rust
pub struct TerminalOutput {
    content: String,
    max_height: f32,        // Default: 300.0
    loading: bool,
    error_mode: bool,       // Red text for stderr
}

impl TerminalOutput {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            max_height: 300.0,
            loading: false,
            error_mode: false,
        }
    }
    
    pub fn with_max_height(mut self, height: f32) -> Self {
        self.max_height = height;
        self
    }
    
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }
    
    pub fn error(mut self, is_error: bool) -> Self {
        self.error_mode = is_error;
        self
    }
    
    pub fn show(&self, ui: &mut egui::Ui) {
        use egui::ScrollArea;
        
        ScrollArea::vertical()
            .max_height(self.max_height)
            .show(ui, |ui| {
                let text_color = if self.error_mode {
                    egui::Color32::from_rgb(220, 50, 50) // Red for errors
                } else {
                    ui.visuals().text_color()
                };
                
                ui.colored_label(
                    text_color,
                    egui::RichText::new(&self.content).monospace()
                );
                
                if self.loading {
                    ui.add_space(8.0);
                    ui.spinner();
                }
            });
    }
}
```

**Usage Example:**
```rust
// Normal output
TerminalOutput::new(linux_output)
    .with_max_height(250.0)
    .show(ui);

// Error output
TerminalOutput::new(error_msg)
    .error(true)
    .show(ui);
```

### 2. EmojiProgressBar Extension (Modified)

**Purpose:** Add SSH-specific progress logic to existing component.

**Modification:** Add `from_ssh_row()` method to `mobile/src/ui_components/emoji_progress.rs`

**New Method:**
```rust
impl EmojiProgressBar {
    /// Create progress bar from SSH row data
    /// Shows: Linux → Docker → Dure progression
    pub fn from_ssh_row(row: &SshRowData) -> Self {
        let mut bar = Self::new();
        
        // Step 1: Linux detected
        bar.add_step("Linux", if row.linux_detected {
            ProgressState::Completed
        } else {
            ProgressState::Pending
        });
        
        // Step 2: Docker installed
        bar.add_step("Docker", if row.docker_enabled {
            ProgressState::Completed
        } else if row.linux_detected {
            ProgressState::Pending  // Available to install
        } else {
            ProgressState::Pending  // Need Linux first
        });
        
        // Step 3: Dure running
        bar.add_step("Dure", if row.dure_wss_enabled {
            ProgressState::Completed
        } else if row.docker_enabled {
            ProgressState::Pending  // Available to install
        } else {
            ProgressState::Pending  // Need Docker first
        });
        
        bar
    }
}
```

**Usage in Status Column:**
```rust
.cell_widget(|ui| {
    let progress = EmojiProgressBar::from_ssh_row(&row)
        .compact(true);
    progress.show(ui);
})
```

### 3. Tab UI (Built-in egui widgets)

**Implementation:** Use egui's selectable labels, no custom component.

**Tab Selector:**
```rust
fn render_tab_selector(ui: &mut egui::Ui, selected_tab: &mut usize, row: &SshRowData) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        
        // Linux tab (always enabled)
        if ui.selectable_label(*selected_tab == 0, "Linux").clicked() {
            *selected_tab = 0;
        }
        
        // Docker tab (enabled if Docker installed)
        ui.add_enabled_ui(row.docker_enabled, |ui| {
            let response = ui.selectable_label(*selected_tab == 1, "Docker");
            if !row.docker_enabled {
                response.on_hover_text("Install Docker first from Linux tab");
            } else if response.clicked() {
                *selected_tab = 1;
            }
        });
        
        // Dure tab (enabled if Dure running)
        ui.add_enabled_ui(row.dure_wss_enabled, |ui| {
            let response = ui.selectable_label(*selected_tab == 2, "Dure");
            if !row.dure_wss_enabled {
                response.on_hover_text("Install Dure first from Docker tab");
            } else if response.clicked() {
                *selected_tab = 2;
            }
        });
    });
}
```

## Data Model Changes

### Extended SshRowData

Add fields to store command outputs and operation states:

```rust
#[derive(Clone, Debug, Default)]
struct SshRowData {
    // ... existing fields ...
    
    // NEW: Command outputs (cached)
    linux_output: Option<String>,           // System info script output
    linux_output_error: Option<String>,     // Error if command failed
    docker_output: Option<String>,          // docker ps -a output
    docker_output_error: Option<String>,    // Error if command failed
    dure_output: Option<String>,            // ss -nltup output
    dure_output_error: Option<String>,      // Error if command failed
    
    // NEW: Per-tab operation states
    linux_refreshing: bool,
    docker_refreshing: bool,
    dure_refreshing: bool,
    docker_installing: bool,                // Docker daemon install in progress
    dure_installing: bool,                  // xmpp-proxy-stack install in progress
    
    // NEW: Last refresh timestamps (for staleness indicators)
    linux_last_refresh: Option<i64>,        // Unix timestamp
    docker_last_refresh: Option<i64>,
    dure_last_refresh: Option<i64>,
}
```

### System Info Script

The Linux tab executes this bash script and displays the output:

```bash
#!/bin/bash
clear
echo "=============================="
echo "      SYSTEM INFORMATION      "
echo "=============================="

echo -e "\n[+] IP Address:"
ip -br address || hostname -I

echo -e "\n[+] IP(EXT) Address:"
curl -s icanhazip.com

echo -e "\n[+] CPU Usage:"
top -bn1 | grep "Cpu(s)"

echo -e "\n[+] Memory and Swap Usage:"
free -h

echo -e "\n[+] Disk Space Usage:"
df -h /

echo -e "\n[+] Top 5 Memory-Consuming Processes:"
ps -eo pid,comm,%mem,%cpu --sort=-%mem | head -n 6

echo "=============================="
```

### Service Detection Logic

**Docker detection:**
```bash
command -v docker &> /dev/null && docker --version
```
If exit code is 0, set `docker_enabled = true`.

**Dure detection:**
```bash
docker ps -a | grep xmpp-proxy-stack | wc -l
```
If count > 0, set `dure_wss_enabled = true`.

## UI Layout

### Drawer Structure

Each drawer contains:
1. **Compact horizontal tabs** at top (~24px height)
2. **Tab content area** below (terminal output + buttons)
3. **Responsive to drawer width**

### Tab 1: Linux (Always Enabled)

**Layout:**
```
┌─────────────────────────────────┐
│ [Linux] Docker  Dure            │ ← Tabs (compact)
├─────────────────────────────────┤
│ ╔═══════════════════════════╗  │
│ ║ ============================║ │
│ ║ SYSTEM INFORMATION         ║ │
│ ║ ============================║ │
│ ║                            ║ │ ← System info
│ ║ [+] IP Address:            ║ │   (scrollable,
│ ║ eth0  192.168.1.100        ║ │    monospace)
│ ║ ...                        ║ │
│ ╚═══════════════════════════╝  │
│                                 │
│ [Refresh] [Install Docker]      │ ← Actions
└─────────────────────────────────┘
```

**Content:**
- Terminal output from system info script
- If `linux_output.is_none()`: "Click Refresh to load"
- If `linux_output_error.is_some()`: Show error in red
- Staleness warning if `linux_last_refresh` > 1 hour

**Actions:**
- **Refresh** button (always available)
  - State: "Refresh" | "Refreshing..." (disabled)
  - On click: Set `linux_refreshing = true`, execute script
- **Install Docker** button (only if `!docker_enabled`)
  - State: "Install Docker" | "Installing..." (disabled)
  - On click: Set `docker_installing = true`, execute install script
  - Disappears when `docker_enabled = true`

### Tab 2: Docker (Enabled if docker_enabled)

**Layout:**
```
┌─────────────────────────────────┐
│ Linux [Docker] Dure             │
├─────────────────────────────────┤
│ ╔═══════════════════════════╗  │
│ ║ CONTAINER ID  IMAGE       ║ │
│ ║ 3f8a9b2c      nginx:latest║ │ ← docker ps -a
│ ║ 7d4e1a5f      redis:alpine║ │   (scrollable,
│ ║ ...                        ║ │    monospace)
│ ╚═══════════════════════════╝  │
│                                 │
│ [Refresh] [Install Dure]        │
└─────────────────────────────────┘
```

**Content:**
- Terminal output from `docker ps -a`
- If `docker_output.is_none()`: "Click Refresh to load"
- If `docker_output_error.is_some()`: Show error in red

**Actions:**
- **Refresh** button
  - Execute: `docker ps -a`
- **Install Dure** button (only if `!dure_wss_enabled`)
  - **Placeholder for now** - shows "Not yet implemented"
  - Future: Clone xmpp-proxy-stack, run docker-compose

**Disabled state:**
- Tab grayed out if `!docker_enabled`
- Hover tooltip: "Install Docker first from Linux tab"

### Tab 3: Dure (Enabled if dure_wss_enabled)

**Layout:**
```
┌─────────────────────────────────┐
│ Linux  Docker [Dure]            │
├─────────────────────────────────┤
│ ╔═══════════════════════════╗  │
│ ║ Netid State  Local Address ║ │
│ ║ tcp   LISTEN 0.0.0.0:80    ║ │ ← ss -nltup
│ ║ tcp   LISTEN 0.0.0.0:443   ║ │   (scrollable,
│ ║ ...                        ║ │    monospace)
│ ╚═══════════════════════════╝  │
│                                 │
│ [Refresh]                       │
└─────────────────────────────────┘
```

**Content:**
- Terminal output from `ss -nltup` (full, unfiltered)
- If `dure_output.is_none()`: "Click Refresh to load"

**Actions:**
- **Refresh** button
  - Execute: `ss -nltup`

**Disabled state:**
- Tab grayed out if `!dure_wss_enabled`
- Hover tooltip: "Install Dure first from Docker tab"

### Status Column (Table)

**Before:** Text like "Connected" or "Offline"

**After:** Emoji progress bar showing Linux→Docker→Dure status

```rust
// In table column definition
.column("Status", 300.0 * width_ratio, false)

// In row rendering
.cell_widget(|ui| {
    let progress = EmojiProgressBar::from_ssh_row(&row)
        .compact(true);
    progress.show(ui);
})
```

**Visual states:**
- ⚪⚪⚪ = None detected
- ✅⚪⚪ = Linux only
- ✅✅⚪ = Linux + Docker
- ✅✅✅ = Linux + Docker + Dure

## State Management & Refresh Flow

### Per-Tab Refresh Flow

When user clicks a tab's Refresh button:

1. **Optimistic update:** Set `linux_refreshing = true`
2. **Button state:** "Refresh" → "Refreshing..." (disabled)
3. **Keep old content visible:** Don't clear `linux_output`
4. **Send SSH command:** Execute via `calc::ssh::run_command()`
5. **On success:**
   - Store output in `linux_output`
   - Set `linux_last_refresh = current_timestamp()`
   - Clear `linux_refreshing = false`
6. **On failure:**
   - Store error in `linux_output_error`
   - Clear `linux_refreshing = false`
7. **Button returns:** "Refreshing..." → "Refresh" (enabled)

### Initial Load (Drawer Open)

When drawer opens:

1. **Check for cached data:**
   - If `linux_output.is_some()` → Show stale data immediately
   - If `linux_output.is_none()` → Show "Click Refresh to load"

2. **Auto-refresh in background:**
   - Trigger refresh for **currently selected tab only**
   - Don't refresh all tabs (saves SSH commands)
   - User can manually refresh other tabs

3. **Staleness indicator:**
   - If `linux_last_refresh` is > 1 hour old, show warning:
   ```rust
   if let Some(last_refresh) = row.linux_last_refresh {
       let elapsed_secs = current_time() - last_refresh;
       
       if elapsed_secs > 3600 {
           ui.colored_label(
               egui::Color32::from_rgb(255, 152, 0),
               format!("⚠ Data from {} ago", format_duration(elapsed_secs))
           );
       }
   }
   ```

### Docker Install Flow

1. User clicks "Install Docker" on Linux tab
2. Set `docker_installing = true` (button → "Installing...")
3. Send SSH command:
   ```bash
   curl -fsSL https://get.docker.com | sh
   ```
4. On success:
   - Set `docker_enabled = true`
   - Clear `docker_installing = false`
   - **Show notification:** "Docker installed. May require SSH reconnect or system reboot."
   - Docker tab becomes enabled
5. On failure:
   - Store error in `linux_output_error`
   - Clear `docker_installing = false`
   - Show error in terminal output

### Dure Install Flow (Placeholder)

1. User clicks "Install Dure" on Docker tab
2. Set `dure_installing = true`
3. **For now:** Show "Not yet implemented" message
4. **Future:** Clone xmpp-proxy-stack repo, run docker-compose
5. Detection: Run `docker ps -a | grep xmpp-proxy-stack | wc -l`
6. If count > 0: Set `dure_wss_enabled = true`, enable Dure tab

### Operation State Tracking (Simplified)

Unlike platform tab's `OperationState` enum, use simple boolean flags:

```rust
// Platform approach (complex, not used here):
enum OperationState {
    Idle,
    InProgress { operation: String, started_at: i64 },
    Completed { operation: String, completed_at: i64 },
    Failed { operation: String, error: String, failed_at: i64 },
}

// SSH approach (simple):
struct SshRowData {
    linux_refreshing: bool,     // Show spinner on Refresh button
    docker_refreshing: bool,
    dure_refreshing: bool,
    docker_installing: bool,    // Show spinner on Install Docker button
    dure_installing: bool,
}
```

**Rationale:** SSH operations are simpler. No need for timestamps, auto-clear, or complex state machines. Just show spinner while running, return to normal when done.

## Integration Points

### Files to Modify

**1. Drawer Rendering**

**File:** `mobile/src/ui_tabs/ssh.rs:2595-2690`

**Current:** `render_drawer_content()` shows flat list

**New:** Tab-based structure

```rust
fn render_drawer_content(ui: &mut egui::Ui, row: &SshRowData, idx: usize) {
    ui.add_space(8.0);
    
    // Get selected tab from temp data (per-host persistence)
    let mut selected_tab = ui.data(|d| 
        d.get_temp::<usize>(egui::Id::new(format!("ssh_tab_{}", idx)))
            .unwrap_or(0)
    );
    
    // Tab selector
    render_tab_selector(ui, &mut selected_tab, row);
    
    ui.separator();
    ui.add_space(8.0);
    
    // Tab content
    match selected_tab {
        0 => render_linux_tab(ui, row, idx),
        1 => render_docker_tab(ui, row, idx),
        2 => render_dure_tab(ui, row, idx),
        _ => {}
    }
    
    // Save selected tab
    ui.data_mut(|d| d.insert_temp(
        egui::Id::new(format!("ssh_tab_{}", idx)), 
        selected_tab
    ));
}
```

**2. Status Column**

**File:** `mobile/src/ui_tabs/ssh.rs:988-1024` (in `render_table`)

**Current:** Text-based status

**New:** Emoji progress bar

```rust
.column("Status", 300.0 * width_ratio, false)

// In row loop:
.cell_widget(|ui| {
    let progress = EmojiProgressBar::from_ssh_row(&row)
        .compact(true);
    progress.show(ui);
})
```

### New Functions to Add

**Tab Rendering:**
```rust
fn render_tab_selector(ui: &mut egui::Ui, selected_tab: &mut usize, row: &SshRowData);
fn render_linux_tab(ui: &mut egui::Ui, row: &SshRowData, idx: usize);
fn render_docker_tab(ui: &mut egui::Ui, row: &SshRowData, idx: usize);
fn render_dure_tab(ui: &mut egui::Ui, row: &SshRowData, idx: usize);
```

**Action Handlers:**
```rust
fn handle_linux_refresh(ui: &mut egui::Ui, row: &SshRowData, idx: usize);
fn handle_docker_refresh(ui: &mut egui::Ui, row: &SshRowData, idx: usize);
fn handle_dure_refresh(ui: &mut egui::Ui, row: &SshRowData, idx: usize);
fn handle_docker_install(ui: &mut egui::Ui, row: &SshRowData, idx: usize);
fn handle_dure_install(ui: &mut egui::Ui, row: &SshRowData, idx: usize);
```

### ViewModel Integration

**No changes needed** - reuse existing SSH command execution:

```rust
// Existing function in calc::ssh
pub fn run_command(
    host: &str,
    port: u16,
    command: &str,
    auth: SshAuth,
) -> Result<String, String>
```

Call from UI layer when buttons clicked.

## Error Handling

### Error Display Strategy

**Principle:** Always show actual command output (stdout + stderr). No silent failures, no generic messages.

### Error Scenarios

**1. SSH Connection Failure**

**Display:**
```
✗ SSH connection failed: Connection timed out
```

**Implementation:**
```rust
TerminalOutput::new(error_msg)
    .error(true)  // Red text
    .show(ui);
```

**2. Command Not Found**

**Display:**
```
bash: docker: command not found
```

**Action:** Show raw stderr, keep tab enabled, user can troubleshoot.

**3. Permission Denied**

**Display:**
```
docker: permission denied while trying to connect to the Docker daemon socket
```

**Action:** Show error, don't retry, user fixes permissions and refreshes.

**4. Partial Command Failure**

**Display:**
```
[+] IP Address:
eth0  192.168.1.100

[+] IP(EXT) Address:
curl: (28) Connection timed out

[+] CPU Usage:
Cpu(s): 5.2%us, 2.1%sy...
```

**Action:** Show full output including failed parts.

### Staleness Warnings

**When data is > 1 hour old:**

```rust
if let Some(last_refresh) = row.linux_last_refresh {
    let elapsed_secs = chrono::Utc::now().timestamp() - last_refresh;
    
    if elapsed_secs > 3600 {
        ui.colored_label(
            egui::Color32::from_rgb(255, 152, 0),
            format!("⚠ Data from {} ago", format_duration(elapsed_secs))
        );
    }
}
```

### Install Failures

**Docker install fails:**

**Display:**
```
✗ Docker installation failed:
E: Unable to locate package docker-ce

Hint: Check your apt sources
```

**Action:** Button returns to "Install Docker", user can retry.

### Tab Disabled States

**Docker tab when Docker not installed:**

```rust
ui.add_enabled_ui(row.docker_enabled, |ui| {
    let response = ui.selectable_label(selected_tab == 1, "Docker");
    if !row.docker_enabled {
        response.on_hover_text("Install Docker first from Linux tab");
    }
});
```

## Testing Strategy

### Unit Tests

**1. TerminalOutput Component**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_terminal_output_creation() {
        let output = TerminalOutput::new("test");
        assert_eq!(output.content, "test");
        assert_eq!(output.max_height, 300.0);
        assert!(!output.loading);
        assert!(!output.error_mode);
    }
    
    #[test]
    fn test_terminal_output_builder() {
        let output = TerminalOutput::new("test")
            .with_max_height(200.0)
            .loading(true)
            .error(true);
        
        assert_eq!(output.max_height, 200.0);
        assert!(output.loading);
        assert!(output.error_mode);
    }
}
```

**2. EmojiProgressBar SSH Integration**

```rust
#[cfg(test)]
mod ssh_tests {
    use super::*;
    
    #[test]
    fn test_from_ssh_row_all_enabled() {
        let row = SshRowData {
            linux_detected: true,
            docker_enabled: true,
            dure_wss_enabled: true,
            ..Default::default()
        };
        
        let progress = EmojiProgressBar::from_ssh_row(&row);
        assert_eq!(progress.steps.len(), 3);
        assert!(progress.steps.iter().all(|s| 
            matches!(s.state, ProgressState::Completed)
        ));
    }
    
    #[test]
    fn test_from_ssh_row_partial() {
        let row = SshRowData {
            linux_detected: true,
            docker_enabled: true,
            dure_wss_enabled: false,
            ..Default::default()
        };
        
        let progress = EmojiProgressBar::from_ssh_row(&row);
        assert!(matches!(progress.steps[0].state, ProgressState::Completed));
        assert!(matches!(progress.steps[1].state, ProgressState::Completed));
        assert!(matches!(progress.steps[2].state, ProgressState::Pending));
    }
}
```

### Manual Testing Checklist

**Drawer Tabs:**
- [ ] Linux tab shows system info output
- [ ] Docker tab disabled when Docker not installed
- [ ] Docker tab enabled after Docker install
- [ ] Dure tab disabled when xmpp-proxy-stack not running
- [ ] Tab selection persists within session (per host)
- [ ] Tabs have compact height

**Refresh Buttons:**
- [ ] Linux refresh: "Refresh" → "Refreshing..." → "Refresh"
- [ ] Old content stays visible during refresh
- [ ] New content replaces old after success
- [ ] Error shown in red monospace text
- [ ] Each tab refreshes independently

**Install Buttons:**
- [ ] "Install Docker" appears on Linux tab when needed
- [ ] Button → "Installing..." during install
- [ ] Button disappears after success
- [ ] Docker tab becomes enabled
- [ ] "Install Dure" button on Docker tab (placeholder message)

**Status Column:**
- [ ] Shows ⚪⚪⚪ for fresh host
- [ ] Shows ✅⚪⚪ after Linux detected
- [ ] Shows ✅✅⚪ after Docker installed
- [ ] Shows ✅✅✅ after Dure detected
- [ ] Compact horizontal layout

**Error Handling:**
- [ ] SSH timeout shows connection error
- [ ] Command not found shows stderr
- [ ] Permission denied shows real error
- [ ] Partial output shows mixed success/failure
- [ ] Staleness warning after 1+ hour

**Edge Cases:**
- [ ] Empty output (no error, just blank)
- [ ] Very long output scrolls properly
- [ ] Unicode/emoji in command output
- [ ] Multiple rapid refresh clicks
- [ ] Drawer close during refresh

## Implementation Phases

### Phase 1: Foundation (Days 1-2)

**Tasks:**
- [ ] Add new fields to `SshRowData` (outputs, errors, flags, timestamps)
- [ ] Create `TerminalOutput` component
- [ ] Add unit tests for `TerminalOutput`
- [ ] Verify compilation

**Commit:** `feat(ssh): add data model for tab-based drawer`

### Phase 2: EmojiProgressBar Integration (Day 2)

**Tasks:**
- [ ] Add `from_ssh_row()` method to `EmojiProgressBar`
- [ ] Add unit tests
- [ ] Update table Status column to use emoji progress
- [ ] Manual test: Status column shows progress

**Commit:** `feat(ssh): add emoji progress to Status column`

### Phase 3: Tab UI Structure (Days 3-4)

**Tasks:**
- [ ] Refactor `render_drawer_content()` with tab selector
- [ ] Create stub tab render functions
- [ ] Implement tab enable/disable logic
- [ ] Add tab selection persistence
- [ ] Manual test: Tabs work, disabled tabs show tooltips

**Commit:** `feat(ssh): add tab-based drawer UI structure`

### Phase 4: Linux Tab Implementation (Day 4)

**Tasks:**
- [ ] Implement `render_linux_tab()`
- [ ] Add Refresh and Install Docker buttons
- [ ] Implement action handlers
- [ ] Wire up in main `ui()` loop
- [ ] Manual test: Linux tab works

**Commit:** `feat(ssh): implement Linux tab with system info`

### Phase 5: Docker Tab Implementation (Day 5)

**Tasks:**
- [ ] Implement `render_docker_tab()`
- [ ] Add Refresh and Install Dure buttons
- [ ] Implement handlers
- [ ] Wire up actions
- [ ] Manual test: Docker tab works

**Commit:** `feat(ssh): implement Docker tab with container list`

### Phase 6: Dure Tab Implementation (Day 5)

**Tasks:**
- [ ] Implement `render_dure_tab()`
- [ ] Add Refresh button
- [ ] Implement handler
- [ ] Wire up action
- [ ] Manual test: Dure tab works

**Commit:** `feat(ssh): implement Dure tab with port status`

### Phase 7: Command Execution Integration (Days 6-7)

**Tasks:**
- [ ] Implement system info script execution
- [ ] Implement Docker detection
- [ ] Implement docker ps execution
- [ ] Implement Dure detection
- [ ] Implement ss execution
- [ ] Implement Docker install (placeholder)
- [ ] Handle all action flags in main loop
- [ ] Integration test: Full flow works

**Commit:** `feat(ssh): wire up SSH command execution for all tabs`

### Phase 8: Auto-Refresh on Drawer Open (Day 7)

**Tasks:**
- [ ] Track drawer open/close state
- [ ] Trigger refresh on first open
- [ ] Show stale data while refreshing
- [ ] Only refresh selected tab
- [ ] Manual test: Auto-refresh works

**Commit:** `feat(ssh): auto-refresh tab on drawer open`

### Phase 9: Polish & Error Handling (Day 8)

**Tasks:**
- [ ] Add staleness warnings
- [ ] Improve error messages
- [ ] Add button loading states
- [ ] Test all error scenarios
- [ ] Add tooltips to disabled tabs
- [ ] Manual test: Errors handled well

**Commit:** `fix(ssh): improve error handling and staleness indicators`

### Phase 10: Testing & Documentation (Day 9)

**Tasks:**
- [ ] Run all unit tests
- [ ] Complete manual testing checklist
- [ ] Add inline documentation
- [ ] Test on multiple platforms
- [ ] Fix any bugs found

**Commit:** `docs(ssh): add inline documentation for drawer refactor`

### Estimated Timeline

- **Days 1-2:** Foundation + emoji progress (2 days)
- **Days 3-6:** Tab UI + all three tabs (4 days)
- **Days 6-8:** Command execution + auto-refresh + polish (3 days)
- **Day 9:** Testing + documentation (1 day)

**Total: ~9 days** (approximately 1.5 weeks with buffer)

## Rollout Strategy

**Branch:** `feature/ssh-drawer-refactor`

**Steps:**
1. Create feature branch from `main`
2. Implement phases 1-10
3. Manual testing on Desktop Linux
4. Code review
5. Merge to `main` via PR
6. Monitor for issues

**No feature flag needed** - active development phase, breaking changes acceptable.

**Rollback plan:** Revert merge commit if critical issues found.

## Success Criteria

### Functional
- [ ] Emoji progress bar shows correct Linux→Docker→Dure state
- [ ] Drawer displays 3 tabs with correct enable/disable logic
- [ ] Terminal output shows monospace scrollable content
- [ ] Refresh buttons work per-tab with loading states
- [ ] Install Docker button triggers installation
- [ ] Auto-refresh on drawer open works
- [ ] Errors show actual stderr in red monospace

### Non-Functional
- [ ] Components reusable (TerminalOutput, EmojiProgressBar)
- [ ] No performance regression
- [ ] Code coverage >70% for new components
- [ ] Works on Desktop Linux/macOS/Windows

### User Experience
- [ ] User sees instant feedback on button clicks
- [ ] Command output is readable and scrollable
- [ ] Tab navigation is intuitive
- [ ] Error messages are helpful and actionable
- [ ] Staleness warnings prevent confusion

## Future Enhancements

**Not in scope for this refactor:**

1. **Event-based architecture:** ViewModel fires events when operations complete (like platform tab)
2. **Actual Dure install:** Complete xmpp-proxy-stack installation workflow
3. **Live updates:** WebSocket or polling for real-time status
4. **Log streaming:** Show docker logs in Dure tab
5. **Service management:** Start/stop/restart buttons for containers
6. **Custom commands:** User-defined tab with custom SSH commands
7. **Multi-host operations:** Batch refresh across multiple hosts

## References

- **Platform refactor spec:** `docs/superpowers/specs/2026-08-04-platform-drawer-refactor-design.md`
- **Current SSH implementation:** `mobile/src/ui_tabs/ssh.rs`
- **EmojiProgressBar component:** `mobile/src/ui_components/emoji_progress.rs`
- **StatusGrid component:** `mobile/src/ui_components/status_grid.rs` (reference, not used here)
- **SSH command execution:** `mobile/src/calc/ssh.rs`

## Open Questions

None - design approved.

## Changelog

- **2026-08-06:** Initial design approved by user
