# SSH Tab Redesign - Comprehensive Host Management

**Date:** 2026-10-02  
**Author:** Claude Sonnet 4.5  
**Status:** Draft  
**Architecture Path:** Modified Approach B (Unified Drawer Components)

---

## Executive Summary

Redesign the SSH tab to match the Platform tab's UX pattern with comprehensive host management operations. Implements 12 SSH operations (Refresh, SSH Check, Edit, Delete, Check/Install/Remove Base/Docker/Dure), unified drawer system with 6 tabs, operation logging infrastructure, and consistent UI spacing.

**Key Objectives:**
1. Unified drawer component architecture (reusable across Platform/SSH/future tabs)
2. 12 SSH operations with proper error handling and logging
3. 6 drawer tabs: Status, Logs, Operations, Host, Docker, Dure
4. Consistent button padding and layout with Platform tab
5. Database migration for operation source tracking (platform vs SSH)

---

## 1. Database Schema Changes

### Current OperationLog Schema

```rust
pub struct OperationLog {
    pub id: i64,
    pub project_id: String,           // Currently: GCP project ID
    pub operation_type: String,        // e.g., "create_vm", "update_firewall"
    pub external_system: String,       // e.g., "gcp", "cloudflare"
    pub status: String,                // "running", "success", "failed"
    pub started_at: i64,
    pub completed_at: Option<i64>,
    pub error_message: Option<String>,
    pub details: Option<String>,
}
```

### New Schema (Extended)

```rust
pub struct OperationLog {
    pub id: i64,
    // NEW: Distinguish platform vs SSH operations
    pub operation_source: String,      // "platform" | "ssh"
    // NEW: Unified identifier (platform_id OR ssh_host)
    pub relevant_id: String,           // "gcp-project-123" or "192.168.1.100"
    
    // Existing fields (unchanged)
    pub operation_type: String,        // "create_vm", "check_base", "install_docker"
    pub external_system: String,       // "gcp", "ssh", "cloudflare"
    pub status: String,
    pub started_at: i64,
    pub completed_at: Option<i64>,
    pub error_message: Option<String>,
    pub details: Option<String>,
}
```

### Migration Strategy

**SQL Migration:**
```sql
-- Step 1: Add new columns (nullable initially)
ALTER TABLE operation_logs ADD COLUMN operation_source TEXT;
ALTER TABLE operation_logs ADD COLUMN relevant_id TEXT;

-- Step 2: Backfill existing records (all are platform operations)
UPDATE operation_logs 
SET operation_source = 'platform', 
    relevant_id = project_id;

-- Step 3: Make columns non-nullable
ALTER TABLE operation_logs ALTER COLUMN operation_source SET NOT NULL;
ALTER TABLE operation_logs ALTER COLUMN relevant_id SET NOT NULL;

-- Step 4: Add index for efficient filtering
CREATE INDEX idx_operation_logs_source_id 
ON operation_logs(operation_source, relevant_id);
```

**Rust Migration File:** `mobile/src/storage/migrations/YYYYMMDD_add_operation_source.rs`

### Helper Functions

```rust
impl NewOperationLog {
    // For platform operations
    pub fn platform(platform_id: impl Into<String>, operation_type: impl Into<String>) -> Self {
        Self {
            operation_source: "platform".to_string(),
            relevant_id: platform_id.into(),
            operation_type: operation_type.into(),
            external_system: "gcp".to_string(),
            status: OperationStatus::Running,
            error_message: None,
            details: None,
        }
    }
    
    // For SSH operations
    pub fn ssh(ssh_host: impl Into<String>, operation_type: impl Into<String>) -> Self {
        Self {
            operation_source: "ssh".to_string(),
            relevant_id: ssh_host.into(),
            operation_type: operation_type.into(),
            external_system: "ssh".to_string(),
            status: OperationStatus::Running,
            error_message: None,
            details: None,
        }
    }
}

// Query helpers
pub fn get_ssh_operations(ssh_host: &str, limit: i64) -> Result<Vec<OperationLog>> {
    // WHERE operation_source='ssh' AND relevant_id=?
}

pub fn get_platform_operations(platform_id: &str, limit: i64) -> Result<Vec<OperationLog>> {
    // WHERE operation_source='platform' AND relevant_id=?
}
```

### Backward Compatibility

**Platform code changes:**
- Update `viewmodel/platform/actor.rs` to use `NewOperationLog::platform()`
- No functional changes, just constructor swap
- Existing queries still work (filter by relevant_id)

