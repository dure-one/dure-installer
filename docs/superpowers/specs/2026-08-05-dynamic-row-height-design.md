# Platform Table Dynamic Row Height Design Specification

**Date:** 2026-08-05  
**Author:** Claude Sonnet 4.5  
**Status:** Approved  
**Related Plan:** Will be created in `docs/superpowers/plans/2026-08-05-dynamic-row-height.md`

---

## Executive Summary

Enable dynamic row heights in the platform tab data_table to prevent button clipping in the Operations column. This is a minimal configuration change leveraging existing auto-height functionality in egui-material3.

**Problem:** Platform table rows have fixed height (~52px), causing wrapped buttons in the Operations column to be clipped and hidden from view.

**Solution:** Enable `.auto_row_height(true)` on the data_table, allowing rows to expand vertically to fit their content while maintaining a 52px minimum.

**Impact:** 2-line code change in platform.rs, no API modifications, no breaking changes.

---

## Background

### Current State

The platform tab displays GCP projects in a Material Design data_table with four columns:
- **Project** (150px): Project ID
- **Type** (80px): Platform type (GCP)
- **Steps** (250px): EmojiProgressBar showing OAuth/Project/VM/Firewall/SSH status
- **Operations** (260px): Action buttons for managing the platform

The Operations column can contain up to 8 buttons depending on platform state:
1. **Refresh** (always enabled)
2. **Add VM** (desktop only, enabled when no VM exists)
3. **Scan VMs** (enabled when project selected)
4. **Firewall** (enabled when project selected and firewall not updated)
5. **Restart** (enabled when VM exists)
6. **Del VM** (enabled when VM exists)
7. **Billing** (desktop only, enabled when project selected)
8. **Regen** (currently commented out)

These buttons use `ui.horizontal_wrapped()` to wrap to multiple lines when they don't fit in the column width (260px). However, the data_table currently uses fixed row height, causing wrapped buttons to be clipped.

### Recent Related Work

- **2026-08-04:** Platform drawer refactor (commit 2637c34) replaced text-based status with EmojiProgressBar and StatusGrid components
- **2026-08-05:** Operations column removed ScrollArea to allow wrapping (line 1296 comment: "Remove ScrollArea to allow dynamic row height")

The foundation is in place - buttons now wrap correctly. The remaining issue is that row height doesn't expand to show the wrapped content.

---

## Requirements

### Functional Requirements

**FR-1:** All buttons in the Operations column must be fully visible without clipping  
**FR-2:** Row height must dynamically adapt to the number of wrapped button rows  
**FR-3:** Rows with few buttons should remain compact (maintain ~52px minimum)  
**FR-4:** Solution must work on all platforms (Desktop Linux/macOS/Windows, Android, WASM)

### Non-Functional Requirements

**NFR-1:** No performance regression in table scrolling  
**NFR-2:** No visual flicker during height recalculation  
**NFR-3:** Solution must be simple and maintainable (prefer configuration over custom code)  
**NFR-4:** No breaking changes to ViewModel API or data structures

### Constraints

**C-1:** Must use existing egui-material3 APIs (no fork modifications required)  
**C-2:** Must preserve existing drawer functionality (independent of row height)  
**C-3:** Must follow Material Design 3 spacing and sizing guidelines  
**C-4:** Must maintain visual consistency across different platform states

---

## Design Decision

### Approach Selection

Three approaches were considered:

| Approach | Pros | Cons | Decision |
|----------|------|------|----------|
| **A: Auto Row Height** | Simple (1 line), native feature, truly dynamic | Uneven row heights | ✅ **SELECTED** |
| **B: Fixed Tall Rows** | Uniform appearance | Wastes space, might still clip | ❌ Rejected |
| **C: Hybrid with Max** | Balanced | Requires API changes, complex | ❌ Rejected |

**Rationale for Selection:**
- Auto-height is a native, battle-tested feature in egui-material3 (see datatable.rs lines 632-642)
- Uneven row heights are standard in Material Design for dynamic content
- Zero implementation risk - pure configuration change
- Performance optimizations (row height caching) already built-in

### Architecture

**Component:** egui-material3 MaterialDataTable (datatable.rs)

**Existing API Used:**
```rust
pub fn auto_row_height(mut self, enabled: bool) -> Self {
    self.auto_height = enabled;
    if enabled {
        // Set a minimal default height to allow content-based sizing
        self.theme.data_row_min_height = Some(20.0);
    }
    self
}

pub fn min_row_height(mut self, height: f32) -> Self {
    self.theme.data_row_min_height = Some(height);
    self
}
```

