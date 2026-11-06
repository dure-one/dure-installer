//! Tests for DNS wizard dialog

use dure::ui_dlg::dns_wizard::{DnsWizard, DnsWizardState};

#[test]
fn test_dns_wizard_creation() {
    let wizard = DnsWizard::new("test_profile".to_string());
    assert_eq!(wizard.state(), &DnsWizardState::CloudflareAuth);
}

#[test]
fn test_dns_wizard_initial_state_is_auth() {
    let wizard = DnsWizard::new("test".to_string());
    assert!(matches!(wizard.state(), DnsWizardState::CloudflareAuth));
}

#[test]
fn test_dns_wizard_state_enum() {
    // Test that all wizard states are accessible
    let _auth = DnsWizardState::CloudflareAuth;
    let _domain = DnsWizardState::DomainRegistration;
    let _vm = DnsWizardState::VmConfiguration;
    let _confirm = DnsWizardState::Confirmation;
    let _complete = DnsWizardState::Complete;
    let _error = DnsWizardState::Error("test error".to_string());

    // Test state equality
    assert_eq!(DnsWizardState::CloudflareAuth, DnsWizardState::CloudflareAuth);
    assert_ne!(DnsWizardState::CloudflareAuth, DnsWizardState::Complete);
}

#[test]
fn test_step1_cannot_proceed_without_oauth() {
    let wizard = DnsWizard::new("test".to_string());
    assert!(!wizard.can_proceed_from_step1());
}

#[test]
fn test_step1_cannot_proceed_without_account_id() {
    let mut wizard = DnsWizard::new("test".to_string());
    wizard.set_oauth_result_for_test();
    assert!(!wizard.can_proceed_from_step1());
}

#[test]
fn test_step1_can_proceed_after_oauth_and_account_id() {
    let mut wizard = DnsWizard::new("test".to_string());
    wizard.set_oauth_result_for_test();
    wizard.set_account_id_for_test("account123".to_string());
    assert!(wizard.can_proceed_from_step1());
}

// Step 2 tests
#[test]
fn test_step2_domain_input() {
    let mut wizard = DnsWizard::new("test".to_string());
    wizard.set_domain_input_for_test("example.com".to_string());
    assert_eq!(wizard.domain_input(), "example.com");
}

#[test]
fn test_step2_can_proceed_with_selected_domain() {
    let mut wizard = DnsWizard::new("test".to_string());
    wizard.select_domain_for_test("dure.co".to_string());
    assert!(wizard.can_proceed_from_step2());
}

#[test]
fn test_step2_cannot_proceed_without_domain() {
    let wizard = DnsWizard::new("test".to_string());
    assert!(!wizard.can_proceed_from_step2());
}

// Step 3 tests
#[test]
fn test_step3_cannot_proceed_without_vm() {
    let wizard = DnsWizard::new("test".to_string());
    assert!(!wizard.can_proceed_from_step3());
}

#[test]
fn test_step3_can_proceed_with_selected_vm() {
    let mut wizard = DnsWizard::new("test".to_string());
    wizard.select_vm_for_test("dure-432719".to_string(), "35.212.193.170".to_string());
    assert!(wizard.can_proceed_from_step3());
}

#[test]
fn test_step3_selected_vm_info() {
    let mut wizard = DnsWizard::new("test".to_string());
    wizard.select_vm_for_test("dure-432719".to_string(), "35.212.193.170".to_string());
    assert_eq!(wizard.selected_vm(), Some("dure-432719"));
    assert_eq!(wizard.selected_vm_ip(), Some("35.212.193.170"));
}

// Step 4 tests
#[test]
fn test_step4_dns_not_configured_initially() {
    let wizard = DnsWizard::new("test".to_string());
    assert!(!wizard.dns_configured());
}

#[test]
fn test_step4_dns_configured_after_setup() {
    let mut wizard = DnsWizard::new("test".to_string());
    wizard.set_dns_configured_for_test(true);
    assert!(wizard.dns_configured());
}

#[test]
fn test_step4_can_add_after_dns_configured() {
    let mut wizard = DnsWizard::new("test".to_string());
    assert!(!wizard.can_proceed_from_step4());

    wizard.set_dns_configured_for_test(true);
    assert!(wizard.can_proceed_from_step4());
}

// Domain binding tests
#[test]
fn test_save_domain_binding() {
    let wizard = DnsWizard::new("test".to_string());
    let (vm_name, domain) = wizard.get_binding_data();
    assert!(vm_name.is_none());
    assert!(domain.is_none());
}

#[test]
fn test_get_binding_data_when_selected() {
    let mut wizard = DnsWizard::new("test".to_string());
    wizard.select_vm_for_test("vm-1".to_string(), "1.2.3.4".to_string());
    wizard.select_domain_for_test("example.com".to_string());

    let (vm_name, domain) = wizard.get_binding_data();
    assert_eq!(vm_name, Some("vm-1".to_string()));
    assert_eq!(domain, Some("example.com".to_string()));
}

#[test]
fn test_dns_wizard_vm_info_loading() {
    let mut wizard = DnsWizard::new("test".to_string());

    // Test load_available_vms method
    let vms = vec![
        ("vm-1".to_string(), "1.2.3.4".to_string(), "us-west1-a".to_string()),
        ("vm-2".to_string(), "5.6.7.8".to_string(), "us-east1-b".to_string()),
    ];

    wizard.load_available_vms(vms);
    // If this compiles and runs without panic, the public API is working
}

#[test]
fn test_dns_wizard_multiple_instances() {
    let wizard1 = DnsWizard::new("profile1".to_string());
    let wizard2 = DnsWizard::new("profile2".to_string());

    // Each wizard should have independent state
    assert!(matches!(wizard1.state(), DnsWizardState::CloudflareAuth));
    assert!(matches!(wizard2.state(), DnsWizardState::CloudflareAuth));
}

#[test]
fn test_dns_wizard_state_clone() {
    let wizard = DnsWizard::new("test".to_string());
    let state = wizard.state().clone();
    assert_eq!(state, DnsWizardState::CloudflareAuth);
}

#[test]
fn test_dns_wizard_empty_vm_list() {
    let mut wizard = DnsWizard::new("test".to_string());
    wizard.load_available_vms(vec![]);
    // Should not panic with empty VM list
}

#[test]
fn test_dns_wizard_state_machine() {
    let wizard = DnsWizard::new("test".to_string());

    // Verify state machine starts at CloudflareAuth
    assert!(matches!(wizard.state(), DnsWizardState::CloudflareAuth));

    // State progression would be:
    // CloudflareAuth -> DomainRegistration -> VmConfiguration -> Confirmation -> Complete
    // This is enforced by the UI logic (can_go_next/can_go_back)
}
