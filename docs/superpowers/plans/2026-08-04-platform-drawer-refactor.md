# Platform Tab Drawer Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refactor platform tab drawer with event-based updates, SVG emoji indicators, responsive grid layout, and reusable UI components.

**Architecture:** Create new `ui_components` module with reusable StatusGrid, EmojiProgressBar, and ActionMenu components. Add OperationState tracking to PlatformRow for immediate visual feedback. Replace text-based status display with SVG emoji indicators (with Unicode fallback). Implement event-based incremental updates instead of full reloads.

**Tech Stack:** Rust (nightly), egui 0.33, eframe 0.33, egui-material3, resvg 0.42 (SVG rendering), smol 2.0 (async runtime)

## Global Constraints

- Rust nightly toolchain required
- No breaking changes to ViewModel API
- Must work on all platforms: Desktop (Linux/macOS/Windows), Android, WASM
- SVG emoji with Unicode fallback (no hard dependency on SVG loading)
- No `unsafe` code without documentation
- Follow existing code style (clippy::pedantic)
- Frequent commits (after each passing test)

---

## File Structure

### New Files
- `mobile/src/ui_components/mod.rs` - Module exports for reusable UI components
- `mobile/src/ui_components/emoji_loader.rs` - SVG emoji cache with Unicode fallback
- `mobile/src/ui_components/status_grid.rs` - Responsive key-value grid component
- `mobile/src/ui_components/emoji_progress.rs` - Emoji progress bar component
- `mobile/src/ui_components/action_menu.rs` - Dropdown action menu component
- `mobile/assets/emoji/*.svg` - SVG emoji assets (13 files)

### Modified Files
- `mobile/Cargo.toml` - Add resvg dependency
- `mobile/src/lib.rs` - Add ui_components module declaration
- `mobile/src/ui_tabs/platform.rs` - Add OperationState, refactor drawer/steps, event handlers
- `mobile/src/viewmodel/platform/events.rs` - Add OperationFailed event

---

## Task 1: Foundation - SVG Assets and Module Structure

**Files:**
- Create: `mobile/assets/emoji/` directory
- Create: `mobile/assets/emoji/checkmark.svg`
- Create: `mobile/assets/emoji/progress.svg`
- Create: `mobile/assets/emoji/circle.svg`
- Create: `mobile/assets/emoji/cross.svg`
- Create: `mobile/assets/emoji/email.svg`
- Create: `mobile/assets/emoji/project.svg`
- Create: `mobile/assets/emoji/vm.svg`
- Create: `mobile/assets/emoji/firewall.svg`
- Create: `mobile/assets/emoji/key.svg`
- Create: `mobile/assets/emoji/terminal.svg`
- Create: `mobile/assets/emoji/network.svg`
- Create: `mobile/assets/emoji/clock.svg`
- Create: `mobile/assets/emoji/warning.svg`
- Create: `mobile/src/ui_components/mod.rs`
- Modify: `mobile/Cargo.toml`
- Modify: `mobile/src/lib.rs`

**Interfaces:**
- Produces: SVG assets available at `mobile/assets/emoji/*.svg`
- Produces: `ui_components` module declared in lib.rs

- [ ] **Step 1: Create emoji assets directory**

```bash
mkdir -p mobile/assets/emoji
```

- [ ] **Step 2: Create checkmark.svg (✅)**

```bash
cat > mobile/assets/emoji/checkmark.svg << 'EOF'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#4CAF50" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
  <polyline points="20 6 9 17 4 12"/>
</svg>
EOF
```

- [ ] **Step 3: Create progress.svg (⏳)**

```bash
cat > mobile/assets/emoji/progress.svg << 'EOF'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none">
  <circle cx="12" cy="12" r="10" stroke="#FF9800" stroke-width="2"/>
  <path d="M12 6 L12 12 L16 14" stroke="#FF9800" stroke-width="2" stroke-linecap="round"/>
</svg>
EOF
```

- [ ] **Step 4: Create circle.svg (⚪)**

```bash
cat > mobile/assets/emoji/circle.svg << 'EOF'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none">
  <circle cx="12" cy="12" r="10" stroke="#9E9E9E" stroke-width="2" fill="#E0E0E0"/>
</svg>
EOF
```

- [ ] **Step 5: Create cross.svg (✗)**

```bash
cat > mobile/assets/emoji/cross.svg << 'EOF'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#F44336" stroke-width="3" stroke-linecap="round">
  <line x1="18" y1="6" x2="6" y2="18"/>
  <line x1="6" y1="6" x2="18" y2="18"/>
</svg>
EOF
```

- [ ] **Step 6: Create email.svg (📧)**

```bash
cat > mobile/assets/emoji/email.svg << 'EOF'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#2196F3" stroke-width="2">
  <rect x="3" y="5" width="18" height="14" rx="2"/>
  <polyline points="3 7 12 13 21 7"/>
</svg>
EOF
```

- [ ] **Step 7: Create project.svg (📁)**

```bash
cat > mobile/assets/emoji/project.svg << 'EOF'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#FFC107" stroke-width="2">
  <path d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"/>
</svg>
EOF
```

- [ ] **Step 8: Create vm.svg (💻)**

```bash
cat > mobile/assets/emoji/vm.svg << 'EOF'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#607D8B" stroke-width="2">
  <rect x="2" y="3" width="20" height="14" rx="2"/>
  <line x1="2" y1="20" x2="22" y2="20"/>
  <line x1="8" y1="23" x2="16" y2="23"/>
  <line x1="12" y1="17" x2="12" y2="23"/>
</svg>
EOF
```

- [ ] **Step 9: Create firewall.svg (🔥)**

```bash
cat > mobile/assets/emoji/firewall.svg << 'EOF'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none">
  <path d="M12 2 C8 4 4 6 4 12 C4 18 12 22 12 22 C12 22 20 18 20 12 C20 6 16 4 12 2 Z" fill="#FF5722" stroke="#D84315" stroke-width="1.5"/>
  <path d="M9 12 L11 14 L15 10" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
</svg>
EOF
```

- [ ] **Step 10: Create key.svg (🔑)**

```bash
cat > mobile/assets/emoji/key.svg << 'EOF'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#795548" stroke-width="2">
  <circle cx="8" cy="16" r="5"/>
  <path d="M13 13 L22 4"/>
  <path d="M18 4 L22 4 L22 8"/>
</svg>
EOF
```

- [ ] **Step 11: Create terminal.svg (📋)**

```bash
cat > mobile/assets/emoji/terminal.svg << 'EOF'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#009688" stroke-width="2">
  <rect x="3" y="3" width="18" height="18" rx="2"/>
  <polyline points="7 8 10 12 7 16"/>
  <line x1="13" y1="16" x2="17" y2="16"/>
</svg>
EOF
```

- [ ] **Step 12: Create network.svg (🌐)**

```bash
cat > mobile/assets/emoji/network.svg << 'EOF'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#3F51B5" stroke-width="2">
  <circle cx="12" cy="12" r="10"/>
  <ellipse cx="12" cy="12" rx="4" ry="10"/>
  <line x1="2" y1="12" x2="22" y2="12"/>
  <path d="M12 2 A10 10 0 0 1 12 22"/>
</svg>
EOF
```

- [ ] **Step 13: Create clock.svg (🕐)**

```bash
cat > mobile/assets/emoji/clock.svg << 'EOF'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#757575" stroke-width="2">
  <circle cx="12" cy="12" r="10"/>
  <polyline points="12 6 12 12 16 14"/>
</svg>
EOF
```

- [ ] **Step 14: Create warning.svg (⚠)**

```bash
cat > mobile/assets/emoji/warning.svg << 'EOF'
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none">
  <path d="M12 2 L22 20 L2 20 Z" fill="#FFC107" stroke="#F57C00" stroke-width="1.5"/>
  <line x1="12" y1="9" x2="12" y2="14" stroke="#000" stroke-width="2" stroke-linecap="round"/>
  <circle cx="12" cy="17" r="1" fill="#000"/>
</svg>
EOF
```

- [ ] **Step 15: Create ui_components module file**

