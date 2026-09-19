# TODO: Complete Platform Drawer Logs UI Integration

## Status
- **Created**: 2026-09-19
- **Backend**: ✅ Complete
- **UI Integration**: ❌ Incomplete

## Completed Work (This PR)

### Backend Infrastructure ✅
1. **LogBuffer** - Ring buffer with 1000-line capacity per project
2. **LogActor** - HashMap-based log storage by project_id
3. **Global sender** - Fire-and-forget logging via OnceLock
4. **Macros** - `dure_info!(project_id = expr, "msg")` syntax
5. **Bulk replacement** - All log:: calls replaced with dure_* macros
6. **DrawerActor wiring** - Can send LogCommand::GetLogs to LogActor
7. **GCP tagging** - VM/billing/DNS operations tagged with project_id

### Architecture
```
UI → DrawerCommand::LoadLogs → DrawerActor → LogCommand::GetLogs → LogActor
                                                                       ↓
                                                          HashMap<project_id, LogBuffer>
```

## Missing Pieces (UI Integration)

### 1. UI Command Sending
**File**: `mobile/src/ui_tabs/platform_drawer.rs`
**Function**: `render_logs_tab()`

Update signature to accept drawer_tx and add project selector + refresh button.

### 2. ViewModel Event Forwarding
**File**: `mobile/src/viewmodel/mod.rs`

Add handling for `ViewModelEvent::Log(LogEvent::LogsRetrieved)` to forward to drawer.

### 3. Drawer Actor Event Handling
**File**: `mobile/src/viewmodel/platform/drawer_types.rs`

Add `DrawerCommand::UpdateLogs` variant.

### 4. Auto-Load on Tab Switch
Load logs automatically when Logs tab opens.

## Testing Checklist

- [ ] Logs appear in drawer Logs tab
- [ ] Project selector works
- [ ] Refresh button updates logs
- [ ] GCP operations show tagged logs
- [ ] Ring buffer limits to 1000 lines

## Estimated Effort
- **Time**: 1-2 hours
- **Files**: 4-5 files
- **Complexity**: Medium (plumbing)
