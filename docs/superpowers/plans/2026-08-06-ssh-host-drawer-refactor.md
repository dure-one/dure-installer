# SSH Host Drawer Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refactor SSH tab drawer to use tab-based layout (Linux/Docker/Dure) with terminal-style command output, add emoji progress indicators, and provide per-tab refresh/install actions.

**Architecture:** Pragmatic hybrid approach - reuse platform components (EmojiProgressBar), create lightweight TerminalOutput component, use egui built-in widgets for tabs, simple boolean state tracking (no complex OperationState enum).

**Tech Stack:** Rust (nightly), egui 0.33, eframe 0.33, egui-material3, existing calc::ssh module for command execution

## Global Constraints

- Rust nightly toolchain required
- Desktop-only feature (not mobile/WASM)
- Follow TDD: write test first, verify failure, implement, verify pass, commit
- No new clippy warnings
- Reuse existing components where appropriate (EmojiProgressBar, StatusGrid pattern)
- No breaking changes to ViewModel API
- Terminal output must be monospace and scrollable
- Show raw error messages (no silent failures)

---

## File Structure

**New Files:**
- `mobile/src/ui_components/terminal_output.rs` - Terminal output display component

**Modified Files:**
- `mobile/src/ui_components/emoji_progress.rs` - Add `from_ssh_row()` method
- `mobile/src/ui_components/mod.rs` - Export TerminalOutput
- `mobile/src/ui_tabs/ssh.rs` - Add data fields, refactor drawer, update Status column

**Test Files:**
- Unit tests in `terminal_output.rs`
- Unit tests in `emoji_progress.rs` (SSH section)

---

## Task 1: Data Model Foundation

**Files:**
- Modify: `mobile/src/ui_tabs/ssh.rs:39-69` (SshRowData struct)

**Interfaces:**
- Consumes: Existing SshRowData fields
- Produces: Extended SshRowData with output fields, refresh flags, timestamps

- [ ] **Step 1: Add output fields to SshRowData**

Add after line 69 in `mobile/src/ui_tabs/ssh.rs`:

```rust
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
```

- [ ] **Step 2: Update Default implementation**

Find `impl Default for SshRowData` (around line 248) and add fields to default values:

```rust
linux_output: None,
linux_output_error: None,
docker_output: None,
docker_output_error: None,
dure_output: None,
dure_output_error: None,
linux_refreshing: false,
docker_refreshing: false,
dure_refreshing: false,
docker_installing: false,
dure_installing: false,
linux_last_refresh: None,
docker_last_refresh: None,
dure_last_refresh: None,
```

- [ ] **Step 3: Verify compilation**

Run: `cd mobile && cargo check --lib`

Expected: Success with no errors

- [ ] **Step 4: Commit**

```bash
git add mobile/src/ui_tabs/ssh.rs
git commit -m "feat(ssh): add data model fields for tab-based drawer

Add output, error, refresh state, and timestamp fields to SshRowData
for Linux/Docker/Dure tab content caching and operation tracking.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 2: TerminalOutput Component

**Files:**
- Create: `mobile/src/ui_components/terminal_output.rs`
- Modify: `mobile/src/ui_components/mod.rs`

**Interfaces:**
- Consumes: Nothing (standalone component)
- Produces: `TerminalOutput` struct with builder methods: `new()`, `with_max_height()`, `loading()`, `error()`, `show()`

- [ ] **Step 1: Write failing tests for TerminalOutput**

Create `mobile/src/ui_components/terminal_output.rs`:

```rust
//! Terminal output display component

use eframe::egui;

