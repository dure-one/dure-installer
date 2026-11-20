//! Tests for mycart business logic
//!
//! Covers SSH host resolution and crontab parsing

#[cfg(test)]
mod tests {
    use crate::calc::mycart::{resolve_ssh_host_for_site};
    use crate::config::{AppConfig, MycartSiteConfig, SshHostConfig};

    fn create_test_ssh_host(host: &str, platform_id: Option<&str>) -> SshHostConfig {
        SshHostConfig {
            host: host.to_string(),
            password: None,
            private_key_path: None,
            keyring_domain: None,
            port: 22,
            initialized: true,
            last_status: None,
            platform_id: platform_id.map(String::from),
            docker_containers: Vec::new(),
            last_ssh_check: None,
            last_host_check: None,
            last_docker_check: None,
            last_dure_check: None,
        }
    }

    fn create_test_site(
        site_address: &str,
        ssh_host: Option<&str>,
        platform_id: Option<&str>,
    ) -> MycartSiteConfig {
        MycartSiteConfig {
            site_address: site_address.to_string(),
            email: "admin@test.com".to_string(),
            encrypted_password: "test".to_string(),
            jwt_token: Some("test_token".to_string()),
            created_at: 0,
            updated_at: 0,
            platform_id: platform_id.map(String::from),
            ssh_host: ssh_host.map(String::from),
        }
    }

    #[test]
    fn test_resolve_ssh_host_explicit_binding() {
        // Arrange
        let mut config = AppConfig::default();
        config.ssh_hosts.push(create_test_ssh_host("ubuntu@34.134.123.45", Some("test-project")));
        config.ssh_hosts.push(create_test_ssh_host("admin@dure.co", None));

        let site = create_test_site("dure.co", Some("34.134.123.45"), Some("test-project"));

        // Act
        let result = resolve_ssh_host_for_site(&config, &site);

        // Assert
        assert!(result.is_some(), "Should find SSH host via explicit binding");
        assert_eq!(result.unwrap().host, "ubuntu@34.134.123.45");
    }

    #[test]
    fn test_resolve_ssh_host_fallback_domain_match() {
        // Arrange
        let mut config = AppConfig::default();
        config.ssh_hosts.push(create_test_ssh_host("ubuntu@dure.co", None));
        config.ssh_hosts.push(create_test_ssh_host("admin@other.com", None));

        let site = create_test_site("dure.co", None, None);

        // Act
        let result = resolve_ssh_host_for_site(&config, &site);

        // Assert
        assert!(result.is_some(), "Should find SSH host via domain fallback");
        assert_eq!(result.unwrap().host, "ubuntu@dure.co");
    }

    #[test]
    fn test_resolve_ssh_host_fallback_ip_match() {
        // Arrange
        let mut config = AppConfig::default();
        config.ssh_hosts.push(create_test_ssh_host("ubuntu@34.134.123.45", None));

        let site = create_test_site("34.134.123.45", None, None);

        // Act
        let result = resolve_ssh_host_for_site(&config, &site);

        // Assert
        assert!(result.is_some(), "Should find SSH host via IP fallback");
        assert_eq!(result.unwrap().host, "ubuntu@34.134.123.45");
    }

    #[test]
    fn test_resolve_ssh_host_no_match() {
        // Arrange
        let mut config = AppConfig::default();
        config.ssh_hosts.push(create_test_ssh_host("ubuntu@other.com", None));

        let site = create_test_site("dure.co", None, None);

        // Act
        let result = resolve_ssh_host_for_site(&config, &site);

        // Assert
        assert!(result.is_none(), "Should return None when no match found");
    }

    #[test]
    fn test_resolve_ssh_host_explicit_takes_precedence() {
        // Arrange
        let mut config = AppConfig::default();
        config.ssh_hosts.push(create_test_ssh_host("ubuntu@34.134.123.45", None));
        config.ssh_hosts.push(create_test_ssh_host("admin@dure.co", None));

        // Explicit binding to IP, even though domain would also match
        let site = create_test_site("dure.co", Some("34.134.123.45"), None);

        // Act
        let result = resolve_ssh_host_for_site(&config, &site);

        // Assert
        assert!(result.is_some(), "Should find SSH host");
        assert_eq!(result.unwrap().host, "ubuntu@34.134.123.45",
            "Explicit binding should take precedence over domain fallback");
    }
}

#[cfg(test)]
mod crontab_parsing_tests {
    use serde_json::json;

    // Test will be added for crontab response parsing
    // This validates the API response format handling

    #[test]
    fn test_crontab_response_array_format() {
        // Arrange
        let response_json = json!({
            "success": true,
            "result": [
                {
                    "job_id": "cleanup",
                    "name": "Cleanup old logs",
                    "enabled": true,
                    "interval": "0 2 * * *",
                    "last_run": null
                },
                {
                    "job_id": "backup",
                    "name": "Database backup",
                    "enabled": false,
                    "interval": "0 3 * * *",
                    "last_run": 1234567890
                }
            ],
            "message": "Success"
        });

        // Act
        let success = response_json["success"].as_bool().unwrap();
        let result = &response_json["result"];

        // Assert
        assert!(success);
        assert!(result.is_array(), "Result should be an array");
        assert_eq!(result.as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_crontab_response_map_format() {
        // Arrange - API returns map with job IDs as keys
        let response_json = json!({
            "success": true,
            "result": {
                "cleanup": {
                    "job_id": "cleanup",
                    "name": "Cleanup old logs",
                    "enabled": true,
                    "interval": "0 2 * * *",
                    "last_run": null
                },
                "backup": {
                    "job_id": "backup",
                    "name": "Database backup",
                    "enabled": false,
                    "interval": "0 3 * * *",
                    "last_run": 1234567890
                }
            },
            "message": "Success"
        });

        // Act
        let success = response_json["success"].as_bool().unwrap();
        let result = &response_json["result"];

        // Assert
        assert!(success);
        assert!(result.is_object(), "Result should be an object/map");
        assert_eq!(result.as_object().unwrap().len(), 2);
    }
}