**How It Works:**
1. egui's layout system calculates natural size for each cell's content
2. With `auto_row_height(true)`, row height = `max(min_row_height, content_height)`
3. Operations column's `horizontal_wrapped` UI requests space for all wrapped buttons
4. Row heights are cached in `DataTableState.cached_row_heights` (line 120 of datatable.rs)
5. Cache invalidates only when row data changes (detected via layout_cache_hash)

---

## Implementation

### File Changes

**Modified Files:**
- `mobile/src/ui_tabs/platform.rs` (1 location, 2 lines added)

**No New Files Required**

### Code Changes

**Location:** `mobile/src/ui_tabs/platform.rs:1273`

**Before:**
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

**After:**
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

**Diff:**
```diff
 let mut table = data_table()
     .id(table_id)
     .allow_selection(false)
     .allow_drawer(true)
+    .auto_row_height(true)      // Enable dynamic row heights
+    .min_row_height(52.0)       // Maintain MD3 minimum height
     .column("Project", 150.0 * width_ratio, false)
```

### Configuration Details

**`.auto_row_height(true)`:**
- Enables content-based row height calculation
- Row height = max(min_row_height, natural_content_height)
- Triggers row height caching for performance

**`.min_row_height(52.0)`:**
- Enforces Material Design 3 minimum row height (52dp)
- Prevents rows from collapsing too small
- Ensures touch targets remain adequately sized

---

## Visual Specifications

### Row Height Behavior

| Scenario | Button Count | Expected Height | Rationale |
|----------|-------------|-----------------|-----------|
| Single row of buttons | 1-3 buttons | ~52px (minimum) | Maintains compact layout |
| Two rows of buttons | 4-6 buttons | ~80-100px | 2 × button height + spacing |
| Three rows of buttons | 7-8 buttons | ~130-150px | 3 × button height + spacing |

**Button Dimensions:**
- Height: ~28px (MaterialButton.small())
- Padding: 6px horizontal, 2px vertical (see platform.rs:1300-1301)
- Spacing: 2px horizontal, 2px vertical (see platform.rs:1298-1299)

### Visual Examples

**Compact Row (No VM, Project Selected):**
```
┌─────────────┬──────┬───────────────┬──────────────────────────────┐
│ my-project  │ GCP  │ ✅📧🔵🔵🔵    │ [Refresh] [Scan VMs] [Bill.] │ ~52px
└─────────────┴──────┴───────────────┴──────────────────────────────┘
```

**Expanded Row (VM Exists, All Buttons):**
```
┌─────────────┬──────┬───────────────┬──────────────────────────────┐
│ my-project  │ GCP  │ ✅✅✅✅🔵    │ [Refresh] [Scan VMs] [Fire.] │
│             │      │               │ [Restart] [Del VM] [Billing] │ ~80px
└─────────────┴──────┴───────────────┴──────────────────────────────┘
```

### Material Design Compliance

- **Spacing:** 2px item spacing (compact button layout)
- **Minimum Height:** 52px (MD3 data table row minimum)
- **Vertical Alignment:** Center (existing VAlign::Center preserved)
- **Touch Targets:** 48dp minimum maintained by button size + row padding

---

## Testing Strategy

### Manual Testing Scenarios

**Test 1: Button Wrapping on Desktop**
- **Setup:** Desktop build, window width ~800px
- **Steps:**
  1. Create platform with no VM (shows Add VM, Scan VMs, Refresh, Billing)
  2. Verify all 4 buttons visible, no clipping
  3. Add VM (shows Scan VMs, Refresh, Restart, Del VM, Firewall, Billing - 6 buttons)
  4. Verify all 6 buttons visible, row expanded to ~2 button rows
- **Expected:** All buttons fully visible, row height ~80-100px

**Test 2: Window Resize Behavior**
- **Setup:** Desktop build with VM created
- **Steps:**
  1. Start with wide window (~1200px) - buttons in 1-2 rows
  2. Resize window narrower (~600px) - buttons wrap to 3+ rows
  3. Verify row height increases to show all buttons
  4. Resize wider - verify row height decreases smoothly
- **Expected:** No clipping at any window width, smooth re-layout

