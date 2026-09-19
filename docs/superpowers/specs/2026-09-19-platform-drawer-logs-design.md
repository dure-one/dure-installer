# Platform Drawer Stdout Logs Design

**Date:** 2026-09-19  
**Author:** Claude Sonnet 4.5  
**Status:** Design Approved

## Overview

This spec defines the implementation of project-specific stdout log buffering and display in the platform drawer. The system captures logs from custom `dure_*` macros, routes them to per-project buffers, and displays them in the drawer's Logs tab.

## Goals

1. **Replace all `log::` calls** with `dure_*` macros (dure_debug, dure_info, dure_warn, dure_error)
2. **Filter logs by project** - Show only logs related to the selected GCP project
3. **Buffer logs per project** - Maintain 1000-line ring buffer per project (in-memory)
4. **Display in drawer** - Show filtered logs in the existing Logs tab

## Non-Goals

- Log persistence to database (in-memory only)
- Per-drawer log level filtering (use global log level setting)
- Log export or download features
- Real-time log streaming (poll-based UI updates sufficient)

## Architecture Overview

### System Components

```
┌─────────────────────────────────────────────────────────┐
│                     Application                          │
│                                                          │
│  ┌──────────────┐        ┌──────────────┐              │
│  │ dure_* macro │───────▶│  LogActor    │              │
│  │ (any module) │  logs  │              │              │
│  └──────────────┘        │ - Buffers    │              │
│                          │ - Routes     │              │
│  ┌──────────────┐        │ - Manages    │              │
│  │ Drawer UI    │◀───────│   overflow   │              │
│  │ (Logs tab)   │ query  └──────────────┘              │
│  └──────────────┘                                       │
└─────────────────────────────────────────────────────────┘
```

### Key Design Decisions

1. **LogActor owns all project log buffers** - Single source of truth, no shared mutable state
2. **Macros are fire-and-forget** - Send log via channel, don't wait for response (non-blocking)
3. **Ring buffer per project** - `VecDeque<String>` with 1000 line cap, FIFO eviction
4. **Global buffer for untagged logs** - Project ID `"__global__"` for logs without explicit project
5. **Integration with existing logger** - `CombinedLogger` routes to both stdout AND LogActor
6. **Explicit project tagging** - Macros accept optional project_id parameter for routing

### Module Structure

```
mobile/src/
├── logging.rs              # Modified: macros with optional project_id
├── log_capture.rs          # Modified: route logs to LogActor
├── viewmodel/
│   ├── log/
│   │   ├── mod.rs         # NEW: LogActor initialization and helpers
│   │   ├── actor.rs       # NEW: Actor message loop
│   │   └── types.rs       # NEW: Commands, Events, LogBuffer
│   └── mod.rs             # Modified: spawn LogActor, add log_tx
└── ui_tabs/
    └── platform_drawer.rs # Modified: query LogActor for logs
```

## Component Design

### LogActor Types (`viewmodel/log/types.rs`)

#### Commands

```rust
pub enum LogCommand {
    /// Append a log line to a project's buffer
    AppendLog {
        project_id: Option<String>,  // None = global buffer
        level: LogLevel,
        module: String,
        message: String,
    },
    
    /// Get logs for a project (for drawer UI)
    GetLogs {
        project_id: String,
        limit: usize,  // Max lines to return
    },
    
    /// Clear logs for a project
    ClearLogs {
        project_id: String,
    },
    
    /// Get list of projects with logs
    ListProjects,
}
```

#### Events

```rust
pub enum LogEvent {
    /// Logs retrieved successfully
    LogsRetrieved {
        project_id: String,
        lines: Vec<String>,
    },
    
    /// Project list retrieved
    ProjectList {
        project_ids: Vec<String>,
    },
}
```

#### LogLevel

```rust
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}
```

#### LogBuffer

