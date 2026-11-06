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

                        // Theme Mode Selector
                        ui.horizontal(|ui| {
                            ui.label(tr!("theme-mode"));

                            if let Ok(mut theme) = get_global_theme().lock() {
                                let mut selected_mode = Self::theme_mode_to_string(theme.theme_mode);

                                egui::ComboBox::from_id_salt("theme_mode_selector")
                                    .selected_text(&selected_mode)
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            &mut selected_mode,
                                            "Light".to_string(),
                                            "Light",
                                        );
                                        ui.selectable_value(
                                            &mut selected_mode,
                                            "Dark".to_string(),
                                            "Dark",
                                        );
                                        ui.selectable_value(
                                            &mut selected_mode,
                                            "Auto".to_string(),
                                            "Auto",
                                        );
                                    });

                                if selected_mode != settings.theme_mode {
                                    settings.theme_mode = selected_mode.clone();
                                    theme.theme_mode = Self::string_to_theme_mode(&selected_mode);
                                    self.theme_to_apply = Some(selected_mode.clone());
                                }
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
}
