#![cfg(test)]

use crate::config::CloudPlatformConfig;
use crate::viewmodel::platform::{PlatformCommand, PlatformEvent};
use anyhow::{anyhow, Result};
use std::collections::VecDeque;

/// Mock platform with full OAuth and VM
pub fn mock_platform_connected() -> CloudPlatformConfig {
    CloudPlatformConfig {
        name: "test-gcp".to_string(),
        platform_type: "gcp".to_string(),
        gcp_oauth_access_token: Some("mock_token".to_string()),
        gcp_oauth_refresh_token: Some("mock_refresh".to_string()),
        gcp_oauth_token_expiry: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
        gcp_connected_email: Some("test@example.com".to_string()),
        gcp_selected_project_id: Some("test-project-123".to_string()),
        vms: vec![
            crate::config::VmConfig {
                name: "test-vm".to_string(),
                zone: "us-central1-a".to_string(),
                machine_type: "e2-micro".to_string(),
                external_ip: Some("203.0.113.42".to_string()),
                ..Default::default()
            }
        ],
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
        name: "test-gcp".to_string(),
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
        self.responses.pop_front()
            .ok_or_else(|| anyhow!("No more mock responses"))
    }
}
