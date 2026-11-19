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
            let key_trimmed = key.trim();
            let value_trimmed = value.trim();

            // Strip surrounding quotes if present (single or double)
            let value_unquoted = if (value_trimmed.starts_with('"') && value_trimmed.ends_with('"'))
                || (value_trimmed.starts_with('\'') && value_trimmed.ends_with('\''))
            {
                &value_trimmed[1..value_trimmed.len() - 1]
            } else {
                value_trimmed
            };

            let mut field = EnvField::new(key_trimmed, value_unquoted);

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
        // .env format: quotes optional, most parsers handle spaces/special chars correctly
        // Only quote if value has leading/trailing whitespace or starts with quote
        let value = &field.value;
        let needs_quotes = value != value.trim() || value.starts_with('"') || value.starts_with('\'');

        if needs_quotes {
            output.push_str(&format!("{}=\"{}\"\n", field.key, value));
        } else {
            output.push_str(&format!("{}={}\n", field.key, value));
        }
    }

    output
}

/// Extract domain from env fields
///
/// Looks for MYCART_DOMAIN or XMPP_DOMAIN and returns first non-empty value
pub fn get_domain_from_env(fields: &[EnvField]) -> Option<String> {
    for key in &["MYCART_DOMAIN", "XMPP_DOMAIN"] {
        if let Some(field) = fields.iter().find(|f| f.key == *key) {
            let trimmed = field.value.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

/// Get environment variable value by key
pub fn get_env_value(fields: &[EnvField], key: &str) -> Option<String> {
    fields
        .iter()
        .find(|f| f.key == key)
        .map(|f| f.value.clone())
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
    fn test_generate_with_spaces() {
        let fields = vec![
            EnvField::new("MSG", "Hello world"),
            EnvField::new("PATH", "/usr/bin"),
            EnvField::new("SPACED", " value "),
        ];

        let output = generate_env_file(&fields);
        // Spaces in middle: no quotes needed (.env parsers handle this)
        assert!(output.contains("MSG=Hello world\n"));
        assert!(output.contains("PATH=/usr/bin\n"));
        // Leading/trailing spaces: needs quotes
        assert!(output.contains("SPACED=\" value \"\n"));
    }

    #[test]
    fn test_parse_quoted_values() {
        let input = "KEY1=\"value with spaces\"\nKEY2='single quoted'\nKEY3=unquoted\n";
        let fields = parse_env_file(input);

        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].value, "value with spaces");
        assert_eq!(fields[1].value, "single quoted");
        assert_eq!(fields[2].value, "unquoted");
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

    #[test]
    fn test_get_domain_from_env_mycart() {
        let fields = vec![
            EnvField::new("OTHER_KEY", "value"),
            EnvField::new("MYCART_DOMAIN", "dure.co"),
            EnvField::new("XMPP_DOMAIN", "xmpp.dure.co"),
        ];

        let domain = get_domain_from_env(&fields);
        assert_eq!(domain, Some("dure.co".to_string()));
    }

    #[test]
    fn test_get_domain_from_env_xmpp_fallback() {
        let fields = vec![
            EnvField::new("OTHER_KEY", "value"),
            EnvField::new("XMPP_DOMAIN", "xmpp.dure.co"),
        ];

        let domain = get_domain_from_env(&fields);
        assert_eq!(domain, Some("xmpp.dure.co".to_string()));
    }

    #[test]
    fn test_get_domain_from_env_empty() {
        let fields = vec![
            EnvField::new("MYCART_DOMAIN", ""),
            EnvField::new("XMPP_DOMAIN", "   "),
        ];

        let domain = get_domain_from_env(&fields);
        assert_eq!(domain, None);
    }

    #[test]
    fn test_get_domain_from_env_none() {
        let fields = vec![EnvField::new("OTHER_KEY", "value")];

        let domain = get_domain_from_env(&fields);
        assert_eq!(domain, None);
    }

    #[test]
    fn test_get_env_value_exists() {
        let fields = vec![
            EnvField::new("KEY1", "value1"),
            EnvField::new("KEY2", "value2"),
        ];

        let value = get_env_value(&fields, "KEY2");
        assert_eq!(value, Some("value2".to_string()));
    }

    #[test]
    fn test_get_env_value_missing() {
        let fields = vec![EnvField::new("KEY1", "value1")];

        let value = get_env_value(&fields, "MISSING");
        assert_eq!(value, None);
    }

    #[test]
    fn test_get_env_value_empty() {
        let fields = vec![EnvField::new("KEY1", "")];

        let value = get_env_value(&fields, "KEY1");
        assert_eq!(value, Some("".to_string()));
    }
}