```rust
/// Ring buffer for project logs (1000 line cap)
pub struct LogBuffer {
    lines: VecDeque<String>,
    capacity: usize,  // Default 1000
}

impl LogBuffer {
    pub fn new() -> Self {
        Self::with_capacity(1000)
    }
    
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            lines: VecDeque::with_capacity(capacity),
            capacity,
        }
    }
    
    /// Push a line to the buffer (FIFO eviction when full)
    pub fn push(&mut self, line: String) {
        if self.lines.len() >= self.capacity {
            self.lines.pop_front();  // Drop oldest
        }
        self.lines.push_back(line);
    }
    
    /// Get most recent lines (up to limit)
    pub fn get_lines(&self, limit: usize) -> Vec<String> {
        self.lines.iter()
            .rev()           // Newest first
            .take(limit)
            .rev()           // Restore chronological order
            .cloned()
            .collect()
    }
    
    pub fn clear(&mut self) {
        self.lines.clear();
    }
    
    pub fn len(&self) -> usize {
        self.lines.len()
    }
}
```

### Modified Macros (`logging.rs`)

#### Macro Signature (Backward Compatible)

```rust
// Usage patterns:
dure_info!("message");                    // ← global buffer
dure_info!(project_id, "message");        // ← project-specific buffer
dure_info!(Some(id), "message");          // ← optional handling
```

#### Implementation (non-WASM)

```rust
#[cfg(not(target_family = "wasm"))]
#[macro_export]
macro_rules! dure_info {
    // With explicit project_id
    ($project_id:expr, $($arg:tt)*) => {
        {
            let module = $crate::logging::module_name(module_path!());
            let message = format!($($arg)*);
            
            // Send to LogActor (fire-and-forget)
            $crate::viewmodel::log::append_log(
                Some($project_id.to_string()),
                $crate::viewmodel::log::LogLevel::Info,
                module.clone(),
                message.clone(),
            );
            
            // Also log to stdout (existing behavior)
            log::info!("[{}] {}", 
                $crate::logging::truncate_module_name(&module), 
                message
            )
        }
    };
    
    // Without project_id (backward compatible)
    ($($arg:tt)*) => {
        {
            let module = $crate::logging::module_name(module_path!());
            let message = format!($($arg)*);
            
            // Send to global buffer
            $crate::viewmodel::log::append_log(
                None,
                $crate::viewmodel::log::LogLevel::Info,
                module.clone(),
                message.clone(),
            );
            
            // Also log to stdout
            log::info!("[{}] {}", 
                $crate::logging::truncate_module_name(&module), 
                message
            )
        }
    };
}

// Repeat for dure_debug!, dure_warn!, dure_error! with appropriate LogLevel
```

#### Implementation (WASM)

```rust
#[cfg(target_family = "wasm")]
#[macro_export]
macro_rules! dure_info {
    // With project_id
    ($project_id:expr, $($arg:tt)*) => {
        {
            let module = $crate::logging::module_name(module_path!());
            let message = format!($($arg)*);
            
            // WASM: No LogActor (runs in browser)
            // Just log to console
            web_sys::console::log_1(
                &format!("[{}] {}", 
                    $crate::logging::truncate_module_name(&module), 
                    message
                ).into()
            )
        }
    };
    
    // Without project_id
    ($($arg:tt)*) => {
        dure_info!(None::<String>, $($arg)*)
    };
}
```

### LogActor Implementation (`viewmodel/log/actor.rs`)