---

## 2. Shared Drawer Components Architecture

### Component Structure

**New directory:** `mobile/src/ui_components/drawer/`

```
ui_components/
├── drawer/
│   ├── mod.rs              // Public exports
│   ├── tab_bar.rs          // Reusable tab switcher
│   ├── status_utils.rs     // Status display helpers (no emojis)
│   ├── logs_renderer.rs    // Log scrolling + auto-refresh
│   └── operations_renderer.rs  // Operation history table
├── action_menu.rs          // Existing (reused)
├── mod.rs
└── ...
```

### TabBar Component (Trait-Based)

```rust
// ui_components/drawer/tab_bar.rs
pub struct TabBar<T: DrawerTabTrait> {
    spacing: f32,
}

pub trait DrawerTabTrait: Copy + PartialEq {
    fn as_str(&self) -> &'static str;
    fn all() -> &'static [Self];
}

impl<T: DrawerTabTrait> TabBar<T> {
    pub fn new() -> Self {
        Self { spacing: 8.0 }
    }
    
    pub fn show(&self, ui: &mut egui::Ui, current_tab: T) -> Option<T> {
        let mut clicked_tab = None;
        
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = self.spacing;
            
            for tab in T::all() {
                let is_selected = current_tab == *tab;
                if ui.selectable_label(is_selected, tab.as_str()).clicked() && !is_selected {
                    clicked_tab = Some(*tab);
                }
            }
        });
        
        clicked_tab
    }
}
```

**Usage Pattern:**
```rust
// Platform drawer implements trait
impl DrawerTabTrait for platform::DrawerTab {
    fn as_str(&self) -> &'static str { /* existing */ }
    fn all() -> &'static [Self] { 
        &[Self::Status, Self::Logs, Self::Operations]
    }
}

// SSH drawer implements trait
impl DrawerTabTrait for ssh::DrawerTab {
    fn as_str(&self) -> &'static str { /* existing */ }
    fn all() -> &'static [Self] {
        &[Self::Status, Self::Logs, Self::Operations, 
          Self::Host, Self::Docker, Self::Dure]
    }
}
```

### StatusUtils Module (No Emojis)

```rust
// ui_components/drawer/status_utils.rs
use egui::{RichText, Color32, Ui};

pub struct StatusLine {
    label: String,
    value: String,
    color: Option<Color32>,
}

impl StatusLine {
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            color: None,
        }
    }
    
    pub fn warning(mut self) -> Self {
        self.color = Some(Color32::from_rgb(255, 193, 7));
        self
    }
    
    pub fn error(mut self) -> Self {
        self.color = Some(Color32::from_rgb(244, 67, 54));
        self
    }
    
    pub fn show(&self, ui: &mut Ui) {
        let text = format!("{}: {}", self.label, self.value);
        if let Some(color) = self.color {
            ui.label(RichText::new(text).color(color));
        } else {
            ui.label(text);
        }
        ui.add_space(4.0);
    }
}

pub fn format_elapsed(seconds: i64) -> String {
    if seconds < 60 {
        "just now".to_string()
    } else if seconds < 3600 {
        format!("{} min ago", seconds / 60)
    } else if seconds < 86400 {
        format!("{} hours ago", seconds / 3600)
    } else {
        format!("{} days ago", seconds / 86400)
    }
}

pub fn staleness_color(seconds: i64, ui: &Ui) -> Color32 {
    if seconds < 3600 {
        ui.style().visuals.text_color()
    } else {
        Color32::from_rgb(255, 193, 7)
    }
}
```

### LogsRenderer Component

