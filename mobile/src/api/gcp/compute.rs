//! GCP Compute Engine API module
//!
//! Handles VM instances, firewall rules, and zone/region operations.

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use urlencoding;

use super::{GCP_COMPUTE_API_BASE, GcpRestClient};

// ============================================================================
// Instance Types
// ============================================================================

/// Instance creation request
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceRequest {
    pub name: String,
    pub machine_type: String, // e.g., "zones/us-central1-a/machineTypes/e2-micro"
    pub disks: Vec<AttachedDisk>,
    pub network_interfaces: Vec<NetworkInterface>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Tags>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachedDisk {
    pub boot: bool,
    pub auto_delete: bool,
    pub initialize_params: InitializeParams,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeParams {
    pub source_image: String, // e.g., "projects/debian-cloud/global/images/debian-11-bullseye-v20240219"
    pub disk_size_gb: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkInterface {
    pub network: String, // e.g., "global/networks/default"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_configs: Option<Vec<AccessConfig>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessConfig {
    #[serde(rename = "type")]
    pub type_: String, // "ONE_TO_ONE_NAT"
    pub name: String, // "External NAT"
}

#[derive(Debug, Serialize)]
pub struct Tags {
    pub items: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct Metadata {
    pub items: Vec<MetadataItem>,
}

#[derive(Debug, Serialize)]
pub struct MetadataItem {
    pub key: String,
    pub value: String,
}

/// Instance response
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Instance {
    pub id: String,
    pub name: String,
    pub machine_type: String,
    pub zone: String,
    pub status: String,
    #[serde(default)]
    pub network_interfaces: Vec<NetworkInterfaceResponse>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkInterfaceResponse {
    #[serde(rename = "networkIP", default)]
    pub network_ip: Option<String>,
    #[serde(default)]
    pub access_configs: Vec<AccessConfigResponse>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessConfigResponse {
    #[serde(rename = "natIP")]
    pub nat_ip: Option<String>,
}

/// Instance list response
#[derive(Debug, Deserialize)]
pub struct InstanceList {
    #[serde(default)]
    pub items: Vec<Instance>,
}

// ============================================================================
// Firewall Types
// ============================================================================

/// Firewall rule request
#[derive(Debug, Serialize)]
pub struct FirewallRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "direction")]
    pub direction: String, // "INGRESS" or "EGRESS"
    pub priority: u32,
    #[serde(rename = "targetTags")]
    pub target_tags: Vec<String>,
    pub allowed: Vec<FirewallAllowed>,
    #[serde(rename = "sourceRanges")]
    pub source_ranges: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallAllowed {
    #[serde(rename = "IPProtocol")]
    pub ip_protocol: String, // "tcp", "udp", "icmp", "all"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ports: Option<Vec<String>>,
}

/// GCP Firewall Rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallRule {
    pub name: String,
    pub allowed: Vec<FirewallAllowed>,
    #[serde(rename = "sourceRanges")]
    pub source_ranges: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct FirewallListResponse {
    items: Option<Vec<FirewallRule>>,
}

/// Firewall list response
#[derive(Debug, Deserialize)]
pub struct ListFirewallsResponse {
    pub items: Option<Vec<Firewall>>,
}

#[derive(Debug, Deserialize)]
pub struct Firewall {
    pub name: String,
    #[serde(rename = "targetTags")]
    pub target_tags: Option<Vec<String>>,
}

// ============================================================================
// Operation Types
// ============================================================================

/// Operation response (for async operations)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Operation {
    #[serde(default)]
    pub id: Option<String>, // Only present in ComputeEngine operations
    pub name: String,
    #[serde(default)]
    pub status: Option<String>, // "PENDING", "RUNNING", "DONE" (ComputeEngine)
    #[serde(default)]
    pub done: Option<bool>, // ResourceManager uses this instead of status
    #[serde(default)]
    pub error: Option<OperationError>,
}

impl Operation {
    /// Returns true if the operation is complete
    /// ResourceManager operations use `done`, ComputeEngine uses `status == "DONE"`
    pub fn is_done(&self) -> bool {
        self.done.unwrap_or(false) || self.status.as_deref() == Some("DONE")
    }

    /// Returns true if the operation has an error
    pub fn has_error(&self) -> bool {
        self.error.is_some()
    }

    /// Returns a status string for display
    pub fn status_string(&self) -> String {
        if let Some(status) = &self.status {
            status.clone()
        } else if let Some(done) = self.done {
            if done {
                "DONE".to_string()
            } else {
                "PENDING".to_string()
            }
        } else {
            "UNKNOWN".to_string()
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct OperationError {
    pub errors: Vec<ErrorDetail>,
}

#[derive(Debug, Deserialize)]
pub struct ErrorDetail {
    #[serde(default)]
    pub code: Option<String>,
    pub message: String,
}

// ============================================================================
// Region/Zone Types
// ============================================================================

/// Region list response
#[derive(Debug, Deserialize)]
pub struct RegionList {
    #[serde(default)]
    pub items: Vec<Region>,
}

#[derive(Debug, Deserialize)]
pub struct Region {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub zones: Vec<String>,
}

/// Zone list response
#[derive(Debug, Deserialize)]
pub struct ZoneList {
    #[serde(default)]
    pub items: Vec<Zone>,
}

#[derive(Debug, Deserialize)]
pub struct Zone {
    pub name: String,
    pub description: String,
    pub region: String,
}

// ============================================================================
// Image Types
// ============================================================================

/// Image list response from GCP API
#[derive(Debug, Deserialize, Default)]
pub struct ImageList {
    #[serde(default)]
    pub items: Vec<Image>,
}

/// GCP Compute Engine image metadata
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    pub name: String,
    pub description: Option<String>,
    pub self_link: String,
    pub creation_timestamp: String,
    pub architecture: Option<String>,
    pub family: Option<String>,
    pub deprecated: Option<DeprecatedStatus>,
}

impl Default for Image {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: None,
            self_link: String::new(),
            creation_timestamp: String::new(),
            architecture: None,
            family: None,
            deprecated: None,
        }
    }
}

impl Image {
    /// Check if image is deprecated
    pub fn is_deprecated(&self) -> bool {
        self.deprecated
            .as_ref()
            .and_then(|d| d.state.as_ref())
            .map(|s| !s.is_empty())
            .unwrap_or(false)
    }

    /// Check if image was created within last 6 months
    pub fn is_recent(&self) -> bool {
        if let Ok(created) = chrono::DateTime::parse_from_rfc3339(&self.creation_timestamp) {
            let six_months_ago = chrono::Utc::now() - chrono::Duration::days(180);
            created.with_timezone(&chrono::Utc) > six_months_ago
        } else {
            false
        }
    }

    /// Get human-readable family name for UI grouping
    pub fn family_group(&self) -> String {
        match self.family.as_deref() {
            Some("debian-13") => "Debian 13".to_string(),
            Some("debian-12") => "Debian 12".to_string(),
            Some("ubuntu-2404-lts") => "Ubuntu 24.04 LTS".to_string(),
            Some("ubuntu-2204-lts") => "Ubuntu 22.04 LTS".to_string(),
            Some(other) => other.replace('-', " ").to_uppercase(),
            None => self.name.clone(),
        }
    }

    /// Get display name with creation date for UI
    pub fn display_name(&self) -> String {
        let date = self.creation_timestamp.split('T').next().unwrap_or("");
        format!("{} ({})", self.family_group(), date)
    }
}

/// Deprecation status
#[derive(Debug, Clone, Deserialize)]
pub struct DeprecatedStatus {
    pub state: Option<String>,
}

// ============================================================================
// Helper Implementations
// ============================================================================

impl InstanceRequest {
    /// Create a basic Debian instance
    pub fn debian_micro(name: String, zone: String) -> Self {
        Self {
            name,
            machine_type: format!("zones/{}/machineTypes/e2-micro", zone),
            disks: vec![AttachedDisk {
                boot: true,
                auto_delete: true,
                initialize_params: InitializeParams {
                    source_image: "projects/debian-cloud/global/images/family/debian-11"
                        .to_string(),
                    disk_size_gb: "10".to_string(),
                },
            }],
            network_interfaces: vec![NetworkInterface {
                network: "global/networks/default".to_string(),
                access_configs: Some(vec![AccessConfig {
                    type_: "ONE_TO_ONE_NAT".to_string(),
                    name: "External NAT".to_string(),
                }]),
            }],
            tags: Some(Tags {
                items: vec![
                    "dure".to_string(),         // Dure firewall rule
                    "http-server".to_string(),  // Allow HTTP
                    "https-server".to_string(), // Allow HTTPS
                ],
            }),
            metadata: None,
        }
    }
}

impl Instance {
    /// Get external IP address
    pub fn external_ip(&self) -> Option<String> {
        self.network_interfaces
            .first()?
            .access_configs
            .first()?
            .nat_ip
            .clone()
    }

    /// Get internal IP address
    pub fn internal_ip(&self) -> Option<String> {
        self.network_interfaces
            .first()
            .and_then(|ni| ni.network_ip.clone())
    }
}

// ============================================================================
// API Methods on GcpRestClient
// ============================================================================

impl GcpRestClient {
    /// Create VM instance
    ///
    /// API: POST /projects/{project}/zones/{zone}/instances
    pub fn create_instance(
        &self,
        project_id: &str,
        zone: &str,
        instance: &InstanceRequest,
    ) -> Result<Operation> {
        let url = format!(
            "{}/projects/{}/zones/{}/instances",
            GCP_COMPUTE_API_BASE, project_id, zone
        );

        let body = serde_json::to_string(instance)?;
        let response = self.post(&url, &body)?;

        if response.status() != 200 {
            let error_text = response.into_string().unwrap_or_default();

            // Detect Compute Engine API not enabled error
            if error_text.contains("Compute Engine API")
                && (error_text.contains("not been used") || error_text.contains("disabled"))
            {
                let activation_url = format!(
                    "https://console.developers.google.com/apis/api/compute.googleapis.com/overview?project={}",
                    project_id
                );

                return Err(anyhow::anyhow!(
                    "Compute Engine API is not enabled in project '{}'.\n\n\
                     To fix this (one-time setup):\n\
                     1. Open: {}\n\
                     2. Click 'Enable API'\n\
                     3. Wait a few minutes for changes to propagate\n\
                     4. Return here and click 'Create Server' again\n\n\
                     Note: This needs to be done once per GCP project.",
                    project_id,
                    activation_url
                ));
            }

            return Err(anyhow::anyhow!("Failed to create instance: {}", error_text));
        }

        let operation: Operation = response.into_json()?;
        Ok(operation)
    }

    /// List VM instances
    ///
    /// API: GET /projects/{project}/zones/{zone}/instances
    pub fn list_instances(&self, project_id: &str, zone: &str) -> Result<InstanceList> {
        let url = format!(
            "{}/projects/{}/zones/{}/instances",
            GCP_COMPUTE_API_BASE, project_id, zone
        );

        let response = self.get(&url)?;
        let list: InstanceList = response.into_json()?;
        Ok(list)
    }

    /// Get VM instance details
    ///
    /// API: GET /projects/{project}/zones/{zone}/instances/{instance}
    pub fn get_instance(
        &self,
        project_id: &str,
        zone: &str,
        instance_name: &str,
    ) -> Result<Instance> {
        let url = format!(
            "{}/projects/{}/zones/{}/instances/{}",
            GCP_COMPUTE_API_BASE, project_id, zone, instance_name
        );

        let response = self.get(&url)?;
        let instance: Instance = response.into_json()?;
        Ok(instance)
    }

    /// Delete VM instance
    ///
    /// API: DELETE /projects/{project}/zones/{zone}/instances/{instance}
    pub fn delete_instance(
        &self,
        project_id: &str,
        zone: &str,
        instance_name: &str,
    ) -> Result<Operation> {
        let url = format!(
            "{}/projects/{}/zones/{}/instances/{}",
            GCP_COMPUTE_API_BASE, project_id, zone, instance_name
        );

        let response = self.delete(&url)?;
        let operation: Operation = response.into_json()?;
        Ok(operation)
    }

    /// Reset (hard reboot) VM instance
    ///
    /// API: POST /projects/{project}/zones/{zone}/instances/{instance}/reset
    pub fn reset_instance(
        &self,
        project_id: &str,
        zone: &str,
        instance_name: &str,
    ) -> Result<Operation> {
        let url = format!(
            "{}/projects/{}/zones/{}/instances/{}/reset",
            GCP_COMPUTE_API_BASE, project_id, zone, instance_name
        );

        let response = self.post(&url, "")?;
        let operation: Operation = response.into_json()?;
        Ok(operation)
    }

    /// Wait for operation to complete
    ///
    /// API: GET /projects/{project}/zones/{zone}/operations/{operation}
    pub fn wait_for_operation(
        &self,
        project_id: &str,
        zone: &str,
        operation_name: &str,
        timeout_secs: u64,
    ) -> Result<Operation> {
        let url = format!(
            "{}/projects/{}/zones/{}/operations/{}",
            GCP_COMPUTE_API_BASE, project_id, zone, operation_name
        );

        let start = std::time::Instant::now();

        loop {
            let response = self.get(&url)?;
            let operation: Operation = response.into_json()?;

            if operation.is_done() {
                return Ok(operation);
            }

            if start.elapsed().as_secs() > timeout_secs {
                return Err(anyhow::anyhow!("Operation timed out"));
            }

            // Poll every 2 seconds
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    }

    /// Wait for a global operation to complete (for firewall, network operations)
    ///
    /// API: GET /projects/{project}/global/operations/{operation}
    pub fn wait_for_global_operation(
        &self,
        project_id: &str,
        operation_name: &str,
    ) -> Result<Operation> {
        let timeout_secs = 120; // 2 minutes
        let url = format!(
            "{}/projects/{}/global/operations/{}",
            GCP_COMPUTE_API_BASE, project_id, operation_name
        );

        let start = std::time::Instant::now();

        loop {
            let response = self.get(&url)?;
            let operation: Operation = response.into_json()?;

            if operation.is_done() {
                return Ok(operation);
            }

            if start.elapsed().as_secs() > timeout_secs {
                return Err(anyhow::anyhow!("Operation timed out"));
            }

            // Poll every 2 seconds
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    }

    /// List available regions
    ///
    /// API: GET /projects/{project}/regions
    pub fn list_regions(&self, project_id: &str) -> Result<RegionList> {
        let url = format!("{}/projects/{}/regions", GCP_COMPUTE_API_BASE, project_id);

        let response = self.get(&url)?;
        let list: RegionList = response.into_json()?;
        Ok(list)
    }

    /// List available zones
    ///
    /// API: GET /projects/{project}/zones
    pub fn list_zones(&self, project_id: &str) -> Result<ZoneList> {
        let url = format!("{}/projects/{}/zones", GCP_COMPUTE_API_BASE, project_id);

        let response = self.get(&url)?;
        let list: ZoneList = response.into_json()?;
        Ok(list)
    }

    /// List firewalls with optional filter
    pub fn list_firewalls(
        &self,
        project_id: &str,
        filter_name: Option<&str>,
    ) -> Result<ListFirewallsResponse> {
        let mut url = format!(
            "{}/projects/{}/global/firewalls",
            GCP_COMPUTE_API_BASE, project_id
        );

        if let Some(name) = filter_name {
            url.push_str(&format!("?filter=name%3D{}", urlencoding::encode(name)));
        }

        let response = self.get(&url)?;
        Ok(response.into_json()?)
    }

    /// Create a firewall rule
    pub fn create_firewall(
        &self,
        project_id: &str,
        firewall_data: &FirewallRequest,
    ) -> Result<Operation> {
        let url = format!(
            "{}/projects/{}/global/firewalls",
            GCP_COMPUTE_API_BASE, project_id
        );

        let response = ureq::post(&url)
            .set("Authorization", &format!("Bearer {}", self.access_token))
            .set("Content-Type", "application/json")
            .send_json(firewall_data)?;

        let operation: Operation = response.into_json()?;

        // Wait for global operation to complete
        self.wait_for_global_operation(project_id, &operation.name)
    }

    /// List firewall rules for a project
    pub fn list_firewall_rules(&self, project_id: &str) -> Result<Vec<FirewallRule>> {
        let url = format!(
            "{}/projects/{}/global/firewalls",
            GCP_COMPUTE_API_BASE, project_id
        );

        let response = self.get(&url)?;
        let list_response: FirewallListResponse = response.into_json()?;

        Ok(list_response.items.unwrap_or_default())
    }

    /// Check if an IP is whitelisted for SSH (port 22) in firewall rules
    pub fn check_ip_whitelisted(&self, project_id: &str, ip: &str) -> Result<bool> {
        let rules = self.list_firewall_rules(project_id)?;

        for rule in rules {
            // Check if rule allows SSH (port 22)
            let allows_ssh = rule.allowed.iter().any(|a| {
                a.ip_protocol.to_lowercase() == "tcp"
                    && a.ports
                        .as_ref()
                        .map_or(false, |ports| ports.iter().any(|p| p == "22"))
            });

            if allows_ssh {
                if let Some(ranges) = &rule.source_ranges {
                    if super::ip_in_ranges(ip, ranges) {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    /// Add an IP to the SSH firewall whitelist
    pub fn add_ip_to_firewall(&self, project_id: &str, ip: &str) -> Result<()> {
        let rules = self.list_firewall_rules(project_id)?;

        // Find specifically the "allow-ssh-dure" rule
        let ssh_rule = rules.iter().find(|rule| rule.name == "allow-ssh-dure");

        if let Some(rule) = ssh_rule {
            // Update existing rule
            let mut updated_ranges = rule.source_ranges.clone().unwrap_or_default();
            let ip_cidr = format!("{}/32", ip);

            if !updated_ranges.contains(&ip_cidr) {
                updated_ranges.push(ip_cidr);

                let body = serde_json::json!({
                    "sourceRanges": updated_ranges,
                });

                let url = format!(
                    "{}/projects/{}/global/firewalls/{}",
                    GCP_COMPUTE_API_BASE, project_id, rule.name
                );

                dure_debug!(
                    "Updating firewall rule '{}' with IP: {}",
                    rule.name, ip
                );
                dure_debug!("PATCH URL: {}", url);
                dure_debug!("Body: {}", body.to_string());

                let response = self.patch(&url, &body.to_string())?;
                let response_text = response.into_string().unwrap_or_default();
                dure_debug!("Response: {}", response_text);
            } else {
                dure_debug!("IP {} already in firewall rule '{}'", ip, rule.name);
            }
        } else {
            // Create new SSH rule
            let body = serde_json::json!({
                "name": "allow-ssh-dure",
                "allowed": [{
                    "IPProtocol": "tcp",
                    "ports": ["22"]
                }],
                "sourceRanges": [format!("{}/32", ip)],
                "direction": "INGRESS",
                "network": "global/networks/default",
            });

            let url = format!(
                "{}/projects/{}/global/firewalls",
                GCP_COMPUTE_API_BASE, project_id
            );

            dure_debug!(
                "Creating new firewall rule 'allow-ssh-dure' with IP: {}",
                ip
            );
            dure_debug!("POST URL: {}", url);
            dure_debug!("Body: {}", body.to_string());

            let response = self.post(&url, &body.to_string())?;
            let response_text = response.into_string().unwrap_or_default();
            dure_debug!("Response: {}", response_text);
        }

        Ok(())
    }

    /// List images from a public image project (debian-cloud, ubuntu-os-cloud, etc.)
    ///
    /// API: GET /projects/{project}/global/images
    pub fn list_images(&self, image_project: &str) -> Result<ImageList> {
        // Use server-side filtering for architecture only
        // Note: deprecated.state filtering doesn't work reliably, so we filter client-side
        let filter = "architecture = \"X86_64\"";
        let url = format!(
            "{}/projects/{}/global/images?filter={}",
            GCP_COMPUTE_API_BASE,
            image_project,
            urlencoding::encode(filter)
        );

        let response = self.get(&url)?;
        let list: ImageList = response.into_json()?;
        Ok(list)
    }

    /// Get filtered list of recent Debian and Ubuntu images
    ///
    /// Server-side filters (via API query parameters):
    /// - Architecture: X86_64 only
    ///
    /// Client-side filters:
    /// - Age: Created within last 6 months (180 days)
    /// - Status: Not deprecated or obsolete
    pub fn list_debian_ubuntu_images(&self) -> Result<Vec<Image>> {
        let mut all_images = Vec::new();
        let mut errors = Vec::new();

        // Fetch Debian images
        match self.list_images("debian-cloud") {
            Ok(list) => {
                dure_info!("Fetched {} Debian images", list.items.len());
                all_images.extend(list.items);
            }
            Err(e) => {
                let err_msg = format!("Failed to fetch Debian images: {}", e);
                dure_warn!("{}", err_msg);
                errors.push(err_msg);
            }
        }

        // Fetch Ubuntu images
        match self.list_images("ubuntu-os-cloud") {
            Ok(list) => {
                dure_info!("Fetched {} Ubuntu images", list.items.len());
                all_images.extend(list.items);
            }
            Err(e) => {
                let err_msg = format!("Failed to fetch Ubuntu images: {}", e);
                dure_warn!("{}", err_msg);
                errors.push(err_msg);
            }
        }

        // If both failed, return error
        if all_images.is_empty() && !errors.is_empty() {
            return Err(anyhow::anyhow!("Failed to fetch images: {}", errors.join("; ")));
        }

        dure_info!("Total images before filtering: {}", all_images.len());

        // Sample first few images to debug architecture values
        if !all_images.is_empty() {
            let sample = &all_images[0];
            dure_info!(
                "Sample image: name={}, arch={:?}, deprecated={:?}, created={}",
                sample.name,
                sample.architecture,
                sample.deprecated,
                sample.creation_timestamp
            );
        }

        // Apply client-side filters (server-side only handles architecture)
        let mut stats = (0, 0); // (old_rejected, deprecated_rejected)
        let filtered: Vec<Image> = all_images
            .into_iter()
            .filter(|img| {
                let is_recent = img.is_recent();
                let not_deprecated = !img.is_deprecated();

                if !is_recent {
                    stats.0 += 1;
                }
                if !not_deprecated {
                    stats.1 += 1;
                }

                is_recent && not_deprecated
            })
            .collect();

        dure_info!("Images after filtering: {}", filtered.len());
        dure_info!(
            "Filter stats - Old rejected: {}, Deprecated/Obsolete rejected: {}",
            stats.0,
            stats.1
        );

        Ok(filtered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_default() {
        let img = Image::default();
        assert_eq!(img.name, "");
        assert_eq!(img.self_link, "");
        assert!(img.architecture.is_none());
        assert!(img.deprecated.is_none());
    }

    #[test]
    fn test_image_is_deprecated() {
        // Active image (no deprecated field)
        let active = Image {
            deprecated: None,
            ..Default::default()
        };
        assert!(!active.is_deprecated());

        // Deprecated image
        let deprecated = Image {
            deprecated: Some(DeprecatedStatus {
                state: Some("DEPRECATED".to_string()),
            }),
            ..Default::default()
        };
        assert!(deprecated.is_deprecated());

        // Edge case: empty state string
        let edge = Image {
            deprecated: Some(DeprecatedStatus {
                state: Some("".to_string()),
            }),
            ..Default::default()
        };
        assert!(!edge.is_deprecated());
    }

    #[test]
    fn test_image_is_recent() {
        use chrono::{Duration, Utc};

        // Recent image (3 months old)
        let recent = Image {
            creation_timestamp: (Utc::now() - Duration::days(90)).to_rfc3339(),
            ..Default::default()
        };
        assert!(recent.is_recent());

        // Old image (7 months old)
        let old = Image {
            creation_timestamp: (Utc::now() - Duration::days(210)).to_rfc3339(),
            ..Default::default()
        };
        assert!(!old.is_recent());

        // Invalid timestamp
        let invalid = Image {
            creation_timestamp: "not-a-date".to_string(),
            ..Default::default()
        };
        assert!(!invalid.is_recent());
    }

    #[test]
    fn test_image_family_group() {
        let debian = Image {
            family: Some("debian-13".to_string()),
            ..Default::default()
        };
        assert_eq!(debian.family_group(), "Debian 13");

        let ubuntu = Image {
            family: Some("ubuntu-2404-lts".to_string()),
            ..Default::default()
        };
        assert_eq!(ubuntu.family_group(), "Ubuntu 24.04 LTS");

        let unknown = Image {
            family: Some("custom-os-v1".to_string()),
            ..Default::default()
        };
        assert_eq!(unknown.family_group(), "CUSTOM OS V1");

        let no_family = Image::default();
        assert_eq!(no_family.family_group(), "");
    }

    #[test]
    fn test_image_display_name() {
        let img = Image {
            name: "debian-13-bookworm-v20260615".to_string(),
            family: Some("debian-13".to_string()),
            creation_timestamp: "2026-06-15T10:00:00.000Z".to_string(),
            ..Default::default()
        };
        assert_eq!(img.display_name(), "Debian 13 (2026-06-15)");

        let no_date = Image {
            family: Some("debian-13".to_string()),
            creation_timestamp: "".to_string(),
            ..Default::default()
        };
        assert_eq!(no_date.display_name(), "Debian 13 ()");
    }

    #[test]
    fn test_parse_image_list_response() {
        let json = r#"{
            "items": [
                {
                    "name": "debian-13-bookworm-v20260615",
                    "selfLink": "projects/debian-cloud/global/images/debian-13-bookworm-v20260615",
                    "creationTimestamp": "2026-06-15T10:00:00.000Z",
                    "architecture": "X86_64",
                    "family": "debian-13"
                }
            ]
        }"#;

        let list: ImageList = serde_json::from_str(json).unwrap();
        assert_eq!(list.items.len(), 1);
        assert_eq!(list.items[0].name, "debian-13-bookworm-v20260615");
        assert_eq!(list.items[0].architecture.as_deref(), Some("X86_64"));
    }

    #[test]
    fn test_parse_deprecated_image() {
        let json = r#"{
            "name": "old-image",
            "selfLink": "projects/debian-cloud/global/images/old-image",
            "creationTimestamp": "2020-01-01T00:00:00.000Z",
            "architecture": "X86_64",
            "deprecated": {
                "state": "DEPRECATED"
            }
        }"#;

        let img: Image = serde_json::from_str(json).unwrap();
        assert!(img.is_deprecated());
    }

    #[test]
    fn test_list_images_method_signature() {
        // This test verifies the method signature exists
        // We can't test actual API calls without mocking, so we just check compilation
        fn _check_signature(client: &GcpRestClient) {
            let _: Result<ImageList> = client.list_images("debian-cloud");
        }
    }

    #[test]
    fn test_image_architecture_filter() {
        let x86 = Image {
            architecture: Some("X86_64".to_string()),
            ..Default::default()
        };
        let arm = Image {
            architecture: Some("ARM64".to_string()),
            ..Default::default()
        };
        let none = Image {
            architecture: None,
            ..Default::default()
        };

        // Test the filter logic inline
        assert!(
            x86.architecture
                .as_ref()
                .map(|a| a.to_uppercase() == "X86_64")
                .unwrap_or(false)
        );

        assert!(
            !arm.architecture
                .as_ref()
                .map(|a| a.to_uppercase() == "X86_64")
                .unwrap_or(false)
        );

        assert!(
            !none
                .architecture
                .as_ref()
                .map(|a| a.to_uppercase() == "X86_64")
                .unwrap_or(false)
        );
    }

    #[test]
    fn test_list_debian_ubuntu_images_signature() {
        // Verify method signature exists
        fn _check_signature(client: &GcpRestClient) {
            let _: Result<Vec<Image>> = client.list_debian_ubuntu_images();
        }
    }
}