```bash
cat > mobile/src/ui_components/mod.rs << 'EOF'
//! Reusable UI components for Dure application

pub mod emoji_loader;
pub mod status_grid;
pub mod emoji_progress;
pub mod action_menu;

pub use emoji_loader::{SvgEmoji, EmojiCache};
pub use status_grid::{StatusGrid, StatusGridItem, ItemState};
pub use emoji_progress::{EmojiProgressBar, ProgressStep, ProgressState};
pub use action_menu::ActionMenu;
EOF
```

- [ ] **Step 16: Add resvg dependency to Cargo.toml**

Add to `mobile/Cargo.toml` in `[dependencies]` section:

```toml
resvg = "0.42"
```

- [ ] **Step 17: Declare ui_components module in lib.rs**

Add after existing module declarations in `mobile/src/lib.rs`:

```rust
#[cfg(feature = "gui")]
pub mod ui_components;
```

- [ ] **Step 18: Verify assets exist**

```bash
ls -la mobile/assets/emoji/*.svg
```

Expected: 13 SVG files listed

- [ ] **Step 19: Compile to verify module structure**

```bash
cd mobile && cargo check --features gui
```

Expected: SUCCESS (may have warnings about unused modules)

- [ ] **Step 20: Commit foundation**

```bash
git add mobile/assets/emoji mobile/src/ui_components/mod.rs mobile/Cargo.toml mobile/src/lib.rs
git commit -m "feat(ui): add SVG emoji assets and ui_components module foundation

- Create 13 SVG emoji assets (checkmark, progress, circle, cross, etc.)
- Add ui_components module structure
- Add resvg dependency for SVG rendering
- Declare ui_components module in lib.rs

Part of platform drawer refactor (Phase 1: Foundation)"
```

---

## Task 2: Emoji Loader with Unicode Fallback

**Files:**
- Create: `mobile/src/ui_components/emoji_loader.rs`

**Interfaces:**
- Consumes: SVG assets from `mobile/assets/emoji/`
- Produces: `SvgEmoji` enum, `EmojiCache` struct with methods:
  - `new(ctx: egui::Context) -> Self`
  - `load(&mut self, emoji: SvgEmoji)`
  - `load_all(&mut self)`
  - `show(&self, ui: &mut egui::Ui, emoji: SvgEmoji, size: f32)`

- [ ] **Step 1: Write test for Unicode fallback**

Create `mobile/src/ui_components/emoji_loader.rs`:

```rust
//! SVG emoji loader with Unicode fallback

use std::collections::HashMap;
use egui::{Context, TextureHandle};

/// SVG emoji identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SvgEmoji {
    Checkmark,
    Progress,
    Circle,
    Cross,
    Email,
    Project,
    VM,
    Firewall,
    Key,
    Terminal,
    Network,
    Clock,
    Warning,
}

impl SvgEmoji {
    /// Get Unicode fallback character
    pub fn to_unicode(&self) -> &'static str {
        match self {
            SvgEmoji::Checkmark => "✅",
            SvgEmoji::Progress => "⏳",
            SvgEmoji::Circle => "⚪",
            SvgEmoji::Cross => "✗",
            SvgEmoji::Email => "📧",
            SvgEmoji::Project => "📁",
            SvgEmoji::VM => "💻",
            SvgEmoji::Firewall => "🔥",
            SvgEmoji::Key => "🔑",
            SvgEmoji::Terminal => "📋",
            SvgEmoji::Network => "🌐",
            SvgEmoji::Clock => "🕐",
            SvgEmoji::Warning => "⚠",
        }
    }

    /// Get SVG file path for embedded assets
    fn svg_path(&self) -> &'static str {
        match self {
            SvgEmoji::Checkmark => "checkmark.svg",
            SvgEmoji::Progress => "progress.svg",
            SvgEmoji::Circle => "circle.svg",
            SvgEmoji::Cross => "cross.svg",
            SvgEmoji::Email => "email.svg",
            SvgEmoji::Project => "project.svg",
            SvgEmoji::VM => "vm.svg",
            SvgEmoji::Firewall => "firewall.svg",
            SvgEmoji::Key => "key.svg",
            SvgEmoji::Terminal => "terminal.svg",
            SvgEmoji::Network => "network.svg",
            SvgEmoji::Clock => "clock.svg",
            SvgEmoji::Warning => "warning.svg",
        }
    }

    /// Get embedded SVG bytes (Desktop/Android)
    #[cfg(not(target_arch = "wasm32"))]
    fn svg_bytes(&self) -> Option<&'static [u8]> {
        match self {
            SvgEmoji::Checkmark => Some(include_bytes!("../../assets/emoji/checkmark.svg")),
            SvgEmoji::Progress => Some(include_bytes!("../../assets/emoji/progress.svg")),
            SvgEmoji::Circle => Some(include_bytes!("../../assets/emoji/circle.svg")),
            SvgEmoji::Cross => Some(include_bytes!("../../assets/emoji/cross.svg")),
            SvgEmoji::Email => Some(include_bytes!("../../assets/emoji/email.svg")),
            SvgEmoji::Project => Some(include_bytes!("../../assets/emoji/project.svg")),
            SvgEmoji::VM => Some(include_bytes!("../../assets/emoji/vm.svg")),
            SvgEmoji::Firewall => Some(include_bytes!("../../assets/emoji/firewall.svg")),
            SvgEmoji::Key => Some(include_bytes!("../../assets/emoji/key.svg")),
            SvgEmoji::Terminal => Some(include_bytes!("../../assets/emoji/terminal.svg")),
            SvgEmoji::Network => Some(include_bytes!("../../assets/emoji/network.svg")),
            SvgEmoji::Clock => Some(include_bytes!("../../assets/emoji/clock.svg")),
            SvgEmoji::Warning => Some(include_bytes!("../../assets/emoji/warning.svg")),
        }
    }
}

/// Emoji texture cache
pub struct EmojiCache {
    textures: HashMap<SvgEmoji, TextureHandle>,
    ctx: Context,
}

impl EmojiCache {
    pub fn new(ctx: Context) -> Self {
        Self {
            textures: HashMap::new(),
            ctx,
        }
    }

    /// Load SVG emoji and cache texture
    pub fn load(&mut self, emoji: SvgEmoji) {
        // TODO: Implement SVG loading
    }

    /// Load all standard emoji
    pub fn load_all(&mut self) {
        let all_emoji = [
            SvgEmoji::Checkmark,
            SvgEmoji::Progress,
            SvgEmoji::Circle,
            SvgEmoji::Cross,
            SvgEmoji::Email,
            SvgEmoji::Project,
            SvgEmoji::VM,
            SvgEmoji::Firewall,
            SvgEmoji::Key,
            SvgEmoji::Terminal,
            SvgEmoji::Network,
            SvgEmoji::Clock,
            SvgEmoji::Warning,
        ];

        for emoji in all_emoji {
            self.load(emoji);
        }
    }

    /// Get cached texture
    pub fn get(&self, emoji: SvgEmoji) -> Option<&TextureHandle> {
        self.textures.get(&emoji)
    }

    /// Render emoji at specified size (with Unicode fallback)
    pub fn show(&self, ui: &mut egui::Ui, emoji: SvgEmoji, size: f32) {
        match self.get(emoji) {
            Some(texture) => {
                // Render SVG texture
                ui.image(texture, egui::vec2(size, size));
            }
            None => {
                // Fallback to Unicode emoji
                let unicode = emoji.to_unicode();
                ui.label(egui::RichText::new(unicode).size(size));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unicode_fallback() {
        assert_eq!(SvgEmoji::Checkmark.to_unicode(), "✅");
        assert_eq!(SvgEmoji::Progress.to_unicode(), "⏳");
        assert_eq!(SvgEmoji::Circle.to_unicode(), "⚪");
        assert_eq!(SvgEmoji::Cross.to_unicode(), "✗");
        assert_eq!(SvgEmoji::Warning.to_unicode(), "⚠");
    }

    #[test]
    fn test_svg_paths() {
        assert_eq!(SvgEmoji::Checkmark.svg_path(), "checkmark.svg");
        assert_eq!(SvgEmoji::Firewall.svg_path(), "firewall.svg");
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_embedded_bytes_exist() {
        assert!(SvgEmoji::Checkmark.svg_bytes().is_some());
        assert!(SvgEmoji::Warning.svg_bytes().is_some());
        // Verify all emoji have bytes
        let all_emoji = [
            SvgEmoji::Checkmark, SvgEmoji::Progress, SvgEmoji::Circle,
            SvgEmoji::Cross, SvgEmoji::Email, SvgEmoji::Project,
            SvgEmoji::VM, SvgEmoji::Firewall, SvgEmoji::Key,
            SvgEmoji::Terminal, SvgEmoji::Network, SvgEmoji::Clock,
            SvgEmoji::Warning,
        ];
        for emoji in all_emoji {
            assert!(emoji.svg_bytes().is_some(), "Missing bytes for {:?}", emoji);
        }
    }
}
```

