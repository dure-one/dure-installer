# Multi-Profile System Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement multi-profile system enabling users to manage multiple independent configurations, databases, and credentials with password-protected isolation.

**Architecture:** Centralized ProfileManager (pure business logic) + DureApp integration (UI state). Each profile has isolated directory with config.yml, SSH keys, KeePass database (key.kdbx), and SQLite database (dure.db).

**Tech Stack:** Rust (nightly), egui, smol async, keepass crate, Diesel ORM, thiserror

**Spec:** `docs/superpowers/specs/2026-09-20-multi-profile-system-design.md`

## Global Constraints

- Profile directory: `~/.config/dure_installer/{profile_name}/` (underscore not hyphen)
- Profile names: `a-z`, `A-Z`, `0-9`, `-`, `_` only, 1-64 chars, case-sensitive
- Password authentication required for all profile access
- No automatic migration: Old `~/.config/dure-installer/` untouched
- Test coverage: 80% minimum

---

## Implementation Tasks

This plan is split into logical phases. Each task follows TDD: test → fail → implement → pass → commit.

### Phase 1: ProfileManager Core (Tasks 1-6)

Create business logic layer for profile management. No UI dependencies.

### Phase 2: UI Dialogs (Tasks 7-9)

Create egui dialog widgets for login, create, delete operations.

### Phase 3: DureApp Integration (Tasks 10-12)

Wire up profile state, selector widget, and dialog coordination in main UI.

### Phase 4: Tab Updates (Task 13)

Update all tabs to check profile state before operations.

### Phase 5: Database Integration (Task 14)

Connect profile switching to database path changes.

### Phase 6: Testing & Polish (Task 15)

Integration tests and manual verification.

---

**Note:** Full task details with step-by-step TDD workflow, code examples, and test cases are documented in the design spec. Each task should:
1. Write failing tests first
2. Implement minimal code to pass
3. Run tests to verify
4. Commit with descriptive message

For detailed implementation guidance, refer to:
- `docs/superpowers/specs/2026-09-20-multi-profile-system-design.md` (complete design)
- Rust coding standards: `~/.claude/rules/ecc/rust/coding-style.md`
- Testing requirements: `~/.claude/rules/ecc/rust/testing.md`

## Detailed Task Implementation

### Task 1: ProfileManager Core - Data Structures

**Files:**
- Create: `mobile/src/calc/profile.rs`
- Modify: `mobile/src/calc/mod.rs`, `mobile/src/lib.rs`

- [ ] Create `mobile/src/calc/profile.rs` with module doc and imports
- [ ] Define `ProfileContext` struct with all path fields
- [ ] Define `ProfileError` enum with thiserror
- [ ] Add `pub mod profile;` to `mobile/src/calc/mod.rs`
- [ ] Add `get_profiles_base_dir()` to `mobile/src/lib.rs` with test override support
- [ ] Run `cargo check --lib` to verify compilation
- [ ] Commit: "feat(profile): add ProfileContext and ProfileError types"

### Task 2: ProfileManager - Name Validation  

**Files:**
- Modify: `mobile/src/calc/profile.rs`

- [ ] Write failing test `test_validate_name_valid()` and `test_validate_name_invalid()`
- [ ] Run test to verify failure
- [ ] Add `ProfileManager` struct and `validate_name()` method
- [ ] Run test to verify pass
- [ ] Commit: "feat(profile): add ProfileManager with name validation"

### Task 3: ProfileManager - List Profiles

**Files:**
- Modify: `mobile/src/calc/profile.rs`, `mobile/src/lib.rs`

- [ ] Write failing test `test_list_profiles_empty()`
- [ ] Run test to verify failure
- [ ] Implement `ProfileManager::list_profiles()` with directory scanning
- [ ] Implement `validate_profile_dir()` helper
- [ ] Run test to verify pass  
- [ ] Commit: "feat(profile): implement ProfileManager::list_profiles"

### Task 4: ProfileManager - Create Profile

**Files:**
- Modify: `mobile/src/calc/profile.rs`

- [ ] Write failing tests `test_create_profile_success()` and `test_create_profile_invalid_name()`
- [ ] Run tests to verify failure
- [ ] Implement `ProfileManager::create_profile()` main function
- [ ] Implement `generate_keypair()` helper using ed25519-dalek
- [ ] Implement `create_kdbx()` helper using keepass crate
- [ ] Run tests to verify pass
- [ ] Commit: "feat(profile): implement ProfileManager::create_profile"

### Task 5: ProfileManager - Verify Password

**Files:**
- Modify: `mobile/src/calc/profile.rs`

- [ ] Write failing tests `test_verify_password_correct()` and `test_verify_password_incorrect()`
- [ ] Run tests to verify failure
- [ ] Implement `ProfileManager::verify_password()` using keepass crate
- [ ] Run tests to verify pass
- [ ] Commit: "feat(profile): implement ProfileManager::verify_password"

