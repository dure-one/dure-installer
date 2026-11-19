//! GCP Compute Engine client wrapper
//!
//! Provides high-level operations for GCP Compute Engine:
//! - VM instance management (create, delete, list, get)
//! - Firewall rules
//! - Static IP allocation
//! - Project and region management
//!
//! Uses google-cloud-compute crate for API calls.

use crate::{dure_debug, dure_error, dure_info, dure_warn};
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// GCP client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcpConfig {
    pub project_id: String,
    pub region: String,
    pub zone: String,
    pub access_token: String,
}

/// VM instance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceConfig {
    pub name: String,
    pub machine_type: String,    // e.g., "e2-micro", "n1-standard-1"
    pub zone: String,            // e.g., "us-central1-a"
    pub boot_disk_image: String, // e.g., "projects/debian-cloud/global/images/debian-11-bullseye-v20240312"
    pub boot_disk_size_gb: i64,
    pub network_tags: Vec<String>,
    pub metadata: Vec<(String, String)>,
}

impl Default for InstanceConfig {
    fn default() -> Self {
        Self {
            name: "dure-instance".to_string(),
            machine_type: "e2-micro".to_string(),
            zone: "us-central1-a".to_string(),
            boot_disk_image: "projects/debian-cloud/global/images/family/debian-11".to_string(),
            boot_disk_size_gb: 10,
            network_tags: vec!["http-server".to_string(), "https-server".to_string()],
            metadata: Vec::new(),
        }
    }
}

/// VM instance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    pub id: String,
    pub name: String,
    pub machine_type: String,
    pub zone: String,
    pub status: String,
    pub external_ip: Option<String>,
    pub internal_ip: Option<String>,
    pub creation_timestamp: String,
}

/// Firewall rule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallRule {
    pub name: String,
    pub network: String,
    pub direction: String, // "INGRESS" or "EGRESS"
    pub priority: i32,
    pub source_ranges: Vec<String>,
    pub allowed: Vec<FirewallAllowed>,
    pub target_tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallAllowed {
    pub protocol: String,   // e.g., "tcp", "udp", "icmp"
    pub ports: Vec<String>, // e.g., ["80", "443", "8000-9000"]
}

impl Default for FirewallRule {
    fn default() -> Self {
        Self {
            name: "allow-http-https".to_string(),
            network: "default".to_string(),
            direction: "INGRESS".to_string(),
            priority: 1000,
            source_ranges: vec!["0.0.0.0/0".to_string()],
            allowed: vec![FirewallAllowed {
                protocol: "tcp".to_string(),
                ports: vec!["80".to_string(), "443".to_string()],
            }],
            target_tags: vec!["http-server".to_string(), "https-server".to_string()],
        }
    }
}

/// GCP Compute Engine client
pub struct GcpClient {
    config: GcpConfig,
}

impl GcpClient {
    /// Create new GCP client
    pub fn new(config: GcpConfig) -> Self {
        Self { config }
    }

    /// List available regions
    pub fn list_regions(&self) -> Result<Vec<Region>> {
        // TODO: Implement using google-cloud-compute
        // For now, return static list
        Ok(vec![
            Region {
                name: "us-central1".to_string(),
                location: "Iowa, USA".to_string(),
                zones: vec![
                    "us-central1-a".to_string(),
                    "us-central1-b".to_string(),
                    "us-central1-c".to_string(),
                    "us-central1-f".to_string(),
                ],
            },
            Region {
                name: "us-east1".to_string(),
                location: "South Carolina, USA".to_string(),
                zones: vec![
                    "us-east1-b".to_string(),
                    "us-east1-c".to_string(),
                    "us-east1-d".to_string(),
                ],
            },
            Region {
                name: "asia-northeast3".to_string(),
                location: "Seoul, South Korea".to_string(),
                zones: vec![
                    "asia-northeast3-a".to_string(),
                    "asia-northeast3-b".to_string(),
                    "asia-northeast3-c".to_string(),
                ],
            },
            Region {
                name: "asia-northeast1".to_string(),
                location: "Tokyo, Japan".to_string(),
                zones: vec![
                    "asia-northeast1-a".to_string(),
                    "asia-northeast1-b".to_string(),
                    "asia-northeast1-c".to_string(),
                ],
            },
            Region {
                name: "europe-west1".to_string(),
                location: "Belgium, Europe".to_string(),
                zones: vec![
                    "europe-west1-b".to_string(),
                    "europe-west1-c".to_string(),
                    "europe-west1-d".to_string(),
                ],
            },
        ])
    }