```rust
// ui_components/drawer/logs_renderer.rs
use egui::{ScrollArea, Ui, Id, RichText, Color32};

pub struct LogsRenderer {
    title: String,
    refresh_interval_secs: u64,
    max_height: f32,
}

impl LogsRenderer {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            refresh_interval_secs: 1,
            max_height: 400.0,
        }
    }
    
    pub fn refresh_interval(mut self, secs: u64) -> Self {
        self.refresh_interval_secs = secs;
        self
    }
    
    pub fn show(&self, ui: &mut Ui, logs: &[String], refresh_trigger_id: Id) -> bool {
        let mut should_refresh = false;
        
        // Auto-refresh logic
        let refresh_id = ui.id().with("log_refresh");
        let now = std::time::Instant::now();
        should_refresh = ui.data(|d| {
            d.get_temp::<std::time::Instant>(refresh_id)
                .map(|last| now.duration_since(last).as_secs() >= self.refresh_interval_secs)
                .unwrap_or(true)
        });
        
        if should_refresh {
            ui.data_mut(|d| {
                d.insert_temp(refresh_id, now);
                d.insert_temp(refresh_trigger_id, ());
            });
        }
        
        // Header with new logs indicator
        ui.horizontal(|ui| {
            ui.heading(&self.title);
            ui.add_space(8.0);
            
            let log_count_id = ui.id().with("prev_log_count");
            let current_count = logs.len();
            let prev_count = ui.data(|d| d.get_temp::<usize>(log_count_id).unwrap_or(0));
            
            if current_count > prev_count {
                let new_count = current_count - prev_count;
                ui.label(
                    RichText::new(format!("▲ {} new", new_count))
                        .color(Color32::from_rgb(76, 175, 80))
                );
                ui.data_mut(|d| d.insert_temp(log_count_id, current_count));
            }
        });
        
        ui.separator();
        
        ScrollArea::vertical()
            .max_height(self.max_height)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                if logs.is_empty() {
                    ui.label("No logs yet");
                } else {
                    for line in logs {
                        ui.label(line);
                    }
                }
            });
        
        should_refresh
    }
}
```

### OperationsRenderer Component

```rust
// ui_components/drawer/operations_renderer.rs
use crate::storage::models::opslog::{OperationLog, OperationStatus};
use egui::{Color32, RichText, Ui};

pub struct OperationsRenderer {
    max_height: f32,
}

impl OperationsRenderer {
    pub fn new() -> Self {
        Self { max_height: 400.0 }
    }
    
    pub fn show(&self, ui: &mut Ui, operations: &[OperationLog]) {
        ui.heading("Operation History");
        ui.separator();
        
        egui::ScrollArea::vertical()
            .max_height(self.max_height)
            .show(ui, |ui| {
                if operations.is_empty() {
                    ui.label("No operations yet");
                    return;
                }
                
                for op in operations {
                    self.render_operation(ui, op);
                    ui.separator();
                }
            });
    }
    
    fn render_operation(&self, ui: &mut Ui, op: &OperationLog) {
        let status = op.status();
        let (status_text, status_color) = match status {
            OperationStatus::Running => ("RUNNING", Color32::from_rgb(33, 150, 243)),
            OperationStatus::Success => ("SUCCESS", Color32::from_rgb(76, 175, 80)),
            OperationStatus::Failed => ("FAILED", Color32::from_rgb(244, 67, 54)),
        };
        
        ui.horizontal(|ui| {
            ui.label(RichText::new(status_text).color(status_color));
            ui.label(&op.operation_type);
            
            if let Some(duration) = op.duration_ms() {
                ui.label(format!("({}ms)", duration));
            }
        });
        
        ui.label(format!("System: {}", op.external_system));
        
        if let Some(error) = &op.error_message {
            ui.label(RichText::new(format!("Error: {}", error)).color(status_color));
        }
        
        if let Some(details) = &op.details {
            ui.label(format!("Details: {}", details));
        }
    }
}
```

---

## 3. SSH ViewModel Expansion

### Commands Extension

**File:** `mobile/src/viewmodel/ssh/commands.rs`

```rust
#[derive(Debug, Clone)]
pub enum SshCommand {
    // Existing commands
    LoadHosts,
    AddHost { /* ... */ },
    UpdateHost { /* ... */ },
    DeleteHost { host: String },
    
    // NEW: Operation commands
    Refresh { host: String },
    SshCheck { host: String },
    CheckBase { host: String },
    InstallBase { host: String },
    CheckDocker { host: String },
    InstallDocker { host: String },
    RemoveDocker { host: String },
    CheckDure { host: String },
    InstallDure { 
        host: String, 
        env_config: std::collections::HashMap<String, String>,
    },
    RemoveDure { host: String },
    
    // NEW: Drawer commands
    Drawer(DrawerCommand),
}
```

### Events Extension

**File:** `mobile/src/viewmodel/ssh/events.rs`

