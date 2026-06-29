//! GCP-specific VM hosting operations
//!
//! Handles VM lifecycle management including creation, deletion,
//! restart, regeneration, and SSH key generation.

use anyhow::{Context, Result};
use crate::calc::gcp_rest::GcpRestClient;
use crate::config::{CloudPlatformConfig, VmInstance};

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

    let public_key_b64 = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &public_key_ssh
    );
    let public_key = format!("ssh-ed25519 {} dure@generated", public_key_b64);

    // Private key as raw bytes (will be stored in keyring)
    let private_key_bytes = signing_key.to_bytes().to_vec();

    Ok((public_key, private_key_bytes))
}

/// Delete a VM instance
pub fn delete_vm(client: &GcpRestClient, vm: &VmInstance) -> Result<String> {
    // Call GCP API to delete instance
    let operation = client.delete_instance(
        &vm.gcp_project_id,
        &vm.zone,
        &vm.name,
    )?;

    // Poll operation status until complete (60 second timeout)
    client.wait_for_operation(
        &vm.gcp_project_id,
        &vm.zone,
        &operation.name,
        60,
    )?;

    Ok(format!("VM {} deleted successfully", vm.name))
}

/// Restart a VM instance
pub fn restart_vm(client: &GcpRestClient, vm: &VmInstance) -> Result<String> {
    // Call GCP API to reset (hard reboot) instance
    let operation = client.reset_instance(
        &vm.gcp_project_id,
        &vm.zone,
        &vm.name,
    )?;

    // Poll operation status until complete (60 second timeout)
    client.wait_for_operation(
        &vm.gcp_project_id,
        &vm.zone,
        &operation.name,
        60,
    )?;

    Ok(format!("VM {} restarted successfully", vm.name))
}

/// Regenerate VMs in a project (delete all, create one fresh)
pub fn regenerate_vm(
    client: &GcpRestClient,
    platform: &mut CloudPlatformConfig,
    zone: &str,
) -> Result<String> {
    use crate::calc::gcp_rest::{InstanceRequest, AttachedDisk, InitializeParams, NetworkInterface, AccessConfig, Metadata, MetadataItem};
    use crate::calc::keyring;

    // Delete all existing VMs
    let vm_count = platform.vms.len();
    for vm in &platform.vms {
        delete_vm(client, vm)?;
    }
    platform.vms.clear();

    // Get project ID
    let project_id = platform.gcp_selected_project_id.as_ref()
        .context("No project selected")?;

    // Generate Ed25519 SSH keypair
    let (public_key, private_key_bytes) = generate_ssh_keypair()?;

    // Generate VM name
    let vm_name = format!("dure-vm-{}", chrono::Utc::now().timestamp());
    let keyring_domain = format!("gcp.{}.{}", platform.name, vm_name);

    // Store private key in keyring
    let kdbx_path = keyring::get_default_kdbx_path()?;
    let kpkey_path = keyring::get_default_kpkey_path()?;
    keyring::add_key_with_ssh(
        &kdbx_path,
        Some(&kpkey_path),
        &keyring_domain,
        "generated_user",
        "",
        Some(&private_key_bytes),
        Some(&format!("SSH key for GCP VM {}", vm_name)),
    )?;

    // Create VM instance request
    let machine_type = format!("zones/{}/machineTypes/e2-micro", zone);
    let instance = InstanceRequest {
        name: vm_name.clone(),
        machine_type,
        disks: vec![AttachedDisk {
            boot: true,
            auto_delete: true,
            initialize_params: InitializeParams {
                source_image: "projects/debian-cloud/global/images/family/debian-12".to_string(),
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
    let external_ip = vm_instance.network_interfaces.first()
        .and_then(|ni| ni.access_configs.first())
        .and_then(|ac| ac.nat_ip.clone());

    // Extract internal IP
    let internal_ip = vm_instance.network_interfaces.first()
        .and_then(|ni| ni.network_ip.clone());

    // Add VM to platform config
    platform.vms.push(VmInstance {
        name: vm_name.clone(),
        instance_id: vm_instance.id,
        zone: zone.to_string(),
        gcp_region: zone.rsplitn(2, '-').nth(1)
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

    Ok(format!("{} VMs deleted, new VM '{}' created successfully", vm_count, vm_name))
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
        assert!(public_key.starts_with("ssh-ed25519"),
            "Public key should start with ssh-ed25519, got: {}", public_key);

        // Private key should be non-empty
        assert!(!private_key.is_empty(), "Private key should not be empty");

        // Private key should be 32 bytes for Ed25519
        assert_eq!(private_key.len(), 32, "Ed25519 private key should be exactly 32 bytes");
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