**Test 3: Platform State Transitions**
- **Setup:** Desktop build
- **Steps:**
  1. Start with no VM (compact row)
  2. Click "Add VM", wait for operation to complete
  3. Verify row expands as new buttons become available
  4. Click "Del VM", wait for completion
  5. Verify row compacts as buttons disappear
- **Expected:** Row height animates smoothly during state transitions

**Test 4: Drawer Independence**
- **Setup:** Desktop build with VM
- **Steps:**
  1. Click row to expand drawer
  2. Verify data row height remains ~80-100px (not affected by drawer)
  3. Verify drawer shows StatusGrid correctly
  4. Close drawer, verify data row height unchanged
- **Expected:** Data row and drawer heights are independent

**Test 5: Performance - Large Table Scrolling**
- **Setup:** Desktop build with 10+ platform rows
- **Steps:**
  1. Create multiple platforms with varying states (some with VM, some without)
  2. Scroll rapidly through the table
  3. Monitor for flicker or lag
- **Expected:** Smooth 60fps scrolling, no visual glitches, cached heights prevent recalculation

**Test 6: Mobile/WASM Platform Filtering**
- **Setup:** Android or WASM build
- **Steps:**
  1. Verify desktop-only buttons (Add VM, Billing) are hidden
  2. Check remaining buttons (Refresh, Scan VMs, Firewall, Restart, Del VM) are visible
  3. Verify row height adjusts to fewer buttons
- **Expected:** All visible buttons fully shown, row height appropriate for platform

### Acceptance Criteria

- ✅ **AC-1:** All buttons in Operations column are fully visible without vertical clipping
- ✅ **AC-2:** Rows with 1-3 buttons remain compact (~52px height)
- ✅ **AC-3:** Rows with 4+ wrapped buttons expand to 2-3 button rows (~80-150px)
- ✅ **AC-4:** Window resize triggers smooth row height recalculation
- ✅ **AC-5:** Drawer expansion/collapse does not affect data row height
- ✅ **AC-6:** Table scrolling remains smooth (60fps) with no flicker
- ✅ **AC-7:** Works identically on Desktop (Linux/macOS/Windows), Android, and WASM

---

## Risk Analysis

### Risk Matrix

| Risk | Likelihood | Impact | Mitigation | Rollback |
|------|-----------|--------|------------|----------|
| Uneven row heights disrupt visual hierarchy | Low | Minor | Standard MD3 pattern for dynamic content | Remove 2 lines |
| Performance regression on large tables | Very Low | Minor | Row height caching already implemented | Remove 2 lines |
| Layout issues on mobile/WASM | Very Low | Minor | egui layout is platform-agnostic | Remove 2 lines |
| First-frame height calculation flicker | Very Low | Negligible | Caching prevents repeated calculation | None needed |

### Rollback Plan

**Simple Revert:**
```diff
 let mut table = data_table()
     .id(table_id)
     .allow_selection(false)
     .allow_drawer(true)
-    .auto_row_height(true)      // Remove if issues arise
-    .min_row_height(52.0)       // Remove if issues arise
     .column("Project", 150.0 * width_ratio, false)
```

**No Data Migration Needed:** This is a pure UI change with no persistent state impact.

---

## Performance Considerations

### Row Height Caching

**Mechanism:** (from datatable.rs:118-129)
```rust
pub struct DataTableState {
    /// Cached row heights to avoid recalculating text layout every frame
    #[serde(skip)]
    pub cached_row_heights: Vec<f32>,
    /// Hash of layout-affecting properties to detect when recalculation is needed
    #[serde(skip)]
    pub layout_cache_hash: u64,
    // ... other fields
}
```

**How It Works:**
1. First frame: Calculate each row's natural height, store in `cached_row_heights`
2. Generate `layout_cache_hash` from row count, column widths, data
3. Subsequent frames: Reuse cached heights if hash matches
4. Hash mismatch (data changed): Recalculate and update cache

**Benchmark Expectations:**
- **Initial render:** ~1ms per row for height calculation (one-time)
- **Cached frames:** ~0.01ms per row (lookup only, 100× faster)
- **100-row table:** ~100ms first frame, <1ms subsequent frames
- **Scroll performance:** 60fps maintained (cached heights used)

### Memory Impact

**Additional Memory per Table:**
- `Vec<f32>` for cached_row_heights: 4 bytes × row_count
- Example: 100 rows = 400 bytes (negligible)

---

## Edge Cases