```rust
#[derive(Debug, Clone)]
pub enum SshEvent {
    // Existing events
    HostsLoaded { hosts: Vec<SshHostInfo> },
    HostAdded { host: String },
    HostUpdated { host: String },
    HostDeleted { host: String },
    
    // NEW: Operation events
    RefreshStarted { host: String },
    RefreshCompleted {
        host: String,
        ssh_connected: bool,
        base_installed: bool,
        docker_installed: bool,
        dure_installed: bool,
    },
    
    SshCheckCompleted {
        host: String,
        connected: bool,
        error: Option<String>,
    },
    
    BaseCheckCompleted {
        host: String,
        installed: bool,
        missing_packages: Vec<String>,
    },
    
    BaseInstallCompleted {
        host: String,
        success: bool,
        error: Option<String>,
    },
    
    DockerCheckCompleted {
        host: String,
        installed: bool,
        version: Option<String>,
    },
    
    DockerInstallCompleted {
        host: String,
        success: bool,
        error: Option<String>,
    },
    
    DockerRemoveCompleted {
        host: String,
        success: bool,
        error: Option<String>,
    },
    
    DureCheckCompleted {
        host: String,
        installed: bool,
        running: bool,
        services: Vec<String>,
    },
    
    DureInstallCompleted {
        host: String,
        success: bool,
        error: Option<String>,
    },
    
    DureRemoveCompleted {
        host: String,
        success: bool,
        error: Option<String>,
    },
    
    OperationStarted {
        host: String,
        operation: String,
    },
    
    OperationFailed {
        host: String,
        operation: String,
        error: String,
    },
    
    Drawer(DrawerEvent),
    Error { operation: String, error: String },
}
```

### SshRow State Extension

**File:** `mobile/src/ui_tabs/ssh.rs`

```rust
#[derive(Clone, Debug)]
pub struct SshRow {
    // Existing fields
    pub host: String,
    pub port: u16,
    pub platform_id: Option<String>,
    pub ssh_connected: bool,
    pub base_installed: bool,
    pub docker_installed: bool,
    pub dure_installed: bool,
    pub ssh_private_key: Option<String>,
    pub drawer_open: bool,
    pub drawer_state: DrawerState,
    
    // NEW: Operation state tracking
    pub operation_state: OperationState,
    
    // NEW: Last check results for Status tab
    pub last_ssh_check: Option<SshCheckResult>,
    pub last_base_check: Option<BaseCheckResult>,
    pub last_docker_check: Option<DockerCheckResult>,
    pub last_dure_check: Option<DureCheckResult>,
}

#[derive(Clone, Debug)]
pub struct SshCheckResult {
    pub connected: bool,
    pub error: Option<String>,
    pub checked_at: i64,
}

#[derive(Clone, Debug)]
pub struct BaseCheckResult {
    pub installed: bool,
    pub missing_packages: Vec<String>,
    pub checked_at: i64,
}

#[derive(Clone, Debug)]
pub struct DockerCheckResult {
    pub installed: bool,
    pub version: Option<String>,
    pub checked_at: i64,
}

#[derive(Clone, Debug)]
pub struct DureCheckResult {
    pub installed: bool,
    pub running: bool,
    pub services: Vec<String>,
    pub checked_at: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OperationState {
    Idle,
    InProgress { operation: String, started_at: i64 },
    Completed { operation: String, completed_at: i64 },
    Failed { operation: String, error: String, failed_at: i64 },
}
```

---

## 4. SSH Operations (Calc Layer)

### Calc Module Structure

**New file:** `mobile/src/calc/ssh_ops.rs`

All operations use `russh` for remote execution, `anyhow` for error handling, and operation logging via `dure_info!` macro.

### Core Operations

#### 1. SSH Connection Check
```rust
pub async fn check_ssh_connection(host: &str, port: u16) -> Result<bool>
```
- Simple `echo ok` command
- Returns true if connection succeeds
- Logs error on failure

#### 2. Check Base Packages
```rust
pub async fn check_base_packages(host: &str) -> Result<Vec<String>>
```
- Required: extrepo, git, iptables, nftables, bpfcc-tools, moreutils
- Returns list of missing packages
- Uses `dpkg -l | grep` for detection

#### 3. Install Base Packages
```rust
pub async fn install_base_packages(host: &str) -> Result<()>
```
- Installs all base packages via apt-get
- Configures network logging: `tcpconnect-bpfcc | ts >> /var/log/dure-network.log`
- Downloads rc.local template from pastebin
- All-in-one operation (network logging included)

#### 4. Check Docker
```rust
pub async fn check_docker_installed(host: &str) -> Result<Option<String>>
```
- Returns docker version if installed
- None if not installed