```rust
use super::types::*;
use smol::channel::{Receiver, Sender};
use std::collections::HashMap;

/// LogActor message loop
pub async fn log_actor_loop(
    mut cmd_rx: Receiver<LogCommand>,
    event_tx: Sender<crate::viewmodel::ViewModelEvent>,
) {
    let mut buffers: HashMap<String, LogBuffer> = HashMap::new();
    
    while let Ok(cmd) = cmd_rx.recv().await {
        match cmd {
            LogCommand::AppendLog { project_id, level, module, message } => {
                let key = project_id.unwrap_or_else(|| "__global__".to_string());
                let buffer = buffers.entry(key).or_insert_with(LogBuffer::new);
                
                // Format: [LEVEL] [Module] message
                let formatted = format!("[{:5}] [{}] {}", 
                    level.as_str(), module, message);
                buffer.push(formatted);
            }
            
            LogCommand::GetLogs { project_id, limit } => {
                let lines = buffers
                    .get(&project_id)
                    .map(|buf| buf.get_lines(limit))
                    .unwrap_or_default();
                
                let _ = event_tx.send(
                    crate::viewmodel::ViewModelEvent::Log(
                        LogEvent::LogsRetrieved { project_id, lines }
                    )
                ).await;
            }
            
            LogCommand::ClearLogs { project_id } => {
                buffers.remove(&project_id);
            }
            
            LogCommand::ListProjects => {
                let project_ids: Vec<String> = buffers.keys().cloned().collect();
                let _ = event_tx.send(
                    crate::viewmodel::ViewModelEvent::Log(
                        LogEvent::ProjectList { project_ids }
                    )
                ).await;
            }
        }
    }
}
```

### ViewModel Integration (`viewmodel/mod.rs`)

#### Add LogEvent to ViewModelEvent

```rust
pub enum ViewModelEvent {
    Platform(platform::PlatformEvent),
    Ssh(ssh::SshEvent),
    Ns(ns::NsEvent),
    Wss(wss::WssEvent),
    Log(log::LogEvent),  // NEW
}
```

#### Add log_tx to ViewModel

```rust
pub struct ViewModel {
    platform_tx: Sender<platform::PlatformCommand>,
    drawer_tx: Sender<platform::DrawerCommand>,
    ssh_tx: Sender<ssh::SshCommand>,
    ns_tx: Sender<ns::NsCommand>,
    wss_tx: Sender<wss::WssCommand>,
    log_tx: Sender<log::LogCommand>,  // NEW
    
    event_rx: Receiver<ViewModelEvent>,
    state: ViewModelState,
    runtime_handle: Option<RuntimeHandle>,
    
    #[cfg(feature = "gui")]
    egui_ctx: Option<egui::Context>,
}
```

#### Spawn LogActor in ViewModel::new()

```rust
pub fn new(ctx: egui::Context) -> Self {
    let (platform_tx, platform_rx) = smol::channel::unbounded();
    let (drawer_tx, drawer_rx) = smol::channel::unbounded();
    let (ssh_tx, ssh_rx) = smol::channel::unbounded();
    let (ns_tx, ns_rx) = smol::channel::unbounded();
    let (wss_tx, wss_rx) = smol::channel::unbounded();
    let (log_tx, log_rx) = smol::channel::unbounded();  // NEW
    let (event_tx, event_rx) = smol::channel::unbounded();
    
    // Initialize global log sender for macros
    log::init_log_sender(log_tx.clone());  // NEW
    
    let runtime_handle = std::thread::spawn(move || {
        smol::block_on(async {
            // Spawn actors
            let event_tx_platform = event_tx.clone();
            smol::spawn(async move {
                platform::actor::platform_actor_loop(
                    platform_rx, drawer_rx, event_tx_platform
                ).await;
            }).detach();
            
            let event_tx_log = event_tx.clone();  // NEW
            smol::spawn(async move {              // NEW
                log::actor::log_actor_loop(log_rx, event_tx_log).await;
            }).detach();
            
            // ... other actors ...
        })
    });
    
    Self {
        platform_tx,
        drawer_tx,
        ssh_tx,
        ns_tx,
        wss_tx,
        log_tx,  // NEW
        event_rx,
        state: ViewModelState::default(),
        runtime_handle: Some(RuntimeHandle::Native(runtime_handle)),
        egui_ctx: Some(ctx),
    }
}
```

#### Add log command sender method

```rust
impl ViewModel {
    /// Send log command to LogActor
    pub fn send_log_command(&self, cmd: log::LogCommand) -> anyhow::Result<()> {
        self.log_tx.try_send(cmd)
            .map_err(|e| anyhow::anyhow!("Failed to send log command: {}", e))
    }
}
```