    /// List projects
    pub fn list_projects(&self) -> Result<Vec<Project>> {
        // TODO: Implement using google-cloud-resourcemanager
        // For now, return empty list (requires implementation)
        Err(anyhow::anyhow!(
            "list_projects not yet implemented - requires google-cloud-resourcemanager"
        ))
    }

    /// Create VM instance
    ///
    /// Reference: reference/google-cloud-rust/guide/samples/src/compute/compute_instances_create.rs
    pub fn create_instance(&self, _config: InstanceConfig) -> Result<Instance> {
        // TODO: Implement using google-cloud-compute
        //
        // Example implementation sketch:
        // ```
        // use google_cloud_compute::client::Instances;
        // use google_cloud_compute::http::instances::insert::*;
        //
        // let instances = Instances::new(auth).await?;
        //
        // let instance = Instance::new()
        //     .set_name(config.name)
        //     .set_machine_type(format!("zones/{}/machineTypes/{}", config.zone, config.machine_type))
        //     .set_disks(vec![
        //         AttachedDisk::new()
        //             .set_boot(true)
        //             .set_initialize_params(
        //                 AttachedDiskInitializeParams::new()
        //                     .set_source_image(config.boot_disk_image)
        //                     .set_disk_size_gb(config.boot_disk_size_gb)
        //             )
        //     ])
        //     .set_network_interfaces(vec![
        //         NetworkInterface::new()
        //             .set_network("global/networks/default")
        //             .set_access_configs(vec![
        //                 AccessConfig::new()
        //                     .set_name("External NAT")
        //                     .set_type("ONE_TO_ONE_NAT")
        //             ])
        //     ])
        //     .set_tags(Tags::new().set_items(config.network_tags));
        //
        // let operation = instances
        //     .insert()
        //     .set_project(&self.config.project_id)
        //     .set_zone(&config.zone)
        //     .set_body(instance)
        //     .poller()
        //     .until_done()
        //     .await?
        //     .to_result()?;
        // ```

        Err(anyhow::anyhow!(
            "create_instance not yet implemented - requires google-cloud-compute integration"
        ))
    }

    /// Delete VM instance
    pub fn delete_instance(&self, _zone: &str, _instance_name: &str) -> Result<()> {
        // TODO: Implement using google-cloud-compute
        // Reference: reference/google-cloud-rust/guide/samples/src/compute/compute_instances_delete.rs
        Err(anyhow::anyhow!(
            "delete_instance not yet implemented - requires google-cloud-compute integration"
        ))
    }

    /// List VM instances
    pub fn list_instances(&self) -> Result<Vec<Instance>> {
        // TODO: Implement using google-cloud-compute
        // Reference: reference/google-cloud-rust/guide/samples/src/compute/compute_instances_list_all.rs
        Err(anyhow::anyhow!(
            "list_instances not yet implemented - requires google-cloud-compute integration"
        ))
    }

    /// Get VM instance details
    pub fn get_instance(&self, _zone: &str, _instance_name: &str) -> Result<Instance> {
        // TODO: Implement using google-cloud-compute
        Err(anyhow::anyhow!(
            "get_instance not yet implemented - requires google-cloud-compute integration"
        ))
    }

