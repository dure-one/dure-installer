//! .env file parser for Install Dure wizard
//!
//! Parses .env.example files into structured fields with comments

use serde::{Deserialize, Serialize};

/// A single environment variable field
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnvField {
    /// Variable name (e.g., "DB_HOST")
    pub key: String,
    /// Current value
    pub value: String,
    /// Comment line above this field (if any)
    pub comment: Option<String>,
}

impl EnvField {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            comment: None,
        }
    }

    pub fn with_comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }
}

/// Parse .env file content into structured fields
///
/// # Format
/// ```text
/// # This is a comment
/// KEY=VALUE
/// ANOTHER_KEY=another value
/// ```
///
/// # Returns
/// Vector of EnvField with keys, values, and associated comments
pub fn parse_env_file(content: &str) -> Vec<EnvField> {
    let mut fields = Vec::new();
    let mut pending_comment: Option<String> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            pending_comment = None;
            continue;
        }

        // Comment line
        if trimmed.starts_with('#') {
            let comment_text = trimmed.trim_start_matches('#').trim();
            pending_comment = Some(comment_text.to_string());
            continue;
        }

        // KEY=VALUE line
        if let Some((key, value)) = trimmed.split_once('=') {
            let mut field = EnvField::new(key.trim(), value.trim());

            if let Some(comment) = pending_comment.take() {
                field.comment = Some(comment);
            }

            fields.push(field);
        }

        // Clear pending comment if line was not KEY=VALUE
        pending_comment = None;
    }

    fields
}

/// Generate .env file content from fields
pub fn generate_env_file(fields: &[EnvField]) -> String {
    let mut output = String::new();

    for field in fields {
        // Add comment if present
        if let Some(comment) = &field.comment {
            output.push_str(&format!("# {}\n", comment));
        }

        // Add KEY=VALUE
        output.push_str(&format!("{}={}\n", field.key, field.value));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_env() {
        let input = "DB_HOST=localhost\nDB_PORT=5432\n";
        let fields = parse_env_file(input);

        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].key, "DB_HOST");
        assert_eq!(fields[0].value, "localhost");
        assert_eq!(fields[1].key, "DB_PORT");
        assert_eq!(fields[1].value, "5432");
    }

    #[test]
    fn test_parse_with_comments() {
        let input = r#"# Database host
DB_HOST=localhost
# Database port
DB_PORT=5432
"#;
        let fields = parse_env_file(input);

        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].key, "DB_HOST");
        assert_eq!(fields[0].comment, Some("Database host".to_string()));
        assert_eq!(fields[1].key, "DB_PORT");
        assert_eq!(fields[1].comment, Some("Database port".to_string()));
    }

    #[test]
    fn test_parse_with_empty_lines() {
        let input = r#"
DB_HOST=localhost

DB_PORT=5432

"#;
        let fields = parse_env_file(input);
        assert_eq!(fields.len(), 2);
    }

    #[test]
    fn test_parse_with_spaces() {
        let input = "  KEY1  =  value1  \n  KEY2=value2\n";
        let fields = parse_env_file(input);

        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].key, "KEY1");
        assert_eq!(fields[0].value, "value1");
    }

    #[test]
    fn test_parse_empty_value() {
        let input = "EMPTY_KEY=\n";
        let fields = parse_env_file(input);

        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].key, "EMPTY_KEY");
        assert_eq!(fields[0].value, "");
    }

    #[test]
    fn test_parse_value_with_equals() {
        let input = "URL=https://example.com?foo=bar\n";
        let fields = parse_env_file(input);

        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].key, "URL");
        assert_eq!(fields[0].value, "https://example.com?foo=bar");
    }

    #[test]
    fn test_generate_env_file() {
        let fields = vec![
            EnvField::new("KEY1", "value1").with_comment("First key"),
            EnvField::new("KEY2", "value2"),
        ];

        let output = generate_env_file(&fields);
        let expected = "# First key\nKEY1=value1\nKEY2=value2\n";
        assert_eq!(output, expected);
    }

    #[test]
    fn test_roundtrip() {
        let input = r#"# Comment 1
KEY1=value1
# Comment 2
KEY2=value2
"#;
        let fields = parse_env_file(input);
        let output = generate_env_file(&fields);
        assert_eq!(output, input);
    }
}