### Task 6: ProfileManager - Delete Profile

**Files:**
- Modify: `mobile/src/calc/profile.rs`

- [ ] Write failing tests `test_delete_profile()` and `test_delete_profile_not_found()`
- [ ] Run tests to verify failure
- [ ] Implement `ProfileManager::delete_profile()`
- [ ] Run all profile tests to verify pass
- [ ] Commit: "feat(profile): implement ProfileManager::delete_profile"

### Task 7: UI - ProfileLoginDialog

**Files:**
- Create: `mobile/src/ui_dlg/profile_login.rs`
- Modify: `mobile/src/ui_dlg/mod.rs`

- [ ] Create `profile_login.rs` with `ProfileLoginDialog` struct
- [ ] Implement `show()` method returning `Option<String>`
- [ ] Implement `set_error()` and `reset()` methods
- [ ] Export in `ui_dlg/mod.rs`
- [ ] Run `cargo check --lib` to verify
- [ ] Commit: "feat(ui): add ProfileLoginDialog widget"

### Task 8: UI - ProfileCreateDialog

**Files:**
- Create: `mobile/src/ui_dlg/profile_create.rs`
- Modify: `mobile/src/ui_dlg/mod.rs`

- [ ] Create `profile_create.rs` with `ProfileCreateDialog` struct
- [ ] Implement `show()` method with inline validation
- [ ] Implement `reset()` method
- [ ] Export in `ui_dlg/mod.rs`
- [ ] Run `cargo check --lib` to verify
- [ ] Commit: "feat(ui): add ProfileCreateDialog widget"

### Task 9: UI - ProfileDeleteDialog

**Files:**
- Create: `mobile/src/ui_dlg/profile_delete.rs`  
- Modify: `mobile/src/ui_dlg/mod.rs`

- [ ] Create `profile_delete.rs` with `ProfileDeleteDialog` struct
- [ ] Implement `show()` method returning `Option<bool>`
- [ ] Export in `ui_dlg/mod.rs`
- [ ] Run `cargo check --lib` to verify
- [ ] Commit: "feat(ui): add ProfileDeleteDialog widget"

### Task 10: DureApp - Add Profile State

**Files:**
- Modify: `mobile/src/dure_stt.rs`

- [ ] Add profile dialog state fields to DureApp struct
- [ ] Add `current_profile: Option<ProfileContext>` field
- [ ] Add `pending_profile_name: Option<String>` field
- [ ] Initialize in Default impl
- [ ] Run `cargo check` to verify
- [ ] Commit: "feat(profile): add profile state to DureApp"

### Task 11: DureApp - Profile Selector

**Files:**
- Modify: `mobile/src/dure.rs`

- [ ] Add profile selector ComboBox in top-left of window
- [ ] Show current profile or "None"
- [ ] List profiles from `ProfileManager::list_profiles()`
- [ ] Add "+ Create New Profile" option
- [ ] Add "Delete Profile" option (when profile selected)
- [ ] Run `cargo check` to verify
- [ ] Commit: "feat(ui): add profile selector ComboBox"

### Task 12: DureApp - Dialog Coordination

**Files:**
- Modify: `mobile/src/dure.rs`

- [ ] Add login dialog logic (show, verify password, handle success/failure)
- [ ] Add create dialog logic (show, create profile, auto-login)
- [ ] Add delete dialog logic (show, unload if active, delete)
- [ ] Connect profile selection to login dialog trigger
- [ ] Run `cargo check` to verify
- [ ] Commit: "feat(profile): add dialog coordination logic"

### Task 13: Tabs - Profile State Checks

**Files:**
- Modify: `mobile/src/ui_tabs/platform.rs`, `ssh.rs`, `ns.rs`, `site.rs`

For each tab:
- [ ] Add profile check at start of `ui()` method
- [ ] Show "No profile selected" message when None
- [ ] Return early if no profile
- [ ] Run `cargo check` to verify
- [ ] Commit: "feat(tabs): add profile state checks to all tabs"

### Task 14: Database Integration

**Files:**
- Modify: `mobile/src/calc/db.rs`

- [ ] Make `set_db_path()` public
- [ ] Add profile path update in DureApp profile load logic
- [ ] Test database switching between profiles
- [ ] Run `cargo check` to verify
- [ ] Commit: "feat(profile): connect profile to database path"

### Task 15: Integration Testing

**Files:**
- Create: `tests/profile_integration.rs`

- [ ] Write integration test for full create → login → delete workflow
- [ ] Write test for multi-profile isolation
- [ ] Run `cargo test --test profile_integration`
- [ ] Manual testing: create profiles, switch, delete
- [ ] Commit: "test(profile): add integration tests"

## Execution Complete

After all tasks are done, use `superpowers:finishing-a-development-branch` to complete the work.