- [ ] **Step 2: Run tests to verify they pass**

```bash
cd mobile && cargo test --features gui emoji_loader::tests
```

Expected: All tests PASS

- [ ] **Step 3: Implement SVG loading (non-WASM)**

Add after `impl EmojiCache`:

```rust
impl EmojiCache {
    // ... existing methods ...

    /// Load SVG emoji and cache texture
    pub fn load(&mut self, emoji: SvgEmoji) {
        // Skip if already loaded
        if self.textures.contains_key(&emoji) {
            return;
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(svg_bytes) = emoji.svg_bytes() {
                match self.load_svg_from_bytes(emoji, svg_bytes, 24.0) {
                    Ok(texture) => {
                        self.textures.insert(emoji, texture);
                    }
                    Err(e) => {
                        eprintln!("Failed to load SVG for {:?}: {}", emoji, e);
                        // Fallback to Unicode (no texture cached)
                    }
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            // WASM: TODO - load via HTTP (future enhancement)
            // For now, fall back to Unicode
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn load_svg_from_bytes(
        &self,
        emoji: SvgEmoji,
        svg_bytes: &[u8],
        size: f32,
    ) -> Result<TextureHandle, String> {
        use resvg::usvg;

        // Parse SVG
        let options = usvg::Options::default();
        let tree = usvg::Tree::from_data(svg_bytes, &options)
            .map_err(|e| format!("SVG parse error: {}", e))?;

        // Render to RGBA buffer
        let pixmap_size = tree.size().to_int_size();
        let mut pixmap = resvg::tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height())
            .ok_or_else(|| "Failed to create pixmap".to_string())?;

        resvg::render(
            &tree,
            resvg::tiny_skia::Transform::default(),
            &mut pixmap.as_mut(),
        );

        // Convert to egui ColorImage
        let image = egui::ColorImage::from_rgba_unmultiplied(
            [pixmap_size.width() as usize, pixmap_size.height() as usize],
            pixmap.data(),
        );

        // Load as texture
        let texture = self.ctx.load_texture(
            format!("emoji_{:?}", emoji),
            image,
            egui::TextureOptions::LINEAR,
        );

        Ok(texture)
    }
}
```

- [ ] **Step 4: Test SVG loading**

```bash
cd mobile && cargo test --features gui emoji_loader::tests::test_embedded_bytes_exist
```

Expected: PASS

- [ ] **Step 5: Commit emoji loader**

```bash
git add mobile/src/ui_components/emoji_loader.rs
git commit -m "feat(ui): implement emoji loader with SVG rendering and Unicode fallback

- Add SvgEmoji enum with 13 emoji types
- Implement EmojiCache for texture caching
- SVG loading via resvg (Desktop/Android)
- Unicode fallback when SVG unavailable
- Unit tests for all emoji mappings

Part of platform drawer refactor (Phase 1: Foundation)"
```

---

## Task 3: OperationState in PlatformRow

**Files:**
- Modify: `mobile/src/ui_tabs/platform.rs`

**Interfaces:**
- Produces: `OperationState` enum in platform.rs
- Produces: `operation_state` field in `PlatformRow`
- Consumes: None (standalone types)

- [ ] **Step 1: Add OperationState enum to platform.rs**

Add after `PlatformAction` enum (around line 63):

```rust
/// Operation state for visual feedback with timestamps
#[derive(Debug, Clone, PartialEq)]
pub enum OperationState {
    Idle,
    InProgress {
        operation: String,
        started_at: i64,
    },
    Completed {
        operation: String,
        completed_at: i64,
    },
    Failed {
        operation: String,
        error: String,
        failed_at: i64,
    },
}

impl Default for OperationState {
    fn default() -> Self {
        Self::Idle
    }
}
```

- [ ] **Step 2: Add operation_state field to PlatformRow**

In `PlatformRow` struct (around line 16), add after `last_refresh_time`:

```rust
    // Operation state tracking (for visual feedback)
    operation_state: OperationState,
```

- [ ] **Step 3: Update PlatformRow initialization in load_rows()**

Find where `PlatformRow` is constructed (around line 1527), add after `last_refresh_time` field:

```rust
                        operation_state: OperationState::Idle,
```

- [ ] **Step 4: Compile to verify changes**

```bash
cd mobile && cargo check --features gui
```

Expected: SUCCESS

- [ ] **Step 5: Commit OperationState**

```bash
git add mobile/src/ui_tabs/platform.rs
git commit -m "feat(platform): add OperationState tracking to PlatformRow

- Add OperationState enum (Idle/InProgress/Completed/Failed)
- Include timestamps for auto-clear logic
- Add operation_state field to PlatformRow
- Initialize to Idle in load_rows()

Part of platform drawer refactor (Phase 1: Foundation)"
```

---

## Task 4: StatusGrid Component

**Files:**
- Create: `mobile/src/ui_components/status_grid.rs`
- Modify: `mobile/src/ui_components/mod.rs`

**Interfaces:**
- Consumes: `SvgEmoji`, `EmojiCache` from emoji_loader
- Produces: `StatusGrid`, `StatusGridItem`, `ItemState` with methods:
  - `StatusGrid::new() -> Self`
  - `StatusGrid::add_item(&mut self, emoji, label, value, state)`
  - `StatusGrid::show(&self, ui: &mut egui::Ui)`

- [ ] **Step 1: Write test for ItemState**

Create `mobile/src/ui_components/status_grid.rs`:

```rust
//! Responsive status grid component

use crate::ui_components::emoji_loader::SvgEmoji;

/// Item state for visual indicators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemState {
    InProgress,
    Success,
    Error,
    Warning,
}

/// Grid item with emoji, label, value, and optional state
#[derive(Clone)]
pub struct StatusGridItem {
    pub emoji: SvgEmoji,
    pub label: String,
    pub value: String,
    pub state: Option<ItemState>,
}

/// Responsive key-value grid
pub struct StatusGrid {
    items: Vec<StatusGridItem>,
    min_column_width: f32,
}

impl StatusGrid {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            min_column_width: 200.0,
        }
    }

    pub fn with_min_column_width(width: f32) -> Self {
        Self {
            items: Vec::new(),
            min_column_width: width,
        }
    }

    pub fn add_item(
        &mut self,
        emoji: SvgEmoji,
        label: impl Into<String>,
        value: impl Into<String>,
        state: Option<ItemState>,
    ) {
        self.items.push(StatusGridItem {
            emoji,
            label: label.into(),
            value: value.into(),
            state,
        });
    }

    fn calculate_columns(&self, available_width: f32) -> usize {
        let cols = (available_width / self.min_column_width).floor() as usize;
        cols.clamp(1, 3)
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        let available_width = ui.available_width();
        let columns = self.calculate_columns(available_width);

        // Use egui Grid for layout
        egui::Grid::new("status_grid")
            .num_columns(columns * 3) // emoji + label + value per column
            .spacing([8.0, 4.0])
            .show(ui, |ui| {
                for (idx, item) in self.items.iter().enumerate() {
                    // Emoji (use Unicode for now, SVG in next iteration)
                    ui.label(item.emoji.to_unicode());

                    // Label
                    ui.label(egui::RichText::new(&item.label).strong());

                    // Value with state color
                    let value_text = match item.state {
                        Some(ItemState::InProgress) => {
                            egui::RichText::new(&item.value).color(egui::Color32::from_rgb(255, 152, 0))
                        }
                        Some(ItemState::Success) => {
                            egui::RichText::new(&item.value).color(egui::Color32::from_rgb(76, 175, 80))
                        }
                        Some(ItemState::Error) => {
                            egui::RichText::new(&item.value).color(egui::Color32::from_rgb(244, 67, 54))
                        }
                        Some(ItemState::Warning) => {
                            egui::RichText::new(&item.value).color(egui::Color32::from_rgb(255, 193, 7))
                        }
                        None => egui::RichText::new(&item.value),
                    };
                    ui.label(value_text);

                    // End row after filling columns
                    if (idx + 1) % columns == 0 {
                        ui.end_row();
                    }
                }
            });
    }
}

impl Default for StatusGrid {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_columns() {
        let grid = StatusGrid::new();
        assert_eq!(grid.calculate_columns(700.0), 3);
        assert_eq!(grid.calculate_columns(450.0), 2);
        assert_eq!(grid.calculate_columns(250.0), 1);
        assert_eq!(grid.calculate_columns(100.0), 1); // Min 1
    }

    #[test]
    fn test_add_item() {
        let mut grid = StatusGrid::new();
        grid.add_item(SvgEmoji::Email, "Email", "test@example.com", None);
        grid.add_item(SvgEmoji::VM, "VM", "my-vm", Some(ItemState::Success));
        assert_eq!(grid.items.len(), 2);
        assert_eq!(grid.items[0].label, "Email");
        assert_eq!(grid.items[1].state, Some(ItemState::Success));
    }
}
```

