//! GCP-specific platform operations
//!
//! Handles platform-level GCP operations including status computation,
//! project selection, and OAuth management.

use crate::config::CloudPlatformConfig;
use anyhow::Result;

/// Compute platform-level status string
pub fn compute_platform_status(platform: &CloudPlatformConfig, total_projects: usize) -> String {
    format!("{} Projects", total_projects)
}

/// Compute project-level status string
pub fn compute_project_status(
    platform: &CloudPlatformConfig,
    current_ip: &str,
    whitelisted: bool,
) -> String {
    let vm_count = platform.vms.len();
    let firewall_status = if whitelisted {
        format!("✓ GCP Firewall Whitelisted({})", current_ip)
    } else {
        "✗ GCP Firewall Not Whitelisted".to_string()
    };

    format!("{} VM\n{}", vm_count, firewall_status)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_platform_status() {
        let platform = CloudPlatformConfig {
            name: "test".to_string(),
            platform_type: "gcp".to_string(),
            gcp_selected_project_id: Some("dure".to_string()),
            vms: vec![],
            ..Default::default()
        };

        let status = compute_platform_status(&platform, 8);
        assert_eq!(status, "8 Projects");
    }

    #[test]
    fn test_compute_project_status_whitelisted() {
        let platform = CloudPlatformConfig {
            name: "test".to_string(),
            platform_type: "gcp".to_string(),
            gcp_selected_project_id: Some("dure".to_string()),
            vms: vec![],
            ..Default::default()
        };

        let status = compute_project_status(&platform, "117.53.222.116", true);
        assert!(status.contains("0 VM"));
        assert!(status.contains("✓ GCP Firewall Whitelisted(117.53.222.116)"));
    }

    #[test]
    fn test_compute_project_status_not_whitelisted() {
        let platform = CloudPlatformConfig {
            name: "test".to_string(),
            platform_type: "gcp".to_string(),
            gcp_selected_project_id: Some("dure".to_string()),
            vms: vec![],
            ..Default::default()
        };

        let status = compute_project_status(&platform, "192.168.1.1", false);
        assert!(status.contains("0 VM"));
        assert!(status.contains("✗ GCP Firewall Not Whitelisted"));
    }
}
