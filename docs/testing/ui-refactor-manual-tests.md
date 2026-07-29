# SSH and Domains Tab UI Refactor - Manual Test Checklist

## SSH Tab Testing

### Width Flexibility
- [ ] Resize window from 800px to 1920px - columns scale proportionally
- [ ] Very narrow window (< 500px) - verify text wraps, no horizontal scroll
- [ ] Wide window (> 2000px) - columns don't exceed 2x base width
- [ ] Operations buttons still clickable at all window sizes
- [ ] Text in Status column wraps properly when narrow

### No Regressions
- [ ] Drawer opens/closes correctly
- [ ] Host connection test works
- [ ] Refresh button updates status
- [ ] Delete host removes entry
- [ ] Add host dialog works

## Domains Tab Testing

### Main Table
- [ ] Main table shows domains with correct Provider formatting
- [ ] Cloudflare displays as "Cloudflare"
- [ ] GCP displays as "Google Cloud (email@example.com)"
- [ ] Porkbun displays as "Porkbun"
- [ ] DuckDNS displays as "DuckDNS"

### Drawer Functionality
- [ ] Expand drawer - nested records table appears
- [ ] Empty domain (no records) - shows "No records yet" message
- [ ] Drawer state persists when switching tabs and returning
- [ ] Multiple drawers can be open simultaneously
- [ ] Nested records table scrolls if > 10 records

### Operations
- [ ] Add Record button - opens dialog with domain pre-filled
- [ ] Nameservers button - opens nameserver comparison dialog
- [ ] Delete Domain button - removes domain and updates table
- [ ] Delete record (🗑 icon) - removes record from nested table

### Edge Cases
- [ ] Zero state: No domains → empty table with helpful message
- [ ] Very long domain name (> 50 chars) - verify text wrapping
- [ ] 50+ records in domain - verify smooth scrolling
- [ ] Rapid drawer toggle (10x) - no UI glitches
- [ ] Window resize while drawer open - nested table adjusts

## Performance Testing

### Domains Tab Large Dataset
- [ ] Load 100 domains with 20 records each
- [ ] Table renders in < 200ms (subjective, should feel instant)
- [ ] Drawer expansion is instant (< 50ms)
- [ ] Smooth scrolling (60 FPS, no jank)

## Browser Compatibility (WASM)
- [ ] N/A - Desktop only feature

## Results

Date tested: ____________
Tester: ____________
All tests passed: [ ] Yes [ ] No

Issues found:
1. 
2. 
3. 