#### 5. Install Docker
```rust
pub async fn install_docker(host: &str) -> Result<()>
```
- Enables extrepo docker-ce
- Installs docker-ce
- Creates docker user and group
- Configures systemd service

#### 6. Remove Docker
```rust
pub async fn remove_docker(host: &str) -> Result<()>
```
- Stops and disables service
- Purges docker-ce packages
- Runs autoremove

#### 7. Check Dure
```rust
pub async fn check_dure_installed(host: &str) -> Result<(bool, bool, Vec<String>)>
```
- Checks /srv/dure-mycart exists
- Runs docker-compose ps
- Returns (installed, running, services)

#### 8. Install Dure
```rust
pub async fn install_dure(
    host: &str,
    env_config: &HashMap<String, String>,
) -> Result<()>
```
- Clones dure-mycart repository to /srv
- Creates .env file from config
- Runs docker-compose up -d as docker user

#### 9. Remove Dure
```rust
pub async fn remove_dure(host: &str) -> Result<()>
```
- Runs docker-compose down

### Drawer Data Fetchers

#### 10. Get Host Info
```rust
pub async fn get_host_info(host: &str) -> Result<HostInfo>
```
- OS version, uptime, load average, memory, disk usage
- For Host tab display

#### 11. Get Docker Containers
```rust
pub async fn get_docker_containers(host: &str) -> Result<String>
```
- Raw `docker ps -a` output
- For Docker tab display (monospace)

#### 12. Get Dure Compose Status
```rust
pub async fn get_dure_compose_status(host: &str) -> Result<(String, String)>
```
- `docker-compose ps` + `docker-compose logs --tail=50`
- For Dure tab display

#### 13. Get Network Log
```rust
pub async fn get_network_log(host: &str, lines: usize) -> Result<String>
```
- Fetches /var/log/dure-network.log (last 100 lines)
- For Host tab display

### Error Handling Pattern

All operations:
- Start with `log_operation_start(host, operation_type)`
- Execute via russh
- Complete with `log_operation_complete(op_id, success, error)`
- Return `Result<T>` for proper error propagation

### Retry Logic

Critical operations (Install Base, Install Docker) use retry helper:
```rust
async fn execute_ssh_command_with_retry(
    host: &str,
    command: &str,
    max_retries: u32,
) -> Result<String>
```
- 3 retries with 2-second delay
- For transient network failures

---

## 5. SSH Drawer Tabs

### Tab 1: Status (Inline Errors, No Emojis)

**Features:**
- IP address display
- SSH connection status with inline error
- Base/Docker/Dure installation status
- Missing packages list if available
- Operation state (InProgress/Completed/Failed)
- SSH action menu (copy key, copy command)

**Error Display:**
```
SSH: Failed (connection refused)
Base: Not Installed
  Missing: git, iptables, bpfcc-tools
Operation: Install Base failed
  Error: apt-get update returned exit code 100
```

### Tab 2: Logs (Auto-Refresh)

**Features:**
- Uses shared `LogsRenderer` component
- Auto-refresh every 1 second
- New logs indicator
- Filters stdout logs by SSH host
- Stick-to-bottom scrolling

### Tab 3: Operations (History Table)

**Features:**
- Uses shared `OperationsRenderer` component
- Shows operation history filtered by `operation_source='ssh'` AND `relevant_id=host`
- Color-coded status (Running/Success/Failed)
- Duration display
- Error messages inline

### Tab 4: Host (System Info + Network Log)

**Features:**
- System information (OS, uptime, load, memory, disk)
- Network log display (/var/log/dure-network.log)
- Static snapshot (manual refresh button)
- Always fetches latest 100 lines
- Monospace display for log content

### Tab 5: Docker (Raw Output)

**Features:**
- Docker version display
- Raw `docker ps -a` output (monospace)
- Manual refresh button
- No table formatting (show actual terminal output)

### Tab 6: Dure (Status + Logs)

**Features:**
- Installation status
- Running status
- `docker-compose ps` output (monospace)
- `docker-compose logs --tail=50` output (monospace)
- Manual refresh button
- Split display: status section + logs section

### DrawerState Extension

