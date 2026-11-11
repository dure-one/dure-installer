use crate::Settings;
use crate::ui_dlg::settings_stt::*;
use eframe::egui;
use egui_i18n::tr;
use egui_material3::{MaterialButton, ThemeMode, get_global_theme};
use sys_locale::get_locale;

impl DlgSettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self) {
        self.save_clicked = false;
        self.theme_to_apply = None;
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    fn detect_system_language() -> String {
        match get_locale().as_deref() {
            Some("ko_KR") | Some("ko-KR") | Some("ko") => "ko-KR".to_string(),
            Some("en_US") | Some("en-US") | Some("en_GB") | Some("en-GB") | Some("en") | _ => {
                "en-US".to_string()
            }
        }
    }

    fn theme_mode_to_string(mode: ThemeMode) -> String {
        match mode {
            ThemeMode::Light => "Light".to_string(),
            ThemeMode::Dark => "Dark".to_string(),
            ThemeMode::Auto => "Auto".to_string(),
        }
    }

    fn string_to_theme_mode(s: &str) -> ThemeMode {
        match s {
            "Light" => ThemeMode::Light,
            "Dark" => ThemeMode::Dark,
            _ => ThemeMode::Auto,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, settings: &mut Settings) {
        if !self.open {
            return;
        }

        self.save_clicked = false;
        self.theme_to_apply = None;

        let mut close_clicked = false;

        egui::Window::new(tr!("settings"))
            .id(egui::Id::new("settings_window"))
            .title_bar(false)
            .resizable(true)
            .collapsible(false)
            .scroll([false, false])
            .min_width(400.0)
            .min_height(300.0)
            .show(ctx, |ui| {
                ui.heading(tr!("settings"));
                ui.add_space(8.0);

                let max_height = ui.available_height() - 50.0;

                egui::ScrollArea::both()
                    .id_salt("settings_scroll")
                    .max_height(max_height)
                    .show(ui, |ui| {
                        // Language Selector
                        ui.horizontal(|ui| {
                            ui.label(tr!("language"));
                            let mut selected_lang = settings.language.clone();

                            egui::ComboBox::from_label("   ")
                                .selected_text(match selected_lang.as_str() {
                                    "Auto" => "Auto",
                                    "en-US" => "English",
                                    "ko-KR" => "한국어",
                                    _ => &selected_lang,
                                })
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut selected_lang,
                                        "Auto".to_string(),
                                        "Auto",
                                    );
                                    ui.selectable_value(
                                        &mut selected_lang,
                                        "en-US".to_string(),
                                        "English",
                                    );
                                    ui.selectable_value(
                                        &mut selected_lang,
                                        "ko-KR".to_string(),
                                        "한국어",
                                    );
                                });

                            if selected_lang != settings.language {
                                settings.language = selected_lang.clone();
                                let language_to_apply = if selected_lang == "Auto" {
                                    Self::detect_system_language()
                                } else {
                                    selected_lang
                                };
                                egui_i18n::set_language(&language_to_apply);
                            }
                        });

                        ui.add_space(8.0);

                        // Color Mode Selector
                        ui.horizontal(|ui| {
                            ui.label(tr!("color-mode"));
                            if let Ok(mut theme) = get_global_theme().lock() {
                                let light_selected = theme.theme_mode == ThemeMode::Light;
                                if ui
                                    .selectable_label(light_selected, tr!("light-mode"))
                                    .clicked()
                                {
                                    theme.theme_mode = ThemeMode::Light;
                                    settings.theme_mode =
                                        Self::theme_mode_to_string(ThemeMode::Light);
                                }

                                let auto_selected = theme.theme_mode == ThemeMode::Auto;
                                if ui
                                    .selectable_label(auto_selected, tr!("auto-mode"))
                                    .clicked()
                                {
                                    theme.theme_mode = ThemeMode::Auto;
                                    settings.theme_mode =
                                        Self::theme_mode_to_string(ThemeMode::Auto);
                                }

                                let dark_selected = theme.theme_mode == ThemeMode::Dark;
                                //if ui
                                //    .selectable_label(dark_selected, tr!("dark-mode"))
                                //    .clicked()
                                //{
                                //    theme.theme_mode = ThemeMode::Dark;
                                //    settings.theme_mode =
                                //        Self::theme_mode_to_string(ThemeMode::Dark);
                                //}
                            }
                        });

                        ui.add_space(8.0);

                        // Theme Mode Selector (Material Colors)
                        ui.horizontal(|ui| {
                            ui.label(tr!("theme-mode"));
                            let mut selected_theme = settings.theme_name.clone();
                            egui::ComboBox::from_id_salt("theme_selector")
                                .selected_text(match selected_theme.as_str() {
                                    "green" => "Green",
                                    "lightblue" => "Light Blue",
                                    "lightpink" => "Light Pink",
                                    "yellow" => "Yellow",
                                    _ => "Default",
                                })
                                .show_ui(ui, |ui| {
                                    for (value, label) in [
                                        ("default", "Default"),
                                        ("green", "Green"),
                                        ("lightblue", "Light Blue"),
                                        ("lightpink", "Light Pink"),
                                        ("yellow", "Yellow"),
                                    ] {
                                        if ui
                                            .selectable_value(
                                                &mut selected_theme,
                                                value.to_string(),
                                                label,
                                            )
                                            .clicked()
                                        {
                                            settings.theme_name = value.to_string();
                                            self.theme_to_apply = Some(value.to_string());
                                        }
                                    }
                                });
                        });

                        ui.add_space(8.0);

                        // Text Style Override Selector
                        ui.horizontal(|ui| {
                            let mut override_text_style = ui.style().override_text_style.clone();
                            ui.label(tr!("text-style"));
                            egui::ComboBox::from_id_salt("override_text_style")
                                .selected_text(match &override_text_style {
                                    None => "None".to_owned(),
                                    Some(s) => s.to_string(),
                                })
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut override_text_style, None, "None");
                                    let all_text_styles = ui.style().text_styles();
                                    for style in all_text_styles {
                                        use egui_material3::theme::get_global_color;
                                        let text = egui::RichText::new(style.to_string())
                                            .text_style(style.clone())
                                            .color(get_global_color("onSurface"));
                                        ui.selectable_value(
                                            &mut override_text_style,
                                            Some(style),
                                            text,
                                        );
                                    }
                                });
                            let text_style = override_text_style.clone();
                            ui.ctx().style_mut(|s| {
                                s.override_text_style = text_style.clone();
                            });

                            let style_string = match text_style {
                                None => String::new(),
                                Some(s) => s.to_string(),
                            };
                            if style_string != settings.override_text_style {
                                settings.override_text_style = style_string;
                            }
                        });