### Helper Module (`viewmodel/log/mod.rs`)

```rust
//! Log buffering actor for project-specific stdout logs

pub mod actor;
mod types;

pub use types::{LogCommand, LogEvent, LogLevel, LogBuffer};

use smol::channel::Sender;
use std::sync::OnceLock;

static LOG_SENDER: OnceLock<Sender<LogCommand>> = OnceLock::new();

/// Initialize log sender (called once at startup in ViewModel::new)
pub fn init_log_sender(tx: Sender<LogCommand>) {
    LOG_SENDER.set(tx).ok();
}

/// Helper for macros - fire-and-forget log append
pub fn append_log(
    project_id: Option<String>,
    level: LogLevel,
    module: String,
    message: String,
) {
    if let Some(tx) = LOG_SENDER.get() {
        let _ = tx.try_send(LogCommand::AppendLog {
            project_id,
            level,
            module,
            message,
        });
        // Fire-and-forget: if channel is full, drop the log
        // (prefer dropping logs over blocking application)
    }
}
```

### Drawer Integration

#### DrawerCommand Extension (`platform/drawer_types.rs`)

Already has `LoadLogs` command - no changes needed:

```rust
pub enum DrawerCommand {
    SwitchTab { tab: DrawerTab },
    LoadOperations { project_id: String, limit: i64 },
    LoadLogs { project_id: String, limit: usize },  // ✓ Already exists
    Refresh,
}
```

#### DrawerEvent Extension (`platform/drawer_types.rs`)

Already has `LogsLoaded` event - no changes needed:

```rust
pub enum DrawerEvent {
    TabSwitched { tab: DrawerTab },
    OperationsLoaded { project_id: String, logs: Vec<OperationLog> },
    LogsLoaded { project_id: String, lines: Vec<String> },  // ✓ Already exists
    Error { operation: String, error: String },
}
```

#### Drawer Actor Handling (`platform/actor.rs`)

Add handler for `DrawerCommand::LoadLogs`:

```rust
// In platform_actor_loop, handle DrawerCommand
match drawer_cmd {
    DrawerCommand::SwitchTab { tab } => {
        // ... existing code ...
    }
    
    DrawerCommand::LoadLogs { project_id, limit } => {
        // Forward to LogActor
        if let Some(log_tx) = vm_log_tx {
            let _ = log_tx.send(log::LogCommand::GetLogs {
                project_id,
                limit,
            }).await;
        }
    }
    
    // ... other commands ...
}
```

#### UI Rendering (`ui_tabs/platform_drawer.rs`)

Already implemented - just needs to trigger LoadLogs when tab opens:

```rust
fn render_logs_tab(ui: &mut egui::Ui, drawer_state: &DrawerState) {
    ui.heading("Stdout Logs");
    ui.add_space(8.0);
    
    if drawer_state.loading {
        ui.spinner();
        ui.label("Loading logs...");
        return;
    }
    
    if drawer_state.logs.is_empty() {
        ui.label("No logs available");
        return;
    }
    
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::monospace(12.0));
            
            for line in &drawer_state.logs {
                ui.label(line);
            }
        });
}
```

## Data Flow

### Log Emission Flow

```
┌─────────────────────────────────────────────────────────────┐
│ 1. Code calls macro                                          │
│    dure_info!(project_id, "Creating VM")                    │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 2. Macro expands to:                                         │
│    - Call viewmodel::log::append_log() (async send)         │
│    - Call log::info!() for stdout (synchronous)             │
└────────────────────┬────────────────────────────────────────┘
                     │
        ┌────────────┴────────────┐
        │                         │
        ▼                         ▼
┌──────────────┐         ┌──────────────┐
│ LogActor     │         │ stdout/      │
│ - Receives   │         │ logcat       │
│ - Routes to  │         │ (existing)   │
│   buffer     │         └──────────────┘
│ - Manages    │
│   overflow   │
└──────────────┘
```

