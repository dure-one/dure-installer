//! Tests for api/gcp/billing module

use dure::api::gcp::billing::{BillingAccount, ProjectBillingInfo};

#[test]
fn test_billing_account_structure() {
    let account = BillingAccount {
        name: "billingAccounts/012345-ABCDEF-678901".to_string(),
        display_name: "My Billing Account".to_string(),
        open: true,
        master_billing_account: None,
    };

    assert_eq!(account.display_name, "My Billing Account");
    assert!(account.open);
    assert_eq!(account.id(), Some("012345-ABCDEF-678901"));
}

#[test]
fn test_project_billing_info_structure() {
    let info = ProjectBillingInfo {
        name: "projects/my-project/billingInfo".to_string(),
        project_id: "my-project".to_string(),
        billing_account_name: Some("billingAccounts/012345-ABCDEF-678901".to_string()),
        billing_enabled: true,
    };

    assert_eq!(info.project_id, "my-project");
    assert!(info.billing_enabled);
}