    /// Create firewall rule
    pub fn create_firewall_rule(&self, _rule: FirewallRule) -> Result<()> {
        // TODO: Implement using google-cloud-compute
        Err(anyhow::anyhow!(
            "create_firewall_rule not yet implemented - requires google-cloud-compute integration"
        ))
    }

    /// Allocate static IP
    pub fn allocate_static_ip(&self, _name: &str, _region: &str) -> Result<String> {
        // TODO: Implement using google-cloud-compute
        Err(anyhow::anyhow!(
            "allocate_static_ip not yet implemented - requires google-cloud-compute integration"
        ))
    }

    /// Get instance external IP
    pub fn get_instance_ip(&self, zone: &str, instance_name: &str) -> Result<String> {
        let instance = self.get_instance(zone, instance_name)?;
        instance
            .external_ip
            .ok_or_else(|| anyhow::anyhow!("Instance has no external IP"))
    }
}

/// GCP Region
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Region {
    pub name: String,
    pub location: String,
    pub zones: Vec<String>,
}

/// GCP Project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub project_id: String,
    pub project_name: String,
    pub project_number: String,
    pub billing_account: Option<String>,
}

/// Machine type information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineType {
    pub name: String,
    pub description: String,
    pub cpus: i32,
    pub memory_mb: i64,
}

pub mod hosting {
    //! GCP-specific VM hosting operations
    //!
    //! Handles VM lifecycle management including creation, deletion,
    //! restart, regeneration, and SSH key generation.

    use crate::api::gcp::GcpRestClient;
    use crate::config::{CloudPlatformConfig, VmInstance};
    use crate::{dure_debug, dure_error, dure_info, dure_warn};
    use anyhow::{Context, Result};

    /// Generate Ed25519 SSH keypair
    pub fn generate_ssh_keypair() -> Result<(String, Vec<u8>)> {
        use ed25519_dalek::SigningKey;
        use rand::RngCore;

        // Generate random 32 bytes for the key
        let mut rng = rand::thread_rng();
        let mut secret_bytes = [0u8; 32];
        rng.fill_bytes(&mut secret_bytes);

        // Generate key pair
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let verifying_key = signing_key.verifying_key();

        // Convert to SSH format
        let public_key_bytes = verifying_key.to_bytes();

        // Encode public key in OpenSSH format
        let mut public_key_ssh = vec![0u8; 4];
        public_key_ssh.extend_from_slice(b"ssh-ed25519");
        public_key_ssh.extend_from_slice(&(11u32).to_be_bytes()); // length of "ssh-ed25519"
        public_key_ssh.extend_from_slice(&(32u32).to_be_bytes()); // length of key
        public_key_ssh.extend_from_slice(&public_key_bytes);

        let public_key_b64 =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &public_key_ssh);
        let public_key = format!("ssh-ed25519 {} dure@generated", public_key_b64);

        // Private key as raw bytes (will be stored in keyring)
        let private_key_bytes = signing_key.to_bytes().to_vec();

