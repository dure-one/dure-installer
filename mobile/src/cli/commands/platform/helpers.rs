//! Helper functions for formatting and validation

use crate::config::CloudPlatformConfig;
use anyhow::{anyhow, Result};

/// Format connection progress steps with status indicators
pub fn format_steps(platform: &CloudPlatformConfig) -> String {
    let gcp = if platform.gcp_oauth_access_token.is_some() { "✓" } else { "✗" };
    let proj = if platform.gcp_selected_project_id.is_some() { "✓" } else { "✗" };
    let vm = if !platform.vms.is_empty() { "✓" } else { "✗" };

    // Firewall check simplified for now (would need GCP API call)
    let firewall = "?";

    // SSH ready if VM has external IP
    let ssh = if platform.vms.first().and_then(|v| v.external_ip.as_ref()).is_some() {
        "✓"
    } else {
        "✗"
    };

    format!(
        "{} GCP Connected → {} Project Created → {} VM Created → {} Firewall Rules Updated → {} SSH Connected",
        gcp, proj, vm, firewall, ssh
    )
}

/// Format drawer content showing platform hierarchy
pub fn format_drawer_content(platform: &CloudPlatformConfig) -> String {
    let mut output = String::new();

    // Level 1: Email
    if let Some(email) = &platform.gcp_connected_email {
        output.push_str(&format!("{}\n", email));
    } else {
        output.push_str("Not connected\n");
    }

    // Level 2: Selected project
    if let Some(project_id) = &platform.gcp_selected_project_id {
        output.push_str(&format!("  └─ Project: {} (selected)\n", project_id));

        // Level 3: VM details
        if let Some(vm) = platform.vms.first() {
            let vm_display = if let Some(external_ip) = &vm.external_ip {
                format!("     └─ VM: {} ({})\n", vm.name, external_ip)
            } else {
                format!("     └─ VM: {} (no external IP)\n", vm.name)
            };
            output.push_str(&vm_display);
        } else {
            output.push_str("     └─ No VM created\n");
        }
    } else {
        output.push_str("  └─ No project selected\n");
    }

    output
}

/// Validate platform is ready for the requested operation
pub fn validate_platform_ready(
    platform: &CloudPlatformConfig,
    operation: &str,
) -> Result<()> {
    // List/show commands don't require validation
    if ["list", "show", "delete"].contains(&operation) {
        return Ok(());
    }

    // Check OAuth
    if platform.gcp_oauth_access_token.is_none() {
        return Err(anyhow!(
            "Platform '{}' is not connected\n\
             Run 'dure platform init {}' to authenticate",
            platform.name, platform.name
        ));
    }

    // Check token expiry
    if let Some(expiry) = platform.gcp_oauth_token_expiry {
        if expiry < chrono::Utc::now().timestamp() {
            return Err(anyhow!(
                "OAuth token expired\n\
                 Run 'dure platform init {}' to reconnect",
                platform.name
            ));
        }
    }

    // Check project for VM/firewall/billing operations
    let project_required = ["addvm", "firewall", "restart", "delvm", "billing"];
    if project_required.contains(&operation) {
        if platform.gcp_selected_project_id.is_none() {
            return Err(anyhow!(
                "No project selected for platform '{}'\n\
                 Run 'dure platform init {}' to select a project",
                platform.name, platform.name
            ));
        }
    }

    Ok(())
}
