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
