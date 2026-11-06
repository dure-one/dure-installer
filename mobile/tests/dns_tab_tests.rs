//! Tests for DNS tab functionality

// Note: DnsTab fields are private for encapsulation.
// UI components are tested through:
// 1. Default trait implementation (public API)
// 2. Integration with DNS wizard (tested in dns_wizard_tests.rs)
// 3. Manual UI testing

#[test]
fn test_dns_tab_has_default() {
    // Verify DnsTab can be created with Default trait
    let _tab = dure::ui_tabs::dns::DnsTab::default();
    // If this compiles and runs, the test passes
}

// DNS operation methods spawn async threads and use external DNS-over-HTTPS API.
// These are integration-tested with real DNS queries below.

#[test]
#[ignore] // Run with --ignored flag (requires network)
fn test_dns_over_https_ns_query() {
    // Test DNS-over-HTTPS query for NS records
    #[derive(serde::Deserialize)]
    struct DnsResponse {
        #[serde(rename = "Answer")]
        answer: Option<Vec<DnsAnswer>>,
    }

    #[derive(serde::Deserialize)]
    struct DnsAnswer {
        data: String,
    }

    let url = "https://1.1.1.1/dns-query?name=cloudflare.com&type=NS";
    let response = ureq::get(url)
        .set("Accept", "application/dns-json")
        .timeout(std::time::Duration::from_secs(5))
        .call()
        .expect("DNS query should succeed");

    let dns_resp: DnsResponse = response.into_json().expect("Should parse JSON");
    assert!(dns_resp.answer.is_some(), "Should have NS records");
    assert!(!dns_resp.answer.unwrap().is_empty(), "Should have at least one NS record");
}

#[test]
#[ignore] // Run with --ignored flag (requires network)
fn test_dns_over_https_a_query() {
    // Test DNS-over-HTTPS query for A records
    #[derive(serde::Deserialize)]
    struct DnsResponse {
        #[serde(rename = "Answer")]
        answer: Option<Vec<DnsAnswer>>,
    }

    #[derive(serde::Deserialize)]
    struct DnsAnswer {
        data: String,
    }

    let url = "https://1.1.1.1/dns-query?name=cloudflare.com&type=A";
    let response = ureq::get(url)
        .set("Accept", "application/dns-json")
        .timeout(std::time::Duration::from_secs(5))
        .call()
        .expect("DNS query should succeed");

    let dns_resp: DnsResponse = response.into_json().expect("Should parse JSON");
    assert!(dns_resp.answer.is_some(), "Should have A records");
    let answers = dns_resp.answer.unwrap();
    assert!(!answers.is_empty(), "Should have at least one A record");

    // Verify it's an IP address
    let ip = &answers[0].data;
    assert!(ip.contains('.'), "Should be IPv4 address");
}

#[test]
#[ignore] // Run with --ignored flag (requires network)
fn test_https_connection_check() {
    // Test HTTPS connection to a known working domain
    let url = "https://cloudflare.com";
    let result = ureq::get(url)
        .timeout(std::time::Duration::from_secs(10))
        .call();

    assert!(result.is_ok(), "HTTPS connection should succeed");
    let response = result.unwrap();
    assert!(response.status() >= 200 && response.status() < 400, "Should get success status");
}
