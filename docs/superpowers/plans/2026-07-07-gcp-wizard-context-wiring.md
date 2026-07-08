# GCP Wizard Context Wiring Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Wire up `with_platform_context()` constructor to "Add VM" button to skip account/project steps when platform already has OAuth and project configured.

**Architecture:** Modify `show_gcp_wizard()` method in `platform.rs` to conditionally use `GcpWizard::with_platform_context()` when platform has OAuth tokens and project ID in config, falling back to `GcpWizard::new()` otherwise for backward compatibility.

**Tech Stack:** Rust nightly, egui 0.33, existing `GcpWizard` constructors, `load_config()` function, `OAuthResult` from `crate::api::gcp::oauth`

## Global Constraints

- Rust nightly toolchain required
- No changes to existing wizard constructors (`GcpWizard::new()`, `GcpWizard::with_platform_context()`)
- No changes to button handler or temp data format
- Backward compatible with all platform states (no OAuth, OAuth only, OAuth + project)
- Safe degradation on config errors (fall back to full 5-step wizard)
- Only modify `show_gcp_wizard()` method in `mobile/src/ui_tabs/platform.rs`

---

### Task 1: Conditional Wizard Constructor Logic

**Files:**
- Modify: `mobile/src/ui_tabs/platform.rs:1819-1829`

**Interfaces:**
- Consumes: `load_config()` from `mobile/src/config.rs`, `GcpWizard::new()` and `GcpWizard::with_platform_context()` from `mobile/src/ui_dlg/platform_gcp.rs`, `OAuthResult` from `mobile/src/api/gcp/oauth.rs`
- Produces: Modified `show_gcp_wizard(&mut self, platform_name: String)` method that conditionally uses `with_platform_context()` when OAuth tokens and project ID exist in config

- [x] **Step 1: Read current implementation**

Read the current `show_gcp_wizard()` method to understand existing structure:

```bash
cd /home/wj/work/dure/mobile
```

Expected: Method at lines 1819-1829 shows:
- Creates wizard with `GcpWizard::new(platform_name)`
- Attempts to load OAuth from config (deprecated NO-OP)
- Calls `wizard.show()` and stores in `self.gcp_wizard`

- [x] **Step 2: Implement conditional constructor logic**

Replace the `show_gcp_wizard()` method with logic that checks for OAuth and project ID:

```rust
fn show_gcp_wizard(&mut self, platform_name: String) {
    // Try to load config and find platform with OAuth + project
    let mut wizard = if let Ok((app_config, _)) = load_config() {
        // Find platform by name
        if let Some(platform) = app_config.platforms.iter()
            .find(|p| p.name == platform_name)
        {
            // Check if platform has OAuth tokens and project ID
            if let (Some(access_token), Some(refresh_token), Some(token_expiry), Some(project_id)) = (
                &platform.gcp_oauth_access_token,
                &platform.gcp_oauth_refresh_token,
                platform.gcp_oauth_token_expiry,
                &platform.gcp_selected_project_id,
            ) {
                // Construct OAuthResult and use with_platform_context
                let oauth_result = crate::api::gcp::oauth::OAuthResult {
                    access_token: access_token.clone(),
                    refresh_token: refresh_token.clone(),
                    expires_at: token_expiry as u64,
                };

                GcpWizard::with_platform_context(
                    platform_name,
                    project_id.clone(),
                    oauth_result,
                )
            } else {
                // Missing OAuth or project, use full wizard
                GcpWizard::new(platform_name)
            }
        } else {
            // Platform not found in config, use full wizard
            GcpWizard::new(platform_name)
        }
    } else {
        // Config load failed, use full wizard
        GcpWizard::new(platform_name)
    };

    wizard.show();
    self.gcp_wizard = Some(wizard);
}
```

**Key Changes:**
- Load config and find platform by name
- Check for all required OAuth fields (`gcp_oauth_access_token`, `gcp_oauth_refresh_token`, `gcp_oauth_token_expiry`) and project ID (`gcp_selected_project_id`)
- If all present: construct `OAuthResult` from `crate::api::gcp::oauth` and call `with_platform_context()`
- Otherwise: fall back to `new()` for full wizard flow
- Safe degradation: any error (config not found, platform not found, missing fields) → use `new()`

**Note:** The field is `app_config.platforms` (not `cloud_platforms`) and OAuth expiry is `gcp_oauth_token_expiry` (type `Option<i64>`) which needs to be cast to `u64` for `OAuthResult.expires_at`.

- [x] **Step 3: Build verification**

Build the project to verify compilation:

Run:
```bash
cd /home/wj/work/dure/mobile
cargo build
```

Expected: Build succeeds with no compilation errors related to `show_gcp_wizard()`, `OAuthResult`, or `CloudPlatformConfig` fields.