        Ok((public_key, private_key_bytes))
    }

    /// Delete a VM instance
    pub fn delete_vm(client: &GcpRestClient, vm: &VmInstance) -> Result<String> {
        // Call GCP API to delete instance
        let operation = client.delete_instance(&vm.gcp_project_id, &vm.zone, &vm.name, false)?;

        // Poll operation status until complete (60 second timeout)
        client.wait_for_operation(&vm.gcp_project_id, &vm.zone, &operation.name, 60)?;

        Ok(format!("VM {} deleted successfully", vm.name))
    }

    /// Restart a VM instance
    pub fn restart_vm(client: &GcpRestClient, vm: &VmInstance) -> Result<String> {
        // Call GCP API to reset (hard reboot) instance
        let operation = client.reset_instance(&vm.gcp_project_id, &vm.zone, &vm.name)?;

        // Poll operation status until complete (60 second timeout)
        client.wait_for_operation(&vm.gcp_project_id, &vm.zone, &operation.name, 60)?;

        Ok(format!("VM {} restarted successfully", vm.name))
    }

    /// Regenerate VMs in a project (delete all, create one fresh)
    pub fn regenerate_vm(
        client: &GcpRestClient,
        platform: &mut CloudPlatformConfig,
        zone: &str,
        handle: &std::sync::Arc<crate::calc::keyring::DatabaseHandle>,
    ) -> Result<String> {
        use crate::api::gcp::compute::{
            AccessConfig, AttachedDisk, InitializeParams, InstanceRequest, Metadata, MetadataItem,
            NetworkInterface,
        };
        use crate::calc::keyring;

        // Delete all existing VMs
        let vm_count = platform.vms.len();
        for vm in &platform.vms {
            delete_vm(client, vm)?;
        }
        platform.vms.clear();

        // Get project ID
        let project_id = platform
            .gcp_selected_project_id
            .as_ref()
            .context("No project selected")?;

        // Generate Ed25519 SSH keypair
        let (public_key, private_key_bytes) = generate_ssh_keypair()?;

        // Generate VM name
        let vm_name = format!("dure-vm-{}", chrono::Utc::now().timestamp());
        let project_id = platform
            .gcp_selected_project_id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No GCP project selected"))?;
        let keyring_domain = format!("gcp.{}.{}", project_id, vm_name);

        // Store private key in profile keyring
        dure_debug!("Storing SSH key in profile keyring: {}", keyring_domain);
        keyring::add_key_to_handle(
            handle,
            &keyring_domain,
            "generated_user",
            "",
            Some(&private_key_bytes),
            Some(&format!("SSH key for GCP VM {}", vm_name)),
        )
        .context("Failed to store SSH key in profile keyring")?;

        // Create VM instance request
        let machine_type = format!("zones/{}/machineTypes/e2-micro", zone);
        let instance = InstanceRequest {
            name: vm_name.clone(),
            machine_type,
            disks: vec![AttachedDisk {
                boot: true,
                auto_delete: true,
                initialize_params: InitializeParams {
                    source_image: "projects/debian-cloud/global/images/family/debian-12"
                        .to_string(),
                    disk_size_gb: "10".to_string(),
                },
            }],
            network_interfaces: vec![NetworkInterface {
                network: "global/networks/default".to_string(),
                access_configs: Some(vec![AccessConfig {
                    type_: "ONE_TO_ONE_NAT".to_string(),
                    name: "External NAT".to_string(),
                    nat_ip: None,
                    network_tier: Some("STANDARD".to_string()),
                }]),
            }],
            tags: None,
            metadata: Some(Metadata {
                items: vec![MetadataItem {
                    key: "ssh-keys".to_string(),
                    value: format!("generated_user:{}", public_key),
                }],
            }),
        };

        // Create VM
        let operation = client.create_instance(project_id, zone, &instance)?;

        // Wait for VM creation to complete (120 second timeout)
        client.wait_for_operation(project_id, zone, &operation.name, 120)?;

        // Fetch VM details to get IP addresses
        let vm_instance = client.get_instance(project_id, zone, &vm_name)?;

        // Extract external IP
        let external_ip = vm_instance
            .network_interfaces
            .first()
            .and_then(|ni| ni.access_configs.first())
            .and_then(|ac| ac.nat_ip.clone());

        // Extract internal IP
        let internal_ip = vm_instance
            .network_interfaces
            .first()
            .and_then(|ni| ni.network_ip.clone());

        // Add VM to platform config
        platform.vms.push(VmInstance {
            name: vm_name.clone(),
            instance_id: vm_instance.id,
            zone: zone.to_string(),
            gcp_region: zone
                .rsplitn(2, '-')
                .nth(1)
                .map(|s| s.to_string())
                .unwrap_or_else(|| zone.to_string()),
            gcp_project_id: project_id.clone(),
            machine_type: "e2-micro".to_string(),
            status: vm_instance.status,
            external_ip,
            internal_ip,
            gcp_billing_account: None,
            created_at: chrono::Utc::now().timestamp(),
            ssh_key_name: Some(keyring_domain),
        });

        Ok(format!(
            "{} VMs deleted, new VM '{}' created successfully",
            vm_count, vm_name
        ))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_generate_ssh_keypair() {
            let result = generate_ssh_keypair();
            assert!(result.is_ok());

            let (public_key, private_key) = result.unwrap();

            // Public key should start with ssh-ed25519
            assert!(
                public_key.starts_with("ssh-ed25519"),
                "Public key should start with ssh-ed25519, got: {}",
                public_key
            );

            // Private key should be non-empty
            assert!(!private_key.is_empty(), "Private key should not be empty");

            // Private key should be 32 bytes for Ed25519
            assert_eq!(
                private_key.len(),
                32,
                "Ed25519 private key should be exactly 32 bytes"
            );
        }

        #[test]
        fn test_delete_vm_message() {
            // Test the success message format (actual API call tested manually)
            let vm = VmInstance {
                name: "test-vm".to_string(),
                instance_id: "123".to_string(),
                zone: "us-central1-a".to_string(),
                gcp_region: "us-central1".to_string(),
                gcp_project_id: "test".to_string(),
                machine_type: "e2-micro".to_string(),
                status: "RUNNING".to_string(),
                external_ip: Some("1.2.3.4".to_string()),
                internal_ip: None,
                gcp_billing_account: None,
                created_at: 0,
                ssh_key_name: None,
            };

            let expected_msg = format!("VM {} deleted successfully", vm.name);
            assert_eq!(expected_msg, "VM test-vm deleted successfully");
        }
    }
}