/// Terminal-style output display (monospace, scrollable)
pub struct TerminalOutput {
    content: String,
    max_height: f32,
    loading: bool,
    error_mode: bool,
}

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
    fn test_terminal_output_builder_max_height() {
        let output = TerminalOutput::new("test").with_max_height(200.0);
        assert_eq!(output.max_height, 200.0);
    }
    
    #[test]
    fn test_terminal_output_builder_loading() {
        let output = TerminalOutput::new("test").loading(true);
        assert!(output.loading);
    }
    
    #[test]
    fn test_terminal_output_builder_error() {
        let output = TerminalOutput::new("test").error(true);
        assert!(output.error_mode);
    }
    
    #[test]
    fn test_terminal_output_builder_chain() {
        let output = TerminalOutput::new("test")
            .with_max_height(200.0)
            .loading(true)
            .error(true);
        
        assert_eq!(output.content, "test");
        assert_eq!(output.max_height, 200.0);
        assert!(output.loading);
        assert!(output.error_mode);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd mobile && cargo test terminal_output --lib -- --nocapture`

Expected: Compilation error - methods not found

- [ ] **Step 3: Implement TerminalOutput methods**

Add implementation before the `#[cfg(test)]` section:

```rust
impl TerminalOutput {
    /// Create new terminal output display
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            max_height: 300.0,
            loading: false,
            error_mode: false,
        }
    }
    
    /// Set maximum height for scrollable area
    pub fn with_max_height(mut self, height: f32) -> Self {
        self.max_height = height;
        self
    }
    
    /// Set loading state (shows spinner)
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }
    
    /// Set error mode (red text)
    pub fn error(mut self, is_error: bool) -> Self {
        self.error_mode = is_error;
        self
    }
    
    /// Render the terminal output
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

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd mobile && cargo test terminal_output --lib -- --nocapture`

Expected: All tests pass

- [ ] **Step 5: Export TerminalOutput from mod.rs**

Add to `mobile/src/ui_components/mod.rs`:

```rust
mod terminal_output;
pub use terminal_output::TerminalOutput;
```

- [ ] **Step 6: Verify compilation**

Run: `cd mobile && cargo check --lib`

Expected: Success

- [ ] **Step 7: Commit**

```bash
git add mobile/src/ui_components/terminal_output.rs mobile/src/ui_components/mod.rs
git commit -m "feat(ui): add TerminalOutput component

Create terminal-style output display component with:
- Monospace scrollable text
- Optional error mode (red text)
- Optional loading spinner
- Builder pattern API

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 3: EmojiProgressBar SSH Integration

**Files:**
- Modify: `mobile/src/ui_components/emoji_progress.rs` (add method and tests)

**Interfaces:**
- Consumes: `SshRowData` (from ui_tabs::ssh)
- Produces: `EmojiProgressBar::from_ssh_row(row: &SshRowData) -> Self`

- [ ] **Step 1: Write failing tests for from_ssh_row**

Add at end of `mobile/src/ui_components/emoji_progress.rs`:

```rust
#[cfg(test)]
mod ssh_tests {
    use super::*;
    
    // Mock SshRowData for testing
    #[derive(Default)]
    struct SshRowData {
        linux_detected: bool,
        docker_enabled: bool,
        dure_wss_enabled: bool,
    }
    
    #[test]
    fn test_from_ssh_row_none_detected() {
        let row = SshRowData {
            linux_detected: false,
            docker_enabled: false,
            dure_wss_enabled: false,
        };
        
        let progress = EmojiProgressBar::from_ssh_row(&row);
        assert_eq!(progress.steps.len(), 3);
        assert!(progress.steps.iter().all(|s| 
            matches!(s.state, ProgressState::Pending)
        ));
    }
    
    #[test]
    fn test_from_ssh_row_linux_only() {
        let row = SshRowData {
            linux_detected: true,
            docker_enabled: false,
            dure_wss_enabled: false,
        };
        
        let progress = EmojiProgressBar::from_ssh_row(&row);
        assert_eq!(progress.steps.len(), 3);
        assert!(matches!(progress.steps[0].state, ProgressState::Completed));
        assert!(matches!(progress.steps[1].state, ProgressState::Pending));
        assert!(matches!(progress.steps[2].state, ProgressState::Pending));
    }
    
    #[test]
    fn test_from_ssh_row_linux_and_docker() {
        let row = SshRowData {
            linux_detected: true,
            docker_enabled: true,
            dure_wss_enabled: false,
        };
        
        let progress = EmojiProgressBar::from_ssh_row(&row);
        assert!(matches!(progress.steps[0].state, ProgressState::Completed));
        assert!(matches!(progress.steps[1].state, ProgressState::Completed));
        assert!(matches!(progress.steps[2].state, ProgressState::Pending));
    }
    
    #[test]
    fn test_from_ssh_row_all_enabled() {
        let row = SshRowData {
            linux_detected: true,
            docker_enabled: true,
            dure_wss_enabled: true,
        };
        
        let progress = EmojiProgressBar::from_ssh_row(&row);
        assert_eq!(progress.steps.len(), 3);
        assert!(progress.steps.iter().all(|s| 
            matches!(s.state, ProgressState::Completed)
        ));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd mobile && cargo test ssh_tests --lib -- --nocapture`

Expected: Method `from_ssh_row` not found

- [ ] **Step 3: Implement from_ssh_row method**

Add this implementation in the `impl EmojiProgressBar` block:

```rust
/// Create progress bar from SSH row data
/// Shows: Linux → Docker → Dure progression
pub fn from_ssh_row(row: &impl SshRowLike) -> Self {
    let mut bar = Self::new();
    
    // Step 1: Linux detected
    bar.add_step("Linux", if row.linux_detected() {
        ProgressState::Completed
    } else {
        ProgressState::Pending
    });
    
    // Step 2: Docker installed
    bar.add_step("Docker", if row.docker_enabled() {
        ProgressState::Completed
    } else {
        ProgressState::Pending
    });
    
    // Step 3: Dure running
    bar.add_step("Dure", if row.dure_wss_enabled() {
        ProgressState::Completed
    } else {
        ProgressState::Pending
    });
    
    bar
}
```

Add trait before tests:

```rust
/// Trait for SSH row-like data
pub trait SshRowLike {
    fn linux_detected(&self) -> bool;
    fn docker_enabled(&self) -> bool;
    fn dure_wss_enabled(&self) -> bool;
}
```

Update test mock to implement trait:

```rust
impl SshRowLike for SshRowData {
    fn linux_detected(&self) -> bool { self.linux_detected }
    fn docker_enabled(&self) -> bool { self.docker_enabled }
    fn dure_wss_enabled(&self) -> bool { self.dure_wss_enabled }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd mobile && cargo test ssh_tests --lib -- --nocapture`

Expected: All tests pass

- [ ] **Step 5: Verify compilation**

Run: `cd mobile && cargo check --lib`

Expected: Success

- [ ] **Step 6: Commit**

```bash
git add mobile/src/ui_components/emoji_progress.rs
git commit -m "feat(ui): add SSH progress support to EmojiProgressBar

Add from_ssh_row() method to show Linux→Docker→Dure progression.
Uses trait for flexibility with actual SshRowData struct.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 4: Implement SshRowLike for SshRowData

**Files:**
- Modify: `mobile/src/ui_tabs/ssh.rs` (add trait impl after struct)

**Interfaces:**
- Consumes: `SshRowLike` trait from emoji_progress
- Produces: Trait implementation for SshRowData

- [ ] **Step 1: Import SshRowLike trait**

Add to imports at top of `mobile/src/ui_tabs/ssh.rs`:

```rust
use crate::ui_components::SshRowLike;
```

- [ ] **Step 2: Implement SshRowLike for SshRowData**

Add after `SshRowData` struct definition (around line 90):

```rust
impl SshRowLike for SshRowData {
    fn linux_detected(&self) -> bool {
        self.linux_detected
    }
    
    fn docker_enabled(&self) -> bool {
        self.docker_enabled
    }
    
    fn dure_wss_enabled(&self) -> bool {
        self.dure_wss_enabled
    }
}
```

- [ ] **Step 3: Verify compilation**

Run: `cd mobile && cargo check --lib`

Expected: Success

- [ ] **Step 4: Commit**

```bash
git add mobile/src/ui_tabs/ssh.rs
git commit -m "feat(ssh): implement SshRowLike for SshRowData

Enable emoji progress bar integration by implementing trait.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 5: Update Status Column with Emoji Progress

**Files:**
- Modify: `mobile/src/ui_tabs/ssh.rs:988-1024` (render_table method)

**Interfaces:**
- Consumes: `EmojiProgressBar::from_ssh_row()`, `SshRowData`
- Produces: Updated Status column rendering

- [ ] **Step 1: Add EmojiProgressBar import**

Add to imports at top of file:

```rust
use crate::ui_components::EmojiProgressBar;
```

- [ ] **Step 2: Find Status column rendering**

Locate the `.cell` or `.cell_widget` for Status column in `render_table` method (around line 1000-1020).

Current code likely looks like:
```rust
.cell(|ui| {
    ui.label(&status_text);
})
```

- [ ] **Step 3: Replace with emoji progress bar**

Replace the Status column cell rendering with:

```rust
.cell_widget(|ui| {
    let progress = EmojiProgressBar::from_ssh_row(&row)
        .compact(true);
    progress.show(ui);
})
```

- [ ] **Step 4: Verify compilation**

Run: `cd mobile && cargo check --lib`

Expected: Success

- [ ] **Step 5: Manual test (if possible)**

Run: `cd mobile && cargo run --bin dure-desktop`

Navigate to SSH tab, verify Status column shows emoji progress (⚪⚪⚪ or ✅⚪⚪ etc)

- [ ] **Step 6: Commit**

```bash
git add mobile/src/ui_tabs/ssh.rs
git commit -m "feat(ssh): replace Status column with emoji progress

Show Linux→Docker→Dure progression using emoji indicators
instead of text status.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 6: Tab Selector Function

**Files:**
- Modify: `mobile/src/ui_tabs/ssh.rs` (add new function after render_drawer_content)

**Interfaces:**
- Consumes: `selected_tab: &mut usize`, `row: &SshRowData`
- Produces: `render_tab_selector()` function

- [ ] **Step 1: Add render_tab_selector function**

Add after `render_drawer_content` function (around line 2690):

```rust
/// Render tab selector (Linux, Docker, Dure)
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

- [ ] **Step 2: Verify compilation**

Run: `cd mobile && cargo check --lib`

Expected: Success (function unused warning is OK)

- [ ] **Step 3: Commit**

```bash
git add mobile/src/ui_tabs/ssh.rs
git commit -m "feat(ssh): add tab selector rendering function

Render compact horizontal tabs with enable/disable logic
and tooltips for disabled tabs.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 7: Tab Rendering Stub Functions

**Files:**
- Modify: `mobile/src/ui_tabs/ssh.rs` (add stub functions)

**Interfaces:**
- Consumes: `ui: &mut egui::Ui`, `row: &SshRowData`, `idx: usize`
- Produces: `render_linux_tab()`, `render_docker_tab()`, `render_dure_tab()`

- [ ] **Step 1: Add stub tab rendering functions**

Add after `render_tab_selector`:

```rust
/// Render Linux tab content
fn render_linux_tab(ui: &mut egui::Ui, row: &SshRowData, idx: usize) {
    ui.label("Linux tab - TODO");
}

/// Render Docker tab content
fn render_docker_tab(ui: &mut egui::Ui, row: &SshRowData, idx: usize) {
    ui.label("Docker tab - TODO");
}

/// Render Dure tab content
fn render_dure_tab(ui: &mut egui::Ui, row: &SshRowData, idx: usize) {
    ui.label("Dure tab - TODO");
}
```

- [ ] **Step 2: Verify compilation**

Run: `cd mobile && cargo check --lib`

Expected: Success

- [ ] **Step 3: Commit**

```bash
git add mobile/src/ui_tabs/ssh.rs
git commit -m "feat(ssh): add stub tab rendering functions

Create placeholder functions for Linux, Docker, and Dure tabs.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 8: Refactor render_drawer_content

**Files:**
- Modify: `mobile/src/ui_tabs/ssh.rs:2595-2690` (replace function body)

**Interfaces:**
- Consumes: Tab selector and tab render functions
- Produces: Updated drawer with tabs

- [ ] **Step 1: Replace render_drawer_content function body**

Replace the entire function body (keep signature):

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

- [ ] **Step 2: Verify compilation**

Run: `cd mobile && cargo check --lib`

Expected: Success

- [ ] **Step 3: Manual test**

Run: `cd mobile && cargo run --bin dure-desktop`

Open SSH drawer, verify tabs appear and can be clicked

- [ ] **Step 4: Commit**

```bash
git add mobile/src/ui_tabs/ssh.rs
git commit -m "refactor(ssh): replace drawer content with tab-based UI

Replace flat list layout with compact horizontal tabs.
Tab selection persists per-host using temp data.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 9: Linux Tab Implementation

**Files:**
- Modify: `mobile/src/ui_tabs/ssh.rs` (implement render_linux_tab)

**Interfaces:**
- Consumes: `TerminalOutput` component, `SshRowData` fields
- Produces: Full Linux tab with output and buttons

- [ ] **Step 1: Add TerminalOutput import**

Add to imports:

```rust
use crate::ui_components::TerminalOutput;
```

- [ ] **Step 2: Implement render_linux_tab**

Replace stub with full implementation:

```rust
/// Render Linux tab content
fn render_linux_tab(ui: &mut egui::Ui, row: &SshRowData, idx: usize) {
    use egui_material3::MaterialButton;
    use chrono::Utc;
    
    // Staleness warning
    if let Some(last_refresh) = row.linux_last_refresh {
        let elapsed_secs = Utc::now().timestamp() - last_refresh;
        
        if elapsed_secs > 3600 {
            let hours = elapsed_secs / 3600;
            ui.colored_label(
                egui::Color32::from_rgb(255, 152, 0),
                format!("⚠ Data from {} hour{} ago", hours, if hours == 1 { "" } else { "s" })
            );
            ui.add_space(4.0);
        }
    }
    
    // Terminal output
    if let Some(error) = &row.linux_output_error {
        TerminalOutput::new(error)
            .error(true)
            .show(ui);
    } else if let Some(output) = &row.linux_output {
        TerminalOutput::new(output)
            .show(ui);
    } else {
        ui.colored_label(
            ui.visuals().weak_text_color(),
            "Click Refresh to load system information"
        );
    }
    
    ui.add_space(8.0);
    
    // Action buttons
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        
        // Refresh button
        let refresh_label = if row.linux_refreshing {
            "Refreshing..."
        } else {
            "Refresh"
        };
        
        if ui.add_enabled(
            !row.linux_refreshing,
            MaterialButton::outlined(refresh_label).small()
        ).clicked() {
            ui.data_mut(|d| {
                d.insert_temp(
                    egui::Id::new(format!("ssh_linux_refresh_{}", idx)),
                    row.host.clone(),
                )
            });
        }
        
        // Install Docker button (only if not installed)
        if !row.docker_enabled {
            let docker_label = if row.docker_installing {
                "Installing..."
            } else {
                "Install Docker"
            };
            
            if ui.add_enabled(
                !row.docker_installing,
                MaterialButton::outlined(docker_label).small()
            ).clicked() {
                ui.data_mut(|d| {
                    d.insert_temp(
                        egui::Id::new(format!("ssh_docker_install_{}", idx)),
                        row.host.clone(),
                    )
                });
            }
        }
    });
}
```

- [ ] **Step 3: Verify compilation**

Run: `cd mobile && cargo check --lib`

Expected: Success

- [ ] **Step 4: Commit**

```bash
git add mobile/src/ui_tabs/ssh.rs
git commit -m "feat(ssh): implement Linux tab rendering

Show system info output with TerminalOutput component.
Add Refresh and Install Docker buttons with loading states.
Show staleness warnings for data older than 1 hour.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 10: Docker Tab Implementation

**Files:**
- Modify: `mobile/src/ui_tabs/ssh.rs` (implement render_docker_tab)

**Interfaces:**
- Consumes: `TerminalOutput`, `SshRowData` fields
- Produces: Full Docker tab with output and buttons

- [ ] **Step 1: Implement render_docker_tab**

Replace stub with:

```rust
/// Render Docker tab content
fn render_docker_tab(ui: &mut egui::Ui, row: &SshRowData, idx: usize) {
    use egui_material3::MaterialButton;
    
    // Terminal output
    if let Some(error) = &row.docker_output_error {
        TerminalOutput::new(error)
            .error(true)
            .show(ui);
    } else if let Some(output) = &row.docker_output {
        TerminalOutput::new(output)
            .show(ui);
    } else {
        ui.colored_label(
            ui.visuals().weak_text_color(),
            "Click Refresh to load Docker containers"
        );
    }
    
    ui.add_space(8.0);
    
    // Action buttons
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        
        // Refresh button
        let refresh_label = if row.docker_refreshing {
            "Refreshing..."
        } else {
            "Refresh"
        };
        
        if ui.add_enabled(
            !row.docker_refreshing,
            MaterialButton::outlined(refresh_label).small()
        ).clicked() {
            ui.data_mut(|d| {
                d.insert_temp(
                    egui::Id::new(format!("ssh_docker_refresh_{}", idx)),
                    row.host.clone(),
                )
            });
        }
        
        // Install Dure button (only if not running)
        if !row.dure_wss_enabled {
            let dure_label = if row.dure_installing {
                "Installing..."
            } else {
                "Install Dure"
            };
            
            if ui.add_enabled(
                !row.dure_installing,
                MaterialButton::outlined(dure_label).small()
            ).clicked() {
                ui.data_mut(|d| {
                    d.insert_temp(
                        egui::Id::new(format!("ssh_dure_install_{}", idx)),
                        row.host.clone(),
                    )
                });
            }
        }
    });
}
```

- [ ] **Step 2: Verify compilation**

Run: `cd mobile && cargo check --lib`

Expected: Success

- [ ] **Step 3: Commit**

```bash
git add mobile/src/ui_tabs/ssh.rs
git commit -m "feat(ssh): implement Docker tab rendering

Show docker ps -a output with TerminalOutput component.
Add Refresh and Install Dure buttons with loading states.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 11: Dure Tab Implementation

**Files:**
- Modify: `mobile/src/ui_tabs/ssh.rs` (implement render_dure_tab)

**Interfaces:**
- Consumes: `TerminalOutput`, `SshRowData` fields
- Produces: Full Dure tab with output and button

- [ ] **Step 1: Implement render_dure_tab**

Replace stub with:

```rust
/// Render Dure tab content
fn render_dure_tab(ui: &mut egui::Ui, row: &SshRowData, idx: usize) {
    use egui_material3::MaterialButton;
    
    // Terminal output
    if let Some(error) = &row.dure_output_error {
        TerminalOutput::new(error)
            .error(true)
            .show(ui);
    } else if let Some(output) = &row.dure_output {
        TerminalOutput::new(output)
            .show(ui);
    } else {
        ui.colored_label(
            ui.visuals().weak_text_color(),
            "Click Refresh to load port status"
        );
    }
    
    ui.add_space(8.0);
    
    // Action button
    let refresh_label = if row.dure_refreshing {
        "Refreshing..."
    } else {
        "Refresh"
    };
    
    if ui.add_enabled(
        !row.dure_refreshing,
        MaterialButton::outlined(refresh_label).small()
    ).clicked() {
        ui.data_mut(|d| {
            d.insert_temp(
                egui::Id::new(format!("ssh_dure_refresh_{}", idx)),
                row.host.clone(),
            )
        });
    }
}
```

- [ ] **Step 2: Verify compilation**

Run: `cd mobile && cargo check --lib`

Expected: Success

- [ ] **Step 3: Commit**

```bash
git add mobile/src/ui_tabs/ssh.rs
git commit -m "feat(ssh): implement Dure tab rendering

Show ss -nltup output with TerminalOutput component.
Add Refresh button with loading state.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 12: Linux Refresh Action Handler

**Files:**
- Modify: `mobile/src/ui_tabs/ssh.rs` (add handler in ui() method)

**Interfaces:**
- Consumes: Temp data flag `ssh_linux_refresh_{idx}`
- Produces: SSH command execution, row state update

- [ ] **Step 1: Define system info script constant**

Add at top of file after imports:

```rust
/// System information script for Linux tab
const LINUX_SYSTEM_INFO_SCRIPT: &str = r#"#!/bin/bash
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
"#;
```

- [ ] **Step 2: Find ui() method action handlers section**

Locate where other SSH action temp data flags are checked (likely after table rendering, before end of ui() method).

- [ ] **Step 3: Add Linux refresh handler**

Add handler:

```rust
// Handle Linux refresh action
for idx in 0..self.rows.len() {
    if let Some(host) = ui.data_mut(|d| 
        d.remove::<String>(egui::Id::new(format!("ssh_linux_refresh_{}", idx)))
    ) {
        // Set refreshing flag
        if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
            row.linux_refreshing = true;
            
            // Get auth from config
            if let Ok(cfg) = AppConfig::load() {
                if let Some(ssh_cfg) = cfg.ssh_hosts.iter().find(|s| s.host == host) {
                    let host = ssh_cfg.host.clone();
                    let port = ssh_cfg.port;
                    let auth = ssh::auth_from_config(ssh_cfg);
                    
                    // Execute command in background
                    std::thread::spawn(move || {
                        let result = ssh::run_command(
                            &host,
                            port,
                            LINUX_SYSTEM_INFO_SCRIPT,
                            auth,
                        );
                        
                        // Store result (in real implementation, would use event/channel)
                        // For now, this is a placeholder - actual integration in next task
                    });
                }
            }
        }
    }
}
```

- [ ] **Step 4: Verify compilation**

Run: `cd mobile && cargo check --lib`

Expected: Success (may have unused result warning)

- [ ] **Step 5: Commit**

```bash
git add mobile/src/ui_tabs/ssh.rs
git commit -m "feat(ssh): add Linux refresh action handler

Handle ssh_linux_refresh temp data flag, set refreshing state,
and spawn SSH command execution thread (placeholder).

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 13: Docker Refresh Action Handler

**Files:**
- Modify: `mobile/src/ui_tabs/ssh.rs` (add handler in ui() method)

**Interfaces:**
- Consumes: Temp data flag `ssh_docker_refresh_{idx}`
- Produces: Docker ps command execution

- [ ] **Step 1: Add Docker refresh handler**

Add after Linux refresh handler:

```rust
// Handle Docker refresh action
for idx in 0..self.rows.len() {
    if let Some(host) = ui.data_mut(|d| 
        d.remove::<String>(egui::Id::new(format!("ssh_docker_refresh_{}", idx)))
    ) {
        if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
            row.docker_refreshing = true;
            
            if let Ok(cfg) = AppConfig::load() {
                if let Some(ssh_cfg) = cfg.ssh_hosts.iter().find(|s| s.host == host) {
                    let host = ssh_cfg.host.clone();
                    let port = ssh_cfg.port;
                    let auth = ssh::auth_from_config(ssh_cfg);
                    
                    std::thread::spawn(move || {
                        let result = ssh::run_command(
                            &host,
                            port,
                            "docker ps -a",
                            auth,
                        );
                        
                        // Placeholder - actual integration in next task
                    });
                }
            }
        }
    }
}
```

- [ ] **Step 2: Verify compilation**

Run: `cd mobile && cargo check --lib`

Expected: Success

- [ ] **Step 3: Commit**

```bash
git add mobile/src/ui_tabs/ssh.rs
git commit -m "feat(ssh): add Docker refresh action handler

Handle ssh_docker_refresh temp data flag and execute docker ps -a.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 14: Dure Refresh Action Handler

**Files:**
- Modify: `mobile/src/ui_tabs/ssh.rs` (add handler in ui() method)

**Interfaces:**
- Consumes: Temp data flag `ssh_dure_refresh_{idx}`
- Produces: ss -nltup command execution

- [ ] **Step 1: Add Dure refresh handler**

Add after Docker refresh handler:

```rust
// Handle Dure refresh action
for idx in 0..self.rows.len() {
    if let Some(host) = ui.data_mut(|d| 
        d.remove::<String>(egui::Id::new(format!("ssh_dure_refresh_{}", idx)))
    ) {
        if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
            row.dure_refreshing = true;
            
            if let Ok(cfg) = AppConfig::load() {
                if let Some(ssh_cfg) = cfg.ssh_hosts.iter().find(|s| s.host == host) {
                    let host = ssh_cfg.host.clone();
                    let port = ssh_cfg.port;
                    let auth = ssh::auth_from_config(ssh_cfg);
                    
                    std::thread::spawn(move || {
                        let result = ssh::run_command(
                            &host,
                            port,
                            "ss -nltup",
                            auth,
                        );
                        
                        // Placeholder - actual integration in next task
                    });
                }
            }
        }
    }
}
```

- [ ] **Step 2: Verify compilation**

Run: `cd mobile && cargo check --lib`

Expected: Success

- [ ] **Step 3: Commit**

```bash
git add mobile/src/ui_tabs/ssh.rs
git commit -m "feat(ssh): add Dure refresh action handler