### Log Retrieval Flow

```
┌─────────────────────────────────────────────────────────────┐
│ 1. User opens drawer Logs tab for project "my-gcp-123"      │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 2. UI sends DrawerCommand::LoadLogs                          │
│    { project_id: "my-gcp-123", limit: 1000 }                │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 3. DrawerActor/PlatformActor forwards to LogActor            │
│    LogCommand::GetLogs { ... }                               │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 4. LogActor retrieves from buffer                            │
│    - Look up buffer for "my-gcp-123"                         │
│    - Get last 1000 lines (ring buffer)                       │
│    - Send LogEvent::LogsRetrieved via event_tx               │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 5. ViewModel receives event, updates DrawerState             │
│    (via existing event handling mechanism)                   │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 6. UI renders logs in scrollable view                        │
│    (drawer_state.logs already exists)                        │
└─────────────────────────────────────────────────────────────┘
```

### Channel Configuration

- **LogCommand channel**: Unbounded (fire-and-forget, never block)
- **LogEvent channel**: Unified `event_tx` (same as other actors)
- **Buffer capacity per project**: 1000 lines (VecDeque)

## Implementation Strategy

### Phase 1: Infrastructure (New Code)

**Files to create:**
- `mobile/src/viewmodel/log/mod.rs` - Module root + helpers
- `mobile/src/viewmodel/log/types.rs` - Commands, Events, LogBuffer
- `mobile/src/viewmodel/log/actor.rs` - Actor loop

**Files to modify:**
- `mobile/src/viewmodel/mod.rs` - Spawn LogActor, add event variant
- `mobile/src/logging.rs` - Extend macros with optional project_id

**Testing:**
- Unit tests for LogBuffer (overflow, get_lines, clear)
- Unit tests for LogActor (append, retrieve, clear)
- Integration test for macro → actor → retrieval flow

### Phase 2: Macro Replacement (Mechanical)

**Bulk replacement pattern:**
```bash
# Replace all log:: calls with dure_* equivalents
log::info!   → dure_info!
log::debug!  → dure_debug!
log::warn!   → dure_warn!
log::error!  → dure_error!
```

**Files affected:** ~200 files (from grep output)

**Strategy:**
1. Automated search-replace for simple cases
2. Manual review for complex formatting
3. Verify compilation after each batch
4. No project_id tagging yet (all go to global buffer)

### Phase 3: Project Tagging (Selective)

**Add project_id to critical paths:**

**GCP API operations** (`api/gcp/*.rs`, `calc/platform*.rs`):
```rust
// Before
dure_info!("Creating VM instance");

// After
dure_info!(project_id, "Creating VM instance");
```

**Platform ViewModel** (`viewmodel/platform/*.rs`):
- Extract project_id from context
- Pass to all GCP-related log calls

**SSH operations** - Keep global (not project-specific)

### Phase 4: Drawer Integration (Wiring)

**Connect drawer to LogActor:**
1. When Logs tab opens, send `LoadLogs` command with selected project_id
2. Handle `LogsLoaded` event in drawer state
3. Render logs in UI (already implemented)

**Auto-refresh:**
- Poll LogActor every 1-2 seconds when Logs tab is active
- Or: Add log count to drawer state, refresh when changed

## Error Handling

### Fire-and-Forget Philosophy

**Logging should never block the application:**
- `try_send()` on LogCommand channel
- If channel is full/closed → drop the log silently
- No error propagation to caller

### Buffer Overflow

**Ring buffer handles automatically:**
- When 1000 lines reached → drop oldest line
- No error, no warning (expected behavior)
- UI shows "last 1000 lines" indicator

### Missing Project Buffer

**Get logs for unknown project:**
- Return empty `Vec<String>`
- Not an error (project might not have logged yet)

### Actor Shutdown

**Graceful degradation:**
- If LogActor crashes → macros silently fail (logs still go to stdout)
- Application continues working
- Log "LogActor unavailable" to stderr