```rust
pub struct DrawerState {
    pub active_tab: DrawerTab,
    pub ssh_host: Option<String>,
    pub operations: Vec<OperationLog>,
    pub logs: Vec<String>,
    pub loading: bool,
    pub host_info: Option<HostInfo>,
    pub docker_status: Option<DockerStatus>,
    pub containers: Vec<ContainerInfo>,
    pub dure_status: Option<DureStatus>,
    
    // NEW fields for additional tabs
    pub network_log: Vec<String>,
    pub docker_ps_raw: Option<String>,
    pub dure_compose_status: Option<String>,
    pub dure_compose_logs: Vec<String>,
}
```

---

## 6. UI Integration

### Operations Column Layout

**3 rows of buttons, 100px min_row_height (from recent commit):**

**Row 1: Basic Operations**
- Refresh (always)
- SSH Check (always)
- Edit (always)
- Delete (always)

**Row 2: Base + Docker**
- Check Base (always)
- Install Base (only when NOT installed)
- Check Docker (always)
- Install Docker (only when NOT installed)
- Remove Docker (only when installed)

**Row 3: Dure**
- Check Dure (always)
- Install Dure (only when NOT installed)
- Remove Dure (only when installed)

### Button Padding (Platform Tab Standard)

```rust
const BUTTON_VERTICAL_SPACING: f32 = 4.0;    // From platform tab
const BUTTON_HORIZONTAL_SPACING: f32 = 4.0;  // From platform tab
```

Applied to:
- Between button rows
- Between buttons in same row
- Consistent with platform tab spacing

### Platform Badge Display

**Two-line layout in Host column:**
- **Line 1:** Platform badge (if platform_id exists)
- **Line 2:** Host IP address

Uses `egui_material3::badge`:
```rust
badge::show(ui, platform_id, BadgeColor::Primary, BadgeSize::Small);
```

### Status Column (Simple Indicators)

- Colored dot for SSH connection (green=connected, gray=disconnected)
- Text-only status for Base/Docker/Dure (no emojis)
- Warning color for "Missing" states

### Install Dure Dialog

**Two-stage process:**

**Stage 1: Clone Repository**
- Shows spinner
- Background task clones dure-mycart
- Parses .env.example into key-value pairs

**Stage 2: Edit Configuration**
- Generic key-value editor (egui::Grid)
- Pre-populated with .env.example values
- User edits values in text fields
- Install button triggers operation with HashMap

**Implementation:**
```rust
fn render_install_dure_dialog(ui: &mut egui::Ui, state: &mut SshTab) {
    if state.install_dure_env_content.is_empty() {
        // Stage 1: Cloning
        ui.label("Cloning repository...");
        ui.spinner();
    } else {
        // Stage 2: Editing
        let env_vars: Vec<(String, String)> = parse_env(state.install_dure_env_content);
        
        egui::Grid::new("env_grid").show(ui, |ui| {
            for (key, value) in &mut env_vars {
                ui.label(key);
                ui.text_edit_singleline(value);
                ui.end_row();
            }
        });
        
        if ui.button("Install").clicked() {
            let config: HashMap<String, String> = env_vars.into_iter().collect();
            // Trigger InstallDure command
        }
    }
}
```

---

## 7. Error Handling & Operation State

### Three-Tier Error Display

1. **Inline Status Tab**
   - Immediate visual feedback
   - Error appears where status normally shows
   - Example: "SSH: Failed (connection refused)"

2. **Operations Tab**
   - Full error details with timestamps
   - Duration display
   - Color-coded status (Running/Success/Failed)

3. **OperationLog Database**
   - Persistent audit trail
   - Queryable by operation_source and relevant_id
   - Includes error_message and details fields

### Operation State Machine

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum OperationState {
    Idle,
    InProgress { operation: String, started_at: i64 },
    Completed { operation: String, completed_at: i64 },
    Failed { operation: String, error: String, failed_at: i64 },
}