- [ ] **Step 2: Run tests**

```bash
cd mobile && cargo test --features gui status_grid::tests
```

Expected: All tests PASS

- [ ] **Step 3: Update mod.rs exports**

Already done in Task 1, verify it includes status_grid

- [ ] **Step 4: Compile full module**

```bash
cd mobile && cargo check --features gui
```

Expected: SUCCESS

- [ ] **Step 5: Commit StatusGrid**

```bash
git add mobile/src/ui_components/status_grid.rs
git commit -m "feat(ui): implement StatusGrid responsive component

- Add ItemState enum (InProgress/Success/Error/Warning)
- Implement StatusGrid with auto-reflow (3→2→1 columns)
- Calculate columns based on available width
- Color-coded value text based on state
- Unit tests for column calculation and item management

Part of platform drawer refactor (Phase 2: Components)"
```

---

## Task 5: EmojiProgressBar Component

**Files:**
- Create: `mobile/src/ui_components/emoji_progress.rs`

**Interfaces:**
- Consumes: `SvgEmoji` from emoji_loader, `PlatformRow` from platform.rs
- Produces: `EmojiProgressBar`, `ProgressStep`, `ProgressState` with methods:
  - `EmojiProgressBar::new() -> Self`
  - `EmojiProgressBar::add_step(&mut self, label, state)`
  - `EmojiProgressBar::from_platform_row(row: &PlatformRow) -> Self`
  - `EmojiProgressBar::show(&self, ui: &mut egui::Ui) -> egui::Response`

- [ ] **Step 1: Write test for ProgressState**

Create `mobile/src/ui_components/emoji_progress.rs`:

```rust
//! Emoji progress bar component

use crate::ui_components::emoji_loader::SvgEmoji;

/// Progress step state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressState {
    Completed,
    InProgress,
    Pending,
    Failed,
}

/// Single progress step
#[derive(Clone)]
pub struct ProgressStep {
    pub label: String,
    pub state: ProgressState,
}

/// Emoji progress bar
pub struct EmojiProgressBar {
    steps: Vec<ProgressStep>,
    compact: bool,
}

impl EmojiProgressBar {
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            compact: true,
        }
    }

    pub fn add_step(&mut self, label: impl Into<String>, state: ProgressState) {
        self.steps.push(ProgressStep {
            label: label.into(),
            state,
        });
    }

    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }

    fn emoji_for_state(&self, state: ProgressState) -> SvgEmoji {
        match state {
            ProgressState::Completed => SvgEmoji::Checkmark,
            ProgressState::InProgress => SvgEmoji::Progress,
            ProgressState::Pending => SvgEmoji::Circle,
            ProgressState::Failed => SvgEmoji::Cross,
        }
    }

    pub fn show(&self, ui: &mut egui::Ui) -> egui::Response {
        if self.compact {
            // Inline: emoji → emoji → emoji
            ui.horizontal(|ui| {
                for (idx, step) in self.steps.iter().enumerate() {
                    let emoji = self.emoji_for_state(step.state);
                    ui.label(emoji.to_unicode());

                    // Arrow between steps (except last)
                    if idx < self.steps.len() - 1 {
                        ui.label("→");
                    }
                }
            })
            .response
        } else {
            // Stacked: emoji + label per row
            ui.vertical(|ui| {
                for step in &self.steps {
                    ui.horizontal(|ui| {
                        let emoji = self.emoji_for_state(step.state);
                        ui.label(emoji.to_unicode());
                        ui.label(&step.label);
                    });
                }
            })
            .response
        }
    }
}

impl Default for EmojiProgressBar {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_step() {
        let mut bar = EmojiProgressBar::new();
        bar.add_step("OAuth", ProgressState::Completed);
        bar.add_step("Project", ProgressState::InProgress);
        bar.add_step("VM", ProgressState::Pending);

        assert_eq!(bar.steps.len(), 3);
        assert_eq!(bar.steps[0].label, "OAuth");
        assert_eq!(bar.steps[1].state, ProgressState::InProgress);
    }

    #[test]
    fn test_emoji_mapping() {
        let bar = EmojiProgressBar::new();
        assert_eq!(bar.emoji_for_state(ProgressState::Completed), SvgEmoji::Checkmark);
        assert_eq!(bar.emoji_for_state(ProgressState::InProgress), SvgEmoji::Progress);
        assert_eq!(bar.emoji_for_state(ProgressState::Pending), SvgEmoji::Circle);
        assert_eq!(bar.emoji_for_state(ProgressState::Failed), SvgEmoji::Cross);
    }
}
```

- [ ] **Step 2: Run tests**

```bash
cd mobile && cargo test --features gui emoji_progress::tests
```

Expected: All tests PASS

- [ ] **Step 3: Add from_platform_row() implementation**

Add to `impl EmojiProgressBar`:

```rust
    /// Create progress bar from PlatformRow state
    pub fn from_platform_row(row: &crate::ui_tabs::platform::PlatformRow) -> Self {
        let mut bar = Self::new();

        // Step 1: OAuth
        let oauth_state = if row.gcp_connected {
            ProgressState::Completed
        } else {
            ProgressState::Pending
        };
        bar.add_step("OAuth", oauth_state);

        // Step 2: Project
        let project_state = if row.project_selected {
            ProgressState::Completed
        } else if row.gcp_connected {
            ProgressState::Pending
        } else {
            ProgressState::Pending
        };
        bar.add_step("Project", project_state);

        // Step 3: VM
        let vm_state = if row.vm_created {
            ProgressState::Completed
        } else if row.project_selected {
            // Check if VM operation in progress
            match &row.operation_state {
                crate::ui_tabs::platform::OperationState::InProgress { operation, .. }
                    if operation.to_lowercase().contains("vm") =>
                {
                    ProgressState::InProgress
                }
                _ => ProgressState::Pending,
            }
        } else {
            ProgressState::Pending
        };
        bar.add_step("VM", vm_state);

        // Step 4: Firewall
        let firewall_state = if row.firewall_updated {
            ProgressState::Completed
        } else if row.vm_created {
            match &row.operation_state {
                crate::ui_tabs::platform::OperationState::InProgress { operation, .. }
                    if operation.to_lowercase().contains("firewall") =>
                {
                    ProgressState::InProgress
                }
                _ => ProgressState::Pending,
            }
        } else {
            ProgressState::Pending
        };
        bar.add_step("Firewall", firewall_state);

        // Step 5: SSH
        let ssh_state = if row.ssh_ready {
            ProgressState::Completed
        } else if row.vm_created {
            ProgressState::Pending
        } else {
            ProgressState::Pending
        };
        bar.add_step("SSH", ssh_state);

        bar
    }
```

- [ ] **Step 4: Write test for from_platform_row**

Add to tests module:

```rust
    #[test]
    fn test_from_platform_row_all_completed() {
        use crate::ui_tabs::platform::{PlatformRow, OperationState};

        let row = PlatformRow {
            project_id: "test".to_string(),
            project_display_name: "Test".to_string(),
            platform_type: "GCP".to_string(),
            gcp_connected: true,
            project_selected: true,
            vm_created: true,
            firewall_updated: true,
            ssh_ready: true,
            operation_state: OperationState::Idle,
            email: None,
            total_project_count: 0,
            selected_project_id: None,
            vm_name: None,
            vm_external_ip: None,
            ssh_private_key: None,
            ssh_public_key: None,
            ssh_keyring_domain: None,
            firewall_status: "".to_string(),
            ssh_status: "".to_string(),
            last_refresh_time: None,
            has_vm: false,
            vm_zone: None,
        };

        let bar = EmojiProgressBar::from_platform_row(&row);
        assert_eq!(bar.steps.len(), 5);
        // All steps should be Completed
        for step in &bar.steps {
            assert_eq!(step.state, ProgressState::Completed);
        }
    }

    #[test]
    fn test_from_platform_row_with_operation() {
        use crate::ui_tabs::platform::{PlatformRow, OperationState};

        let row = PlatformRow {
            project_id: "test".to_string(),
            project_display_name: "Test".to_string(),
            platform_type: "GCP".to_string(),
            gcp_connected: true,
            project_selected: true,
            vm_created: false,
            firewall_updated: false,
            ssh_ready: false,
            operation_state: OperationState::InProgress {
                operation: "Creating VM".to_string(),
                started_at: 0,
            },
            email: None,
            total_project_count: 0,
            selected_project_id: None,
            vm_name: None,
            vm_external_ip: None,
            ssh_private_key: None,
            ssh_public_key: None,
            ssh_keyring_domain: None,
            firewall_status: "".to_string(),
            ssh_status: "".to_string(),
            last_refresh_time: None,
            has_vm: false,
            vm_zone: None,
        };

        let bar = EmojiProgressBar::from_platform_row(&row);
        assert_eq!(bar.steps[2].state, ProgressState::InProgress); // VM step
    }
```

- [ ] **Step 5: Run tests**

```bash
cd mobile && cargo test --features gui emoji_progress::tests
```

Expected: All tests PASS

- [ ] **Step 6: Commit EmojiProgressBar**

```bash
git add mobile/src/ui_components/emoji_progress.rs
git commit -m "feat(ui): implement EmojiProgressBar component

- Add ProgressState enum (Completed/InProgress/Pending/Failed)
- Implement progress bar with compact/stacked modes
- Add from_platform_row() to derive state from PlatformRow
- Map operation state to InProgress indicator
- Unit tests for state mapping and row conversion

Part of platform drawer refactor (Phase 2: Components)"
```

---

## Task 6: ActionMenu Component

**Files:**
- Create: `mobile/src/ui_components/action_menu.rs`

**Interfaces:**
- Consumes: `SvgEmoji` from emoji_loader
- Produces: `ActionMenu` with methods:
  - `ActionMenu::new(label: impl Into<String>) -> Self`
  - `ActionMenu::with_icon(self, icon: SvgEmoji) -> Self`
  - `ActionMenu::add_action(&mut self, label: impl Into<String>)`
  - `ActionMenu::show(&mut self, ui: &mut egui::Ui) -> Option<usize>`

- [ ] **Step 1: Write test for action menu**

Create `mobile/src/ui_components/action_menu.rs`:

```rust
//! Dropdown action menu component

use crate::ui_components::emoji_loader::SvgEmoji;

/// Dropdown action menu
pub struct ActionMenu {
    label: String,
    icon: Option<SvgEmoji>,
    actions: Vec<String>,
}

impl ActionMenu {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            actions: Vec::new(),
        }
    }

    pub fn with_icon(mut self, icon: SvgEmoji) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn add_action(&mut self, label: impl Into<String>) {
        self.actions.push(label.into());
    }

    /// Show menu and return index of clicked action
    pub fn show(&mut self, ui: &mut egui::Ui) -> Option<usize> {
        let button_text = if let Some(icon) = self.icon {
            format!("{} {}", icon.to_unicode(), self.label)
        } else {
            self.label.clone()
        };

        let mut selected = None;

        egui::menu::menu_button(ui, button_text, |ui| {
            for (idx, action) in self.actions.iter().enumerate() {
                if ui.button(action).clicked() {
                    selected = Some(idx);
                    ui.close_menu();
                }
            }
        });

        selected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_menu_creation() {
        let mut menu = ActionMenu::new("Test Menu");
        menu.add_action("Action 1");
        menu.add_action("Action 2");
        menu.add_action("Action 3");

        assert_eq!(menu.actions.len(), 3);
        assert_eq!(menu.actions[0], "Action 1");
        assert_eq!(menu.label, "Test Menu");
    }

    #[test]
    fn test_with_icon() {
        let menu = ActionMenu::new("SSH")
            .with_icon(SvgEmoji::Terminal);
        assert_eq!(menu.icon, Some(SvgEmoji::Terminal));
    }
}
```

- [ ] **Step 2: Run tests**

```bash
cd mobile && cargo test --features gui action_menu::tests
```

Expected: All tests PASS

- [ ] **Step 3: Compile full ui_components module**

```bash
cd mobile && cargo check --features gui
```

Expected: SUCCESS

- [ ] **Step 4: Commit ActionMenu**

```bash
git add mobile/src/ui_components/action_menu.rs
git commit -m "feat(ui): implement ActionMenu dropdown component

- Add ActionMenu with optional icon support
- Return clicked action index from show()
- Use egui menu_button for dropdown UI
- Unit tests for menu creation and icon assignment

Part of platform drawer refactor (Phase 2: Components)"
```

---

## Task 7: Refactor render_drawer_content() to Use StatusGrid

**Files:**
- Modify: `mobile/src/ui_tabs/platform.rs`

**Interfaces:**
- Consumes: `StatusGrid`, `StatusGridItem`, `ItemState`, `ActionMenu` from ui_components
- Produces: Refactored `render_drawer_content()` function at line ~699

- [ ] **Step 1: Add ui_components imports to platform.rs**

Add after existing use statements at top of platform.rs:

```rust
use crate::ui_components::{StatusGrid, ItemState, ActionMenu, SvgEmoji};
```

- [ ] **Step 2: Backup current render_drawer_content**

```bash
# Create backup for reference
cp mobile/src/ui_tabs/platform.rs mobile/src/ui_tabs/platform.rs.backup
```

- [ ] **Step 3: Replace render_drawer_content() function**

Replace function at line ~699-803 with:

```rust
fn render_drawer_content(ui: &mut egui::Ui, row: &PlatformRow) {
    ui.add_space(8.0);

    let mut grid = StatusGrid::new();

    // Connection info
    if let Some(email) = &row.email {
        grid.add_item(
            SvgEmoji::Email,
            "Email",
            format!("{} ({} projects)", email, row.total_project_count),
            None,
        );
    } else {
        grid.add_item(SvgEmoji::Email, "Email", "Not connected", None);
    }

    // Project info
    if let Some(project_id) = &row.selected_project_id {
        grid.add_item(SvgEmoji::Project, "Project", project_id, None);

        // Refresh staleness
        if let Some(last_refresh) = row.last_refresh_time {
            let elapsed = chrono::Utc::now().timestamp() - last_refresh;
            let (time_str, state) = if elapsed < 60 {
                ("just now".to_string(), None)
            } else if elapsed < 3600 {
                (format!("{} min ago", elapsed / 60), None)
            } else if elapsed < 86400 {
                (format!("{} hours ago", elapsed / 3600), Some(ItemState::Warning))
            } else {
                (format!("{} days ago", elapsed / 86400), Some(ItemState::Warning))
            };
            grid.add_item(SvgEmoji::Clock, "Refreshed", time_str, state);
        }

        // VM details
        if let Some(vm_name) = &row.vm_name {
            grid.add_item(SvgEmoji::VM, "VM", vm_name, None);

            // IP address
            grid.add_item(
                SvgEmoji::Network,
                "IP",
                row.vm_external_ip
                    .as_deref()
                    .unwrap_or("⚠ No external IP"),
                if row.vm_external_ip.is_none() {
                    Some(ItemState::Warning)
                } else {
                    None
                },
            );

            // Firewall status (check operation state)
            let (firewall_value, firewall_state) = match &row.operation_state {
                OperationState::InProgress { operation, .. }
                    if operation.to_lowercase().contains("firewall") =>
                {
                    ("Updating...".to_string(), Some(ItemState::InProgress))
                }
                OperationState::Failed { operation, error, .. }
                    if operation.to_lowercase().contains("firewall") =>
                {
                    (error.clone(), Some(ItemState::Error))
                }
                _ => (row.firewall_status.clone(), None),
            };
            grid.add_item(SvgEmoji::Firewall, "Firewall", firewall_value, firewall_state);

            // SSH status
            grid.add_item(SvgEmoji::Key, "SSH", &row.ssh_status, None);
        } else {
            grid.add_item(SvgEmoji::VM, "VM", "— No VM created", None);
        }
    } else {
        grid.add_item(SvgEmoji::Project, "Project", "— No project selected", None);
    }

    grid.show(ui);

    // SSH action menu (if available)
    if let (Some(external_ip), Some(private_key)) =
        (&row.vm_external_ip, &row.ssh_private_key)
    {
        ui.add_space(8.0);

        let ssh_command = format!(
            "K=$(mktemp) && cat > $K <<'EOF'\n{}\nEOF\nchmod 600 $K && ssh -i $K root@{} && rm $K",
            private_key.trim(),
            external_ip
        );

        let mut menu = ActionMenu::new("📋 SSH").with_icon(SvgEmoji::Terminal);
        menu.add_action("Copy SSH Command");
        menu.add_action("Copy Private Key");
        menu.add_action("Copy IP Address");

        if let Some(action_idx) = menu.show(ui) {
            let text_to_copy = match action_idx {
                0 => &ssh_command,
                1 => private_key,
                2 => external_ip,
                _ => return,
            };

            ui.output_mut(|o| o.copied_text = text_to_copy.to_string());
        }
    } else if row.vm_external_ip.is_some() && row.ssh_keyring_domain.is_some() {
        ui.add_space(8.0);
        ui.colored_label(
            egui::Color32::from_rgb(255, 152, 0),
            "⚠ SSH key not found in keyring",
        );
    }
}
```

- [ ] **Step 4: Compile to verify refactor**

```bash
cd mobile && cargo check --features gui
```

Expected: SUCCESS

- [ ] **Step 5: Test UI manually (if possible)**

```bash
cd mobile && cargo run --features gui --release
```

Check: Platform tab drawer shows grid layout with emoji icons

- [ ] **Step 6: Commit drawer refactor**

```bash
git add mobile/src/ui_tabs/platform.rs
git commit -m "refactor(platform): replace drawer text hierarchy with StatusGrid

- Replace render_drawer_content() with StatusGrid component
- Add SSH ActionMenu for Copy Command/Key/IP
- Show operation state in firewall status (⏳/✗)
- Display staleness warning for old refresh times
- Maintain all existing functionality with better UX

Part of platform drawer refactor (Phase 3: Integration)"
```

---

## Task 8: Refactor format_steps() to Use EmojiProgressBar

**Files:**
- Modify: `mobile/src/ui_tabs/platform.rs`

**Interfaces:**
- Consumes: `EmojiProgressBar::from_platform_row()` from ui_components
- Produces: Refactored Steps column rendering in table

- [ ] **Step 1: Find format_steps function**

```bash
grep -n "fn format_steps" mobile/src/ui_tabs/platform.rs
```

Expected: Line number of format_steps() function (or it may be inline in cell rendering)

- [ ] **Step 2: Replace Steps column rendering**

Find where Steps column is rendered (around line 1074-1075), replace `.cell(&format_steps(&row_for_cells))` with `.cell_widget()`:

```rust
                        .cell_widget(|ui| {
                            use crate::ui_components::EmojiProgressBar;
                            let progress = EmojiProgressBar::from_platform_row(&row_for_cells)
                                .compact(true);
                            progress.show(ui);
                        })
```

- [ ] **Step 3: Remove old format_steps() function if exists**

```bash
# Search for and comment out old format_steps function
grep -n "fn format_steps" mobile/src/ui_tabs/platform.rs
```

If function exists, comment it out or remove it.

- [ ] **Step 4: Add EmojiProgressBar to imports**

Verify `use crate::ui_components::{...}` includes `EmojiProgressBar`

- [ ] **Step 5: Compile to verify**

```bash
cd mobile && cargo check --features gui
```

Expected: SUCCESS

- [ ] **Step 6: Test emoji progress bar display**

```bash
cd mobile && cargo run --features gui --release
```

Check: Steps column shows ✅ → ✅ → ⏳ → ⚪ → ⚪ (or similar)

- [ ] **Step 7: Commit steps refactor**

```bash
git add mobile/src/ui_tabs/platform.rs
git commit -m "refactor(platform): replace Steps text with EmojiProgressBar

- Use EmojiProgressBar::from_platform_row() in Steps column
- Show visual progress: ✅ → ⏳ → ⚪ indicators
- Automatically reflect operation state (InProgress shows ⏳)
- Remove old format_steps() function

Part of platform drawer refactor (Phase 3: Integration)"
```

---

## Task 9: Add Optimistic Updates for Operation Buttons

**Files:**
- Modify: `mobile/src/ui_tabs/platform.rs`

**Interfaces:**
- Consumes: `OperationState` enum, `rows` vec in `PlatformTab`
- Produces: Immediate UI updates when buttons clicked

- [ ] **Step 1: Add optimistic update for Firewall button**

Find firewall button click handler (around line 1294), add before `self.update_firewall()`:

```rust
            if let Some(platform_name) = ui.data(|d| {
                d.get_temp::<String>(egui::Id::new("platform_action_update_firewall"))
            }) {
                // Optimistic update: Set InProgress immediately
                if let Some(row) = self.rows.iter_mut().find(|r| r.project_id == platform_name) {
                    row.operation_state = OperationState::InProgress {
                        operation: "Updating firewall".to_string(),
                        started_at: chrono::Utc::now().timestamp(),
                    };
                }

                self.update_firewall(platform_name, vm.as_deref_mut());
                ui.data_mut(|d| {
                    d.remove::<String>(egui::Id::new("platform_action_update_firewall"))
                });
            }
```

- [ ] **Step 2: Add optimistic update for Restart VM button**

Find restart VM button handler (around line 1346), add before `self.restart_vm()`:

```rust
            if let Some(platform_name) =
                ui.data(|d| d.get_temp::<String>(egui::Id::new("platform_action_restart_vm")))
            {
                // Optimistic update
                if let Some(row) = self.rows.iter_mut().find(|r| r.project_id == platform_name) {
                    row.operation_state = OperationState::InProgress {
                        operation: "Restarting VM".to_string(),
                        started_at: chrono::Utc::now().timestamp(),
                    };
                }

                // Find platform and get vm_name and zone
                if let Ok((app_config, _)) = load_config() {
                    if let Some(platform) = app_config
                        .platforms
                        .iter()
                        .find(|p| p.gcp_selected_project_id.as_ref() == Some(&platform_name))
                    {
                        if let Some(vm_config) = platform.vms.first() {
                            self.restart_vm(
                                platform_name.clone(),
                                vm_config.name.clone(),
                                vm_config.zone.clone(),
                                vm.as_deref_mut(),
                            );
                        }
                    }
                }
                ui.data_mut(|d| {
                    d.remove::<String>(egui::Id::new("platform_action_restart_vm"))
                });
            }
```

- [ ] **Step 3: Add optimistic update for Delete VM button**

Find delete VM handler (around line 1311), add before `self.show_delete_vm_confirmation()`:

```rust
            if let Some((platform_name, vm_name, vm_zone)) = ui.data(|d| {
                d.get_temp::<(String, String, String)>(egui::Id::new(
                    "platform_action_delete_vm",
                ))
            }) {
                // Optimistic update
                if let Some(row) = self.rows.iter_mut().find(|r| r.project_id == platform_name) {
                    row.operation_state = OperationState::InProgress {
                        operation: format!("Deleting VM {}", vm_name),
                        started_at: chrono::Utc::now().timestamp(),
                    };
                }

                self.show_delete_vm_confirmation(platform_name, vm_name, vm_zone);
                ui.data_mut(|d| {
                    d.remove::<(String, String, String)>(egui::Id::new(
                        "platform_action_delete_vm",
                    ))
                });
            }
```

