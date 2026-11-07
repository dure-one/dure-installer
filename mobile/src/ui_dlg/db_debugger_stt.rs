//! Database debugger dialog state

#[derive(Default)]
pub struct DlgDbDebugger {
    pub open: bool,
    pub selected_table: String,
    pub table_names: Vec<String>,
    pub table_data: Vec<Vec<String>>, // Rows of columns
    pub column_names: Vec<String>,
    pub error_message: Option<String>,
    pub key_sequence_count: u8,
    pub last_key_time: f64,
    pub data_loaded: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state() {
        let state = DlgDbDebugger::default();
        assert!(!state.open);
        assert_eq!(state.selected_table, "");
        assert_eq!(state.key_sequence_count, 0);
        assert_eq!(state.last_key_time, 0.0);
        assert!(!state.data_loaded);
    }

    #[test]
    fn test_open_state() {
        let mut state = DlgDbDebugger::default();
        assert!(!state.open);

        state.open = true;
        assert!(state.open);
    }

    #[test]
    fn test_key_sequence_increment() {
        let mut state = DlgDbDebugger::default();
        assert_eq!(state.key_sequence_count, 0);

        state.key_sequence_count += 1;
        assert_eq!(state.key_sequence_count, 1);

        state.key_sequence_count += 1;
        assert_eq!(state.key_sequence_count, 2);
    }

    #[test]
    fn test_key_sequence_reset() {
        let mut state = DlgDbDebugger::default();
        state.key_sequence_count = 3;
        state.last_key_time = 100.0;

        state.key_sequence_count = 0;
        state.last_key_time = 0.0;

        assert_eq!(state.key_sequence_count, 0);
        assert_eq!(state.last_key_time, 0.0);
    }

    #[test]
    fn test_error_message() {
        let mut state = DlgDbDebugger::default();
        assert!(state.error_message.is_none());

        state.error_message = Some("Test error".to_string());
        assert_eq!(state.error_message, Some("Test error".to_string()));
    }
}