impl OperationState {
    pub fn start(operation: impl Into<String>) -> Self;
    pub fn complete(self) -> Self;
    pub fn fail(self, error: impl Into<String>) -> Self;
    pub fn reset(&mut self);
}
```

### Error Handling Flow

**In SshActor:**
```rust
async fn handle_install_base(&mut self, host: String) -> SshEvent {
    // 1. Start operation logging
    let op_id = self.start_operation_log(&host, "install_base").await?;
    
    // 2. Execute operation
    match calc::ssh_ops::install_base_packages(&host).await {
        Ok(()) => {
            // 3a. Complete successfully
            self.complete_operation_log(op_id, true, None).await?;
            SshEvent::BaseInstallCompleted { host, success: true, error: None }
        }
        Err(e) => {
            // 3b. Complete with error
            let error_msg = e.to_string();
            self.complete_operation_log(op_id, false, Some(&error_msg)).await?;
            SshEvent::BaseInstallCompleted { host, success: false, error: Some(error_msg) }
        }
    }
}
```

**In UI (event handler):**
```rust
SshEvent::BaseInstallCompleted { host, success, error } => {
    if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
        if success {
            row.base_installed = true;
            row.operation_state = row.operation_state.clone().complete();
        } else {
            row.operation_state = row.operation_state.clone()
                .fail(error.unwrap_or("Unknown error".to_string()));
        }
    }
}
```

### Retry Logic

For critical operations with potential transient failures:

```rust
async fn execute_ssh_command_with_retry(
    host: &str,
    command: &str,
    max_retries: u32,
) -> Result<String> {
    let mut last_error = None;
    
    for attempt in 0..max_retries {
        match execute_ssh_command(host, command).await {
            Ok(output) => return Ok(output),
            Err(e) => {
                last_error = Some(e);
                if attempt < max_retries - 1 {
                    smol::Timer::after(Duration::from_secs(2)).await;
                }
            }
        }
    }
    
    Err(last_error.unwrap())
}
```

Used for: Install Base, Install Docker (3 retries with 2s delay)

### User Notifications

For critical failures, show toast notification:
```rust
use egui_material3::notification;

