//! Tests for api/gcp/compute module

use dure::api::gcp::compute::{FirewallRule, FirewallAllowed, AccessConfig};

#[test]
fn test_firewall_rule_structure() {
    let rule = FirewallRule {
        name: "allow-ssh".to_string(),
        allowed: vec![FirewallAllowed {
            ip_protocol: "tcp".to_string(),
            ports: Some(vec!["22".to_string()]),
        }],
        source_ranges: Some(vec!["0.0.0.0/0".to_string()]),
    };

    assert_eq!(rule.name, "allow-ssh");
    assert_eq!(rule.allowed[0].ip_protocol, "tcp");
}

// ============================================================================
// AccessConfig Serialization Tests
// ============================================================================

#[test]
fn test_access_config_serialization_with_nat_ip() {
    // Arrange
    let config = AccessConfig {
        type_: "ONE_TO_ONE_NAT".to_string(),
        name: "External NAT".to_string(),
        nat_ip: Some("1.2.3.4".to_string()),
        network_tier: None,
    };

    // Act
    let json = serde_json::to_value(&config).expect("serialization failed");

    // Assert
    assert_eq!(json["type"], "ONE_TO_ONE_NAT");
    assert_eq!(json["name"], "External NAT");
    assert_eq!(json["natIP"], "1.2.3.4");

    // Verify all fields are present
    assert!(json.get("type").is_some(), "type field must be present");
    assert!(json.get("name").is_some(), "name field must be present");
    assert!(json.get("natIP").is_some(), "natIP field must be present");
}

#[test]
fn test_access_config_serialization_without_nat_ip() {
    // Arrange
    let config = AccessConfig {
        type_: "ONE_TO_ONE_NAT".to_string(),
        name: "External NAT".to_string(),
        nat_ip: None,
        network_tier: None,
    };

    // Act
    let json = serde_json::to_value(&config).expect("serialization failed");

    // Assert
    assert_eq!(json["type"], "ONE_TO_ONE_NAT");
    assert_eq!(json["name"], "External NAT");

    // Verify natIP field is skipped (not present) when None
    assert!(json.get("type").is_some(), "type field must be present");
    assert!(json.get("name").is_some(), "name field must be present");
    assert!(json.get("natIP").is_none(), "natIP field must be skipped when None");
}

#[test]
fn test_access_config_serialization_json_string() {
    // Test with nat_ip Some
    let config_with_ip = AccessConfig {
        type_: "ONE_TO_ONE_NAT".to_string(),
        name: "External NAT".to_string(),
        nat_ip: Some("10.0.0.1".to_string()),
        network_tier: None,
    };

    let json_string = serde_json::to_string(&config_with_ip)
        .expect("serialization to string failed");

    // natIP field should be in the JSON string
    assert!(json_string.contains("natIP"), "natIP field must be in JSON string");
    assert!(json_string.contains("10.0.0.1"), "IP address must be in JSON string");

    // Test with nat_ip None
    let config_without_ip = AccessConfig {
        type_: "ONE_TO_ONE_NAT".to_string(),
        name: "External NAT".to_string(),
        nat_ip: None,
        network_tier: None,
    };

    let json_string = serde_json::to_string(&config_without_ip)
        .expect("serialization to string failed");

    // natIP field should NOT be in the JSON string when None
    assert!(!json_string.contains("natIP"), "natIP field must not be in JSON string when None");
}