Handle ssh_dure_refresh temp data flag and execute ss -nltup.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 15: Docker Install Action Handler

**Files:**
- Modify: `mobile/src/ui_tabs/ssh.rs` (add handler in ui() method)

**Interfaces:**
- Consumes: Temp data flag `ssh_docker_install_{idx}`
- Produces: Docker installation command execution

- [ ] **Step 1: Define Docker install script constant**

Add after LINUX_SYSTEM_INFO_SCRIPT:

```rust
/// Docker installation script
const DOCKER_INSTALL_SCRIPT: &str = "curl -fsSL https://get.docker.com | sh";
```

- [ ] **Step 2: Add Docker install handler**

Add after Dure refresh handler:

```rust
// Handle Docker install action
for idx in 0..self.rows.len() {
    if let Some(host) = ui.data_mut(|d| 
        d.remove::<String>(egui::Id::new(format!("ssh_docker_install_{}", idx)))
    ) {
        if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
            row.docker_installing = true;
            
            if let Ok(cfg) = AppConfig::load() {
                if let Some(ssh_cfg) = cfg.ssh_hosts.iter().find(|s| s.host == host) {
                    let host = ssh_cfg.host.clone();
                    let port = ssh_cfg.port;
                    let auth = ssh::auth_from_config(ssh_cfg);
                    
                    std::thread::spawn(move || {
                        let result = ssh::run_command(
                            &host,
                            port,
                            DOCKER_INSTALL_SCRIPT,
                            auth,
                        );
                        
                        // Placeholder - actual integration in next task
                    });
                }
            }
        }
    }
}
```

