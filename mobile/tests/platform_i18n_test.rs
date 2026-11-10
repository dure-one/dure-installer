//! Platform tab i18n translation tests
//!
//! Verifies all user-facing strings use Fluent translation system

use std::fs;

#[test]
fn test_platform_no_hardcoded_strings() {
    // Read platform.rs source
    let platform_source = fs::read_to_string("src/ui_tabs/platform.rs")
        .expect("Failed to read platform.rs");

    // Critical hardcoded strings that must not appear without tr!()
    let forbidden_strings = vec![
        r#"heading("Cloud Platforms")"#,
        r#"label("Manage cloud service"#,
        r#"MaterialButton::outlined("Firewall")"#,
        r#"MaterialButton::outlined("Restart")"#,
        r#"MaterialButton::outlined("Del VM")"#,
        r#"MaterialButton::filled("Add Platform")"#,
        r#"label("Configure a new cloud"#,
        r#"label("Type:")"#,
        r#"label("Project:")"#,
        r#"label("Waiting for authorization"#,
        r#"heading("Initialization Progress")"#,
        r#"heading("Confirm Deletion")"#,
        r#"heading("Select VM to Delete")"#,
        r#"heading("Select Project")"#,
    ];

    let mut found_hardcoded = Vec::new();

    for forbidden in &forbidden_strings {
        if platform_source.contains(forbidden) {
            found_hardcoded.push(*forbidden);
        }
    }

    assert!(
        found_hardcoded.is_empty(),
        "Found hardcoded strings in platform.rs (should use tr! macro):\n{}",
        found_hardcoded.join("\n")
    );
}

#[test]
fn test_en_us_has_required_keys() {
    let en_ftl = fs::read_to_string("assets/languages/fluent/en-US.ftl")
        .expect("Failed to read en-US.ftl");

    let required_keys = vec![
        "platform-main-heading",
        "platform-main-desc",
        "platform-button-add",
        "platform-button-firewall",
        "platform-button-restart",
        "platform-button-del-vm",
        "platform-add-configure",
        "platform-add-type",
        "platform-add-project",
        "platform-add-project-id",
        "platform-add-oauth-waiting",
        "platform-add-oauth-complete",
        "platform-add-oauth-connect",
        "platform-add-oauth-url",
        "platform-progress-heading",
        "platform-progress-platform",
        "platform-delete-vm-confirm",
        "platform-delete-vm-question",
        "platform-delete-vm-zone",
        "platform-delete-vm-select",
        "platform-delete-vm-none",
        "platform-delete-vm-found",
        "platform-delete-vm-yes",
        "platform-delete-vm-cancel",
        "platform-delete-vm-next",
        "platform-delete-vm-manage-ips",
        "platform-select-project-heading",
        "platform-select-project-none",
        "platform-select-project-button",
        "platform-drawer-no-vms",
        "platform-billing-heading",
        "platform-billing-project-id",
        "platform-billing-loading",
        "platform-billing-no-table",
        "platform-billing-error",
        "platform-billing-help",
        "platform-billing-no-data",
    ];

    let mut missing_keys = Vec::new();

    for key in &required_keys {
        let key_pattern = format!("{} =", key);
        if !en_ftl.contains(&key_pattern) {
            missing_keys.push(*key);
        }
    }

    assert!(
        missing_keys.is_empty(),
        "Missing translation keys in en-US.ftl:\n{}",
        missing_keys.join("\n")
    );
}

#[test]
fn test_ko_kr_has_required_keys() {
    let ko_ftl = fs::read_to_string("assets/languages/fluent/ko-KR.ftl")
        .expect("Failed to read ko-KR.ftl");

    let required_keys = vec![
        "platform-main-heading",
        "platform-main-desc",
        "platform-button-add",
        "platform-button-firewall",
        "platform-button-restart",
        "platform-button-del-vm",
        "platform-add-configure",
        "platform-add-type",
        "platform-add-project",
        "platform-add-project-id",
        "platform-add-oauth-waiting",
        "platform-add-oauth-complete",
        "platform-add-oauth-connect",
        "platform-add-oauth-url",
        "platform-progress-heading",
        "platform-progress-platform",
        "platform-delete-vm-confirm",
        "platform-delete-vm-question",
        "platform-delete-vm-zone",
        "platform-delete-vm-select",
        "platform-delete-vm-none",
        "platform-delete-vm-found",
        "platform-delete-vm-yes",
        "platform-delete-vm-cancel",
        "platform-delete-vm-next",
        "platform-delete-vm-manage-ips",
        "platform-select-project-heading",
        "platform-select-project-none",
        "platform-select-project-button",
        "platform-drawer-no-vms",
        "platform-billing-heading",
        "platform-billing-project-id",
        "platform-billing-loading",
        "platform-billing-no-table",
        "platform-billing-error",
        "platform-billing-help",
        "platform-billing-no-data",
    ];

    let mut missing_keys = Vec::new();

    for key in &required_keys {
        let key_pattern = format!("{} =", key);
        if !ko_ftl.contains(&key_pattern) {
            missing_keys.push(*key);
        }
    }

    assert!(
        missing_keys.is_empty(),
        "Missing translation keys in ko-KR.ftl:\n{}",
        missing_keys.join("\n")
    );
}

#[test]
fn test_platform_uses_tr_macro() {
    let platform_source = fs::read_to_string("src/ui_tabs/platform.rs")
        .expect("Failed to read platform.rs");

    // Verify tr! macro is imported and used
    assert!(
        platform_source.contains("use egui_i18n::tr;"),
        "platform.rs must import tr! macro"
    );

    // Count tr! usage - should have many instances
    let tr_count = platform_source.matches("tr!(").count();
    assert!(
        tr_count >= 40,
        "platform.rs should use tr!() macro extensively (found {} uses, expected >= 40)",
        tr_count
    );
}
