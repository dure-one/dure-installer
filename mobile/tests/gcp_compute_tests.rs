//! Tests for api/gcp/compute module

use dure::api::gcp::compute::{FirewallRule, FirewallAllowed};

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