- [ ] **Step 3: Verify compilation**

Run: `cd mobile && cargo check --lib`

Expected: Success

- [ ] **Step 4: Commit**

```bash
git add mobile/src/ui_tabs/ssh.rs
git commit -m "feat(ssh): add Docker install action handler

Handle ssh_docker_install temp data flag and execute Docker
installation script via curl | sh.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 16: Dure Install Action Handler (Placeholder)

**Files:**
- Modify: `mobile/src/ui_tabs/ssh.rs` (add handler in ui() method)

**Interfaces:**
- Consumes: Temp data flag `ssh_dure_install_{idx}`
- Produces: Placeholder message (not yet implemented)

- [ ] **Step 1: Add Dure install handler**

Add after Docker install handler:

```rust
// Handle Dure install action (placeholder)
for idx in 0..self.rows.len() {
    if let Some(host) = ui.data_mut(|d| 
        d.remove::<String>(egui::Id::new(format!("ssh_dure_install_{}", idx)))
    ) {
        if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
            // For now, just show error message
            row.dure_output_error = Some(
                "Dure installation not yet implemented.\n\
                 Future: Will clone xmpp-proxy-stack and run docker-compose.".to_string()
            );
            row.dure_installing = false;
        }
    }
}
```

- [ ] **Step 2: Verify compilation**

Run: `cd mobile && cargo check --lib`

Expected: Success

- [ ] **Step 3: Commit**

```bash
git add mobile/src/ui_tabs/ssh.rs
git commit -m "feat(ssh): add Dure install action handler (placeholder)

