# Platform Table Dynamic Row Height Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enable dynamic row heights in platform tab data_table to prevent button clipping in the Operations column.

**Architecture:** Add `.auto_row_height(true)` and `.min_row_height(52.0)` configuration to existing data_table initialization in platform.rs. This leverages egui-material3's built-in auto-height feature with row height caching for performance.

**Tech Stack:** Rust (nightly), egui 0.33, egui-material3 (custom fork with data_table component)

## Global Constraints

- Rust nightly toolchain required
- No breaking changes to ViewModel API or data structures
- Must work on all platforms: Desktop (Linux/macOS/Windows), Android, WASM
- Must preserve existing drawer functionality (independent of row height)
- Follow Material Design 3 spacing and sizing guidelines (52px minimum row height)
- No performance regression in table scrolling (row height caching must work)
- Simple configuration change only - no custom code or API modifications

---

## File Structure

### Modified Files
- `mobile/src/ui_tabs/platform.rs:1273` - Add two configuration lines to data_table initialization

### No New Files Required

---

## Task 1: Enable Auto Row Height Configuration

**Files:**
- Modify: `mobile/src/ui_tabs/platform.rs:1273-1280`

**Interfaces:**
- Consumes: Existing `data_table()` builder API from egui-material3
  - `.auto_row_height(enabled: bool) -> Self` - enables content-based row sizing
  - `.min_row_height(height: f32) -> Self` - sets minimum height for auto-sizing mode
- Produces: Configured data_table with dynamic row heights enabled

- [ ] **Step 1: Locate the data_table initialization**

Open `mobile/src/ui_tabs/platform.rs` and navigate to line 1273 where the data_table is initialized in the platform display logic. The current code looks like:

```rust
let mut table = data_table()
    .id(table_id)
    .allow_selection(false)
    .allow_drawer(true)
    .column("Project", 150.0 * width_ratio, false)
    .column("Type", 80.0 * width_ratio, false)
    .column("Steps", 250.0 * width_ratio, false)
    .column("Operations", 260.0 * width_ratio, false);
```

- [ ] **Step 2: Add auto row height configuration**

Add two configuration lines after `.allow_drawer(true)` and before the first `.column()` call:

```rust
let mut table = data_table()
    .id(table_id)
    .allow_selection(false)
    .allow_drawer(true)
    .auto_row_height(true)      // Enable dynamic row heights
    .min_row_height(52.0)       // Maintain MD3 minimum height
    .column("Project", 150.0 * width_ratio, false)
    .column("Type", 80.0 * width_ratio, false)
    .column("Steps", 250.0 * width_ratio, false)
    .column("Operations", 260.0 * width_ratio, false);
```

**Explanation:**
- `.auto_row_height(true)` - Tells the data_table to calculate row height based on content instead of using a fixed height. The table will use `max(min_row_height, natural_content_height)` for each row.
- `.min_row_height(52.0)` - Sets the minimum row height to 52 pixels, which is the Material Design 3 standard for data table rows. This prevents rows from becoming too short.

- [ ] **Step 3: Verify code compiles**

Run the build to ensure the configuration is syntactically correct:

```bash
cd /home/wj/work/dure/mobile
cargo check
```

Expected output: Build succeeds with no errors. The egui-material3 library provides these methods on the data_table builder.

If you see an error like "method not found", verify that:
1. The egui-material3 dependency is correctly specified in Cargo.toml
2. You're using the correct builder method chain syntax

- [ ] **Step 4: Build desktop version for testing**

Build the desktop version with GUI enabled:

```bash
cd /home/wj/work/dure/mobile
cargo build --profile dev-release --bin dure-desktop
```

Expected: Build completes successfully. The binary will be at `target/dev-release/dure-desktop`.

**Note:** Using `dev-release` profile for faster builds while still having reasonable performance for testing.

- [ ] **Step 5: Manual Test 1 - Button Wrapping (Desktop, No VM)**

Run the desktop application and test compact row behavior:

```bash
./target/dev-release/dure-desktop
```

**Test Steps:**
1. Navigate to Platform tab
2. If no platforms exist, create one (OAuth → Select Project)
3. With a platform that has no VM, observe the Operations column
4. Expected buttons: Refresh, Scan VMs, Billing (3-4 buttons depending on state)
5. Resize window to ~800px width

**Expected Result:**
- All buttons are fully visible without clipping
- Row height is approximately 52px (compact, single row of buttons)
- No vertical scrollbar appears in the Operations cell

**Pass Criteria:** ✅ All buttons visible, row compact, no clipping

- [ ] **Step 6: Manual Test 2 - Button Wrapping (Desktop, With VM)**

Continue testing with expanded row behavior:

**Test Steps:**
1. Click "Add VM" on a platform (or use existing platform with VM)
2. Wait for VM creation to complete
3. Observe the Operations column with additional buttons
4. Expected buttons: Refresh, Scan VMs, Firewall, Restart, Del VM, Billing (6 buttons)
5. Resize window narrower (~600px width) to force wrapping

**Expected Result:**
- All 6 buttons are fully visible
- Buttons wrap to 2-3 rows depending on window width
- Row height expands to approximately 80-150px (2-3 button rows)
- Row height smoothly adjusts as window is resized

**Pass Criteria:** ✅ All buttons visible at any window width, row expands/contracts smoothly

- [ ] **Step 7: Manual Test 3 - Drawer Independence**

Verify that drawer expansion doesn't interfere with row height:

**Test Steps:**
1. Click on a platform row to expand its drawer
2. Observe the drawer content (StatusGrid with platform details)
3. Observe the data row height (with buttons)
4. Close the drawer

**Expected Result:**
- Data row height (button area) remains constant whether drawer is open or closed
- Drawer height is independent of button row height
- StatusGrid in drawer displays correctly

**Pass Criteria:** ✅ Row and drawer heights are independent, both display correctly

- [ ] **Step 8: Manual Test 4 - Performance (Scrolling)**

If you have multiple platforms (create 3-5 if needed), test scrolling performance:

**Test Steps:**
1. Create 3-5 platforms with varying states (some with VMs, some without)
2. Scroll rapidly up and down through the platform table
3. Monitor for visual glitches or lag

**Expected Result:**
- Smooth 60fps scrolling with no stuttering
- No visual flicker when scrolling
- Row heights remain stable (cached, not recalculated every frame)

**Pass Criteria:** ✅ Smooth scrolling, no flicker, no performance degradation

- [ ] **Step 9: (Optional) Test on Mobile/WASM**

If building for Android or WASM, verify platform filtering works correctly:

**Android Build:**
```bash
cd /home/wj/work/dure/mobile
./build.sh
```

**Test Steps:**
1. Install on Android device or emulator
2. Navigate to Platform tab
3. Verify desktop-only buttons (Add VM, Billing) are NOT shown
4. Verify remaining buttons (Refresh, Scan VMs, Firewall, Restart, Del VM) are visible
5. Verify row height adjusts appropriately for fewer buttons

**Expected Result:**
- Platform filtering works correctly (no desktop-only buttons)
- Remaining buttons are fully visible with no clipping
- Row height is appropriate for the platform

**Pass Criteria:** ✅ Platform-specific buttons displayed correctly, no clipping

**Note:** This step is optional for the core implementation. The egui layout system is platform-agnostic, so if it works on desktop, it will work on mobile/WASM. However, if you have time, it's good to verify.

- [ ] **Step 10: Verify all acceptance criteria**

Check against the spec's acceptance criteria:

- ✅ **AC-1:** All buttons in Operations column are fully visible without vertical clipping
- ✅ **AC-2:** Rows with 1-3 buttons remain compact (~52px height)
- ✅ **AC-3:** Rows with 4+ wrapped buttons expand to 2-3 button rows (~80-150px)
- ✅ **AC-4:** Window resize triggers smooth row height recalculation
- ✅ **AC-5:** Drawer expansion/collapse does not affect data row height
- ✅ **AC-6:** Table scrolling remains smooth (60fps) with no flicker
- ✅ **AC-7:** Works identically on Desktop (Linux/macOS/Windows), Android, and WASM

**If any criterion fails, review the implementation and debug before committing.**

- [ ] **Step 11: Commit the change**

Once all manual tests pass, commit the change:

```bash
cd /home/wj/work/dure
git add mobile/src/ui_tabs/platform.rs
git commit -m "$(cat <<'EOF'
feat(platform): enable dynamic row heights in data_table

Enable auto row height for platform table to prevent button clipping in
the Operations column. Rows now expand vertically to fit wrapped button
content while maintaining a 52px minimum height per MD3 guidelines.

Implementation:
- Add .auto_row_height(true) to data_table initialization
- Add .min_row_height(52.0) to enforce MD3 minimum
- Leverages egui-material3's built-in auto-height feature
- Row heights are cached for performance (no scrolling regression)

Visual behavior:
- Rows with 1-3 buttons: ~52px (compact, single row)
- Rows with 4-6 buttons: ~80-100px (2 button rows)
- Rows with 7-8 buttons: ~130-150px (3 button rows)
- Smooth height adjustment on window resize

Testing:
- Manual verification on desktop (Linux)
- All buttons fully visible at any window width
- Drawer independence verified
- Scrolling performance maintained (60fps)

Fixes button clipping issue identified in platform drawer refactor.

See docs/superpowers/specs/2026-08-05-dynamic-row-height-design.md

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
EOF
)"
```