## Testing Strategy

### Unit Tests

**LogBuffer** (`types.rs`):
- ✅ Push and retrieve lines
- ✅ Ring buffer overflow (1000+ lines)
- ✅ Get with limit
- ✅ Clear buffer

**LogActor** (`actor.rs`):
- ✅ Append log to project buffer
- ✅ Append to global buffer (None project_id)
- ✅ Retrieve logs
- ✅ Clear logs
- ✅ List projects with logs

**Macros** (`logging_test.rs`):
- ✅ Backward compatibility (no project_id)
- ✅ With project_id
- ✅ All log levels (debug/info/warn/error)

### Integration Tests

**End-to-end flow:**
1. Initialize ViewModel (spawns LogActor)
2. Call `dure_info!(project_id, "test")`
3. Send `GetLogs` command
4. Verify `LogsRetrieved` event contains log
5. Verify drawer state updated

### Manual Testing Checklist

- [ ] Start app, verify LogActor spawned
- [ ] Open platform drawer, switch to Logs tab
- [ ] Select project, verify logs appear
- [ ] Generate 1000+ logs, verify ring buffer (oldest dropped)
- [ ] Switch between projects, verify filtering
- [ ] Check global logs (`__global__` buffer)
- [ ] Restart app, verify logs cleared (in-memory only)
- [ ] Check performance with high log volume (10k/sec)

### Edge Cases

1. **Empty buffer**: GetLogs for project with no logs → `[]`
2. **Unknown project**: GetLogs for non-existent project → `[]`
3. **Channel closed**: Macro after LogActor stopped → log dropped, no panic
4. **Rapid logging**: 10,000 logs/sec → no blocking, ring buffer works
5. **Long messages**: 10KB log line → allow (no truncation for now)

## Performance Considerations

### Memory Usage

**Per-project buffer:**
- 1000 lines × ~200 bytes/line = ~200KB per project
- 10 projects = ~2MB total (negligible)

**Channel queue:**
- Unbounded channel can grow if LogActor is slow
- Mitigation: Actor processes commands quickly (just HashMap insert)
- Worst case: Memory spike during log burst, then GC cleans up

### CPU Overhead

**Macro call overhead:**
- Channel send: ~100ns (non-blocking)
- Format string: ~1μs (already done for stdout)
- Total: <2μs per log call (acceptable)

**Actor processing:**
- HashMap insert: O(1)
- VecDeque push: O(1) amortized
- GetLogs: O(min(n, limit)) where n = buffer size
- All operations are fast, no blocking

### UI Responsiveness

**Polling vs Push:**
- Current: Poll GetLogs every 1-2 sec when tab active
- Future optimization: Subscribe to log updates (push-based)
- For MVP, polling is sufficient

## Future Enhancements

**Not in scope for this implementation:**

1. **Log persistence** - Save to SQLite for history
2. **Log export** - Download logs as file
3. **Advanced filtering** - Regex, log level, time range
4. **Log search** - Full-text search in logs
5. **Real-time streaming** - Push-based updates instead of polling
6. **Log compression** - Compress old logs to save memory
7. **Cross-project search** - Search logs across all projects

## Open Questions

None - all design decisions finalized during brainstorming.

## Success Criteria

1. ✅ All `log::` calls replaced with `dure_*` macros
2. ✅ LogActor buffers logs per project (1000 line ring buffer)
3. ✅ Drawer Logs tab displays project-filtered logs
4. ✅ Logs survive while app runs, cleared on restart (in-memory only)
5. ✅ No performance degradation (logging stays fast)
6. ✅ 80%+ test coverage for new code

## References

- Existing drawer implementation: `mobile/src/ui_tabs/platform_drawer.rs`
- Existing logging macros: `mobile/src/logging.rs`
- Actor pattern: `mobile/src/viewmodel/platform/actor.rs`, `mobile/src/viewmodel/ssh/actor.rs`
- Ring buffer: Rust std `VecDeque`