Show 'not yet implemented' message for Dure installation.
Future work: clone xmpp-proxy-stack and run docker-compose.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 17: Command Result Integration (Completion Handler)

**Files:**
- Modify: `mobile/src/ui_tabs/ssh.rs` (update action handlers to store results)

**Interfaces:**
- Consumes: SSH command results
- Produces: Updated row fields (output, error, timestamp, flags)

**Note:** This task integrates actual command results. Since SSH commands run in background threads, we need a mechanism to update row data when they complete. For simplicity, we'll use a polling approach where rows check for completed commands on each frame.

- [ ] **Step 1: Add result channel type**

Add at top of file after imports:

```rust
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// Command result cache (host -> (command_type, result))
type ResultCache = Arc<Mutex<HashMap<String, (CommandType, Result<String, String>)>>>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum CommandType {
    LinuxSystemInfo,
    DockerPs,
    DureSs,
    DockerInstall,
}
```

- [ ] **Step 2: Add result cache field to SshTab**

Add to SshTab struct:

```rust
#[cfg_attr(feature = "serde", serde(skip))]
command_results: ResultCache,
```

- [ ] **Step 3: Initialize result cache in Default**

Add to Default impl:

```rust
command_results: Arc::new(Mutex::new(HashMap::new())),
```

- [ ] **Step 4: Update Linux refresh handler to use cache**

