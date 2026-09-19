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

## Task Summary

**Task 1**: ProfileManager - Core types (ProfileContext, ProfileError)  
**Task 2**: ProfileManager - Name validation  
**Task 3**: ProfileManager - List profiles  
**Task 4**: ProfileManager - Create profile (keys + kdbx + config)  
**Task 5**: ProfileManager - Verify password  
**Task 6**: ProfileManager - Delete profile  
**Task 7**: UI - ProfileLoginDialog  
**Task 8**: UI - ProfileCreateDialog  
**Task 9**: UI - ProfileDeleteDialog  
**Task 10**: DureApp - Add profile state fields  
**Task 11**: DureApp - Profile selector ComboBox  
**Task 12**: DureApp - Dialog coordination logic  
**Task 13**: Tabs - Add profile state checks (platform, ssh, ns, site)  
**Task 14**: Database - Connect profile to db path  
**Task 15**: Integration tests + manual verification  

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-09-20-multi-profile-system.md`.

Two execution options:

**1. Subagent-Driven (recommended)** - Fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

Which approach?