### Edge Case 1: Empty Operations Column
**Scenario:** Row with no enabled buttons (shouldn't occur in practice)  
**Behavior:** Falls back to min_row_height (52px)  
**Handling:** No special code needed - automatic fallback

### Edge Case 2: Extremely Wide Window
**Scenario:** All 8 buttons fit in single row  
**Behavior:** Row uses min_row_height (52px)  
**Handling:** Natural behavior, no clipping

### Edge Case 3: Extremely Narrow Window
**Scenario:** Buttons wrap to 4+ rows  
**Behavior:** Row expands to show all buttons (may exceed 200px)  
**Handling:** Acceptable - ensuring visibility is priority over height constraint

### Edge Case 4: Operation In Progress State
**Scenario:** Only "Refresh" button enabled, others grayed out  
**Behavior:** All buttons still rendered (grayed), row height based on full button count  
**Handling:** Correct behavior - grayed buttons still occupy space

### Edge Case 5: Drawer Height vs Row Height
**Scenario:** Drawer content shorter than expanded data row  
**Behavior:** Drawer and data row heights are independent  
**Handling:** Already supported by egui-material3 drawer implementation

---

## Future Enhancements

**Not in Scope for This Design:**

1. **Max Row Height Constraint:**
   - Would require adding `.max_row_height()` API to datatable.rs
   - Use case: Prevent extremely tall rows in pathological cases
   - Priority: Low (no known need)

2. **Dropdown Menu Alternative:**
   - Replace button row with single "Actions ▼" dropdown
   - Use case: Further space savings
   - Priority: Low (current solution sufficient)

3. **Icon-Only Buttons:**
   - Replace text buttons with icon-only buttons (smaller)
   - Use case: Fit more buttons in single row
   - Priority: Low (text buttons more accessible)

4. **Virtualized Table Rows:**
   - Only render visible rows for extremely large tables (1000+ rows)
   - Use case: Performance optimization for large datasets
   - Priority: Low (current platform counts are <50)

---

## Documentation Impact

**No User Documentation Updates Required:**
- This is an internal UX improvement
- No user-facing feature changes
- No API modifications

**Developer Documentation:**
- This spec serves as documentation
- Commit message will reference this spec
- No additional docs needed

---

## Success Metrics

**Qualitative Metrics:**
- ✅ User can see all operation buttons without scrolling
- ✅ Table maintains professional, organized appearance
- ✅ No user complaints about clipped buttons

**Quantitative Metrics:**
- ✅ Zero button clipping incidents (visual inspection)
- ✅ Table scroll FPS ≥ 60 (performance profiling)
- ✅ Row height calculation < 2ms per row (performance profiling)

---

## References

### Code References
- **datatable.rs:** `/home/wj/work/egui-material3/src/datatable.rs`
  - Lines 632-642: `auto_row_height()` implementation
  - Lines 644-649: `min_row_height()` implementation
  - Lines 118-129: Row height caching in DataTableState
- **platform.rs:** `/home/wj/work/dure/mobile/src/ui_tabs/platform.rs`
  - Line 1273: data_table initialization (modification point)
  - Lines 1295-1450: Operations column widget_cell implementation

### Related Plans
- **2026-08-04:** Platform Drawer Refactor (`docs/superpowers/plans/2026-08-04-platform-drawer-refactor.md`)
  - Context for recent EmojiProgressBar and StatusGrid work

### Material Design Specifications
- **MD3 Data Tables:** https://m3.material.io/components/data-tables
  - Row height: 52dp default, flexible for content
  - Touch targets: 48dp minimum

---

## Appendix: Alternative Approaches Considered

### Alternative 1: ScrollArea Inside Cell
**Description:** Keep fixed row height, add vertical ScrollArea to Operations cell  
**Pros:** Uniform row heights  
**Cons:** Poor UX (nested scrolling), accessibility issues  
**Rejected:** Violates UX best practices

### Alternative 2: Tooltip with All Actions
**Description:** Show only primary actions, rest in tooltip  
**Pros:** Always compact  
**Cons:** Hides functionality, poor discoverability  
**Rejected:** Reduces feature visibility

### Alternative 3: Modify egui-material3
**Description:** Add custom row height calculation logic to datatable.rs  
**Pros:** Full control  
**Cons:** Unnecessary - feature already exists  
**Rejected:** Over-engineering

---

## Changelog

| Date | Author | Change |
|------|--------|--------|
| 2026-08-05 | Claude Sonnet 4.5 | Initial design specification |

---

**END OF SPECIFICATION**
