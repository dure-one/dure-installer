#![cfg(test)]

use crate::config::CloudPlatformConfig;
use crate::viewmodel::platform::{PlatformCommand, PlatformEvent};
use anyhow::{Result, anyhow};
use std::collections::VecDeque;

/// Mock platform with full OAuth and VM
pub fn mock_platform_connected() -> CloudPlatformConfig {
    CloudPlatformConfig {
        platform_type: "gcp".to_string(),
        gcp_oauth_access_token: Some("mock_token".to_string()),
        gcp_oauth_refresh_token: Some("mock_refresh".to_string()),
        gcp_oauth_token_expiry: Some((chrono::Utc::now() + chrono::Duration::hours(1)).timestamp()),
        gcp_connected_email: Some("test@example.com".to_string()),
        gcp_selected_project_id: Some("test-project-123".to_string()),
        vms: vec![crate::config::VmInstance {
            name: "test-vm".to_string(),
            instance_id: "test-vm-id".to_string(),
            zone: "us-central1-a".to_string(),
            gcp_region: "us-central1".to_string(),
            machine_type: "e2-micro".to_string(),
            status: "RUNNING".to_string(),
            external_ip: Some("203.0.113.42".to_string()),
            internal_ip: Some("10.0.0.1".to_string()),
            gcp_project_id: "test-project-123".to_string(),
            gcp_billing_account: None,
            created_at: chrono::Utc::now().timestamp(),
            ssh_key_name: None,
        }],
        ..Default::default()
    }
}

/// Mock platform with OAuth but no VMs
pub fn mock_platform_no_vm() -> CloudPlatformConfig {
    let mut platform = mock_platform_connected();
    platform.vms.clear();
    platform
}

/// Mock platform with no OAuth connection
pub fn mock_platform_disconnected() -> CloudPlatformConfig {
    CloudPlatformConfig {
        platform_type: "gcp".to_string(),
        gcp_oauth_access_token: None,
        ..Default::default()
    }
}

/// Mock ViewModel runner for testing
pub struct MockPlatformRunner {
    pub responses: VecDeque<PlatformEvent>,
}

impl MockPlatformRunner {
    pub fn new() -> Self {
        Self {
            responses: VecDeque::new(),
        }
    }

    pub fn expect_response(&mut self, response: PlatformEvent) {
        self.responses.push_back(response);
    }

    pub async fn execute_command(&mut self, _cmd: PlatformCommand) -> Result<PlatformEvent> {
        let event = self.responses
            .pop_front()
            .ok_or_else(|| anyhow!("No more mock responses"))?;

        // Convert Error events to Err results
        if let PlatformEvent::Error { operation, error } = &event {
            return Err(anyhow!("{}: {}", operation, error));
        }

        Ok(event)
    }
}

#[cfg(test)]
mod helpers_tests {
    use super::*;
    use crate::cli::commands::platform::helpers::*;

    #[test]
    fn test_format_steps_all_complete() {
        let platform = mock_platform_connected();
        let steps = format_steps(&platform);

        assert!(steps.contains("✓"));
        assert!(steps.contains("→"));
        assert!(steps.contains("GCP Connected"));
        assert!(steps.contains("Project Created"));
        assert!(steps.contains("VM Created"));
    }

    #[test]
    fn test_format_steps_no_vm() {
        let platform = mock_platform_no_vm();
        let steps = format_steps(&platform);

        assert!(steps.contains("✓ GCP Connected"));
        assert!(steps.contains("✓ Project Created"));
        assert!(steps.contains("✗ VM Created"));
    }

    #[test]
    fn test_format_steps_disconnected() {
        let platform = mock_platform_disconnected();
        let steps = format_steps(&platform);

        assert!(steps.contains("✗ GCP Connected"));
        assert!(steps.contains("✗ Project Created"));
    }

    #[test]
    fn test_format_drawer_content_connected() {
        let platform = mock_platform_connected();
        let content = format_drawer_content(&platform);

        assert!(content.contains("test@example.com"));
        assert!(content.contains("test-project-123"));
        assert!(content.contains("test-vm"));
        assert!(content.contains("203.0.113.42"));
    }

    #[test]
    fn test_format_drawer_content_no_vm() {
        let platform = mock_platform_no_vm();
        let content = format_drawer_content(&platform);

        assert!(content.contains("test@example.com"));
        assert!(content.contains("test-project-123"));
        assert!(content.contains("No VM created"));
    }

    #[test]
    fn test_validate_platform_not_connected() {
        let platform = mock_platform_disconnected();
        let result = validate_platform_ready(&platform, "addvm");

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not connected"));
        assert!(err.contains("dure platform init"));
    }

