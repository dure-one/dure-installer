//! Configuration migration from V1 (with platform.name) to V2 (project_id-based)

use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::config::{AppConfig, CloudPlatformConfig, VmInstance};

/// Legacy CloudPlatformConfig with 'name' field (V1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudPlatformConfigV1 {
    pub name: String,
    pub platform_type: String,

    // GCP specific
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gcp_oauth_access_token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gcp_oauth_refresh_token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gcp_oauth_token_expiry: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gcp_connected_email: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gcp_selected_project_id: Option<String>,

    // Firebase specific
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub firebase_project_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub firebase_api_key: Option<String>,

    // Supabase specific
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supabase_project_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supabase_api_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supabase_anon_key: Option<String>,

    // Common fields
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_account_json: Option<String>,

    // VM instances (for GCP)
    #[serde(default)]
    pub vms: Vec<VmInstance>,
}

/// Legacy AppConfig with V1 platforms
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfigV1 {
    #[serde(default)]
    pub platforms: Vec<CloudPlatformConfigV1>,
}

/// Migrate V1 platform to V2
fn migrate_platform_v1_to_v2(v1: CloudPlatformConfigV1) -> Option<CloudPlatformConfig> {
    // Only migrate platforms with valid gcp_selected_project_id
    if v1.platform_type == "gcp" && v1.gcp_selected_project_id.is_none() {
        eprintln!("⚠ Skipping platform '{}': no gcp_selected_project_id", v1.name);
        return None;
    }

    Some(CloudPlatformConfig {
        platform_type: v1.platform_type,
        gcp_oauth_access_token: v1.gcp_oauth_access_token,
        gcp_oauth_refresh_token: v1.gcp_oauth_refresh_token,
        gcp_oauth_token_expiry: v1.gcp_oauth_token_expiry,
        gcp_connected_email: v1.gcp_connected_email,
        gcp_selected_project_id: v1.gcp_selected_project_id,
        firebase_project_id: v1.firebase_project_id,
        firebase_api_key: v1.firebase_api_key,
        supabase_project_ref: v1.supabase_project_ref,
        supabase_api_url: v1.supabase_api_url,
        supabase_anon_key: v1.supabase_anon_key,
        api_token: v1.api_token,
        service_account_json: v1.service_account_json,
        vms: v1.vms,
        // New cache fields - start empty
        cached_total_project_count: None,
        cached_vm_status: None,
        cached_firewall_status: None,
        cached_vm_external_ip: None,
        last_status_refresh: None,
    })
}

/// Migrate entire config from V1 to V2
pub fn migrate_config_v1_to_v2(v1: AppConfigV1) -> Result<AppConfig, String> {
    let mut migrated_platforms = Vec::new();
    let mut skipped_count = 0;

    for platform in v1.platforms {
        if let Some(migrated) = migrate_platform_v1_to_v2(platform) {
            migrated_platforms.push(migrated);
        } else {
            skipped_count += 1;
        }
    }

    if skipped_count > 0 {
        eprintln!("⚠ Migration: Skipped {} invalid platform(s)", skipped_count);
    }

    Ok(AppConfig {
        platforms: migrated_platforms,
        ..Default::default()
    })
}

/// Create backup of config file
pub fn backup_config(config_path: &Path) -> Result<(), String> {
    let backup_path = config_path.with_extension("yml.backup");
    std::fs::copy(config_path, &backup_path)
        .map_err(|e| format!("Failed to create backup: {}", e))?;
    eprintln!("✓ Created backup: {}", backup_path.display());
    Ok(())
}

/// Restore config from backup
pub fn restore_from_backup(config_path: &Path) -> Result<(), String> {
    let backup_path = config_path.with_extension("yml.backup");
    std::fs::copy(&backup_path, config_path)
        .map_err(|e| format!("Failed to restore backup: {}", e))?;
    eprintln!("✓ Restored from backup: {}", backup_path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrate_v1_to_v2() {
        let v1_platform = CloudPlatformConfigV1 {
            name: "test-platform".to_string(),
            platform_type: "gcp".to_string(),
            gcp_selected_project_id: Some("my-gcp-project".to_string()),
            gcp_connected_email: Some("test@example.com".to_string()),
            gcp_oauth_access_token: None,
            gcp_oauth_refresh_token: None,
            gcp_oauth_token_expiry: None,
            firebase_project_id: None,
            firebase_api_key: None,
            supabase_project_ref: None,
            supabase_api_url: None,
            supabase_anon_key: None,
            api_token: None,
            service_account_json: None,
            vms: vec![],
        };

        let v1_config = AppConfigV1 {
            platforms: vec![v1_platform],
        };

        let v2_config = migrate_config_v1_to_v2(v1_config).unwrap();

        assert_eq!(v2_config.platforms.len(), 1);
        assert_eq!(v2_config.platforms[0].platform_type, "gcp");
        assert_eq!(v2_config.platforms[0].gcp_selected_project_id, Some("my-gcp-project".to_string()));
        assert_eq!(v2_config.platforms[0].gcp_connected_email, Some("test@example.com".to_string()));
        assert_eq!(v2_config.platforms[0].cached_total_project_count, None);
    }

    #[test]
    fn test_skip_invalid_platform() {
        let v1_platform = CloudPlatformConfigV1 {
            name: "invalid-platform".to_string(),
            platform_type: "gcp".to_string(),
            gcp_selected_project_id: None, // Invalid - no project ID
            gcp_connected_email: None,
            gcp_oauth_access_token: None,
            gcp_oauth_refresh_token: None,
            gcp_oauth_token_expiry: None,
            firebase_project_id: None,
            firebase_api_key: None,
            supabase_project_ref: None,
            supabase_api_url: None,
            supabase_anon_key: None,
            api_token: None,
            service_account_json: None,
            vms: vec![],
        };

        let v1_config = AppConfigV1 {
            platforms: vec![v1_platform],
        };

        let v2_config = migrate_config_v1_to_v2(v1_config).unwrap();

        assert_eq!(v2_config.platforms.len(), 0); // Should skip invalid platform
    }
}
