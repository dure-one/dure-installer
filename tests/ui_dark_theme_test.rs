//! Dark theme color correctness tests
//!
//! Ensures UI components use theme colors instead of hardcoded values,
//! making them readable in both light and dark themes.

#[cfg(test)]
mod dark_theme_tests {
    use std::path::Path;

    /// Test that no UI files use Color32::BLACK for text
    /// (invisible on dark backgrounds)
    #[test]
    fn test_no_hardcoded_black_text() {
        let ui_dirs = ["mobile/src/ui_tabs", "mobile/src/ui_dlg"];
        let mut violations = Vec::new();

        for dir in &ui_dirs {
            if !Path::new(dir).exists() {
                continue;
            }

            let output = std::process::Command::new("grep")
                .args(&["-rn", "Color32::BLACK", dir])
                .output()
                .expect("Failed to run grep");

            if !output.stdout.is_empty() {
                let results = String::from_utf8_lossy(&output.stdout);
                violations.push(format!("Found Color32::BLACK in {}:\n{}", dir, results));
            }
        }

        assert!(
            violations.is_empty(),
            "Hardcoded Color32::BLACK found (invisible on dark theme):\n{}",
            violations.join("\n")
        );
    }

    /// Test that UI files prefer theme colors over hardcoded RGB
    /// Allows some exceptions (row-level status colors, badges)
    #[test]
    fn test_minimal_hardcoded_colors() {
        let ui_dirs = ["mobile/src/ui_tabs", "mobile/src/ui_dlg"];
        let mut hardcoded_count = 0;
        let mut details = Vec::new();

        for dir in &ui_dirs {
            if !Path::new(dir).exists() {
                continue;
            }

            let output = std::process::Command::new("grep")
                .args(&["-rn", "Color32::from_rgb", dir])
                .output()
                .expect("Failed to run grep");

            if !output.stdout.is_empty() {
                let results = String::from_utf8_lossy(&output.stdout);
                let lines: Vec<&str> = results.lines().collect();
                hardcoded_count += lines.len();
                details.push(format!("Found {} hardcoded colors in {}:\n{}",
                    lines.len(), dir, results));
            }
        }

        // Allow some hardcoded colors for row-level status, but warn if excessive
        if hardcoded_count > 5 {
            eprintln!(
                "Warning: {} hardcoded Color32::from_rgb found. Consider using theme colors:\n{}",
                hardcoded_count,
                details.join("\n")
            );
        }
    }

    /// Test that section headers use theme text colors
    #[test]
    fn test_section_headers_use_theme() {
        let file = "mobile/src/ui_tabs/dns_drawer.rs";
        if !Path::new(file).exists() {
            return;
        }

        let content = std::fs::read_to_string(file)
            .expect("Failed to read dns_drawer.rs");

        // Check that "Platform Binding" and "Check Status" headers don't use BLACK
        let problematic_lines: Vec<_> = content
            .lines()
            .enumerate()
            .filter(|(_, line)| {
                (line.contains("Platform Binding") || line.contains("Check Status"))
                    && line.contains("Color32::BLACK")
            })
            .collect();

        assert!(
            problematic_lines.is_empty(),
            "Section headers in dns_drawer.rs use Color32::BLACK (lines: {:?})",
            problematic_lines.iter().map(|(i, _)| i + 1).collect::<Vec<_>>()
        );
    }

    /// Test that colored_label usage is minimal
    /// (colored_label with hardcoded colors won't adapt to theme)
    #[test]
    fn test_colored_label_usage() {
        let ui_dirs = ["mobile/src/ui_tabs", "mobile/src/ui_dlg"];
        let mut colored_label_count = 0;
        let mut details = Vec::new();

        for dir in &ui_dirs {
            if !Path::new(dir).exists() {
                continue;
            }

            let output = std::process::Command::new("grep")
                .args(&["-rn", "colored_label", dir])
                .output()
                .expect("Failed to run grep");

            if !output.stdout.is_empty() {
                let results = String::from_utf8_lossy(&output.stdout);
                let lines: Vec<&str> = results.lines().collect();
                colored_label_count += lines.len();
                details.push(format!("Found {} colored_label calls in {}:\n{}",
                    lines.len(), dir, results));
            }
        }

        // colored_label is OK for semantic status, but prefer badges
        if colored_label_count > 20 {
            eprintln!(
                "Info: {} colored_label calls found. Consider using badge() for status:\n{}",
                colored_label_count,
                details.join("\n")
            );
        }
    }

    /// Test that RichText.color() calls use theme colors
    #[test]
    fn test_richtext_uses_theme_colors() {
        let file = "mobile/src/ui_tabs/dns_drawer.rs";
        if !Path::new(file).exists() {
            return;
        }

        let content = std::fs::read_to_string(file)
            .expect("Failed to read dns_drawer.rs");

        // Count RichText::new().color(Color32::...) patterns
        let bad_patterns = content
            .lines()
            .enumerate()
            .filter(|(_, line)| {
                line.contains("RichText::new")
                    && line.contains(".color(")
                    && (line.contains("Color32::BLACK")
                        || line.contains("Color32::WHITE")
                        || line.contains("Color32::from_rgb"))
                    && !line.contains("ui.visuals()")
            })
            .collect::<Vec<_>>();

        assert!(
            bad_patterns.is_empty(),
            "RichText uses hardcoded colors instead of theme in dns_drawer.rs (lines: {:?})",
            bad_patterns.iter().map(|(i, _)| i + 1).collect::<Vec<_>>()
        );
    }

    /// Test that theme mode changes call apply_theme to update visuals
    #[test]
    fn test_theme_mode_applies_visuals() {
        let file = "mobile/src/ui_dlg/settings.rs";
        if !Path::new(file).exists() {
            return;
        }

        let content = std::fs::read_to_string(file)
            .expect("Failed to read settings.rs");

        // Find theme mode changes (ThemeMode::Light, Dark, Auto)
        let theme_changes: Vec<usize> = content
            .lines()
            .enumerate()
            .filter(|(_, line)| {
                line.contains("theme.theme_mode = ThemeMode::")
            })
            .map(|(i, _)| i)
            .collect();

        assert!(!theme_changes.is_empty(), "No theme mode changes found in settings.rs");

        // For each theme mode change, verify apply_theme is called nearby
        let lines: Vec<&str> = content.lines().collect();
        let mut missing_apply_theme = Vec::new();

        for &change_line in &theme_changes {
            // Check 10 lines after the theme change for apply_theme call
            let end_line = (change_line + 10).min(lines.len());
            let has_apply_theme = lines[change_line..end_line]
                .iter()
                .any(|line| line.contains("apply_theme") || line.contains("Self::apply_theme_to_context"));

            if !has_apply_theme {
                missing_apply_theme.push(change_line + 1); // +1 for 1-based line numbers
            }
        }

        assert!(
            missing_apply_theme.is_empty(),
            "Theme mode changes missing apply_theme call (lines: {:?}). \
            After changing theme_mode, must call apply_theme() to update visuals.",
            missing_apply_theme
        );
    }
}
