# Manual Test: Language Setting Persistence

## Bug Description
Language setting was always reset to English on app restart, even when Korean was saved.

## Root Cause
Settings were loaded from disk but language was never applied to i18n system.

## Fix Applied
- File: `mobile/src/dure.rs` line ~1374
- Added: Call to `crate::i18n::set_language(&self.settings.language)` after loading settings

## Manual Test Steps

### Test 1: Fresh Profile with Korean Language

1. **Build and run the app**
   ```bash
   cargo run --features gui
   ```

2. **Create new profile**
   - Click "Create Profile" button
   - Enter profile name: `test-korean`
   - Enter password
   - Click "Create"

3. **Change language to Korean**
   - Open Settings dialog (gear icon or menu)
   - Change "Language" dropdown to "ko-KR" (Korean)
   - Click "Save"
   - **Verify**: UI immediately switches to Korean

4. **Restart application**
   ```bash
   # Close app
   # Run again
   cargo run --features gui
   ```

5. **Login to profile**
   - Select `test-korean` from profile dropdown
   - Enter password
   - Click "Login"

6. **VERIFY**: UI displays in Korean immediately after login
   - ✅ PASS: UI is in Korean
   - ❌ FAIL: UI is in English (bug not fixed)

### Test 2: Existing Profile Language Persists

1. **With app running and Korean language active**
   - Open Settings
   - Change language to "en-US" (English)
   - Click "Save"
   - **Verify**: UI switches to English

2. **Change back to Korean**
   - Open Settings
   - Change language to "ko-KR"
   - Click "Save"

3. **Restart app and login**
   - Close app
   - Run again: `cargo run --features gui`
   - Login to `test-korean` profile

4. **VERIFY**: UI displays in Korean

### Test 3: Auto Language Detection

1. **Create new profile** `test-auto`

2. **Check default language**
   - Open Settings
   - Default should be "Auto"
   - **Verify**: UI displays in system language (Korean on Korean system, English on English system)

3. **Save with "Auto" setting**
   - Keep language as "Auto"
   - Click "Save"

4. **Restart and verify**
   - Restart app
   - Login to `test-auto`
   - **Verify**: UI displays in system language

## Expected Log Output

With fix applied, you should see in logs:

```
Settings loaded from "/home/user/.config/dure_installer/test-korean/settings.yml"
Settings: show_logs=false, log_level=Error, language=ko-KR
Language applied: ko-KR
Profile loaded successfully: test-korean
```

Without fix (old behavior):

```
Settings loaded from "/home/user/.config/dure_installer/test-korean/settings.yml"
Settings: show_logs=false, log_level=Error
Profile loaded successfully: test-korean
```

## Regression Tests

Verify other settings still work:

1. **Log Level**
   - Change log level to "Debug"
   - Save and restart
   - Verify logs show debug messages

2. **Show Logs Toggle**
   - Enable "Show Logs" checkbox
   - Save and restart
   - Verify log pane appears

3. **Theme**
   - Change theme
   - Save and restart
   - Verify theme persists

## Cleanup

After testing, delete test profiles:

```bash
rm -rf ~/.config/dure_installer/test-korean
rm -rf ~/.config/dure_installer/test-auto
```

## Test Result

- [ ] Test 1: Fresh profile with Korean - PASS
- [ ] Test 2: Language persists across restarts - PASS
- [ ] Test 3: Auto language detection - PASS
- [ ] Regression: Log settings work - PASS
- [ ] Regression: Theme settings work - PASS

**Date Tested**: _____________
**Tester**: _____________
**Result**: PASS / FAIL
**Notes**: _____________