                        ui.add_space(8.0);

                        // Show Logs and Log Level
                        ui.horizontal(|ui| {
                            ui.label(tr!("show-logs"));
                            let show_logs_changed = ui.checkbox(&mut settings.show_logs, tr!("show")).changed();
                            if show_logs_changed {
                                crate::dure_info!("🔧 Show Logs checkbox toggled: show_logs={}", settings.show_logs);
                            }

                            use crate::LogLevel;
                            let current_level = LogLevel::from_str(&settings.log_level);

                            let mut level_changed = false;
                            for (level, label) in [
                                (LogLevel::Error, "ERROR"),
                                (LogLevel::Warn, "WARN"),
                                (LogLevel::Info, "INFO"),
                                (LogLevel::Debug, "DEBUG"),
                                (LogLevel::Trace, "TRACE"),
                            ] {
                                let selected = current_level == level;
                                if ui.selectable_label(selected, label).clicked() {
                                    settings.log_level = level.as_str().to_string();
                                    level_changed = true;
                                    #[cfg(not(target_arch = "wasm32"))]
                                    crate::log_capture::update_log_level(&settings.log_level);
                                }
                            }

                            // Update log settings when show_logs or log_level changes
                            #[cfg(not(target_arch = "wasm32"))]
                            if show_logs_changed || level_changed {
                                crate::log_capture::update_log_settings(
                                    settings.show_logs,
                                    LogLevel::from_str(&settings.log_level)
                                );
                            }
                        });
                    });

                ui.add_space(8.0);

                // Bottom buttons
                ui.horizontal(|ui| {
                    if ui.add(MaterialButton::filled(tr!("ok"))).clicked() {
                        self.save_clicked = true;
                        close_clicked = true;
                    }

                    if ui.add(MaterialButton::outlined("Cancel")).clicked() {
                        close_clicked = true;
                    }
                });
            });

        if close_clicked {
            self.close();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dlg_settings_open_close() {
        let mut dlg = DlgSettings::new();
        dlg.open();
        assert!(dlg.open);
        assert!(!dlg.save_clicked);

        dlg.close();
        assert!(!dlg.open);
    }

    #[test]
    fn test_detect_system_language() {
        let lang = DlgSettings::detect_system_language();
        assert!(lang == "en-US" || lang == "ko-KR");
    }

    #[test]
    fn test_theme_mode_to_string() {
        assert_eq!(DlgSettings::theme_mode_to_string(ThemeMode::Light), "Light");
        assert_eq!(DlgSettings::theme_mode_to_string(ThemeMode::Dark), "Dark");
        assert_eq!(DlgSettings::theme_mode_to_string(ThemeMode::Auto), "Auto");
    }

    #[test]
    fn test_string_to_theme_mode() {
        assert!(matches!(DlgSettings::string_to_theme_mode("Light"), ThemeMode::Light));
        assert!(matches!(DlgSettings::string_to_theme_mode("Dark"), ThemeMode::Dark));
        assert!(matches!(DlgSettings::string_to_theme_mode("Auto"), ThemeMode::Auto));
        assert!(matches!(DlgSettings::string_to_theme_mode("Invalid"), ThemeMode::Auto));
    }

    #[test]
    fn test_theme_selector_integration() {
        use crate::Settings;

        let mut settings = Settings::default();
        assert_eq!(settings.theme_name, "default");

        // Simulate selecting a different theme
        settings.theme_name = "green".to_string();
        assert_eq!(settings.theme_name, "green");

        settings.theme_name = "lightblue".to_string();
        assert_eq!(settings.theme_name, "lightblue");

        settings.theme_name = "lightpink".to_string();
        assert_eq!(settings.theme_name, "lightpink");

        settings.theme_name = "yellow".to_string();
        assert_eq!(settings.theme_name, "yellow");
    }

    #[test]
    fn test_color_mode_and_theme_mode_separate() {
        use crate::Settings;

        let mut settings = Settings::default();

        // Color Mode (Light/Dark/Auto) should be separate from Theme Mode (colors)
        settings.theme_mode = "Dark".to_string();
        settings.theme_name = "green".to_string();

        assert_eq!(settings.theme_mode, "Dark"); // Window color scheme
        assert_eq!(settings.theme_name, "green"); // Material color theme

        // Both can be changed independently
        settings.theme_mode = "Light".to_string();
        assert_eq!(settings.theme_mode, "Light");
        assert_eq!(settings.theme_name, "green"); // Theme name unchanged
    }
}