Replace the thread spawn section in Linux refresh handler:

```rust
let cache = self.command_results.clone();
std::thread::spawn(move || {
    let result = ssh::run_command(
        &host,
        port,
        LINUX_SYSTEM_INFO_SCRIPT,
        auth,
    );
    
    cache.lock().unwrap().insert(
        host.clone(),
        (CommandType::LinuxSystemInfo, result)
    );
});
```

- [ ] **Step 5: Add result processing in ui() method**

Add before action handlers:

```rust
// Process completed commands
let completed: Vec<_> = {
    let mut cache = self.command_results.lock().unwrap();
    cache.drain().collect()
};

for (host, (cmd_type, result)) in completed {
    if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
        match cmd_type {
            CommandType::LinuxSystemInfo => {
                row.linux_refreshing = false;
                match result {
                    Ok(output) => {
                        row.linux_output = Some(output);
                        row.linux_output_error = None;
                        row.linux_last_refresh = Some(chrono::Utc::now().timestamp());
                    }
                    Err(err) => {
                        row.linux_output_error = Some(err);
                    }
                }
            }
            CommandType::DockerPs => {
                row.docker_refreshing = false;
                match result {
                    Ok(output) => {
                        row.docker_output = Some(output);
                        row.docker_output_error = None;
                        row.docker_last_refresh = Some(chrono::Utc::now().timestamp());
                    }
                    Err(err) => {
                        row.docker_output_error = Some(err);
                    }
                }
            }
            CommandType::DureSs => {
                row.dure_refreshing = false;
                match result {
                    Ok(output) => {
                        row.dure_output = Some(output);
                        row.dure_output_error = None;
                        row.dure_last_refresh = Some(chrono::Utc::now().timestamp());
                    }
                    Err(err) => {
                        row.dure_output_error = Some(err);
                    }
                }
            }
            CommandType::DockerInstall => {
                row.docker_installing = false;
                match result {
                    Ok(_) => {
                        // Update docker_enabled flag
                        row.docker_enabled = true;
                        row.linux_output = Some(
                            "Docker installed successfully.\n\
                             Note: May require SSH reconnect or system reboot to take effect.".to_string()
                        );
                    }
                    Err(err) => {
                        row.linux_output_error = Some(format!(
                            "Docker installation failed:\n{}",
                            err
                        ));
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 6: Update Docker refresh handler to use cache**

Update the thread spawn in Docker refresh handler:

```rust
let cache = self.command_results.clone();
std::thread::spawn(move || {
    let result = ssh::run_command(
        &host,
        port,
        "docker ps -a",
        auth,
    );
    
    cache.lock().unwrap().insert(
        host.clone(),
        (CommandType::DockerPs, result)
    );
});
```

- [ ] **Step 7: Update Dure refresh handler to use cache**

Update the thread spawn in Dure refresh handler:

```rust
let cache = self.command_results.clone();
std::thread::spawn(move || {
    let result = ssh::run_command(
        &host,
        port,
        "ss -nltup",
        auth,
    );
    
    cache.lock().unwrap().insert(
        host.clone(),
        (CommandType::DureSs, result)
    );
});
```

- [ ] **Step 8: Update Docker install handler to use cache**

Update the thread spawn in Docker install handler:

```rust
let cache = self.command_results.clone();
std::thread::spawn(move || {
    let result = ssh::run_command(
        &host,
        port,
        DOCKER_INSTALL_SCRIPT,
        auth,
    );
    
    cache.lock().unwrap().insert(
        host.clone(),
        (CommandType::DockerInstall, result)
    );
});
```

- [ ] **Step 9: Verify compilation**

Run: `cd mobile && cargo check --lib`

Expected: Success

- [ ] **Step 10: Manual integration test**

Run: `cd mobile && cargo run --bin dure-desktop`

Test workflow:
1. Add SSH host
2. Open drawer, click Linux tab Refresh
3. Verify output appears in terminal
4. Click Docker tab Refresh (should show error if not installed)
5. Click Install Docker
6. Verify Docker tab becomes enabled after install

- [ ] **Step 11: Commit**

```bash
git add mobile/src/ui_tabs/ssh.rs
git commit -m "feat(ssh): integrate command execution results

Add result cache to store command outputs from background threads.
Process completed commands and update row data (output, error,
timestamp, flags).

Full integration: Linux refresh, Docker refresh, Dure refresh,
and Docker install now update UI when commands complete.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 18: Service Detection Integration

