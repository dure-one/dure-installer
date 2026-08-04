# Platform Drawer Refactor - Testing Checklist

## Functional Requirements

- [ ] **FR1:** Steps column shows emoji progress bar (✅ → ✅ → ⏳ → ⚪ → ⚪)
  - Tested on: [ ] Desktop [ ] Android [ ] WASM

- [ ] **FR2:** Drawer displays compact grid layout (2-3 columns responsive)
  - Tested widths: [ ] >600px (3 col) [ ] 400-600px (2 col) [ ] <400px (1 col)

- [ ] **FR3:** Operation buttons show immediate feedback (⏳ → ✅/✗)
  - Tested ops: [ ] Firewall [ ] Restart [ ] Delete [ ] Scan [ ] Add VM

- [ ] **FR4:** SSH actions dropdown works
  - [ ] Copy Command [ ] Copy Key [ ] Copy IP

- [ ] **FR5:** Grid auto-reflows responsively
  - [ ] Resize window from wide → narrow, verify column change

- [ ] **FR6:** Event-based updates (no polling)
  - [ ] Firewall update shows progress immediately
  - [ ] No full table reload on operations

## Non-Functional Requirements

- [ ] **NFR2:** SVG emoji with Unicode fallback
  - [ ] SVG renders on Desktop
  - [ ] Unicode shows if SVG unavailable

- [ ] **NFR3:** No breaking changes to ViewModel API
  - [ ] Existing event names unchanged
  - [ ] New OperationFailed event optional (not breaking)

- [ ] **NFR4:** Works on all platforms
  - [ ] Desktop Linux
  - [ ] Desktop macOS
  - [ ] Desktop Windows
  - [ ] Android
  - [ ] WASM

## Edge Cases

- [ ] Missing data handling
  - [ ] No email: shows "Not connected"
  - [ ] No VM: shows "— No VM created"
  - [ ] No external IP: shows "⚠ No external IP"
  - [ ] SSH key missing: shows warning

- [ ] Stale data warning
  - [ ] Fresh (<1 hour): no warning
  - [ ] Stale (>1 hour): yellow warning icon

- [ ] Operation failures
  - [ ] Network error: shows ✗ Failed with tooltip
  - [ ] Auto-clear after 10 seconds

- [ ] Auto-clear timing
  - [ ] Completed: clears after 3 seconds
  - [ ] Failed: clears after 10 seconds

- [ ] Concurrent operations
  - [ ] Buttons disabled during InProgress
  - [ ] Refresh always enabled

## Performance

- [ ] No full reloads on events (check via logging)
- [ ] Smooth UI during operations (no flicker)
- [ ] Grid renders quickly on window resize

## Regression Testing

- [ ] All existing platform operations still work
- [ ] Config save/load unchanged
- [ ] ViewModel events fire correctly
- [ ] No console errors or warnings

## Accessibility

- [ ] Unicode emoji visible without SVG
- [ ] Button tooltips present
- [ ] Error messages clear and actionable

## Sign-off

- [ ] All tests passed
- [ ] No regressions found
- [ ] Ready for merge

Tested by: ___________________  
Date: ___________________
