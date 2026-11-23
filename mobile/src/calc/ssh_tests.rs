//! Tests for SSH key loading and tunnel SSH command generation

#[cfg(test)]
mod tests {
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_ssh_key_content_from_path() {
        // Arrange: Create temp file with test key
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let test_key =
            "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA...\n-----END RSA PRIVATE KEY-----";
        temp_file
            .write_all(test_key.as_bytes())
            .expect("Failed to write test key");
        let key_path = temp_file.path().to_str().unwrap().to_string();

        // Act
        let result = super::super::ssh::load_ssh_key_content_from_path(&key_path);

        // Assert
        assert!(result.is_some(), "Should load key from file");
        assert_eq!(result.unwrap(), test_key);
    }

    #[test]
    fn test_load_ssh_key_content_missing_file() {
        // Arrange
        let key_path = "/nonexistent/path/to/key";

        // Act
        let result = super::super::ssh::load_ssh_key_content_from_path(key_path);

        // Assert
        assert!(result.is_none(), "Should return None for missing file");
    }

    #[test]
    fn test_generate_tunnel_ssh_command() {
        // Arrange
        let test_key =
            "-----BEGIN RSA PRIVATE KEY-----\ntest_key_content\n-----END RSA PRIVATE KEY-----";
        let local_port = 54321;

        // Act
        let result = super::super::ssh::generate_tunnel_ssh_command(test_key, local_port);

        // Assert
        assert!(
            result.contains("ssh"),
            "Command should contain 'ssh': {}",
            result
        );
        assert!(
            result.contains(&local_port.to_string()),
            "Command should contain local port {}: {}",
            local_port,
            result
        );
        assert!(
            result.contains(test_key),
            "Command should embed private key"
        );
        assert!(
            result.contains("localhost"),
            "Command should target localhost"
        );
    }

    #[test]
    fn test_generate_tunnel_ssh_command_format() {
        // Arrange
        let test_key = "test_key";
        let local_port = 12345;

        // Act
        let result = super::super::ssh::generate_tunnel_ssh_command(test_key, local_port);

        // Assert - verify it uses heredoc format for key
        assert!(
            result.contains("<<'EOF'") || result.contains("<(cat"),
            "Command should use safe key embedding: {}",
            result
        );
    }
}