**Verification Points:**
- ✅ `OAuthResult` is public in `crate::api::gcp::oauth`
- ✅ `app_config.platforms` exists and is `Vec<CloudPlatformConfig>`
- ✅ `CloudPlatformConfig` has fields: `gcp_oauth_access_token`, `gcp_oauth_refresh_token`, `gcp_oauth_token_expiry`, `gcp_selected_project_id`
- ✅ `GcpWizard::with_platform_context()` accepts `(String, String, OAuthResult)`

- [ ] **Step 4: Manual test - Platform with OAuth + project**

Test the shortened wizard flow (3 steps instead of 5):

**Setup:**
1. Start the application
2. Navigate to Platform tab
3. Create a platform with GCP account connected and project selected:
   - Add new platform (name: "test-platform")
   - Complete OAuth flow (Connect Account step)
   - Select a project (Select Project step)
   - Exit wizard without creating VM

**Test:**
1. Click "Add VM" button in operations column for "test-platform" row
2. Verify wizard opens at "Configure Server" step (step 3/3)
3. Verify progress indicator shows 3 steps total (not 5)
4. Verify "Back" button is hidden
5. Verify image list starts loading immediately

**Expected:**
- Wizard skips "Connect Account" and "Select Project" steps
- User immediately sees server configuration options
- Progress shows "Configure → Create → Complete"

**Actual:**
- [ ] Wizard opened at Configure Server step
- [ ] Progress indicator shows 3 steps
- [ ] Back button hidden
- [ ] Image list loading

- [ ] **Step 5: Manual test - Regression (No OAuth scenario)**

Test the full wizard flow (5 steps) still works:

**Setup:**
1. Start the application
2. Navigate to Platform tab

**Test:**
1. Click "Add Platform" button
2. Select "GCP" platform type
3. Enter platform name (e.g., "new-platform")
4. Verify wizard opens at "Connect Account" step (step 1/5)
5. Verify progress indicator shows 5 steps total
6. Click "Connect Account" button
7. Complete OAuth flow
8. Verify wizard advances to "Select Project" step (step 2/5)

**Expected:**
- Full 5-step wizard flow works as before
- Progress shows "Connect → Project → Configure → Create → Complete"
- No regression in existing functionality

**Actual:**
- [ ] Wizard opened at Connect Account step
- [ ] Progress indicator shows 5 steps
- [ ] OAuth flow works
- [ ] Advanced to Select Project step

- [ ] **Step 6: Commit changes**

Commit the implementation:

```bash
git add mobile/src/ui_tabs/platform.rs
git commit -m "$(cat <<'EOF'
feat: wire up GcpWizard context constructor to Add VM button

Skip Connect Account and Select Project steps when platform already has
OAuth tokens and project ID configured. Falls back to full wizard flow
when OAuth or project missing (backward compatible).

Changes:
- Modify show_gcp_wizard() to load config and check for OAuth + project
- Use with_platform_context() when all credentials present (3-step flow)
- Use new() as fallback for missing credentials (5-step flow)
- Safe degradation on config errors

Refs: docs/superpowers/specs/2026-07-07-gcp-wizard-context-wiring-design.md

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
EOF
)"
```

Expected: Commit created with modified `platform.rs`

---

## Testing Summary

**Unit Tests:** None (spec explicitly states "Unit Tests: None needed - logic too simple, covered by integration")

**Manual Testing:**
1. ✅ Platform with OAuth + project → 3-step wizard (Configure → Create → Complete)
2. ✅ New platform without OAuth → 5-step wizard (Connect → Project → Configure → Create → Complete)
3. ✅ Config load error → 5-step wizard (safe degradation)

**Success Criteria:**
- Clicking "Add VM" on configured platform → starts at Configure step
- Clicking "Add Platform" → GCP → starts at Connect step
- Progress indicator shows correct step count (3 or 5)
- No changes to existing wizard functionality
- Backward compatible with all platform states

## Implementation Notes

**Why no unit tests?**
- UI integration code requires egui context mocking
- Config loading requires file system mocking
- Wizard state requires complex setup
- Manual testing provides better coverage for this use case
- Spec explicitly approves manual-only testing approach

**Type conversions:**
- `gcp_oauth_token_expiry` is `Option<i64>` in config
- `OAuthResult.expires_at` is `u64`
- Cast: `token_expiry as u64` (safe since token expiry is always positive Unix timestamp)

**Backward compatibility:**
- All error paths → use `new()` constructor
- Missing OAuth token → use `new()`
- Missing project ID → use `new()`
- Config not found → use `new()`
- Platform not found → use `new()`

This ensures no user-facing breakage even if config is corrupted or incomplete.
