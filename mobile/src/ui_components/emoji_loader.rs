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

    #[cfg(not(target_arch = "wasm32"))]
    fn load_svg_from_bytes(
        &self,
        emoji: SvgEmoji,
        svg_bytes: &[u8],
        _size: f32,
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

    /// Get cached texture
    pub fn get(&self, emoji: SvgEmoji) -> Option<&TextureHandle> {
        self.textures.get(&emoji)
    }

    /// Render emoji at specified size (with Unicode fallback)
    pub fn show(&self, ui: &mut egui::Ui, emoji: SvgEmoji, size: f32) {
        match self.get(emoji) {
            Some(texture) => {
                // Render SVG texture
                ui.image(egui::ImageSource::Texture(egui::load::SizedTexture {
                    id: texture.id(),
                    size: egui::vec2(size, size),
                }));
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