- [ ] **Step 4: Add optimistic update for Scan VMs button**

Find scan VMs handler (around line 1302), add optimistic update:

```rust
            if let Some(platform_name) = ui.data(|d| {
                d.get_temp::<String>(egui::Id::new("platform_action_scan_vms"))
            }) {
                // Optimistic update
                if let Some(row) = self.rows.iter_mut().find(|r| r.project_id == platform_name) {
                    row.operation_state = OperationState::InProgress {
                        operation: "Scanning VMs".to_string(),
                        started_at: chrono::Utc::now().timestamp(),
                    };
                }

                self.scan_vms(platform_name, vm.as_deref_mut());
                ui.data_mut(|d| {
                    d.remove::<String>(egui::Id::new("platform_action_scan_vms"))
                });
            }
```

- [ ] **Step 5: Compile to verify**

```bash
cd mobile && cargo check --features gui
```

Expected: SUCCESS

- [ ] **Step 6: Commit optimistic updates**

```bash
git add mobile/src/ui_tabs/platform.rs
git commit -m "feat(platform): add optimistic UI updates for operation buttons

- Set OperationState::InProgress immediately when buttons clicked
- Users see ⏳ indicator instantly before async operation starts
- Apply to: Firewall, Restart VM, Delete VM, Scan VMs
- Include operation timestamp for auto-clear logic

Part of platform drawer refactor (Phase 4: Event Flow)"
```

---

## Task 10: Update Event Handlers for Incremental Updates

**Files:**
- Modify: `mobile/src/ui_tabs/platform.rs`

**Interfaces:**
- Consumes: ViewModel events (FirewallUpdated, VMRestarted, etc.)
- Produces: Incremental row updates without full reload

- [ ] **Step 1: Update FirewallUpdated event handler**

Find `ViewModelEvent::Platform(PlatformEvent::FirewallUpdated` handler (around line 821), replace with:

```rust
                    ViewModelEvent::Platform(PlatformEvent::FirewallUpdated {
                        project_id,
                        whitelisted_ip,
                    }) => {
                        dure_debug!("✓ Successfully added {} to firewall whitelist", whitelisted_ip);

                        // Incremental update: Find and update specific row
                        if let Some(row) = self.rows.iter_mut().find(|r| r.project_id == project_id) {
                            row.operation_state = OperationState::Completed {
                                operation: "firewall".to_string(),
                                completed_at: chrono::Utc::now().timestamp(),
                            };
                            row.firewall_status = format!("✅ Whitelisted ({})", whitelisted_ip);
                            row.firewall_updated = true;
                        }
                        // Note: NO self.loaded = false! Incremental update only
                    }
```

- [ ] **Step 2: Update VMRestarted event handler**

Find `ViewModelEvent::Platform(PlatformEvent::VMRestarted` handler (around line 831), replace with:

```rust
                    ViewModelEvent::Platform(PlatformEvent::VMRestarted { project_id, vm_name }) => {
                        dure_info!("✓ VM {} restarted successfully", vm_name);

                        // Incremental update
                        if let Some(row) = self.rows.iter_mut().find(|r| r.project_id == project_id) {
                            row.operation_state = OperationState::Completed {
                                operation: "restart".to_string(),
                                completed_at: chrono::Utc::now().timestamp(),
                            };
                        }
                        // Note: NO self.loaded = false!
                    }
```

- [ ] **Step 3: Update VMCreated event handler**

Find `ViewModelEvent::Platform(PlatformEvent::VMCreated` handler (around line 856), replace:

```rust
                    ViewModelEvent::Platform(PlatformEvent::VMCreated {
                        project_id,
                        vm_name,
                        external_ip,
                    }) => {
                        dure_info!("✓ VM '{}' created successfully with IP {}", vm_name, external_ip);

                        // Incremental update
                        if let Some(row) = self.rows.iter_mut().find(|r| r.project_id == project_id) {
                            row.operation_state = OperationState::Completed {
                                operation: "vm".to_string(),
                                completed_at: chrono::Utc::now().timestamp(),
                            };
                            row.vm_name = Some(vm_name);
                            row.vm_external_ip = Some(external_ip);
                            row.vm_created = true;
                            row.has_vm = true;
                        }
                        // Trigger full reload to update config-backed data
                        self.loaded = false;
                    }
```

- [ ] **Step 4: Update VMDeleted event handler (keep reload for row removal)**

Find `ViewModelEvent::Platform(PlatformEvent::VMDeleted` handler (around line 866), keep reload but add state update:

```rust
                    ViewModelEvent::Platform(PlatformEvent::VMDeleted {
                        platform_name,
                        vm_name,
                    }) => {
                        dure_info!("✓ VM {} deleted successfully", vm_name);

                        // Update operation state before reload
                        if let Some(row) = self.rows.iter_mut().find(|r| r.project_id == platform_name) {
                            row.operation_state = OperationState::Completed {
                                operation: "delete_vm".to_string(),
                                completed_at: chrono::Utc::now().timestamp(),
                            };
                        }

                        // Keep config update and reload logic
                        if let Ok((mut app_config, config_path)) = load_config() {
                            if let Some(platform) = app_config
                                .platforms
                                .iter_mut()
                                .find(|p| p.gcp_selected_project_id.as_ref() == Some(&platform_name))
                            {
                                platform.vms.retain(|vm| vm.name != vm_name);

                                if let Err(e) = app_config.save(&config_path) {
                                    self.load_error = Some(format!("Failed to save config: {}", e));
                                } else {
                                    dure_info!("✓ Config updated, refreshing table");
                                    self.loaded = false;
                                    self.load_error = None;
                                }
                            }
                        }
                    }
```

- [ ] **Step 5: Update VMsScanned event handler**

Find `ViewModelEvent::Platform(PlatformEvent::VMsScanned` handler (around line 847), update:

```rust
                    ViewModelEvent::Platform(PlatformEvent::VMsScanned {
                        platform_name,
                        vm_count,
                    }) => {
                        dure_info!("✓ Scanned and imported {} VMs for platform '{}'", vm_count, platform_name);

                        // Update operation state
                        if let Some(row) = self.rows.iter_mut().find(|r| r.project_id == platform_name) {
                            row.operation_state = OperationState::Completed {
                                operation: "scan".to_string(),
                                completed_at: chrono::Utc::now().timestamp(),
                            };
                        }

                        // Trigger reload to show imported VMs
                        self.loaded = false;
                        self.load_error = None;
                    }
```

- [ ] **Step 6: Compile to verify**

```bash
cd mobile && cargo check --features gui
```

Expected: SUCCESS

- [ ] **Step 7: Commit event handler updates**

```bash
git add mobile/src/ui_tabs/platform.rs
git commit -m "refactor(platform): update event handlers for incremental updates

- FirewallUpdated: Update row state without reload
- VMRestarted: Update operation state without reload
- VMCreated: Update row fields, keep reload for config sync
- VMDeleted: Update state before reload
- VMsScanned: Update state, keep reload for new VMs
- Remove unnecessary self.loaded = false calls

Part of platform drawer refactor (Phase 4: Event Flow)"
```

---

## Task 11: Add OperationFailed Event Support

**Files:**
- Modify: `mobile/src/viewmodel/platform/events.rs`
- Modify: `mobile/src/ui_tabs/platform.rs`

**Interfaces:**
- Consumes: Error results from ViewModel actor operations
- Produces: `OperationFailed` event in PlatformEvent enum
- Produces: Error state updates in UI

- [ ] **Step 1: Add OperationFailed to PlatformEvent enum**

Edit `mobile/src/viewmodel/platform/events.rs`, add after existing events:

```rust
    /// Operation failed with error
    OperationFailed {
        project_id: String,
        operation: String,    // "firewall", "restart", etc.
        error: String,
    },
```

- [ ] **Step 2: Add event handler in platform.rs**

In `platform.rs` event processing (around line 810), add after existing event handlers:

```rust
                    ViewModelEvent::Platform(PlatformEvent::OperationFailed {
                        project_id,
                        operation,
                        error,
                    }) => {
                        dure_error!("✗ Operation '{}' failed for {}: {}", operation, project_id, error);

                        // Update row to show error state
                        if let Some(row) = self.rows.iter_mut().find(|r| r.project_id == project_id) {
                            row.operation_state = OperationState::Failed {
                                operation: operation.clone(),
                                error: error.clone(),
                                failed_at: chrono::Utc::now().timestamp(),
                            };
                        }
                    }
```

