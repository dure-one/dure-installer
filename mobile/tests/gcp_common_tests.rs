//! Tests for api/gcp common module

use dure::api::gcp::{get_current_ip, ip_in_ranges, get_common_machine_types};

#[test]
fn test_get_current_ip() {
    let result = get_current_ip();

    if let Ok(ip) = result {
        // Should be valid IPv4 format
        assert!(ip.contains('.'));
        assert!(!ip.contains('\n'));
        assert!(!ip.contains(' '));

        // Should be parseable as IP
        use std::net::Ipv4Addr;
        let parsed: Result<Ipv4Addr, _> = ip.parse();
        assert!(parsed.is_ok(), "IP should be valid IPv4: {}", ip);
    } else {
        // Allow test to pass if offline
        eprintln!("Skipping IP test (offline): {:?}", result);
    }
}

#[test]
fn test_ip_in_ranges() {
    let ranges = vec!["10.0.0.0/8".to_string(), "117.53.222.116/32".to_string()];

    assert!(ip_in_ranges("117.53.222.116", &ranges));
    assert!(!ip_in_ranges("192.168.1.1", &ranges));
}

#[test]
fn test_common_machine_types() {
    let types = get_common_machine_types();
    assert!(!types.is_empty());
    assert!(types.iter().any(|t| t.name == "e2-micro"));
}