notification::show(
    ui.ctx(),
    format!("Install Base failed on {}", host),
    &error_message,
    notification::NotificationAlign::TopRight,
    5.0,  // 5 seconds
);
```

---

## 8. Testing Strategy

### Coverage Goals

**Minimum 80% coverage** (per common/testing.md)

**Test distribution:**
- Unit tests: 60%
- Integration tests: 30%
- Manual testing: 10% (UI verification)

### Unit Tests

#### OperationLog Model
- `test_operation_log_ssh_constructor()`
- `test_operation_log_platform_constructor()`
- `test_get_ssh_operations()`
- `test_get_platform_operations()`

#### OperationState State Machine
- `test_operation_state_lifecycle()`
- `test_operation_state_failure()`
- `test_operation_state_reset()`

#### DrawerState
- `test_drawer_state_tab_switch()`
- `test_drawer_state_set_ssh_host_clears_cache()`

#### SSH Operations (Mocked)
- `test_check_base_packages_all_installed()`
- `test_check_base_packages_some_missing()`

### Integration Tests

#### SSH Connection (Real)
```rust
#[test]
#[ignore]  // Requires SSH_TEST_HOST env var
fn test_ssh_connection_real()
```

#### Database Migration
```rust
#[test]
fn test_operation_log_migration()
```
- Runs migration
- Inserts platform + SSH operations
- Queries by operation_source

#### ViewModel Event Handling
```rust
#[test]
fn test_ssh_actor_refresh_command()
```

### Manual Testing Checklist

**UI:**
- [ ] SSH tab loads with config hosts
- [ ] Platform badge displays when linked
- [ ] Button padding matches platform tab
- [ ] 3 rows of buttons visible
- [ ] Conditional buttons show/hide correctly

**Operations:**
- [ ] Refresh: 4 checks run, state updates
- [ ] SSH Check: Result in Status tab
- [ ] Check Base: Missing packages shown
- [ ] Install Base: Packages + network logging
- [ ] Install Docker: docker-ce installed
- [ ] Remove Docker: purged
- [ ] Install Dure: .env dialog → install
- [ ] Remove Dure: compose down

**Drawer:**
- [ ] All 6 tabs switch correctly
- [ ] Status: inline errors, no emojis
- [ ] Logs: auto-refresh 1s
- [ ] Operations: filtered by SSH host
- [ ] Host: network log 100 lines
- [ ] Docker: raw ps -a output
- [ ] Dure: compose status + logs

**Error Handling:**
- [ ] SSH failure: inline + Operations tab
- [ ] Install failure: error message shown
- [ ] Retry logic: 3 attempts on network timeout

**Database:**
- [ ] Migration successful
- [ ] Platform ops backfilled
- [ ] SSH ops query works
- [ ] Platform ops still functional

---

## Implementation Phases

### Phase 1: Foundation (Week 1)
1. Database migration
2. Shared drawer components (TabBar, StatusUtils, LogsRenderer, OperationsRenderer)
3. SSH ViewModel expansion (commands, events, state)

### Phase 2: Operations (Week 2)
4. Calc layer: 12 SSH operations
5. SshActor handlers
6. Operation logging integration

### Phase 3: UI (Week 3)
7. SSH drawer tabs (6 tabs)
8. Operations column layout
9. Install Dure dialog
10. Button padding + conditional display

### Phase 4: Migration & Testing (Week 1)
11. Platform drawer migration to shared components
12. Integration tests
13. Manual testing + bug fixes

**Total: 3-4 weeks**

---

## Success Criteria

- [ ] All 12 SSH operations functional
- [ ] 6 drawer tabs render correctly
- [ ] Shared components reused by Platform tab
- [ ] Button padding consistent with Platform tab
- [ ] Database migration successful (no data loss)
- [ ] 80%+ test coverage
- [ ] No regressions in Platform tab
- [ ] Operations logged to database correctly
- [ ] Inline error display works
- [ ] Install Dure .env editor functional

---

## Future Considerations

### NS Tab (Nameserver)
Similar drawer pattern:
- Status, Logs, Operations, DNS Records
- Can reuse shared drawer components

### Site Tab (E-commerce)
Similar drawer pattern:
- Status, Logs, Operations, Products, Orders
- Can reuse shared drawer components

### Drawer Unification Complete
After Platform migration, all tabs use:
- Shared TabBar
- Shared LogsRenderer
- Shared OperationsRenderer
- Consistent UX across all tabs

---

## Risk Mitigation

**Risk: Platform drawer breaks during migration**
- Mitigation: Migrate after SSH drawer proven stable
- Build SSH first, migrate Platform second
- No changes to Platform until SSH tested

**Risk: SSH operations fail on different distributions**
- Mitigation: Test on Debian, Ubuntu, OpenBSD
- Document distribution-specific variations
- Provide fallback commands

**Risk: Database migration loses data**
- Mitigation: Backup before migration
- Test migration on copy of production DB
- Backfill verification queries

**Risk: .env.example format changes**
- Mitigation: Generic key-value parser
- Handle parse errors gracefully
- Allow manual text editing as fallback

---

## Appendix A: File Changes Summary

**New Files:**
- `mobile/src/ui_components/drawer/mod.rs`
- `mobile/src/ui_components/drawer/tab_bar.rs`
- `mobile/src/ui_components/drawer/status_utils.rs`
- `mobile/src/ui_components/drawer/logs_renderer.rs`
- `mobile/src/ui_components/drawer/operations_renderer.rs`
- `mobile/src/calc/ssh_ops.rs`
- `mobile/src/storage/migrations/YYYYMMDD_add_operation_source.rs`

**Modified Files:**
- `mobile/src/ui_tabs/ssh.rs` (operations column, state)
- `mobile/src/ui_tabs/ssh_drawer.rs` (6 tabs)
- `mobile/src/viewmodel/ssh/commands.rs` (12 commands)
- `mobile/src/viewmodel/ssh/events.rs` (operation events)
- `mobile/src/viewmodel/ssh/actor.rs` (handlers)
- `mobile/src/viewmodel/ssh/drawer_types.rs` (DrawerState extension)
- `mobile/src/storage/models/opslog.rs` (constructors, queries)
- `mobile/src/calc/mod.rs` (export ssh_ops)

**Phase 2 (Platform Migration):**
- `mobile/src/ui_tabs/platform_drawer.rs` (use shared components)
- `mobile/src/viewmodel/platform/actor.rs` (use NewOperationLog::platform)

---

## Appendix B: Database Schema

**Before:**
```sql
CREATE TABLE operation_logs (
    id INTEGER PRIMARY KEY,
    project_id TEXT NOT NULL,
    operation_type TEXT NOT NULL,
    external_system TEXT NOT NULL,
    status TEXT NOT NULL,
    started_at INTEGER NOT NULL,
    completed_at INTEGER,
    error_message TEXT,
    details TEXT
);
```

**After:**
```sql
CREATE TABLE operation_logs (
    id INTEGER PRIMARY KEY,
    operation_source TEXT NOT NULL,  -- NEW
    relevant_id TEXT NOT NULL,       -- NEW
    operation_type TEXT NOT NULL,
    external_system TEXT NOT NULL,
    status TEXT NOT NULL,
    started_at INTEGER NOT NULL,
    completed_at INTEGER,
    error_message TEXT,
    details TEXT
);

CREATE INDEX idx_operation_logs_source_id 
ON operation_logs(operation_source, relevant_id);
```

---

**End of Design Document**