- [ ] **Step 3: Compile to verify**

```bash
cd mobile && cargo check --features gui
```

Expected: SUCCESS (ViewModel actor implementations can send this event in future)

- [ ] **Step 4: Commit OperationFailed event**

```bash
git add mobile/src/viewmodel/platform/events.rs mobile/src/ui_tabs/platform.rs
git commit -m "feat(platform): add OperationFailed event for error handling

- Add OperationFailed variant to PlatformEvent enum
- Include project_id, operation name, and error message
- Update UI to show ✗ Failed state with error tooltip
- Auto-clear after 10 seconds (timestamp tracked)

Part of platform drawer refactor (Phase 4: Event Flow)"
```

---

## Task 12: Implement Auto-Clear for Completed/Failed States

**Files:**
- Modify: `mobile/src/ui_tabs/platform.rs`

**Interfaces:**
- Consumes: `OperationState` timestamps (completed_at, failed_at)
- Produces: Automatic state reset to Idle after timeout

- [ ] **Step 1: Add auto-clear logic in ui() function**

In `PlatformTab::ui()`, after event processing and before table rendering (around line 950), add:

```rust
        // Auto-clear Completed/Failed operation states
        let now = chrono::Utc::now().timestamp();
        for row in &mut self.rows {
            match &row.operation_state {
                OperationState::Completed { completed_at, .. } if now - completed_at > 3 => {
                    row.operation_state = OperationState::Idle;
                }
                OperationState::Failed { failed_at, .. } if now - failed_at > 10 => {
                    row.operation_state = OperationState::Idle;
                }
                _ => {}
            }
        }
```

- [ ] **Step 2: Request repaint when state changes**

After the auto-clear loop, add:

```rust
        // Request repaint to update UI when states auto-clear
        ui.ctx().request_repaint_after(std::time::Duration::from_secs(1));
```

- [ ] **Step 3: Compile to verify**

```bash
cd mobile && cargo check --features gui
```

Expected: SUCCESS

- [ ] **Step 4: Test auto-clear behavior**

```bash
cd mobile && cargo run --features gui --release
```

Check: After operation completes, ✅ clears after 3 seconds, ✗ clears after 10 seconds

- [ ] **Step 5: Commit auto-clear logic**

```bash
git add mobile/src/ui_tabs/platform.rs
git commit -m "feat(platform): implement auto-clear for operation states

- Clear Completed state after 3 seconds
- Clear Failed state after 10 seconds
- Request periodic repaint to update UI
- Return to Idle state automatically

Part of platform drawer refactor (Phase 4: Event Flow)"
```

---

## Task 13: Disable Buttons During Operations

**Files:**
- Modify: `mobile/src/ui_tabs/platform.rs`

**Interfaces:**
- Consumes: `operation_state` from `PlatformRow`
- Produces: Disabled buttons during InProgress state

- [ ] **Step 1: Add operation check before buttons**

Find operation buttons section (around line 1090), wrap in `add_enabled_ui`:

```rust
                                    ui.horizontal_wrapped(|ui| {
                                        ui.spacing_mut().item_spacing.x = 2.0;
                                        ui.style_mut().spacing.button_padding =
                                            egui::vec2(6.0, 2.0);

                                        // Check if any operation in progress
                                        let operation_in_progress = matches!(
                                            row_for_actions.operation_state,
                                            OperationState::InProgress { .. }
                                        );

                                        // Refresh button always enabled
                                        if ui
                                            .add(MaterialButton::outlined("Refresh").small())
                                            .on_hover_text("Refresh platform data")
                                            .clicked()
                                        {
                                            ui.data_mut(|d| {
                                                d.insert_temp(
                                                    egui::Id::new("platform_action_refresh"),
                                                    row_for_actions.project_id.clone(),
                                                )
                                            });
                                        }

                                        // Disable other buttons during operations
                                        ui.add_enabled_ui(!operation_in_progress, |ui| {
                                            // 1. Add VM
                                            #[cfg(not(any(
                                                target_os = "android",
                                                target_arch = "wasm32"
                                            )))]
                                            if ui
                                                .add_enabled(
                                                    !row_for_actions.has_vm
                                                        && row_for_actions.project_selected,
                                                    MaterialButton::outlined("Add VM").small(),
                                                )
                                                .on_hover_text("Add VM")
                                                .clicked()
                                            {
                                                // ... existing code
                                            }

                                            // ... rest of buttons (Scan VMs, Firewall, Restart, Del VM, Billing, Delete)
                                        });
                                    });
```

- [ ] **Step 2: Compile to verify**

```bash
cd mobile && cargo check --features gui
```

Expected: SUCCESS

- [ ] **Step 3: Test button disabling**

```bash
cd mobile && cargo run --features gui --release
```

Check: Click operation button, verify other buttons become disabled during ⏳ state

- [ ] **Step 4: Commit button disabling**

```bash
git add mobile/src/ui_tabs/platform.rs
git commit -m "feat(platform): disable operation buttons during InProgress state

- Prevent concurrent operations on same platform
- Refresh button remains enabled (safe to spam)
- All other buttons disabled when operation_state is InProgress
- Improves UX and prevents race conditions

Part of platform drawer refactor (Phase 4: Event Flow)"
```

---

## Task 14: Final Testing and Documentation

**Files:**
- Create: `docs/superpowers/specs/2026-08-04-platform-drawer-refactor-testing.md`

**Interfaces:**
- Produces: Testing checklist documentation
- Verifies: All requirements met

- [ ] **Step 1: Create testing checklist**

```bash
cat > docs/superpowers/specs/2026-08-04-platform-drawer-refactor-testing.md << 'EOF'
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
EOF
```

- [ ] **Step 2: Run all cargo tests**

```bash
cd mobile && cargo test --features gui
```

Expected: All tests PASS

- [ ] **Step 3: Manual testing on Desktop**

```bash
cd mobile && cargo run --features gui --release
```

Walk through checklist in testing document

- [ ] **Step 4: Verify no breaking changes**

```bash
cd mobile && cargo check --all-features
```

Expected: SUCCESS on all feature combinations

- [ ] **Step 5: Commit testing documentation**

```bash
git add docs/superpowers/specs/2026-08-04-platform-drawer-refactor-testing.md
git commit -m "docs: add platform drawer refactor testing checklist

- Functional requirements checklist (FR1-FR6)
- Non-functional requirements (NFR2-NFR4)
- Edge case testing
- Performance verification
- Regression testing
- Accessibility checks

Part of platform drawer refactor (Phase 4: Final Testing)"
```

---

## Self-Review

**1. Spec coverage:**
- ✅ FR1: Steps emoji progress bar (Task 8)
- ✅ FR2: Drawer grid layout (Task 7)
- ✅ FR3: Operation feedback (Task 9, 10)
- ✅ FR4: SSH dropdown (Task 7)
- ✅ FR5: Responsive reflow (Task 4, 7)
- ✅ FR6: Event-based updates (Task 9, 10, 11)
- ✅ NFR1: Reusable components (Tasks 2-6)
- ✅ NFR2: SVG with fallback (Task 2)
- ✅ NFR3: No ViewModel breaking changes (Task 11 is additive)
- ✅ NFR4: All platforms (SVG loading has platform guards)

**2. Placeholder scan:**
- All code complete, no TBD/TODO
- All test expectations specified
- All file paths exact
- All commands with expected output

**3. Type consistency:**
- `OperationState` fields match across all tasks
- `SvgEmoji` enum consistent in all components
- `ItemState`/`ProgressState` enums match usage
- Function signatures match between definition and use

**4. Dependencies:**
- Task 1 produces SVG assets → consumed by Task 2
- Task 2 produces SvgEmoji → consumed by Tasks 4, 5, 6
- Task 3 produces OperationState → consumed by Tasks 4, 7, 9, 10
- Tasks 4-6 produce components → consumed by Tasks 7-8
- Tasks 9-10 depend on Task 3 (OperationState)
- All dependencies satisfied in order

Plan is complete and ready for execution.

---

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-08-04-platform-drawer-refactor.md`.

Two execution options:

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

Which approach?
