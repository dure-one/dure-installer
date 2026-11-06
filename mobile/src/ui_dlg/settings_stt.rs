#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct DlgSettings {
    pub open: bool,
    // Action results
    pub save_clicked: bool,
    pub theme_to_apply: Option<String>,
}

impl Default for DlgSettings {
    fn default() -> Self {
        Self {
            open: false,
            save_clicked: false,
            theme_to_apply: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dlg_settings_default() {
        let dlg = DlgSettings::default();
        assert!(!dlg.open);
        assert!(!dlg.save_clicked);
        assert!(dlg.theme_to_apply.is_none());
    }
}
