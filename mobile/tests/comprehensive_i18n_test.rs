//! Comprehensive i18n tests for platform, SSH, and DNS modules
//!
//! Verifies all user-facing strings use Fluent translation system

use std::fs;

#[test]
fn test_platform_drawer_no_hardcoded_strings() {
    let source = fs::read_to_string("src/ui_tabs/platform_drawer.rs")
        .expect("Failed to read platform_drawer.rs");

    let forbidden = vec![
        r#"StatusLine::new("Refreshed""#,
        r#"StatusLine::new("IP""#,
        r#"StatusLine::new("Firewall""#,
        r#"StatusLine::new("SSH""#,
        r#"StatusLine::new("Operation""#,
        r#"add_action("Copy SSH"#,
        r#"add_action("Copy Private"#,
        r#"add_action("Copy IP"#,
        r#"ActionMenu::new("SSH")"#,
        r#""No external IP""#,
        r#""No VM created""#,
        r#""No project selected""#,
        r#""Not found in keyring""#,
        r#""Updating...""#,
    ];

    let mut found = Vec::new();
    for pattern in &forbidden {
        if source.contains(pattern) {
            found.push(*pattern);
        }
    }

    assert!(
        found.is_empty(),
        "Found hardcoded strings in platform_drawer.rs:\n{}",
        found.join("\n")
    );
}

#[test]
fn test_ssh_no_hardcoded_strings() {
    let source = fs::read_to_string("src/ui_tabs/ssh.rs")
        .expect("Failed to read ssh.rs");

    let forbidden = vec![
        r#"operation: "Refresh""#,
        r#"operation: "SSH Check""#,
        r#"operation: "Base Check""#,
        r#"operation: "Install Base""#,
        r#"operation: "Docker Check""#,
        r#"operation: "Install Docker""#,
        r#""No active profile"#,
        r#""SSH management not available on WASM""#,
        r#""Unknown error""#,
    ];

    let mut found = Vec::new();
    for pattern in &forbidden {
        if source.contains(pattern) {
            found.push(*pattern);
        }
    }

    assert!(
        found.is_empty(),
        "Found hardcoded strings in ssh.rs:\n{}",
        found.join("\n")
    );
}

#[test]
fn test_ssh_drawer_no_hardcoded_strings() {
    let source = fs::read_to_string("src/ui_tabs/ssh_drawer.rs")
        .expect("Failed to read ssh_drawer.rs");

    let forbidden = vec![
        r#"LogsRenderer::new("SSH Host Logs")"#,
        r#"heading("Host Information")"#,
        r#"button("Refresh")"#,
        r#"label(format!("OS: {}"#,
        r#"label(format!("Uptime: {}"#,
        r#"label(format!("External IP: {}"#,
        r#"label(format!("Load: {}"#,
        r#"label(format!("Memory: {}"#,
        r#"label(format!("Disk: {}"#,
        r#"heading("Network Log"#,
    ];

    let mut found = Vec::new();
    for pattern in &forbidden {
        if source.contains(pattern) {
            found.push(*pattern);
        }
    }

    assert!(
        found.is_empty(),
        "Found hardcoded strings in ssh_drawer.rs:\n{}",
        found.join("\n")
    );
}

#[test]
fn test_dns_no_hardcoded_strings() {
    let source = fs::read_to_string("src/ui_tabs/dns.rs")
        .expect("Failed to read dns.rs");

    let forbidden = vec![
        r#"=> "Status""#,
        r#"=> "Logs""#,
        r#"=> "Operations""#,
        r#""Found {} NS record(s)""#,
        r#""No NS records found""#,
        r#""DNS query failed""#,
        r#""A record for root domain""#,
        r#""Cloudflare API check not yet implemented""#,
    ];

    let mut found = Vec::new();
    for pattern in &forbidden {
        if source.contains(pattern) {
            found.push(*pattern);
        }
    }

    assert!(
        found.is_empty(),
        "Found hardcoded strings in dns.rs:\n{}",
        found.join("\n")
    );
}

#[test]
fn test_dns_wizard_no_hardcoded_strings() {
    let source = fs::read_to_string("src/ui_dlg/dns_wizard.rs")
        .expect("Failed to read dns_wizard.rs");

    let forbidden = vec![
        r#"MaterialButton::outlined("Open OAuth Screen")"#,
        r#"MaterialButton::filled("Next")"#,
        r#"MaterialButton::outlined("Previous")"#,
        r#"heading("Step 1: Cloudflare Authentication")"#,
        r#"label("Connect your Cloudflare account"#,
        r#"label("Waiting for operations...")"#,
        r#"label("Loading: Waiting for OAuth"#,
        r#"label("Please complete the authorization"#,
        r#""Success: OAuth successful""#,
        r#""Error: OAuth failed""#,
        r#""API connection verified""#,
        r#""Verifying API connection...""#,
        r#""Account ID:""#,
        r#""Warning: Account ID required""#,
    ];

    let mut found = Vec::new();
    for pattern in &forbidden {
        if source.contains(pattern) {
            found.push(*pattern);
        }
    }

    assert!(
        found.is_empty(),
        "Found hardcoded strings in dns_wizard.rs:\n{}",
        found.join("\n")
    );
}

#[test]
fn test_fluent_keys_exist() {
    let en_ftl = fs::read_to_string("assets/languages/fluent/en-US.ftl")
        .expect("Failed to read en-US.ftl");
    let ko_ftl = fs::read_to_string("assets/languages/fluent/ko-KR.ftl")
        .expect("Failed to read ko-KR.ftl");

    let required_keys = vec![
        // Platform drawer
        "platform-drawer-status-refreshed",
        "platform-drawer-status-ip",
        "platform-drawer-status-no-ip",
        "platform-drawer-status-firewall",
        "platform-drawer-status-ssh",
        "platform-drawer-menu-copy-ssh",
        "platform-drawer-menu-copy-key",
        "platform-drawer-menu-copy-ip",

        // SSH
        "ssh-operation-refresh",
        "ssh-operation-ssh-check",
        "ssh-operation-base-check",
        "ssh-operation-install-base",
        "ssh-operation-docker-check",
        "ssh-operation-install-docker",
        "ssh-host-info-title",
        "ssh-logs-title",
        "ssh-button-refresh",

        // DNS
        "dns-drawer-tab-status",
        "dns-drawer-tab-logs",
        "dns-drawer-tab-operations",
        "dns-check-ns-found",
        "dns-check-ns-none",
        "dns-wizard-step1-title",
        "dns-wizard-button-next",
        "dns-wizard-button-previous",
    ];

    let mut missing_en = Vec::new();
    let mut missing_ko = Vec::new();

    for key in &required_keys {
        let key_pattern = format!("{} =", key);
        if !en_ftl.contains(&key_pattern) {
            missing_en.push(*key);
        }
        if !ko_ftl.contains(&key_pattern) {
            missing_ko.push(*key);
        }
    }

    assert!(
        missing_en.is_empty(),
        "Missing keys in en-US.ftl:\n{}",
        missing_en.join("\n")
    );
    assert!(
        missing_ko.is_empty(),
        "Missing keys in ko-KR.ftl:\n{}",
        missing_ko.join("\n")
    );
}
