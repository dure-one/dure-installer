# GCP Wizard Context Wiring Design

**Date:** 2026-07-07  
**Status:** Approved  
**Goal:** Wire up `with_platform_context()` constructor to "Add VM" button to skip account/project steps when they're already configured.

## Problem Statement

When user clicks "Add VM" button on a platform row that already has:
- Connected Google account (OAuth token)
- Selected GCP project ID

The wizard still shows all 5 steps:
1. Connect Account
2. Select Project  
3. Configure Server
4. Create Server
5. Complete

This is redundant since steps 1-2 are already done. The wizard should skip directly to step 3 (Configure Server).

## Current Implementation

**File:** `mobile/src/ui_tabs/platform.rs`

**Flow:**
1. "Add VM" button click handler (line 1001-1012):
   - Stores only `platform_name` in temp data
   - Uses egui temp storage ID: `"platform_action_add_vm"`

2. Wizard launcher (line 1221-1226):
   - Reads platform_name from temp data
   - Calls `show_gcp_wizard(platform_name)`

3. `show_gcp_wizard()` method (line 1819-1829):
   ```rust
   fn show_gcp_wizard(&mut self, platform_name: String) {
       let mut wizard = GcpWizard::new(platform_name);
       
       // Load OAuth from config if exists
       if let Ok((app_config, _)) = load_config() {
           wizard.load_oauth_from_config(&app_config); // NO-OP (deprecated)
       }
       
       wizard.show();
       self.gcp_wizard = Some(wizard);
   }
   ```

**Issue:** Always uses `GcpWizard::new()` which starts at ConnectAccount state, even when OAuth and project are already configured.

## Solution Design

### Architecture

Modify `show_gcp_wizard()` to conditionally use `with_platform_context()` constructor when platform already has OAuth + project configured.

### Decision Logic

```
IF platform has valid OAuth token AND selected project ID:
    → Use with_platform_context(platform_name, project_id, oauth_result)
    → Wizard starts at ConfigureServer state (step 3/3)
ELSE:
    → Use new(platform_name)  
    → Wizard starts at ConnectAccount state (step 1/5)
```

### Implementation Approach

**Modify:** `show_gcp_wizard()` method in `mobile/src/ui_tabs/platform.rs` (line 1819-1829)

**Steps:**
1. Load config to find platform by name
2. Check if platform has both:
   - `gcp_oauth_access_token` (Some)
   - `gcp_oauth_refresh_token` (Some)
   - `gcp_oauth_expires_at` (Some)
   - `gcp_selected_project_id` (Some)
3. If all present:
   - Construct `OAuthResult { access_token, refresh_token, expires_at }`
   - Call `GcpWizard::with_platform_context(platform_name, project_id, oauth_result)`
4. Otherwise:
   - Fall back to `GcpWizard::new(platform_name)`
5. Call `wizard.show()` and store in `self.gcp_wizard`

### Data Availability

**PlatformRow has:**
- `platform_name`: String ✓
- `email`: Option<String> ✓
- `selected_project_id`: Option<String> ✓
- `gcp_connected`: bool ✓

**But NOT:**
- OAuth access_token (only available in config)
- OAuth refresh_token (only available in config)
- OAuth expires_at (only available in config)

**Solution:** Load from config using platform_name as lookup key.

### Backward Compatibility

- ✅ New platforms without OAuth: Uses `new()` → shows all 5 steps
- ✅ Platforms with OAuth only: Uses `new()` → shows all 5 steps
- ✅ Platforms with OAuth + project: Uses `with_platform_context()` → shows 3 steps
- ✅ No changes to existing wizard constructors
- ✅ No changes to button handler
- ✅ No changes to temp data storage

### Error Handling

**Scenario:** Config file not found or corrupted
- **Action:** Fall back to `GcpWizard::new()`
- **Result:** User sees full 5-step wizard (safe degradation)

**Scenario:** Platform not found in config
- **Action:** Fall back to `GcpWizard::new()`
- **Result:** User sees full 5-step wizard (safe degradation)

**Scenario:** OAuth token expired
- **Action:** Use expired token, let wizard's existing refresh logic handle it
- **Result:** Wizard may show token refresh UI if needed

### Testing Strategy

**Unit Tests:** None needed (logic too simple, covered by integration)

**Manual Testing:**
1. Create platform with OAuth + project configured
2. Click "Add VM" button
3. Verify wizard shows: "Configure → Create → Complete" (3 steps)
4. Verify progress indicator shows 3 steps, not 5
5. Verify "Back" button is hidden
6. Verify Configure step starts with image loading

**Regression Testing:**
1. Create new platform (no OAuth)
2. Click "Add Platform" → "GCP"
3. Verify wizard shows: "Connect → Project → Configure → Create → Complete" (5 steps)
4. Verify all steps work as before

## Technical Constraints

- **No changes to:** Button handler, temp data format, wizard constructors
- **Only modify:** `show_gcp_wizard()` method (8 lines → ~25 lines)
- **Rust nightly:** Required (existing constraint)
- **Config access:** Use existing `load_config()` function

## Success Criteria

✅ Clicking "Add VM" on configured platform → starts at Configure step  
✅ Clicking "Add Platform" → GCP → starts at Connect step  
✅ Progress indicator shows 3 steps (shortened flow) or 5 steps (full flow)  
✅ No changes to existing wizard functionality  
✅ Backward compatible with all platform states

## File Changes Summary

- **Modify:** `mobile/src/ui_tabs/platform.rs` (1 method, ~17 lines added)
- **Total:** 1 file changed

## Dependencies

- Existing: `GcpWizard::with_platform_context()` (already implemented)
- Existing: `load_config()` function
- Existing: `CloudPlatformConfig` struct with OAuth fields

## Implementation Time Estimate

- Implementation: 5 minutes
- Testing: 3 minutes
- **Total:** ~8 minutes