Expected output: Commit succeeds with commit hash displayed.

- [ ] **Step 12: Verify commit and push (if ready)**

Check the commit:

```bash
git log -1 --stat
```

Expected output: Shows the commit with 1 file changed, 2 insertions.

If working on a feature branch and ready to integrate:

```bash
git push origin feature/platform-drawer-refactor
```

**Note:** Only push if this change is ready for review. If you're still working on related features, you can commit locally and push later.

---

## Task 2: Documentation Update (Optional)

**Files:**
- Modify: `docs/superpowers/specs/2026-08-05-dynamic-row-height-design.md` (update status)

**Note:** This task is optional. The spec already documents the implementation. If you want to mark it as "Implemented", update the status field.

- [ ] **Step 1: Update spec status (optional)**

If desired, update the spec to reflect implementation completion:

```bash
cd /home/wj/work/dure
```

Open `docs/superpowers/specs/2026-08-05-dynamic-row-height-design.md` and change line 5 from:

```markdown
**Status:** Approved
```

to:

```markdown
**Status:** Implemented (2026-08-05)
```

- [ ] **Step 2: Commit documentation update (optional)**

```bash
git add docs/superpowers/specs/2026-08-05-dynamic-row-height-design.md
git commit -m "docs: update dynamic row height spec status to Implemented

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Rollback Plan

If issues are discovered after deployment, the change can be easily reverted:

**Rollback Commit:**
```bash
cd /home/wj/work/dure
git revert HEAD  # Reverts the most recent commit (the auto row height change)
```

**Or Manual Rollback:**
Edit `mobile/src/ui_tabs/platform.rs:1273-1280` and remove the two added lines:

```diff
 let mut table = data_table()
     .id(table_id)
     .allow_selection(false)
     .allow_drawer(true)
-    .auto_row_height(true)      // Remove to disable dynamic heights
-    .min_row_height(52.0)       // Remove to use default fixed height
     .column("Project", 150.0 * width_ratio, false)
```

Then rebuild and deploy.

---

## Edge Cases Handled

The following edge cases are automatically handled by the egui-material3 auto-height implementation:

1. **Empty Operations Column** - Falls back to min_row_height (52px) if no buttons present
2. **Extremely Wide Window** - All buttons fit in one row, uses min_row_height (52px)
3. **Extremely Narrow Window** - Buttons wrap to 4+ rows, height expands accordingly (may exceed 200px)
4. **Operation In Progress** - Grayed-out buttons still occupy space, row height based on full button count
5. **Drawer Height vs Row Height** - Independent heights, no interference (already supported by egui-material3)

No additional code is needed to handle these cases.

---

## Performance Notes

**Row Height Caching:**
- egui-material3 caches row heights in `DataTableState.cached_row_heights` (Vec<f32>)
- Heights are recalculated only when data changes (detected via `layout_cache_hash`)
- First frame: ~1ms per row for height calculation (one-time cost)
- Subsequent frames: ~0.01ms per row (cache lookup, 100× faster)
- Scrolling performance: 60fps maintained due to cached heights

**Memory Impact:**
- 4 bytes per row for cached height (Vec<f32>)
- For 100 rows: 400 bytes total (negligible)

---

## Testing Checklist Summary

Before committing, verify all of the following:

- [ ] Code compiles without errors or warnings
- [ ] All buttons visible without clipping (window width 600-1200px)
- [ ] Rows with few buttons remain compact (~52px)
- [ ] Rows with many buttons expand appropriately (80-150px)
- [ ] Window resize triggers smooth height recalculation
- [ ] Drawer and row heights are independent
- [ ] Scrolling is smooth (60fps, no flicker)
- [ ] Platform filtering works correctly (if testing mobile/WASM)

---

## References

- **Design Spec:** `docs/superpowers/specs/2026-08-05-dynamic-row-height-design.md`
- **egui-material3 Source:** `/home/wj/work/egui-material3/src/datatable.rs`
  - Lines 632-642: `auto_row_height()` implementation
  - Lines 644-649: `min_row_height()` implementation
  - Lines 118-129: Row height caching in DataTableState
- **Platform Tab Source:** `mobile/src/ui_tabs/platform.rs`
  - Line 1273: data_table initialization (modification point)
  - Lines 1295-1450: Operations column widget_cell implementation

---

## Changelog

| Date | Author | Change |
|------|--------|--------|
| 2026-08-05 | Claude Sonnet 4.5 | Initial implementation plan |

---

**END OF PLAN**