**Files:**
- Modify: `mobile/src/ui_tabs/ssh.rs` (add detection commands)

**Interfaces:**
- Consumes: SSH command execution
- Produces: Updated `docker_enabled` and `dure_wss_enabled` flags

- [ ] **Step 1: Add detection command types**

Add to CommandType enum:

```rust
DockerDetect,
DureDetect,
```

- [ ] **Step 2: Add detection commands to load_rows**

Find `load_rows()` method and add detection commands after loading SSH hosts:

```rust
// Detect Docker and Dure for each host
for row in &mut self.rows {
    if let Ok(cfg) = AppConfig::load() {
        if let Some(ssh_cfg) = cfg.ssh_hosts.iter().find(|s| s.host == row.host) {
            let host = ssh_cfg.host.clone();
            let port = ssh_cfg.port;
            let auth = ssh::auth_from_config(ssh_cfg);
            
            // Docker detection
            let cache = self.command_results.clone();
            let host_clone = host.clone();
            std::thread::spawn(move || {
                let result = ssh::run_command(
                    &host_clone,
                    port,
                    "command -v docker &> /dev/null && docker --version",
                    auth.clone(),
                );
                
                cache.lock().unwrap().insert(
                    host_clone,
                    (CommandType::DockerDetect, result)
                );
            });
            
            // Dure detection (only if Docker detected)
            if row.docker_enabled {
                let cache = self.command_results.clone();
                std::thread::spawn(move || {
                    let result = ssh::run_command(
                        &host,
                        port,
                        "docker ps -a | grep xmpp-proxy-stack | wc -l",
                        auth,
                    );
                    
                    cache.lock().unwrap().insert(
                        host,
                        (CommandType::DureDetect, result)
                    );
                });
            }
        }
    }
}
```

- [ ] **Step 3: Add detection result processing**

Add to command result processing in ui() method:

```rust
CommandType::DockerDetect => {
    // If command succeeded (exit code 0), Docker is installed
    if result.is_ok() {
        row.docker_enabled = true;
    } else {
        row.docker_enabled = false;
    }
}
CommandType::DureDetect => {
    // If count > 0, Dure is running
    if let Ok(count_str) = result {
        if let Ok(count) = count_str.trim().parse::<i32>() {
            row.dure_wss_enabled = count > 0;
        }
    }
}
```

- [ ] **Step 4: Verify compilation**

Run: `cd mobile && cargo check --lib`

Expected: Success

- [ ] **Step 5: Commit**

```bash
git add mobile/src/ui_tabs/ssh.rs
git commit -m "feat(ssh): add Docker and Dure service detection

Automatically detect Docker installation and Dure containers
on host load using background SSH commands.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 19: Auto-Refresh on Drawer Open

**Files:**
- Modify: `mobile/src/ui_tabs/ssh.rs` (add drawer open detection and auto-refresh)

**Interfaces:**
- Consumes: Drawer state from data table
- Produces: Auto-refresh trigger for selected tab

- [ ] **Step 1: Add drawer state tracking to SshTab**

Add fields:

```rust
#[cfg_attr(feature = "serde", serde(skip))]
drawer_open_state: HashMap<String, bool>,  // host -> is_open

#[cfg_attr(feature = "serde", serde(skip))]
drawer_auto_refreshed: HashSet<String>,    // hosts that were auto-refreshed
```

Initialize in Default:

```rust
drawer_open_state: HashMap::new(),
drawer_auto_refreshed: HashSet::new(),
```

- [ ] **Step 2: Add imports**

```rust
use std::collections::HashSet;
```

- [ ] **Step 3: Detect drawer open state in ui() method**

Add after table rendering, before action handlers:

```rust
// Track drawer open state and trigger auto-refresh
let table_id = egui::Id::new("ssh_table");
if let Some(state) = ui.data(|d| d.get_persisted::<egui_material3::datatable::DataTableState>(table_id)) {
    for (idx, row) in self.rows.iter_mut().enumerate() {
        let is_open = state.drawers.contains(&idx);
        let was_open = self.drawer_open_state.get(&row.host).copied().unwrap_or(false);
        
        // Drawer just opened
        if is_open && !was_open {
            // Get selected tab for this drawer
            let selected_tab = ui.data(|d| 
                d.get_temp::<usize>(egui::Id::new(format!("ssh_tab_{}", idx)))
                    .unwrap_or(0)
            );
            
            // Auto-refresh if not yet refreshed for this drawer open
            let key = format!("{}-{}", row.host, idx);
            if !self.drawer_auto_refreshed.contains(&key) {
                // Check if we should refresh (no data or stale)
                let should_refresh = match selected_tab {
                    0 => {
                        if row.linux_output.is_none() {
                            true
                        } else if let Some(last) = row.linux_last_refresh {
                            let elapsed = chrono::Utc::now().timestamp() - last;
                            elapsed > 3600  // 1 hour
                        } else {
                            false
                        }
                    }
                    1 => row.docker_output.is_none(),
                    2 => row.dure_output.is_none(),
                    _ => false,
                };
                
                if should_refresh {
                    // Trigger refresh for this tab
                    ui.data_mut(|d| {
                        let flag_id = match selected_tab {
                            0 => format!("ssh_linux_refresh_{}", idx),
                            1 => format!("ssh_docker_refresh_{}", idx),
                            2 => format!("ssh_dure_refresh_{}", idx),
                            _ => String::new(),
                        };
                        
                        if !flag_id.is_empty() {
                            d.insert_temp(egui::Id::new(flag_id), row.host.clone());
                        }
                    });
                    
                    self.drawer_auto_refreshed.insert(key);
                }
            }
        }
        
        // Drawer closed
        if !is_open && was_open {
            // Clear auto-refresh flag when drawer closes
            let key = format!("{}-{}", row.host, idx);
            self.drawer_auto_refreshed.remove(&key);
        }
        
        // Update state
        self.drawer_open_state.insert(row.host.clone(), is_open);
    }
}
```

- [ ] **Step 4: Verify compilation**

Run: `cd mobile && cargo check --lib`

Expected: Success

- [ ] **Step 5: Manual test**

Run: `cd mobile && cargo run --bin dure-desktop`

1. Add SSH host
2. Open drawer - should auto-refresh Linux tab
3. Close drawer, reopen - should NOT auto-refresh again (unless >1 hour passed)
4. Switch to Docker tab, close/reopen - should auto-refresh Docker tab

- [ ] **Step 6: Commit**

```bash
git add mobile/src/ui_tabs/ssh.rs
git commit -m "feat(ssh): add auto-refresh on drawer open

