# Comprehensive Refresh Behavior

## Overview

The Refresh button performs a comprehensive health check of the platform:

1. **VM Status Check**: Verifies VM exists and retrieves external IP
2. **Firewall Check**: Confirms current IP is whitelisted for SSH access
3. **SSH Test**: Attempts actual SSH connection to verify end-to-end connectivity

## User Experience

### Visual Feedback

**Immediate (Optimistic):**
- Steps column shows ⏳ for "Refreshing"
- Operation buttons disabled during refresh

**After Completion (3-5 seconds):**
- Steps column updates:
  - ✅ OAuth (if connected)
  - ✅ Project (if selected)
  - ✅/✗ VM (based on existence check)
  - ✅/✗ Firewall (based on whitelist check)
  - ✅/✗ SSH (based on connection test)

**Drawer Updates:**
- VM IP address (fresh from GCP)
- Firewall status: "✅ Whitelisted (1.2.3.4)" or "✗ Not whitelisted"
- SSH status: "✓ Ready" or "✗ Connection failed: <error>"
- Last refresh time: "just now", "2 min ago", etc.

**Auto-Clear:**
- ✅ Completed indicator clears after 3 seconds
- Returns to showing current state

## Technical Details

### Data Flow

1. UI: User clicks Refresh button
2. UI: Set OperationState::InProgress ("Refreshing")
3. UI: Send PlatformCommand::RefreshPlatform to ViewModel
4. ViewModel: Execute three checks in sequence:
   - Query GCP API for VM instances
   - Query GCP API for firewall rules
   - Test SSH connection (5 second timeout)
5. ViewModel: Send PlatformEvent::RefreshCompleted with results
6. UI: Update PlatformRow with fresh data
7. UI: Set OperationState::Completed, auto-clear after 3s

### Error Handling

- **No access token**: Returns empty status (shows ? in UI)
- **GCP API failure**: Logs error, returns failure status
- **SSH timeout**: Returns `connected: false` with timeout error
- **Network failure**: Shows error in SSH status field

### Performance

- **Expected duration**: 3-5 seconds total
- **Timeout**: 5 seconds for SSH test
- **No polling**: Event-driven updates only
- **No full reload**: Incremental PlatformRow updates

## Testing Checklist

- [ ] Refresh with running VM shows all ✅
- [ ] Refresh with stopped VM shows ✗ VM
- [ ] Refresh with non-whitelisted IP shows ✗ Firewall
- [ ] Refresh with unreachable VM shows ✗ SSH
- [ ] Refresh with no VM shows ✗ VM, ✗ Firewall, ✗ SSH
- [ ] Operation buttons disabled during refresh
- [ ] ✅ indicator auto-clears after 3 seconds
- [ ] Last refresh time updates correctly
- [ ] Drawer shows fresh IP address
- [ ] Works on all platforms (Linux, macOS, Windows)