    #[test]
    fn test_validate_platform_no_project() {
        let mut platform = mock_platform_connected();
        platform.gcp_selected_project_id = None;

        let result = validate_platform_ready(&platform, "addvm");

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("No project selected"));
    }

    #[test]
    fn test_validate_platform_ready_success() {
        let platform = mock_platform_connected();
        let result = validate_platform_ready(&platform, "addvm");

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_platform_list_no_validation() {
        let platform = mock_platform_disconnected();
        let result = validate_platform_ready(&platform, "list");

        // List command doesn't require connection
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod runner_tests {
    use super::*;
    use crate::cli::commands::platform::runner::*;

    #[test]
    fn test_runner_creation() {
        let runner = PlatformCliRunner::new();
        // Just verify it compiles and creates successfully
        drop(runner);
    }

    #[test]
    fn test_execute_command_success() {
        smol::block_on(async {
            let mut runner = MockPlatformRunner::new();
            runner.expect_response(PlatformEvent::FirewallUpdated {
                platform_name: "test-gcp".to_string(),
                whitelisted_ip: "203.0.113.42".to_string(),
            });

            let result = runner
                .execute_command(PlatformCommand::UpdateFirewall {
                    platform_name: "test-gcp".to_string(),
                    allow_ip: "203.0.113.42".to_string(),
                })
                .await;

            assert!(result.is_ok());
            if let Ok(PlatformEvent::FirewallUpdated { whitelisted_ip, .. }) = result {
                assert_eq!(whitelisted_ip, "203.0.113.42");
            } else {
                panic!("Expected FirewallUpdated event");
            }
        })
    }

    #[test]
    fn test_execute_command_error() {
        smol::block_on(async {
            let mut runner = MockPlatformRunner::new();
            runner.expect_response(PlatformEvent::Error {
                operation: "update_firewall".to_string(),
                error: "Permission denied".to_string(),
            });

            let result = runner
                .execute_command(PlatformCommand::UpdateFirewall {
                    platform_name: "test-gcp".to_string(),
                    allow_ip: "203.0.113.42".to_string(),
                })
                .await;

            assert!(result.is_err());
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("Permission denied")
            );
        })
    }
}

#[cfg(test)]
mod list_tests {
    use super::*;
    use crate::cli::commands::platform::list::*;

    #[test]
    fn test_list_empty() {
        let config = crate::config::AppConfig {
            platforms: vec![],
            ..Default::default()
        };

        let result = format_platform_list(&config);
        assert!(result.contains("No platforms configured"));
    }

    #[test]
    fn test_list_with_platforms() {
        let config = crate::config::AppConfig {
            platforms: vec![mock_platform_connected(), mock_platform_no_vm()],
            ..Default::default()
        };

        let result = format_platform_list(&config);
        assert!(result.contains("test-gcp"));
        assert!(result.contains("GCP"));
        assert!(result.contains("✓"));
        assert!(result.contains("→"));
    }

    #[test]
    fn test_show_platform_not_found() {
        let config = crate::config::AppConfig {
            platforms: vec![mock_platform_connected()],
            ..Default::default()
        };

        let result = format_platform_show(&config, "nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_show_platform_found() {
        let config = crate::config::AppConfig {
            platforms: vec![mock_platform_connected()],
            ..Default::default()
        };

        let result = format_platform_show(&config, "test-gcp");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("test-gcp"));
        assert!(output.contains("test@example.com"));
        assert!(output.contains("Available Actions"));
    }
}

#[cfg(test)]
mod firewall_tests {
    use super::*;

    #[test]
    fn test_firewall_with_explicit_ip() {
        smol::block_on(async {
            let mut runner = MockPlatformRunner::new();
            runner.expect_response(PlatformEvent::FirewallUpdated {
                platform_name: "test-gcp".to_string(),
                whitelisted_ip: "203.0.113.42".to_string(),
            });

            let platform = mock_platform_connected();
            let result = crate::cli::commands::platform::firewall::execute_firewall_inner(
                &mut runner,
                &platform,
                Some("203.0.113.42".to_string()),
            )
            .await;

            assert!(result.is_ok());
        })
    }

    #[test]
    fn test_firewall_validation_not_connected() {
        let platform = mock_platform_disconnected();
        let result =
            crate::cli::commands::platform::helpers::validate_platform_ready(&platform, "firewall");

        assert!(result.is_err());
    }
}

#[cfg(test)]
mod vm_tests {
    use super::*;

    #[test]
    fn test_addvm_success() {
        smol::block_on(async {
            let mut runner = MockPlatformRunner::new();
            runner.expect_response(PlatformEvent::VMCreated {
                platform_name: "test-gcp".to_string(),
                vm_name: "test-vm".to_string(),
                external_ip: "203.0.113.50".to_string(),
            });

            let platform = mock_platform_no_vm();
            let result = crate::cli::commands::platform::vm::execute_addvm_inner(
                &mut runner,
                &platform,
                "test-vm".to_string(),
                "us-central1-a".to_string(),
                "e2-micro".to_string(),
            )
            .await;

            assert!(result.is_ok());
        })
    }

    #[test]
    fn test_select_vm_single() {
        let platform = mock_platform_connected();
        let result = crate::cli::commands::platform::vm::select_vm(&platform, None);

        assert!(result.is_ok());
        let (name, zone) = result.unwrap();
        assert_eq!(name, "test-vm");
        assert_eq!(zone, "us-central1-a");
    }

    #[test]
    fn test_select_vm_none() {
        let platform = mock_platform_no_vm();
        let result = crate::cli::commands::platform::vm::select_vm(&platform, None);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No VMs found"));
    }
}

#[cfg(test)]
mod billing_tests {
    use super::*;
    use crate::api::gcp::bigquery::BillingRecord;

    #[test]
    fn test_billing_success() {
        smol::block_on(async {
            let mut runner = MockPlatformRunner::new();
            runner.expect_response(PlatformEvent::BillingFetched {
                platform_name: "test-gcp".to_string(),
                records: vec![
                    BillingRecord {
                        month: "2026-07".to_string(),
                        currency: "USD".to_string(),
                        total_net_cost: 12.45,
                    },
                    BillingRecord {
                        month: "2026-06".to_string(),
                        currency: "USD".to_string(),
                        total_net_cost: 11.89,
                    },
                    BillingRecord {
                        month: "2026-05".to_string(),
                        currency: "USD".to_string(),
                        total_net_cost: 13.20,
                    },
                ],
            });

            let platform = mock_platform_connected();
            let result =
                crate::cli::commands::platform::billing::execute_billing_inner(&mut runner, &platform)
                    .await;

            assert!(result.is_ok());
        })
    }
}