Automatically refresh selected tab when drawer opens if:
- No data cached, OR
- Data is stale (>1 hour old for Linux tab)

Only auto-refresh once per drawer open session.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 20: Documentation and Final Testing

**Files:**
- Modify: `mobile/src/ui_components/terminal_output.rs` (add doc comments)
- Modify: `mobile/src/ui_components/emoji_progress.rs` (add doc comments)
- Modify: `mobile/src/ui_tabs/ssh.rs` (add doc comments for new functions)

**Interfaces:**
- Consumes: None
- Produces: Documented code

- [ ] **Step 1: Add module documentation to terminal_output.rs**

Add at top of file:

```rust
//! Terminal output display component
//!
//! Provides a monospace, scrollable display for command output with:
//! - Error mode (red text for stderr)
//! - Loading indicator (spinner)
//! - Configurable max height
//!
//! # Example
//!
//! ```no_run
//! use crate::ui_components::TerminalOutput;
//!
//! // Normal output
//! TerminalOutput::new("command output")
//!     .with_max_height(250.0)
//!     .show(ui);
//!
//! // Error output
//! TerminalOutput::new("error message")
//!     .error(true)
//!     .show(ui);
//! ```
```

- [ ] **Step 2: Add doc comments to EmojiProgressBar::from_ssh_row**

Add before method:

```rust
/// Create progress bar from SSH host data
///
/// Shows Linux → Docker → Dure progression:
/// - ⚪⚪⚪ = None detected
/// - ✅⚪⚪ = Linux only
/// - ✅✅⚪ = Linux + Docker
/// - ✅✅✅ = Linux + Docker + Dure
///
/// # Example
///
/// ```no_run
/// let progress = EmojiProgressBar::from_ssh_row(&row)
///     .compact(true);
/// progress.show(ui);
/// ```
```

- [ ] **Step 3: Add doc comments to tab render functions**

Add before each function in ssh.rs:

```rust
/// Render Linux tab content
///
/// Displays:
/// - System information from bash script
/// - Staleness warning if data > 1 hour old
/// - Refresh button
/// - Install Docker button (if not installed)
```

```rust
/// Render Docker tab content
///
/// Displays:
/// - docker ps -a output
/// - Refresh button
/// - Install Dure button (if not running)
```

```rust
/// Render Dure tab content  
///
/// Displays:
/// - ss -nltup output
/// - Refresh button
```

- [ ] **Step 4: Run all unit tests**

Run: `cd mobile && cargo test --lib`

Expected: All tests pass

- [ ] **Step 5: Run manual testing checklist**

From design spec section "Manual Testing Checklist":

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

- [ ] **Step 6: Check for clippy warnings**

Run: `cd mobile && cargo clippy --lib`

Expected: No new warnings

- [ ] **Step 7: Commit**

```bash
git add mobile/src/ui_components/terminal_output.rs \
        mobile/src/ui_components/emoji_progress.rs \
        mobile/src/ui_tabs/ssh.rs
git commit -m "docs(ssh): add inline documentation for drawer refactor

Add module docs, function docs, and usage examples for:
- TerminalOutput component
- EmojiProgressBar SSH integration
- Tab rendering functions

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Self-Review Checklist

After completing all tasks, verify:

**Spec Coverage:**
- [x] FR1: Status column emoji progress - Task 5
- [x] FR2: 3 compact tabs in drawer - Tasks 6-8
- [x] FR3: Terminal-style output - Tasks 2, 9-11
- [x] FR4: Linux tab (info + buttons) - Task 9
- [x] FR5: Docker tab (ps + buttons) - Task 10
- [x] FR6: Dure tab (ss + button) - Task 11
- [x] FR7: Tab auto-enable - Tasks 6-8, 18
- [x] FR8: Auto-refresh on open - Task 19
- [x] FR9: Per-tab refresh - Tasks 12-14
- [x] FR10: Button loading states - Tasks 9-11

- [x] NFR1: Reuse components - Tasks 3-5
- [x] NFR2: Monospace scrollable - Task 2
- [x] NFR3: Raw errors - Task 2, Tasks 9-11
- [x] NFR4: Cross-platform - All tasks
- [x] NFR5: No ViewModel changes - All tasks

**No Placeholders:**
- All code blocks complete ✅
- All commands have exact paths ✅
- All tests have assertions ✅
- Dure install marked as placeholder (intentional) ✅

**Type Consistency:**
- SshRowData fields match across tasks ✅
- TerminalOutput API consistent ✅
- CommandType enum complete ✅
- Function signatures match ✅

**Implementation Order:**
1. Data model (Task 1) ✅
2. Components (Tasks 2-5) ✅
3. UI structure (Tasks 6-8) ✅
4. Tab content (Tasks 9-11) ✅
5. Actions (Tasks 12-16) ✅
6. Integration (Tasks 17-19) ✅
7. Documentation (Task 20) ✅

---

## Execution Notes

**Estimated Time:** 20 tasks × 20-30 minutes average = ~8-10 hours

**Critical Path:**
1. Data model (Task 1) - foundation
2. TerminalOutput (Task 2) - UI dependency
3. Tab structure (Tasks 6-8) - visual framework
4. Command integration (Task 17) - functional core

**Testing Strategy:**
- Unit tests after each component (Tasks 2-3)
- Manual tests after UI changes (Tasks 5, 8, 19)
- Full integration test at end (Task 20)

**Rollback Points:**
- After Task 5: Status column works, can revert drawer
- After Task 8: Tabs work, can revert command execution
- After Task 17: Full integration, can tune auto-refresh

---

## Post-Implementation

After completing all tasks:

1. **Merge to main:**
   ```bash
   git checkout main
   git pull origin main
   git merge feature/ssh-drawer-refactor
   git push origin main
   ```

2. **Monitor for issues:**
   - Watch for SSH timeout reports
   - Check drawer performance on many hosts
   - Verify tab state persistence

3. **Future enhancements:**
   - Event-based architecture (replace polling)
   - Actual Dure install implementation
   - Live log streaming
   - Service management buttons