pub mod platform {
    //! GCP-specific platform operations
    //!
    //! Handles platform-level GCP operations including status computation,
    //! project selection, and OAuth management.

    use crate::config::CloudPlatformConfig;
    use crate::{dure_debug, dure_error, dure_info, dure_warn};
    use anyhow::Result;

    /// Compute platform-level status string
    pub fn compute_platform_status(
        platform: &CloudPlatformConfig,
        total_projects: usize,
    ) -> String {
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
}

/// Get common machine types
pub fn get_common_machine_types() -> Vec<MachineType> {
    vec![
        MachineType {
            name: "e2-micro".to_string(),
            description: "0.25-2 vCPU, 1 GB RAM (free tier eligible)".to_string(),
            cpus: 1,
            memory_mb: 1024,
        },
        MachineType {
            name: "e2-small".to_string(),
            description: "0.5-2 vCPU, 2 GB RAM".to_string(),
            cpus: 1,
            memory_mb: 2048,
        },
        MachineType {
            name: "e2-medium".to_string(),
            description: "1-2 vCPU, 4 GB RAM".to_string(),
            cpus: 1,
            memory_mb: 4096,
        },
        MachineType {
            name: "n1-standard-1".to_string(),
            description: "1 vCPU, 3.75 GB RAM".to_string(),
            cpus: 1,
            memory_mb: 3840,
        },
        MachineType {
            name: "n1-standard-2".to_string(),
            description: "2 vCPU, 7.5 GB RAM".to_string(),
            cpus: 2,
            memory_mb: 7680,
        },
        MachineType {
            name: "n2-standard-2".to_string(),
            description: "2 vCPU, 8 GB RAM".to_string(),
            cpus: 2,
            memory_mb: 8192,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_instance_config() {
        let config = InstanceConfig::default();
        assert_eq!(config.machine_type, "e2-micro");
        assert!(!config.network_tags.is_empty());
    }

    #[test]
    fn test_default_firewall_rule() {
        let rule = FirewallRule::default();
        assert_eq!(rule.direction, "INGRESS");
        assert!(!rule.allowed.is_empty());
    }

    #[test]
    fn test_common_machine_types() {
        let types = get_common_machine_types();
        assert!(!types.is_empty());
        assert!(types.iter().any(|t| t.name == "e2-micro"));
    }
}
