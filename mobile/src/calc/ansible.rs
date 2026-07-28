//! Ansible management functionality
//!
//! Provides Ansible Galaxy API integration and role lifecycle management

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use crate::config::{SshHostConfig, AnsibleRoleConfig};
use crate::calc::ssh;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnsibleRoleMetadata {
    pub name: String,
    pub description: String,
    pub variables: Vec<(String, String)>, // (name, default_value)
    pub dependencies: Vec<String>,
    pub suggested_ports: Vec<u16>,
}

/// Parse Ansible Galaxy role name
/// Example: "serhii9132.wireguard" -> ("serhii9132", "wireguard")
pub fn parse_galaxy_role(role: &str) -> Result<(&str, &str)> {
    role.split_once('.')
        .ok_or_else(|| anyhow::anyhow!("Invalid Galaxy role format. Expected 'namespace.role'"))
}

/// Generate Ansible playbook with role and variables
pub fn generate_playbook(config: &AnsibleRoleConfig) -> Result<String> {
    let mut playbook = String::from("---\n");
    playbook.push_str("- hosts: localhost\n");
    playbook.push_str("  become: yes\n");
    playbook.push_str("  roles:\n");
    playbook.push_str(&format!("    - {}\n", config.galaxy_name));

    if !config.variables.is_empty() {
        playbook.push_str("  vars:\n");
        for (key, value) in &config.variables {
            playbook.push_str(&format!("    {}: {}\n", key, value));
        }
    }

    Ok(playbook)
}

/// Fetch role metadata from Ansible Galaxy API
pub async fn fetch_ansible_role_metadata(role: &str) -> Result<AnsibleRoleMetadata> {
    let (namespace, name) = parse_galaxy_role(role)?;

    // Call Galaxy API v2
    let url = format!(
        "https://galaxy.ansible.com/api/v2/roles/?name={}&namespace={}",
        name, namespace
    );

    let response: serde_json::Value = ureq::get(&url)
        .call()
        .context("Failed to fetch from Ansible Galaxy")?
        .into_json()?;

    // Check if results found
    let results = response["results"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("No results from Galaxy API"))?;

    if results.is_empty() {
        anyhow::bail!("Role '{}' not found on Ansible Galaxy", role);
    }

    let role_data = &results[0];

    // Parse description
    let description = role_data["description"]
        .as_str()
        .unwrap_or("")
        .to_string();

    // Parse dependencies (if available)
    let dependencies = role_data["summary_fields"]["dependencies"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|d| d.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    // Note: variables and suggested_ports require fetching role files
    // For now, return empty - would need separate API calls
    let variables = vec![];
    let suggested_ports = vec![];

    Ok(AnsibleRoleMetadata {
        name: format!("{}.{}", namespace, name),
        description,
        variables,
        dependencies,
        suggested_ports,
    })
}

/// Install Ansible on remote host
pub async fn install_ansible(host_config: &SshHostConfig) -> Result<Vec<String>> {
    let mut progress = Vec::new();

    progress.push("Updating package list...".to_string());
    ssh::execute_command(host_config, "sudo apt-get update").await?;

    progress.push("Installing prerequisites...".to_string());
    ssh::execute_command(host_config, "sudo apt-get install -y software-properties-common").await?;

    progress.push("Adding Ansible PPA...".to_string());
    ssh::execute_command(host_config, "sudo add-apt-repository --yes --update ppa:ansible/ansible").await?;

    progress.push("Installing Ansible...".to_string());
    ssh::execute_command(host_config, "sudo apt-get install -y ansible").await?;

    progress.push("Ansible installed successfully".to_string());

    Ok(progress)
}

/// Check if Ansible is installed
pub async fn is_ansible_installed(host_config: &SshHostConfig) -> Result<bool> {
    match ssh::execute_command(host_config, "which ansible").await {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Install Ansible Galaxy role
pub async fn install_ansible_role(
    host_config: &SshHostConfig,
    config: &AnsibleRoleConfig,
) -> Result<String> {
    // Install role from Galaxy
    let install_cmd = format!("ansible-galaxy install {}", config.galaxy_name);
    ssh::execute_command(host_config, &install_cmd).await?;

    // Generate playbook with variables
    let playbook = generate_playbook(config)?;
    let playbook_path = format!("/tmp/{}.yml", config.name);

    // Write playbook to remote host
    let write_cmd = format!("cat > {} <<'PLAYBOOK_EOF'\n{}\nPLAYBOOK_EOF", playbook_path, playbook);
    ssh::execute_command(host_config, &write_cmd).await?;

    // Run playbook
    let run_cmd = format!("ansible-playbook {}", playbook_path);
    ssh::execute_command(host_config, &run_cmd).await
}

/// Remove Ansible role
pub async fn remove_ansible_role(
    host_config: &SshHostConfig,
    galaxy_name: &str,
) -> Result<String> {
    let remove_cmd = format!("ansible-galaxy remove {}", galaxy_name);
    ssh::execute_command(host_config, &remove_cmd).await
}

/// List installed roles
pub async fn list_ansible_roles(
    host_config: &SshHostConfig,
) -> Result<Vec<String>> {
    let output = ssh::execute_command(host_config, "ansible-galaxy list").await?;

    let roles: Vec<String> = output
        .lines()
        .filter(|line| !line.starts_with('#') && line.contains(','))
        .filter_map(|line| line.split(',').next().map(|s| s.trim().to_string()))
        .collect();

    Ok(roles)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_galaxy_role_valid() {
        let result = parse_galaxy_role("serhii9132.wireguard");
        assert_eq!(result.unwrap(), ("serhii9132", "wireguard"));
    }

    #[test]
    fn test_parse_galaxy_role_invalid() {
        let result = parse_galaxy_role("invalid-role");
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_playbook_with_variables() {
        let config = AnsibleRoleConfig {
            name: "wg-test".to_string(),
            galaxy_name: "serhii9132.wireguard".to_string(),
            variables: vec![
                ("wireguard_port".to_string(), "51820".to_string()),
                ("wireguard_peers".to_string(), "2".to_string()),
            ],
            ports: vec![51820],
            installed: false,
        };

        let playbook = generate_playbook(&config).unwrap();

        assert!(playbook.contains("---"));
        assert!(playbook.contains("hosts: localhost"));
        assert!(playbook.contains("become: yes"));
        assert!(playbook.contains("serhii9132.wireguard"));
        assert!(playbook.contains("wireguard_port: 51820"));
        assert!(playbook.contains("wireguard_peers: 2"));
    }

    #[test]
    fn test_generate_playbook_without_variables() {
        let config = AnsibleRoleConfig {
            name: "simple".to_string(),
            galaxy_name: "geerlingguy.apache".to_string(),
            variables: vec![],
            ports: vec![],
            installed: false,
        };

        let playbook = generate_playbook(&config).unwrap();

        assert!(playbook.contains("geerlingguy.apache"));
        assert!(!playbook.contains("vars:"));
    }
}
