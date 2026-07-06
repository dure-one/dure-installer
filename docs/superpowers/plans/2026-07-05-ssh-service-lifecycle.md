# SSH Service Lifecycle Management Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement full lifecycle management for Docker, Ansible, and Dure-WSS services on SSH hosts with form-based configuration, API validation, port mapping, and automated dependency installation.

**Architecture:** Hybrid approach - SSH Actor handles async orchestration, calc layer modules (docker, ansible, dure_wss) contain service-specific business logic. UI layer provides modal dialogs for configuration and drawer for status display.

**Tech Stack:** Rust 2021, smol 2.0 async runtime, ureq 2.12 HTTP client, russh SSH, egui + egui-material3 UI, serde YAML config

## Global Constraints

- Rust Edition: 2021
- Async Runtime: smol 2.0 (already in use)
- HTTP Client: ureq 2.12 with rustls (already in use)
- MVVM Pattern: Follow existing pattern (Platform/NS actors)
- No OpenSSL: Use pure Rust libraries (russh, rustls)
- Cross-platform: Desktop (Linux/macOS/Windows), Android, WASM
- TDD: Write tests before implementation
- Config Backward Compatibility: Use `#[serde(default)]` for new fields
- Error Messages: User-friendly, actionable, no technical jargon
- Frequent commits: After each task completion

---

## File Structure

**New Files:**
- `mobile/src/calc/docker.rs` - Docker Hub API client, container management (pull, run, remove, list)
- `mobile/src/calc/ansible.rs` - Ansible Galaxy API client, role management (install, remove, list)
- `mobile/src/calc/dure_wss.rs` - Dure-WSS service management (install, start, stop, status)

**Modified Files:**
- `mobile/src/config.rs` - Add DockerContainerConfig, AnsibleRoleConfig, DureWssConfig structs, extend SshHostConfig
- `mobile/src/viewmodel/ssh/commands.rs` - Add Docker/Ansible/Dure-WSS lifecycle commands
- `mobile/src/viewmodel/ssh/events.rs` - Add service lifecycle events
- `mobile/src/viewmodel/ssh/actor.rs` - Add command handlers for all services
- `mobile/src/viewmodel/mod.rs` - Add public API methods for service management
- `mobile/src/ui_tabs/ssh.rs` - Add install dialogs, dependency confirmation, drawer enhancements

---

### Task 1: Config Model + Docker Calc Layer

**Files:**
- Modify: `mobile/src/config.rs:147-190` (after SshHostConfig definition)
- Create: `mobile/src/calc/docker.rs`
- Create: `mobile/src/calc/docker_tests.rs` (or use #[cfg(test)] mod)

**Interfaces:**
- Consumes: Nothing (foundation task)
- Produces:
  - `DockerContainerConfig` struct
  - `AnsibleRoleConfig` struct
  - `DureWssConfig` struct
  - `SshHostConfig.docker_containers: Vec<DockerContainerConfig>`
  - `SshHostConfig.ansible_roles: Vec<AnsibleRoleConfig>`
  - `SshHostConfig.dure_wss_config: Option<DureWssConfig>`
  - `calc::docker::DockerImageMetadata` struct
  - `calc::docker::parse_docker_image(image: &str) -> Result<(&str, &str)>`
  - `calc::docker::fetch_docker_image_metadata(image: &str) -> Result<DockerImageMetadata>`
  - `calc::docker::install_docker_daemon(host_config: &SshHostConfig) -> Result<Vec<String>>`
  - `calc::docker::is_docker_installed(host_config: &SshHostConfig) -> Result<bool>`
  - `calc::docker::run_docker_container(host_config: &SshHostConfig, config: &DockerContainerConfig) -> Result<String>`
  - `calc::docker::remove_docker_container(host_config: &SshHostConfig, container_name: &str) -> Result<String>`
  - `calc::docker::list_docker_containers(host_config: &SshHostConfig) -> Result<Vec<DockerContainerConfig>>`
  - `calc::docker::build_docker_run_command(config: &DockerContainerConfig) -> String`

- [ ] **Step 1: Write test for parse_docker_image**

```rust
// mobile/src/calc/docker.rs (bottom of file)

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_docker_image_with_namespace() {
        let result = parse_docker_image("linuxserver/wireguard");
        assert_eq!(result.unwrap(), ("linuxserver", "wireguard"));
    }

    #[test]
    fn test_parse_docker_image_official() {
        let result = parse_docker_image("nginx");
        assert_eq!(result.unwrap(), ("library", "nginx"));
    }

    #[test]
    fn test_parse_docker_image_empty() {
        let result = parse_docker_image("");
        assert!(result.is_err());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib calc::docker::tests::test_parse_docker_image`  
Expected: FAIL (module calc::docker does not exist)

- [ ] **Step 3: Add config structs to config.rs**

Edit `mobile/src/config.rs`, add after `SshHostConfig` impl Default block (around line 190):

```rust
/// Docker container configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerContainerConfig {
    /// Unique instance name (e.g., "wireguard-us")
    pub name: String,
    /// Docker Hub image (e.g., "linuxserver/wireguard")
    pub image: String,
    /// Image tag (e.g., "latest", "v1.2.3")
    pub tag: String,
    /// Port mappings (host_port, container_port)
    pub ports: Vec<(u16, u16)>,
    /// Environment variables
    pub env: Vec<(String, String)>,
    /// Container status (running, stopped, error)
    pub status: String,
}

/// Ansible role configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnsibleRoleConfig {
    /// Unique instance name (e.g., "wireguard-main")
    pub name: String,
    /// Galaxy role namespace (e.g., "serhii9132.wireguard")
    pub galaxy_name: String,
    /// Role variables (key-value pairs)
    pub variables: Vec<(String, String)>,
    /// Port mappings for services this role manages
    pub ports: Vec<u16>,
    /// Installation status
    pub installed: bool,
}

/// Dure-WSS service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DureWssConfig {
    /// Domain for TLS cert (e.g., "api.dure.one")
    pub domain: String,
    /// Email for ACME notifications
    pub email: String,
    /// Release channel (stable, dev, beta)
    pub channel: String,
    /// Binary variant (headless, gui)
    pub variant: String,
    /// Service status (running, stopped, not_installed)
    pub status: String,
}
```

- [ ] **Step 4: Extend SshHostConfig with service fields**

Edit `mobile/src/config.rs`, add to `SshHostConfig` struct (after `platform_name` field around line 170):

```rust
    /// Docker containers installed on this host
    #[serde(default)]
    pub docker_containers: Vec<DockerContainerConfig>,
    
    /// Ansible roles installed on this host
    #[serde(default)]
    pub ansible_roles: Vec<AnsibleRoleConfig>,
    
    /// Dure-WSS service configuration
    #[serde(default)]
    pub dure_wss_config: Option<DureWssConfig>,
```

Update `SshHostConfig` impl Default (around line 178):

```rust
impl Default for SshHostConfig {
    fn default() -> Self {
        Self {
            host: String::new(),
            password: None,
            private_key_path: None,
            keyring_domain: None,
            port: default_ssh_port(),
            initialized: false,
            last_status: None,
            platform_name: None,
            docker_containers: Vec::new(),
            ansible_roles: Vec::new(),
            dure_wss_config: None,
        }
    }
}
```

- [ ] **Step 5: Create calc/docker.rs with parse_docker_image**

Create `mobile/src/calc/docker.rs`:

```rust
//! Docker management functionality
//!
//! Provides Docker Hub API integration and container lifecycle management

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use crate::config::{SshHostConfig, DockerContainerConfig};
use crate::calc::ssh;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerImageMetadata {
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub exposed_ports: Vec<u16>,
    pub env_vars: Vec<String>,
}

/// Parse Docker image name into namespace and image
/// Examples: "nginx" -> ("library", "nginx")
///           "linuxserver/wireguard" -> ("linuxserver", "wireguard")
pub fn parse_docker_image(image: &str) -> Result<(&str, &str)> {
    if image.is_empty() {
        anyhow::bail!("Image name cannot be empty");
    }
    
    if let Some((namespace, name)) = image.split_once('/') {
        Ok((namespace, name))
    } else {
        // No slash = official image from library
        Ok(("library", image))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_docker_image_with_namespace() {
        let result = parse_docker_image("linuxserver/wireguard");
        assert_eq!(result.unwrap(), ("linuxserver", "wireguard"));
    }

    #[test]
    fn test_parse_docker_image_official() {
        let result = parse_docker_image("nginx");
        assert_eq!(result.unwrap(), ("library", "nginx"));
    }

    #[test]
    fn test_parse_docker_image_empty() {
        let result = parse_docker_image("");
        assert!(result.is_err());
    }
}
```

- [ ] **Step 6: Add docker module to calc/mod.rs**

Edit `mobile/src/calc/mod.rs`, add after other module declarations:

```rust
pub mod docker;
```

- [ ] **Step 7: Run tests to verify parse_docker_image passes**

Run: `cargo test --lib calc::docker::tests`  
Expected: PASS (3 tests)

- [ ] **Step 8: Write test for build_docker_run_command**

Add to `mobile/src/calc/docker.rs` tests module:

```rust
    #[test]
    fn test_build_docker_run_command() {
        let config = DockerContainerConfig {
            name: "test-container".to_string(),
            image: "nginx".to_string(),
            tag: "latest".to_string(),
            ports: vec![(8080, 80), (8443, 443)],
            env: vec![
                ("ENV_KEY".to_string(), "value".to_string()),
                ("ANOTHER".to_string(), "test".to_string()),
            ],
            status: "running".to_string(),
        };

        let cmd = build_docker_run_command(&config);

        assert!(cmd.contains("docker run -d"));
        assert!(cmd.contains("--name test-container"));
        assert!(cmd.contains("--restart unless-stopped"));
        assert!(cmd.contains("-p 8080:80"));
        assert!(cmd.contains("-p 8443:443"));
        assert!(cmd.contains("-e ENV_KEY=value"));
        assert!(cmd.contains("-e ANOTHER=test"));
        assert!(cmd.contains("nginx:latest"));
    }
```

- [ ] **Step 9: Run test to verify it fails**

Run: `cargo test --lib calc::docker::tests::test_build_docker_run_command`  
Expected: FAIL (function build_docker_run_command not found)

- [ ] **Step 10: Implement build_docker_run_command**

Add to `mobile/src/calc/docker.rs` (before tests module):

```rust
/// Build docker run command (for testing and SSH execution)
pub fn build_docker_run_command(config: &DockerContainerConfig) -> String {
    let mut cmd = format!(
        "docker run -d --name {} --restart unless-stopped",
        config.name
    );
    
    // Add port mappings
    for (host_port, container_port) in &config.ports {
        cmd.push_str(&format!(" -p {}:{}", host_port, container_port));
    }
    
    // Add environment variables
    for (key, value) in &config.env {
        cmd.push_str(&format!(" -e {}={}", key, value));
    }
    
    // Add image
    cmd.push_str(&format!(" {}:{}", config.image, config.tag));
    
    cmd
}
```

- [ ] **Step 11: Run test to verify it passes**

Run: `cargo test --lib calc::docker::tests::test_build_docker_run_command`  
Expected: PASS

- [ ] **Step 12: Implement async Docker functions (stubs for now)**

Add to `mobile/src/calc/docker.rs` (after build_docker_run_command, before tests):

```rust
/// Fetch image metadata from Docker Hub API
pub async fn fetch_docker_image_metadata(image: &str) -> Result<DockerImageMetadata> {
    let (namespace, name) = parse_docker_image(image)?;
    
    // Call Docker Hub API v2
    let url = format!(
        "https://hub.docker.com/v2/repositories/{}/{}",
        namespace, name
    );
    
    // TODO: This is a sync call in async context - needs async HTTP client or runtime::unblock
    // For now, using ureq synchronously
    let response: serde_json::Value = ureq::get(&url)
        .call()
        .context("Failed to fetch from Docker Hub")?
        .into_json()?;
    
    // Extract description
    let description = response["description"]
        .as_str()
        .unwrap_or("")
        .to_string();
    
    // Note: tags require separate API call to /v2/repositories/{namespace}/{name}/tags
    // For now, return "latest" as default
    let tags = vec!["latest".to_string()];
    
    // Note: exposed_ports and env_vars require Dockerfile parsing or image inspection
    // Docker Hub API v2 doesn't expose this directly
    let exposed_ports = vec![];
    let env_vars = vec![];
    
    Ok(DockerImageMetadata {
        name: format!("{}/{}", namespace, name),
        description,
        tags,
        exposed_ports,
        env_vars,
    })
}

/// Install Docker daemon on remote host
pub async fn install_docker_daemon(host_config: &SshHostConfig) -> Result<Vec<String>> {
    let mut progress = Vec::new();
    
    progress.push("Downloading Docker installer...".to_string());
    ssh::execute_command(host_config, "curl -fsSL https://get.docker.com -o get-docker.sh").await?;
    
    progress.push("Installing Docker...".to_string());
    ssh::execute_command(host_config, "sudo sh get-docker.sh").await?;
    
    progress.push("Enabling Docker service...".to_string());
    ssh::execute_command(host_config, "sudo systemctl enable docker").await?;
    
    progress.push("Starting Docker service...".to_string());
    ssh::execute_command(host_config, "sudo systemctl start docker").await?;
    
    progress.push("Adding user to docker group...".to_string());
    ssh::execute_command(host_config, "sudo usermod -aG docker $USER").await?;
    
    progress.push("Docker installed successfully".to_string());
    
    Ok(progress)
}

/// Check if Docker is installed and running
pub async fn is_docker_installed(host_config: &SshHostConfig) -> Result<bool> {
    match ssh::execute_command(host_config, "which docker").await {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Pull and run Docker container
pub async fn run_docker_container(
    host_config: &SshHostConfig,
    config: &DockerContainerConfig,
) -> Result<String> {
    // Pull image
    let pull_cmd = format!("docker pull {}:{}", config.image, config.tag);
    ssh::execute_command(host_config, &pull_cmd).await?;
    
    // Build and execute docker run command
    let run_cmd = build_docker_run_command(config);
    ssh::execute_command(host_config, &run_cmd).await
}

/// Stop and remove Docker container
pub async fn remove_docker_container(
    host_config: &SshHostConfig,
    container_name: &str,
) -> Result<String> {
    // Stop container (ignore errors if already stopped)
    let stop_cmd = format!("docker stop {}", container_name);
    let _ = ssh::execute_command(host_config, &stop_cmd).await;
    
    // Remove container
    let rm_cmd = format!("docker rm {}", container_name);
    ssh::execute_command(host_config, &rm_cmd).await
}

/// List running containers
pub async fn list_docker_containers(
    host_config: &SshHostConfig,
) -> Result<Vec<DockerContainerConfig>> {
    let output = ssh::execute_command(
        host_config,
        "docker ps -a --format '{{.Names}}|{{.Image}}|{{.Status}}'",
    )
    .await?;
    
    let mut containers = Vec::new();
    for line in output.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 3 {
            let (image, tag) = parts[1].split_once(':').unwrap_or((parts[1], "latest"));
            containers.push(DockerContainerConfig {
                name: parts[0].to_string(),
                image: image.to_string(),
                tag: tag.to_string(),
                ports: vec![], // Would need docker inspect to get ports
                env: vec![],
                status: parts[2].to_string(),
            });
        }
    }
    
    Ok(containers)
}
```

- [ ] **Step 13: Verify compilation**

Run: `cargo check --lib`  
Expected: No errors (warnings OK)

- [ ] **Step 14: Commit Task 1**

```bash
git add mobile/src/config.rs mobile/src/calc/docker.rs mobile/src/calc/mod.rs
git commit -m "feat(ssh-services): add config model and Docker calc layer

- Add DockerContainerConfig, AnsibleRoleConfig, DureWssConfig structs
- Extend SshHostConfig with docker_containers, ansible_roles, dure_wss_config
- Implement calc::docker module with Docker Hub API client
- Add parse_docker_image, build_docker_run_command with tests
- Add container lifecycle functions: install, run, remove, list

Tests: cargo test calc::docker::tests passes

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 2: Ansible + Dure-WSS Calc Layer

**Files:**
- Create: `mobile/src/calc/ansible.rs`
- Create: `mobile/src/calc/dure_wss.rs`

**Interfaces:**
- Consumes:
  - `SshHostConfig` (from config.rs)
  - `AnsibleRoleConfig` (from config.rs)
  - `DureWssConfig` (from config.rs)
  - `calc::ssh::execute_command()` (existing)
- Produces:
  - `calc::ansible::AnsibleRoleMetadata` struct
  - `calc::ansible::parse_galaxy_role(role: &str) -> Result<(&str, &str)>`
  - `calc::ansible::fetch_ansible_role_metadata(role: &str) -> Result<AnsibleRoleMetadata>`
  - `calc::ansible::install_ansible(host_config: &SshHostConfig) -> Result<Vec<String>>`
  - `calc::ansible::is_ansible_installed(host_config: &SshHostConfig) -> Result<bool>`
  - `calc::ansible::install_ansible_role(host_config: &SshHostConfig, config: &AnsibleRoleConfig) -> Result<String>`
  - `calc::ansible::generate_playbook(config: &AnsibleRoleConfig) -> Result<String>`
  - `calc::ansible::remove_ansible_role(host_config: &SshHostConfig, galaxy_name: &str) -> Result<String>`
  - `calc::ansible::list_ansible_roles(host_config: &SshHostConfig) -> Result<Vec<String>>`
  - `calc::dure_wss::install_dure_wss(host_config: &SshHostConfig, config: &DureWssConfig) -> Result<Vec<String>>`
  - `calc::dure_wss::get_dure_wss_status(host_config: &SshHostConfig) -> Result<String>`
  - `calc::dure_wss::start_dure_wss(host_config: &SshHostConfig) -> Result<String>`
  - `calc::dure_wss::stop_dure_wss(host_config: &SshHostConfig) -> Result<String>`
  - `calc::dure_wss::restart_dure_wss(host_config: &SshHostConfig) -> Result<String>`
  - `calc::dure_wss::uninstall_dure_wss(host_config: &SshHostConfig) -> Result<String>`

- [ ] **Step 1: Write test for parse_galaxy_role**

Create `mobile/src/calc/ansible.rs`:

```rust
//! Ansible management functionality
//!
//! Provides Ansible Galaxy API integration and role lifecycle management

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
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib calc::ansible::tests::test_parse_galaxy_role`  
Expected: FAIL (module calc::ansible does not exist)

- [ ] **Step 3: Add ansible module to calc/mod.rs**

Edit `mobile/src/calc/mod.rs`:

```rust
pub mod ansible;
```

- [ ] **Step 4: Run tests to verify parse_galaxy_role passes**

Run: `cargo test --lib calc::ansible::tests`  
Expected: PASS (2 tests)

- [ ] **Step 5: Write test for generate_playbook**

Add to `mobile/src/calc/ansible.rs` tests module:

```rust
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
```

- [ ] **Step 6: Run test to verify it fails**

Run: `cargo test --lib calc::ansible::tests::test_generate_playbook`  
Expected: FAIL (function generate_playbook not found)

- [ ] **Step 7: Implement generate_playbook**

Add to `mobile/src/calc/ansible.rs` (before tests module):

```rust
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
```

- [ ] **Step 8: Run test to verify it passes**

Run: `cargo test --lib calc::ansible::tests::test_generate_playbook`  
Expected: PASS (2 tests)

- [ ] **Step 9: Implement async Ansible functions**

Add to `mobile/src/calc/ansible.rs` (after generate_playbook, before tests):

```rust
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
```

- [ ] **Step 10: Create calc/dure_wss.rs**

Create `mobile/src/calc/dure_wss.rs`:

```rust
//! Dure-WSS service management functionality
//!
//! Provides installation and lifecycle management for Dure-WSS service

use anyhow::Result;
use crate::config::{SshHostConfig, DureWssConfig};
use crate::calc::ssh;

/// Install Dure-WSS via official install script
pub async fn install_dure_wss(
    host_config: &SshHostConfig,
    config: &DureWssConfig,
) -> Result<Vec<String>> {
    let mut progress = Vec::new();
    
    // Download and run installer
    progress.push("Downloading Dure-WSS installer...".to_string());
    let install_cmd = format!(
        "curl --proto '=https' --tlsv1.2 -sSf https://run.dure.one | \
         DURE_CHANNEL={} DURE_VARIANT={} sh",
        config.channel, config.variant
    );
    ssh::execute_command(host_config, &install_cmd).await?;
    
    // Configure domain and email
    progress.push("Configuring Dure-WSS...".to_string());
    let config_cmd = format!(
        "dure wss config --domain {} --email {}",
        config.domain, config.email
    );
    ssh::execute_command(host_config, &config_cmd).await?;
    
    // Start service
    progress.push("Starting Dure-WSS service...".to_string());
    ssh::execute_command(host_config, "dure wss start").await?;
    
    progress.push("Dure-WSS installed and started".to_string());
    
    Ok(progress)
}

/// Get Dure-WSS service status
pub async fn get_dure_wss_status(host_config: &SshHostConfig) -> Result<String> {
    ssh::execute_command(host_config, "dure wss status").await
}

/// Start Dure-WSS service
pub async fn start_dure_wss(host_config: &SshHostConfig) -> Result<String> {
    ssh::execute_command(host_config, "dure wss start").await
}

/// Stop Dure-WSS service
pub async fn stop_dure_wss(host_config: &SshHostConfig) -> Result<String> {
    ssh::execute_command(host_config, "dure wss stop").await
}

/// Restart Dure-WSS service
pub async fn restart_dure_wss(host_config: &SshHostConfig) -> Result<String> {
    ssh::execute_command(host_config, "dure wss restart").await
}

/// Uninstall Dure-WSS
pub async fn uninstall_dure_wss(host_config: &SshHostConfig) -> Result<String> {
    // Stop service (ignore errors if already stopped)
    let _ = ssh::execute_command(host_config, "dure wss stop").await;
    
    // Remove binary and config
    ssh::execute_command(host_config, "sudo rm -f /usr/local/bin/dure").await?;
    ssh::execute_command(host_config, "sudo rm -rf ~/.config/dure").await?;
    
    Ok("Dure-WSS uninstalled".to_string())
}
```

- [ ] **Step 11: Add dure_wss module to calc/mod.rs**

Edit `mobile/src/calc/mod.rs`:

```rust
pub mod dure_wss;
```

- [ ] **Step 12: Verify compilation**

Run: `cargo check --lib`  
Expected: No errors (warnings OK)

- [ ] **Step 13: Run all calc tests**

Run: `cargo test --lib calc::`  
Expected: PASS (Docker + Ansible tests)

- [ ] **Step 14: Commit Task 2**

```bash
git add mobile/src/calc/ansible.rs mobile/src/calc/dure_wss.rs mobile/src/calc/mod.rs
git commit -m "feat(ssh-services): add Ansible and Dure-WSS calc layers

- Implement calc::ansible module with Galaxy API client
- Add parse_galaxy_role, generate_playbook with tests
- Add role lifecycle functions: install, remove, list
- Implement calc::dure_wss module with service management
- Add install, start, stop, restart, uninstall, status functions

Tests: cargo test calc:: passes

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 3: Actor Layer - Commands, Events, and Handlers

**Files:**
- Modify: `mobile/src/viewmodel/ssh/commands.rs`
- Modify: `mobile/src/viewmodel/ssh/events.rs`
- Modify: `mobile/src/viewmodel/ssh/actor.rs`

**Interfaces:**
- Consumes:
  - All calc layer functions from Task 1 & 2
  - `SshHostConfig`, `DockerContainerConfig`, `AnsibleRoleConfig`, `DureWssConfig` (from config.rs)
- Produces:
  - `SshCommand` variants: ValidateDockerImage, InstallDockerImage, RemoveDockerContainer, ListDockerContainers, ValidateAnsibleRole, InstallAnsibleRole, RemoveAnsibleRole, ListAnsibleRoles, InstallDureWssService, StartDureWss, StopDureWss, RestartDureWss, UninstallDureWss
  - `SshEvent` variants: DockerImageValidated, DockerDaemonInstallRequired, DockerDaemonInstalled, DockerImageInstalled, DockerContainerRemoved, DockerContainersListed, AnsibleRoleValidated, AnsibleDaemonInstallRequired, AnsibleDaemonInstalled, AnsibleRoleInstalled, AnsibleRoleRemoved, AnsibleRolesListed, DureWssServiceInstalled, DureWssStarted, DureWssStopped, DureWssUninstalled
  - Actor command handlers in `SshActor::run()` loop

- [ ] **Step 1: Add Docker command variants to ssh/commands.rs**

Edit `mobile/src/viewmodel/ssh/commands.rs`, add after existing commands:

```rust
    // Docker Lifecycle
    ValidateDockerImage {
        image: String,
    },
    InstallDockerImage {
        host_name: String,
        container_name: String,
        image: String,
        tag: String,
        ports: Vec<(u16, u16)>,
        env: Vec<(String, String)>,
    },
    RemoveDockerContainer {
        host_name: String,
        container_name: String,
    },
    ListDockerContainers {
        host_name: String,
    },
```

- [ ] **Step 2: Add Ansible command variants**

Add to `mobile/src/viewmodel/ssh/commands.rs`:

```rust
    // Ansible Lifecycle
    ValidateAnsibleRole {
        role: String,
    },
    InstallAnsibleRole {
        host_name: String,
        instance_name: String,
        galaxy_name: String,
        variables: Vec<(String, String)>,
        ports: Vec<u16>,
    },
    RemoveAnsibleRole {
        host_name: String,
        instance_name: String,
    },
    ListAnsibleRoles {
        host_name: String,
    },
```

- [ ] **Step 3: Add Dure-WSS command variants**

Add to `mobile/src/viewmodel/ssh/commands.rs`:

```rust
    // Dure-WSS Lifecycle
    InstallDureWssService {
        host_name: String,
        domain: String,
        email: String,
        channel: String,
        variant: String,
    },
    StartDureWss {
        host_name: String,
    },
    StopDureWss {
        host_name: String,
    },
    RestartDureWss {
        host_name: String,
    },
    UninstallDureWss {
        host_name: String,
    },
```

- [ ] **Step 4: Add Docker event variants to ssh/events.rs**

Edit `mobile/src/viewmodel/ssh/events.rs`, first add imports at top:

```rust
use crate::calc::docker::DockerImageMetadata;
use crate::calc::ansible::AnsibleRoleMetadata;
use crate::config::{DockerContainerConfig, AnsibleRoleConfig};
```

Then add event variants after existing events:

```rust
    // Docker Events
    DockerImageValidated {
        image: String,
        metadata: DockerImageMetadata,
    },
    DockerDaemonInstallRequired {
        host_name: String,
    },
    DockerDaemonInstalled {
        host_name: String,
    },
    DockerImageInstalled {
        host_name: String,
        container_name: String,
    },
    DockerContainerRemoved {
        host_name: String,
        container_name: String,
    },
    DockerContainersListed {
        host_name: String,
        containers: Vec<DockerContainerConfig>,
    },
```

- [ ] **Step 5: Add Ansible event variants**

Add to `mobile/src/viewmodel/ssh/events.rs`:

```rust
    // Ansible Events
    AnsibleRoleValidated {
        role: String,
        metadata: AnsibleRoleMetadata,
    },
    AnsibleDaemonInstallRequired {
        host_name: String,
    },
    AnsibleDaemonInstalled {
        host_name: String,
    },
    AnsibleRoleInstalled {
        host_name: String,
        instance_name: String,
    },
    AnsibleRoleRemoved {
        host_name: String,
        instance_name: String,
    },
    AnsibleRolesListed {
        host_name: String,
        roles: Vec<String>,
    },
```

- [ ] **Step 6: Add Dure-WSS event variants**

Add to `mobile/src/viewmodel/ssh/events.rs`:

```rust
    // Dure-WSS Events
    DureWssServiceInstalled {
        host_name: String,
        domain: String,
    },
    DureWssStarted {
        host_name: String,
    },
    DureWssStopped {
        host_name: String,
    },
    DureWssUninstalled {
        host_name: String,
    },
```

- [ ] **Step 7: Add imports to ssh/actor.rs**

Edit `mobile/src/viewmodel/ssh/actor.rs`, add imports at top:

```rust
use crate::calc::{docker, ansible, dure_wss};
use crate::config::{DockerContainerConfig, AnsibleRoleConfig, DureWssConfig};
```

- [ ] **Step 8: Implement ValidateDockerImage handler**

Add to `mobile/src/viewmodel/ssh/actor.rs` in the command match block:

```rust
            SshCommand::ValidateDockerImage { image } => {
                self.send_event(SshEvent::Progress {
                    operation: "ValidateDockerImage".to_string(),
                    progress: 0.5,
                    status: format!("Fetching metadata for {}", image),
                });

                match runtime::unblock(move || {
                    smol::block_on(docker::fetch_docker_image_metadata(&image))
                })
                .await
                {
                    Ok(metadata) => {
                        self.send_event(SshEvent::DockerImageValidated {
                            image: image.clone(),
                            metadata,
                        });
                    }
                    Err(e) => {
                        self.send_event(SshEvent::Error {
                            operation: "ValidateDockerImage".to_string(),
                            error: e.to_string(),
                        });
                    }
                }
            }
```

- [ ] **Step 9: Implement InstallDockerImage handler with dependency check**

Add to `mobile/src/viewmodel/ssh/actor.rs`:

```rust
            SshCommand::InstallDockerImage {
                host_name,
                container_name,
                image,
                tag,
                ports,
                env,
            } => {
                // Load config to get host
                let host_config = match self.load_host_config(&host_name) {
                    Ok(cfg) => cfg,
                    Err(e) => {
                        self.send_event(SshEvent::Error {
                            operation: "InstallDockerImage".to_string(),
                            error: format!("Host not found: {}", e),
                        });
                        continue;
                    }
                };

                // Check port conflicts
                if let Err(conflict) = self.check_port_conflicts(&host_name, &ports) {
                    self.send_event(SshEvent::Error {
                        operation: "InstallDockerImage".to_string(),
                        error: conflict,
                    });
                    continue;
                }

                // Check if Docker is installed
                let host_config_clone = host_config.clone();
                match runtime::unblock(move || {
                    smol::block_on(docker::is_docker_installed(&host_config_clone))
                })
                .await
                {
                    Ok(true) => {
                        // Docker installed, proceed to image installation
                    }
                    Ok(false) => {
                        // Docker not installed, send event requiring confirmation
                        self.send_event(SshEvent::DockerDaemonInstallRequired {
                            host_name: host_name.clone(),
                        });
                        
                        // Install Docker daemon
                        self.send_event(SshEvent::Progress {
                            operation: "InstallDocker".to_string(),
                            progress: 0.2,
                            status: "Installing Docker daemon...".to_string(),
                        });

                        let host_config_clone2 = host_config.clone();
                        match runtime::unblock(move || {
                            smol::block_on(docker::install_docker_daemon(&host_config_clone2))
                        })
                        .await
                        {
                            Ok(_) => {
                                self.send_event(SshEvent::DockerDaemonInstalled {
                                    host_name: host_name.clone(),
                                });
                            }
                            Err(e) => {
                                self.send_event(SshEvent::Error {
                                    operation: "InstallDocker".to_string(),
                                    error: e.to_string(),
                                });
                                continue;
                            }
                        }
                    }
                    Err(e) => {
                        self.send_event(SshEvent::Error {
                            operation: "CheckDocker".to_string(),
                            error: e.to_string(),
                        });
                        continue;
                    }
                }

                // Install Docker image
                self.send_event(SshEvent::Progress {
                    operation: "InstallDockerImage".to_string(),
                    progress: 0.5,
                    status: format!("Pulling image {}:{}", image, tag),
                });

                let container_config = DockerContainerConfig {
                    name: container_name.clone(),
                    image: image.clone(),
                    tag: tag.clone(),
                    ports: ports.clone(),
                    env: env.clone(),
                    status: "running".to_string(),
                };

                let host_config_clone3 = host_config.clone();
                match runtime::unblock(move || {
                    smol::block_on(docker::run_docker_container(
                        &host_config_clone3,
                        &container_config,
                    ))
                })
                .await
                {
                    Ok(_) => {
                        // Save container to config
                        if let Err(e) = self.save_docker_container(&host_name, container_config) {
                            self.send_event(SshEvent::Error {
                                operation: "SaveConfig".to_string(),
                                error: e.to_string(),
                            });
                        }

                        self.send_event(SshEvent::DockerImageInstalled {
                            host_name,
                            container_name,
                        });
                    }
                    Err(e) => {
                        self.send_event(SshEvent::Error {
                            operation: "InstallDockerImage".to_string(),
                            error: e.to_string(),
                        });
                    }
                }
            }
```

- [ ] **Step 10: Add helper methods to SshActor**

Add these methods to `mobile/src/viewmodel/ssh/actor.rs` impl block (before run() method):

```rust
    fn load_host_config(&self, host_name: &str) -> anyhow::Result<SshHostConfig> {
        let (config, _) = crate::config::load_config()?;
        config
            .ssh_hosts
            .iter()
            .find(|h| h.host == *host_name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Host '{}' not found", host_name))
    }

    fn check_port_conflicts(
        &self,
        host_name: &str,
        new_ports: &[(u16, u16)],
    ) -> Result<(), String> {
        let (config, _) = crate::config::load_config().map_err(|e| e.to_string())?;
        let host_config = config
            .ssh_hosts
            .iter()
            .find(|h| h.host == *host_name)
            .ok_or_else(|| format!("Host '{}' not found", host_name))?;

        let mut allocated_ports = std::collections::HashSet::new();

        // Collect ports from Docker containers
        for container in &host_config.docker_containers {
            for (host_port, _) in &container.ports {
                allocated_ports.insert(*host_port);
            }
        }

        // Collect ports from Ansible roles
        for role in &host_config.ansible_roles {
            for port in &role.ports {
                allocated_ports.insert(*port);
            }
        }

        // Check new ports against allocated ports
        for (host_port, _) in new_ports {
            if allocated_ports.contains(host_port) {
                return Err(format!(
                    "Port {} already in use on host '{}'",
                    host_port, host_name
                ));
            }
        }

        Ok(())
    }

    fn save_docker_container(
        &self,
        host_name: &str,
        container: DockerContainerConfig,
    ) -> anyhow::Result<()> {
        let (mut config, config_path) = crate::config::load_config()?;

        if let Some(host) = config.ssh_hosts.iter_mut().find(|h| h.host == *host_name) {
            host.docker_containers.push(container);
            crate::config::save_config(&config, &config_path)?;
        }

        Ok(())
    }

    fn save_ansible_role(
        &self,
        host_name: &str,
        role: AnsibleRoleConfig,
    ) -> anyhow::Result<()> {
        let (mut config, config_path) = crate::config::load_config()?;

        if let Some(host) = config.ssh_hosts.iter_mut().find(|h| h.host == *host_name) {
            host.ansible_roles.push(role);
            crate::config::save_config(&config, &config_path)?;
        }

        Ok(())
    }

    fn save_dure_wss_config(
        &self,
        host_name: &str,
        dure_config: DureWssConfig,
    ) -> anyhow::Result<()> {
        let (mut config, config_path) = crate::config::load_config()?;

        if let Some(host) = config.ssh_hosts.iter_mut().find(|h| h.host == *host_name) {
            host.dure_wss_config = Some(dure_config);
            crate::config::save_config(&config, &config_path)?;
        }

        Ok(())
    }
```

- [ ] **Step 11: Implement remaining Docker command handlers**

Add to `mobile/src/viewmodel/ssh/actor.rs`:

```rust
            SshCommand::RemoveDockerContainer {
                host_name,
                container_name,
            } => {
                let host_config = match self.load_host_config(&host_name) {
                    Ok(cfg) => cfg,
                    Err(e) => {
                        self.send_event(SshEvent::Error {
                            operation: "RemoveDockerContainer".to_string(),
                            error: e.to_string(),
                        });
                        continue;
                    }
                };

                let container_name_clone = container_name.clone();
                match runtime::unblock(move || {
                    smol::block_on(docker::remove_docker_container(
                        &host_config,
                        &container_name_clone,
                    ))
                })
                .await
                {
                    Ok(_) => {
                        // Remove from config
                        let (mut config, config_path) = match crate::config::load_config() {
                            Ok(c) => c,
                            Err(e) => {
                                self.send_event(SshEvent::Error {
                                    operation: "LoadConfig".to_string(),
                                    error: e.to_string(),
                                });
                                continue;
                            }
                        };

                        if let Some(host) = config.ssh_hosts.iter_mut().find(|h| h.host == host_name) {
                            host.docker_containers.retain(|c| c.name != container_name);
                            let _ = crate::config::save_config(&config, &config_path);
                        }

                        self.send_event(SshEvent::DockerContainerRemoved {
                            host_name,
                            container_name,
                        });
                    }
                    Err(e) => {
                        self.send_event(SshEvent::Error {
                            operation: "RemoveDockerContainer".to_string(),
                            error: e.to_string(),
                        });
                    }
                }
            }

            SshCommand::ListDockerContainers { host_name } => {
                let host_config = match self.load_host_config(&host_name) {
                    Ok(cfg) => cfg,
                    Err(e) => {
                        self.send_event(SshEvent::Error {
                            operation: "ListDockerContainers".to_string(),
                            error: e.to_string(),
                        });
                        continue;
                    }
                };

                match runtime::unblock(move || {
                    smol::block_on(docker::list_docker_containers(&host_config))
                })
                .await
                {
                    Ok(containers) => {
                        self.send_event(SshEvent::DockerContainersListed {
                            host_name,
                            containers,
                        });
                    }
                    Err(e) => {
                        self.send_event(SshEvent::Error {
                            operation: "ListDockerContainers".to_string(),
                            error: e.to_string(),
                        });
                    }
                }
            }
```

- [ ] **Step 12: Implement Ansible command handlers**

Add to `mobile/src/viewmodel/ssh/actor.rs`:

```rust
            SshCommand::ValidateAnsibleRole { role } => {
                self.send_event(SshEvent::Progress {
                    operation: "ValidateAnsibleRole".to_string(),
                    progress: 0.5,
                    status: format!("Fetching metadata for {}", role),
                });

                match runtime::unblock(move || {
                    smol::block_on(ansible::fetch_ansible_role_metadata(&role))
                })
                .await
                {
                    Ok(metadata) => {
                        self.send_event(SshEvent::AnsibleRoleValidated {
                            role: role.clone(),
                            metadata,
                        });
                    }
                    Err(e) => {
                        self.send_event(SshEvent::Error {
                            operation: "ValidateAnsibleRole".to_string(),
                            error: e.to_string(),
                        });
                    }
                }
            }

            SshCommand::InstallAnsibleRole {
                host_name,
                instance_name,
                galaxy_name,
                variables,
                ports,
            } => {
                let host_config = match self.load_host_config(&host_name) {
                    Ok(cfg) => cfg,
                    Err(e) => {
                        self.send_event(SshEvent::Error {
                            operation: "InstallAnsibleRole".to_string(),
                            error: e.to_string(),
                        });
                        continue;
                    }
                };

                // Check port conflicts
                let ports_for_check: Vec<(u16, u16)> = ports.iter().map(|p| (*p, *p)).collect();
                if let Err(conflict) = self.check_port_conflicts(&host_name, &ports_for_check) {
                    self.send_event(SshEvent::Error {
                        operation: "InstallAnsibleRole".to_string(),
                        error: conflict,
                    });
                    continue;
                }

                // Check if Ansible is installed
                let host_config_clone = host_config.clone();
                match runtime::unblock(move || {
                    smol::block_on(ansible::is_ansible_installed(&host_config_clone))
                })
                .await
                {
                    Ok(true) => {
                        // Ansible installed, proceed
                    }
                    Ok(false) => {
                        // Install Ansible
                        self.send_event(SshEvent::AnsibleDaemonInstallRequired {
                            host_name: host_name.clone(),
                        });

                        let host_config_clone2 = host_config.clone();
                        match runtime::unblock(move || {
                            smol::block_on(ansible::install_ansible(&host_config_clone2))
                        })
                        .await
                        {
                            Ok(_) => {
                                self.send_event(SshEvent::AnsibleDaemonInstalled {
                                    host_name: host_name.clone(),
                                });
                            }
                            Err(e) => {
                                self.send_event(SshEvent::Error {
                                    operation: "InstallAnsible".to_string(),
                                    error: e.to_string(),
                                });
                                continue;
                            }
                        }
                    }
                    Err(e) => {
                        self.send_event(SshEvent::Error {
                            operation: "CheckAnsible".to_string(),
                            error: e.to_string(),
                        });
                        continue;
                    }
                }

                // Install role
                let role_config = AnsibleRoleConfig {
                    name: instance_name.clone(),
                    galaxy_name: galaxy_name.clone(),
                    variables: variables.clone(),
                    ports: ports.clone(),
                    installed: true,
                };

                let host_config_clone3 = host_config.clone();
                match runtime::unblock(move || {
                    smol::block_on(ansible::install_ansible_role(
                        &host_config_clone3,
                        &role_config,
                    ))
                })
                .await
                {
                    Ok(_) => {
                        if let Err(e) = self.save_ansible_role(&host_name, role_config) {
                            self.send_event(SshEvent::Error {
                                operation: "SaveConfig".to_string(),
                                error: e.to_string(),
                            });
                        }

                        self.send_event(SshEvent::AnsibleRoleInstalled {
                            host_name,
                            instance_name,
                        });
                    }
                    Err(e) => {
                        self.send_event(SshEvent::Error {
                            operation: "InstallAnsibleRole".to_string(),
                            error: e.to_string(),
                        });
                    }
                }
            }

            SshCommand::RemoveAnsibleRole {
                host_name,
                instance_name,
            } => {
                let (config, _) = match crate::config::load_config() {
                    Ok(c) => c,
                    Err(e) => {
                        self.send_event(SshEvent::Error {
                            operation: "RemoveAnsibleRole".to_string(),
                            error: e.to_string(),
                        });
                        continue;
                    }
                };

                let host_config = match config.ssh_hosts.iter().find(|h| h.host == host_name) {
                    Some(h) => h.clone(),
                    None => {
                        self.send_event(SshEvent::Error {
                            operation: "RemoveAnsibleRole".to_string(),
                            error: format!("Host '{}' not found", host_name),
                        });
                        continue;
                    }
                };

                // Find galaxy_name from config
                let galaxy_name = host_config
                    .ansible_roles
                    .iter()
                    .find(|r| r.name == instance_name)
                    .map(|r| r.galaxy_name.clone());

                if let Some(gname) = galaxy_name {
                    match runtime::unblock(move || {
                        smol::block_on(ansible::remove_ansible_role(&host_config, &gname))
                    })
                    .await
                    {
                        Ok(_) => {
                            // Remove from config
                            let (mut config, config_path) = match crate::config::load_config() {
                                Ok(c) => c,
                                Err(e) => {
                                    self.send_event(SshEvent::Error {
                                        operation: "LoadConfig".to_string(),
                                        error: e.to_string(),
                                    });
                                    continue;
                                }
                            };

                            if let Some(host) = config.ssh_hosts.iter_mut().find(|h| h.host == host_name) {
                                host.ansible_roles.retain(|r| r.name != instance_name);
                                let _ = crate::config::save_config(&config, &config_path);
                            }

                            self.send_event(SshEvent::AnsibleRoleRemoved {
                                host_name,
                                instance_name,
                            });
                        }
                        Err(e) => {
                            self.send_event(SshEvent::Error {
                                operation: "RemoveAnsibleRole".to_string(),
                                error: e.to_string(),
                            });
                        }
                    }
                }
            }

            SshCommand::ListAnsibleRoles { host_name } => {
                let host_config = match self.load_host_config(&host_name) {
                    Ok(cfg) => cfg,
                    Err(e) => {
                        self.send_event(SshEvent::Error {
                            operation: "ListAnsibleRoles".to_string(),
                            error: e.to_string(),
                        });
                        continue;
                    }
                };

                match runtime::unblock(move || {
                    smol::block_on(ansible::list_ansible_roles(&host_config))
                })
                .await
                {
                    Ok(roles) => {
                        self.send_event(SshEvent::AnsibleRolesListed {
                            host_name,
                            roles,
                        });
                    }
                    Err(e) => {
                        self.send_event(SshEvent::Error {
                            operation: "ListAnsibleRoles".to_string(),
                            error: e.to_string(),
                        });
                    }
                }
            }
```

- [ ] **Step 13: Implement Dure-WSS command handlers**

Add to `mobile/src/viewmodel/ssh/actor.rs`:

```rust
            SshCommand::InstallDureWssService {
                host_name,
                domain,
                email,
                channel,
                variant,
            } => {
                let host_config = match self.load_host_config(&host_name) {
                    Ok(cfg) => cfg,
                    Err(e) => {
                        self.send_event(SshEvent::Error {
                            operation: "InstallDureWssService".to_string(),
                            error: e.to_string(),
                        });
                        continue;
                    }
                };

                let dure_config = DureWssConfig {
                    domain: domain.clone(),
                    email: email.clone(),
                    channel: channel.clone(),
                    variant: variant.clone(),
                    status: "running".to_string(),
                };

                let dure_config_clone = dure_config.clone();
                match runtime::unblock(move || {
                    smol::block_on(dure_wss::install_dure_wss(&host_config, &dure_config_clone))
                })
                .await
                {
                    Ok(_) => {
                        if let Err(e) = self.save_dure_wss_config(&host_name, dure_config) {
                            self.send_event(SshEvent::Error {
                                operation: "SaveConfig".to_string(),
                                error: e.to_string(),
                            });
                        }

                        self.send_event(SshEvent::DureWssServiceInstalled {
                            host_name,
                            domain,
                        });
                    }
                    Err(e) => {
                        self.send_event(SshEvent::Error {
                            operation: "InstallDureWssService".to_string(),
                            error: e.to_string(),
                        });
                    }
                }
            }

            SshCommand::StartDureWss { host_name } => {
                let host_config = match self.load_host_config(&host_name) {
                    Ok(cfg) => cfg,
                    Err(e) => {
                        self.send_event(SshEvent::Error {
                            operation: "StartDureWss".to_string(),
                            error: e.to_string(),
                        });
                        continue;
                    }
                };

                match runtime::unblock(move || {
                    smol::block_on(dure_wss::start_dure_wss(&host_config))
                })
                .await
                {
                    Ok(_) => {
                        self.send_event(SshEvent::DureWssStarted { host_name });
                    }
                    Err(e) => {
                        self.send_event(SshEvent::Error {
                            operation: "StartDureWss".to_string(),
                            error: e.to_string(),
                        });
                    }
                }
            }

            SshCommand::StopDureWss { host_name } => {
                let host_config = match self.load_host_config(&host_name) {
                    Ok(cfg) => cfg,
                    Err(e) => {
                        self.send_event(SshEvent::Error {
                            operation: "StopDureWss".to_string(),
                            error: e.to_string(),
                        });
                        continue;
                    }
                };

                match runtime::unblock(move || {
                    smol::block_on(dure_wss::stop_dure_wss(&host_config))
                })
                .await
                {
                    Ok(_) => {
                        self.send_event(SshEvent::DureWssStopped { host_name });
                    }
                    Err(e) => {
                        self.send_event(SshEvent::Error {
                            operation: "StopDureWss".to_string(),
                            error: e.to_string(),
                        });
                    }
                }
            }

            SshCommand::RestartDureWss { host_name } => {
                let host_config = match self.load_host_config(&host_name) {
                    Ok(cfg) => cfg,
                    Err(e) => {
                        self.send_event(SshEvent::Error {
                            operation: "RestartDureWss".to_string(),
                            error: e.to_string(),
                        });
                        continue;
                    }
                };

                match runtime::unblock(move || {
                    smol::block_on(dure_wss::restart_dure_wss(&host_config))
                })
                .await
                {
                    Ok(_) => {
                        // Send both stopped and started events
                        self.send_event(SshEvent::DureWssStopped {
                            host_name: host_name.clone(),
                        });
                        self.send_event(SshEvent::DureWssStarted { host_name });
                    }
                    Err(e) => {
                        self.send_event(SshEvent::Error {
                            operation: "RestartDureWss".to_string(),
                            error: e.to_string(),
                        });
                    }
                }
            }

            SshCommand::UninstallDureWss { host_name } => {
                let host_config = match self.load_host_config(&host_name) {
                    Ok(cfg) => cfg,
                    Err(e) => {
                        self.send_event(SshEvent::Error {
                            operation: "UninstallDureWss".to_string(),
                            error: e.to_string(),
                        });
                        continue;
                    }
                };

                match runtime::unblock(move || {
                    smol::block_on(dure_wss::uninstall_dure_wss(&host_config))
                })
                .await
                {
                    Ok(_) => {
                        // Remove from config
                        let (mut config, config_path) = match crate::config::load_config() {
                            Ok(c) => c,
                            Err(e) => {
                                self.send_event(SshEvent::Error {
                                    operation: "LoadConfig".to_string(),
                                    error: e.to_string(),
                                });
                                continue;
                            }
                        };

                        if let Some(host) = config.ssh_hosts.iter_mut().find(|h| h.host == host_name) {
                            host.dure_wss_config = None;
                            let _ = crate::config::save_config(&config, &config_path);
                        }

                        self.send_event(SshEvent::DureWssUninstalled { host_name });
                    }
                    Err(e) => {
                        self.send_event(SshEvent::Error {
                            operation: "UninstallDureWss".to_string(),
                            error: e.to_string(),
                        });
                    }
                }
            }
```

- [ ] **Step 14: Verify compilation**

Run: `cargo check --lib`  
Expected: No errors (may have warnings about unused code)

- [ ] **Step 15: Commit Task 3**

```bash
git add mobile/src/viewmodel/ssh/commands.rs mobile/src/viewmodel/ssh/events.rs mobile/src/viewmodel/ssh/actor.rs
git commit -m "feat(ssh-services): implement actor layer for service lifecycle

- Add Docker/Ansible/Dure-WSS command variants to SshCommand
- Add corresponding event variants to SshEvent  
- Implement all command handlers in SshActor
- Add dependency auto-install logic (Docker, Ansible)
- Add port conflict detection across services
- Add config persistence helpers (save_docker_container, save_ansible_role, save_dure_wss_config)

Actor layer complete, ready for ViewModel API integration.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 4: ViewModel Public API

**Files:**
- Modify: `mobile/src/viewmodel/mod.rs`

**Interfaces:**
- Consumes:
  - All SshCommand variants from Task 3
  - `ViewModel.ssh_sender` (existing channel to SSH actor)
- Produces:
  - `ViewModel::validate_docker_image(&self, image: String)`
  - `ViewModel::install_docker_image(&self, host: String, container_name: String, image: String, tag: String, ports: Vec<(u16, u16)>, env: Vec<(String, String)>)`
  - `ViewModel::remove_docker_container(&self, host: String, container_name: String)`
  - `ViewModel::list_docker_containers(&self, host: String)`
  - `ViewModel::validate_ansible_role(&self, role: String)`
  - `ViewModel::install_ansible_role(&self, host: String, instance_name: String, galaxy_name: String, variables: Vec<(String, String)>, ports: Vec<u16>)`
  - `ViewModel::remove_ansible_role(&self, host: String, instance_name: String)`
  - `ViewModel::list_ansible_roles(&self, host: String)`
  - `ViewModel::install_dure_wss(&self, host: String, domain: String, email: String, channel: String, variant: String)`
  - `ViewModel::start_dure_wss(&self, host: String)`
  - `ViewModel::stop_dure_wss(&self, host: String)`
  - `ViewModel::restart_dure_wss(&self, host: String)`
  - `ViewModel::uninstall_dure_wss(&self, host: String)`

- [ ] **Step 1: Add Docker ViewModel methods**

Edit `mobile/src/viewmodel/mod.rs`, add methods to impl ViewModel block:

```rust
    // Docker Lifecycle Management

    /// Validate Docker image and fetch metadata from Docker Hub
    pub fn validate_docker_image(&self, image: String) {
        if let Some(sender) = &self.ssh_sender {
            let _ = sender.send_blocking(SshCommand::ValidateDockerImage { image });
        }
    }

    /// Install Docker image on host (auto-installs Docker if needed)
    pub fn install_docker_image(
        &self,
        host: String,
        container_name: String,
        image: String,
        tag: String,
        ports: Vec<(u16, u16)>,
        env: Vec<(String, String)>,
    ) {
        if let Some(sender) = &self.ssh_sender {
            let _ = sender.send_blocking(SshCommand::InstallDockerImage {
                host_name: host,
                container_name,
                image,
                tag,
                ports,
                env,
            });
        }
    }

    /// Remove Docker container from host
    pub fn remove_docker_container(&self, host: String, container_name: String) {
        if let Some(sender) = &self.ssh_sender {
            let _ = sender.send_blocking(SshCommand::RemoveDockerContainer {
                host_name: host,
                container_name,
            });
        }
    }

    /// List Docker containers on host
    pub fn list_docker_containers(&self, host: String) {
        if let Some(sender) = &self.ssh_sender {
            let _ = sender.send_blocking(SshCommand::ListDockerContainers { host_name: host });
        }
    }
```

- [ ] **Step 2: Add Ansible ViewModel methods**

Add to `mobile/src/viewmodel/mod.rs`:

```rust
    // Ansible Lifecycle Management

    /// Validate Ansible role and fetch metadata from Galaxy
    pub fn validate_ansible_role(&self, role: String) {
        if let Some(sender) = &self.ssh_sender {
            let _ = sender.send_blocking(SshCommand::ValidateAnsibleRole { role });
        }
    }

    /// Install Ansible role on host (auto-installs Ansible if needed)
    pub fn install_ansible_role(
        &self,
        host: String,
        instance_name: String,
        galaxy_name: String,
        variables: Vec<(String, String)>,
        ports: Vec<u16>,
    ) {
        if let Some(sender) = &self.ssh_sender {
            let _ = sender.send_blocking(SshCommand::InstallAnsibleRole {
                host_name: host,
                instance_name,
                galaxy_name,
                variables,
                ports,
            });
        }
    }

    /// Remove Ansible role from host
    pub fn remove_ansible_role(&self, host: String, instance_name: String) {
        if let Some(sender) = &self.ssh_sender {
            let _ = sender.send_blocking(SshCommand::RemoveAnsibleRole {
                host_name: host,
                instance_name,
            });
        }
    }

    /// List Ansible roles installed on host
    pub fn list_ansible_roles(&self, host: String) {
        if let Some(sender) = &self.ssh_sender {
            let _ = sender.send_blocking(SshCommand::ListAnsibleRoles { host_name: host });
        }
    }
```

- [ ] **Step 3: Add Dure-WSS ViewModel methods**

Add to `mobile/src/viewmodel/mod.rs`:

```rust
    // Dure-WSS Lifecycle Management

    /// Install Dure-WSS service on host
    pub fn install_dure_wss(
        &self,
        host: String,
        domain: String,
        email: String,
        channel: String,
        variant: String,
    ) {
        if let Some(sender) = &self.ssh_sender {
            let _ = sender.send_blocking(SshCommand::InstallDureWssService {
                host_name: host,
                domain,
                email,
                channel,
                variant,
            });
        }
    }

    /// Start Dure-WSS service
    pub fn start_dure_wss(&self, host: String) {
        if let Some(sender) = &self.ssh_sender {
            let _ = sender.send_blocking(SshCommand::StartDureWss { host_name: host });
        }
    }

    /// Stop Dure-WSS service
    pub fn stop_dure_wss(&self, host: String) {
        if let Some(sender) = &self.ssh_sender {
            let _ = sender.send_blocking(SshCommand::StopDureWss { host_name: host });
        }
    }

    /// Restart Dure-WSS service
    pub fn restart_dure_wss(&self, host: String) {
        if let Some(sender) = &self.ssh_sender {
            let _ = sender.send_blocking(SshCommand::RestartDureWss { host_name: host });
        }
    }

    /// Uninstall Dure-WSS service
    pub fn uninstall_dure_wss(&self, host: String) {
        if let Some(sender) = &self.ssh_sender {
            let _ = sender.send_blocking(SshCommand::UninstallDureWss { host_name: host });
        }
    }
```

- [ ] **Step 4: Verify compilation**

Run: `cargo check --lib`  
Expected: No errors

- [ ] **Step 5: Commit Task 4**

```bash
git add mobile/src/viewmodel/mod.rs
git commit -m "feat(ssh-services): add ViewModel public API for service management

Add 13 public methods to ViewModel for Docker/Ansible/Dure-WSS lifecycle:
- Docker: validate, install, remove, list
- Ansible: validate, install, remove, list
- Dure-WSS: install, start, stop, restart, uninstall

ViewModel API complete and ready for UI integration.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

Due to the length of this plan, I'll continue with the remaining tasks (UI implementation) in a follow-up. The plan is getting very long. Should I:


### Task 5: Docker UI Dialog

**Files:**
- Modify: `mobile/src/ui_tabs/ssh.rs`

**Interfaces:**
- Consumes:
  - `ViewModel` public API from Task 4
  - `SshEvent` variants from Task 3
  - `DockerImageMetadata` from Task 1
- Produces:
  - `render_docker_install_dialog()` method
  - Dialog state fields in `SshTab` struct
  - Event handlers for Docker events

- [ ] **Step 1: Add Docker dialog state to SshTab struct**

Edit `mobile/src/ui_tabs/ssh.rs`, add fields to `SshTab` struct:

```rust
    // Docker Install Dialog
    #[cfg_attr(feature = "serde", serde(skip))]
    show_docker_install_dialog: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    docker_install_host_idx: Option<usize>,
    #[cfg_attr(feature = "serde", serde(skip))]
    docker_image_input: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    docker_container_name: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    docker_tag: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    docker_metadata: Option<DockerImageMetadata>,
    #[cfg_attr(feature = "serde", serde(skip))]
    docker_port_mappings: Vec<(String, String)>, // (host_port, container_port)
    #[cfg_attr(feature = "serde", serde(skip))]
    docker_env_vars: Vec<(String, String)>,
    #[cfg_attr(feature = "serde", serde(skip))]
    docker_validating: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    docker_validation_error: Option<String>,
```

- [ ] **Step 2: Update SshTab Default impl**

Update `impl Default for SshTab`, add initializers:

```rust
            show_docker_install_dialog: false,
            docker_install_host_idx: None,
            docker_image_input: String::new(),
            docker_container_name: String::new(),
            docker_tag: "latest".to_string(),
            docker_metadata: None,
            docker_port_mappings: Vec::new(),
            docker_env_vars: Vec::new(),
            docker_validating: false,
            docker_validation_error: None,
```

- [ ] **Step 3: Add Docker event handling to handle_event method**

Add to existing `handle_event` method in `ssh.rs`:

```rust
                SshEvent::DockerImageValidated { image, metadata } => {
                    self.docker_validating = false;
                    self.docker_validation_error = None;
                    self.docker_metadata = Some(metadata.clone());
                    
                    // Pre-populate port mappings from exposed ports
                    self.docker_port_mappings.clear();
                    for port in &metadata.exposed_ports {
                        self.docker_port_mappings.push((port.to_string(), port.to_string()));
                    }
                    
                    // Pre-populate env vars
                    self.docker_env_vars.clear();
                    for env_var in &metadata.env_vars {
                        self.docker_env_vars.push((env_var.clone(), String::new()));
                    }
                }
                SshEvent::DockerImageInstalled { host_name, container_name } => {
                    self.show_docker_install_dialog = false;
                    // Refresh host list
                    self.load_rows();
                }
                SshEvent::DockerDaemonInstalled { host_name } => {
                    // Docker installed, proceeding with image install
                }
                SshEvent::Error { operation, error } if operation.contains("Docker") => {
                    if operation == "ValidateDockerImage" {
                        self.docker_validating = false;
                        self.docker_validation_error = Some(error.clone());
                    }
                }
```

- [ ] **Step 4: Implement render_docker_install_dialog method**

Add method to `ssh.rs`:

```rust
    fn render_docker_install_dialog(&mut self, ctx: &egui::Context, vm: Option<&mut ViewModel>) {
        if !self.show_docker_install_dialog {
            return;
        }

        let mut open = true;
        egui::Window::new("Install Docker Image")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 8.0;

                    // Image input
                    ui.label("Docker Image:");
                    let image_response = ui.text_edit_singleline(&mut self.docker_image_input);
                    
                    // Validate on blur
                    if image_response.lost_focus() && !self.docker_image_input.is_empty() {
                        if let Some(vm) = vm {
                            self.docker_validating = true;
                            self.docker_validation_error = None;
                            vm.validate_docker_image(self.docker_image_input.clone());
                        }
                    }

                    if self.docker_validating {
                        ui.spinner();
                        ui.label("Fetching image metadata...");
                    }

                    if let Some(error) = &self.docker_validation_error {
                        ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
                    }

                    // Show metadata if validated
                    if let Some(metadata) = &self.docker_metadata {
                        ui.separator();
                        ui.label(format!("Description: {}", metadata.description));
                        
                        ui.horizontal(|ui| {
                            ui.label("Tag:");
                            egui::ComboBox::from_id_source("docker_tag")
                                .selected_text(&self.docker_tag)
                                .show_ui(ui, |ui| {
                                    for tag in &metadata.tags {
                                        ui.selectable_value(&mut self.docker_tag, tag.clone(), tag);
                                    }
                                });
                        });
                    }

                    // Container name
                    ui.label("Container Name:");
                    ui.text_edit_singleline(&mut self.docker_container_name);
                    ui.label("(Unique identifier for this instance)");

                    // Port mappings
                    if !self.docker_port_mappings.is_empty() {
                        ui.separator();
                        ui.label("Port Mappings (host:container):");
                        
                        let mut to_remove = None;
                        for (idx, (host_port, container_port)) in self.docker_port_mappings.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.text_edit_singleline(host_port);
                                ui.label(":");
                                ui.text_edit_singleline(container_port);
                                if ui.button("❌").clicked() {
                                    to_remove = Some(idx);
                                }
                            });
                        }
                        
                        if let Some(idx) = to_remove {
                            self.docker_port_mappings.remove(idx);
                        }

                        if ui.button("➕ Add Port Mapping").clicked() {
                            self.docker_port_mappings.push((String::new(), String::new()));
                        }
                    }

                    // Environment variables
                    if !self.docker_env_vars.is_empty() || self.docker_metadata.is_some() {
                        ui.separator();
                        ui.label("Environment Variables:");
                        
                        let mut to_remove_env = None;
                        for (idx, (key, value)) in self.docker_env_vars.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.text_edit_singleline(key);
                                ui.label("=");
                                ui.text_edit_singleline(value);
                                if ui.button("❌").clicked() {
                                    to_remove_env = Some(idx);
                                }
                            });
                        }
                        
                        if let Some(idx) = to_remove_env {
                            self.docker_env_vars.remove(idx);
                        }

                        if ui.button("➕ Add Environment Variable").clicked() {
                            self.docker_env_vars.push((String::new(), String::new()));
                        }
                    }

                    ui.separator();

                    ui.horizontal(|ui| {
                        let can_install = self.docker_metadata.is_some() 
                            && !self.docker_container_name.is_empty()
                            && !self.docker_validating;

                        if ui.add_enabled(can_install, MaterialButton::filled("Install")).clicked() {
                            if let Some(vm) = vm {
                                if let Some(host_idx) = self.docker_install_host_idx {
                                    let host_name = self.rows[host_idx].host.clone();
                                    
                                    // Parse ports
                                    let ports: Vec<(u16, u16)> = self.docker_port_mappings
                                        .iter()
                                        .filter_map(|(h, c)| {
                                            let hp = h.parse::<u16>().ok()?;
                                            let cp = c.parse::<u16>().ok()?;
                                            Some((hp, cp))
                                        })
                                        .collect();

                                    // Filter out empty env vars
                                    let env: Vec<(String, String)> = self.docker_env_vars
                                        .iter()
                                        .filter(|(k, _)| !k.is_empty())
                                        .cloned()
                                        .collect();

                                    vm.install_docker_image(
                                        host_name,
                                        self.docker_container_name.clone(),
                                        self.docker_image_input.clone(),
                                        self.docker_tag.clone(),
                                        ports,
                                        env,
                                    );
                                }
                            }
                        }

                        if ui.button("Cancel").clicked() {
                            open = false;
                        }
                    });
                });
            });

        if !open {
            self.show_docker_install_dialog = false;
            self.docker_image_input.clear();
            self.docker_container_name.clear();
            self.docker_tag = "latest".to_string();
            self.docker_metadata = None;
            self.docker_port_mappings.clear();
            self.docker_env_vars.clear();
            self.docker_validating = false;
            self.docker_validation_error = None;
        }
    }
```

- [ ] **Step 5: Add "Install Docker Image" button to drawer operations**

In the drawer rendering section, add button for Docker:

```rust
            if ui.add(MaterialButton::outlined("Install Docker Image").small()).clicked() {
                self.show_docker_install_dialog = true;
                self.docker_install_host_idx = Some(row_idx);
            }
```

- [ ] **Step 6: Call render_docker_install_dialog in ui() method**

Add to `ui()` method after existing dialog renders:

```rust
        self.render_docker_install_dialog(ui.ctx(), vm.as_deref_mut());
```

- [ ] **Step 7: Verify compilation**

Run: `cargo check --lib`  
Expected: No errors

- [ ] **Step 8: Commit Task 5**

```bash
git add mobile/src/ui_tabs/ssh.rs
git commit -m "feat(ssh-ui): add Docker install dialog

- Add dialog state fields to SshTab struct
- Implement render_docker_install_dialog with validation
- Add image validation on blur (fetches metadata)
- Add port mapping and env var configuration forms
- Add event handling for DockerImageValidated, DockerImageInstalled
- Add "Install Docker Image" button to drawer operations

Docker UI complete and functional.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 6: Ansible UI Dialog

**Files:**
- Modify: `mobile/src/ui_tabs/ssh.rs`

**Interfaces:**
- Consumes:
  - `ViewModel` Ansible methods from Task 4
  - `SshEvent` Ansible variants from Task 3
  - `AnsibleRoleMetadata` from Task 2
- Produces:
  - `render_ansible_install_dialog()` method
  - Ansible dialog state fields in `SshTab` struct

- [ ] **Step 1: Add Ansible dialog state to SshTab struct**

Edit `mobile/src/ui_tabs/ssh.rs`:

```rust
    // Ansible Install Dialog
    #[cfg_attr(feature = "serde", serde(skip))]
    show_ansible_install_dialog: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    ansible_install_host_idx: Option<usize>,
    #[cfg_attr(feature = "serde", serde(skip))]
    ansible_role_input: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    ansible_instance_name: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    ansible_metadata: Option<AnsibleRoleMetadata>,
    #[cfg_attr(feature = "serde", serde(skip))]
    ansible_variables: Vec<(String, String)>,
    #[cfg_attr(feature = "serde", serde(skip))]
    ansible_ports: Vec<String>,
    #[cfg_attr(feature = "serde", serde(skip))]
    ansible_validating: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    ansible_validation_error: Option<String>,
```

- [ ] **Step 2: Update SshTab Default impl for Ansible**

```rust
            show_ansible_install_dialog: false,
            ansible_install_host_idx: None,
            ansible_role_input: String::new(),
            ansible_instance_name: String::new(),
            ansible_metadata: None,
            ansible_variables: Vec::new(),
            ansible_ports: Vec::new(),
            ansible_validating: false,
            ansible_validation_error: None,
```

- [ ] **Step 3: Add Ansible event handling**

Add to `handle_event` method:

```rust
                SshEvent::AnsibleRoleValidated { role, metadata } => {
                    self.ansible_validating = false;
                    self.ansible_validation_error = None;
                    self.ansible_metadata = Some(metadata.clone());
                    
                    // Pre-populate variables from metadata
                    self.ansible_variables.clear();
                    for (name, default_value) in &metadata.variables {
                        self.ansible_variables.push((name.clone(), default_value.clone()));
                    }
                    
                    // Pre-populate ports
                    self.ansible_ports.clear();
                    for port in &metadata.suggested_ports {
                        self.ansible_ports.push(port.to_string());
                    }
                }
                SshEvent::AnsibleRoleInstalled { host_name, instance_name } => {
                    self.show_ansible_install_dialog = false;
                    self.load_rows();
                }
                SshEvent::AnsibleDaemonInstalled { host_name } => {
                    // Ansible installed, proceeding with role install
                }
                SshEvent::Error { operation, error } if operation.contains("Ansible") => {
                    if operation == "ValidateAnsibleRole" {
                        self.ansible_validating = false;
                        self.ansible_validation_error = Some(error.clone());
                    }
                }
```

- [ ] **Step 4: Implement render_ansible_install_dialog**

```rust
    fn render_ansible_install_dialog(&mut self, ctx: &egui::Context, vm: Option<&mut ViewModel>) {
        if !self.show_ansible_install_dialog {
            return;
        }

        let mut open = true;
        egui::Window::new("Install Ansible Role")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 8.0;

                    // Role input
                    ui.label("Ansible Galaxy Role (namespace.role):");
                    let role_response = ui.text_edit_singleline(&mut self.ansible_role_input);
                    ui.label("Example: serhii9132.wireguard");
                    
                    // Validate on blur
                    if role_response.lost_focus() && !self.ansible_role_input.is_empty() {
                        if let Some(vm) = vm {
                            self.ansible_validating = true;
                            self.ansible_validation_error = None;
                            vm.validate_ansible_role(self.ansible_role_input.clone());
                        }
                    }

                    if self.ansible_validating {
                        ui.spinner();
                        ui.label("Fetching role metadata...");
                    }

                    if let Some(error) = &self.ansible_validation_error {
                        ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
                    }

                    // Show metadata if validated
                    if let Some(metadata) = &self.ansible_metadata {
                        ui.separator();
                        ui.label(format!("Description: {}", metadata.description));
                        
                        if !metadata.dependencies.is_empty() {
                            ui.label(format!("Dependencies: {}", metadata.dependencies.join(", ")));
                        }
                    }

                    // Instance name
                    ui.label("Instance Name:");
                    ui.text_edit_singleline(&mut self.ansible_instance_name);
                    ui.label("(Unique identifier for this role instance)");

                    // Variables
                    if !self.ansible_variables.is_empty() {
                        ui.separator();
                        ui.label("Role Variables:");
                        
                        for (key, value) in &mut self.ansible_variables {
                            ui.horizontal(|ui| {
                                ui.label(format!("{}:", key));
                                ui.text_edit_singleline(value);
                            });
                        }
                    }

                    // Ports
                    if !self.ansible_ports.is_empty() || self.ansible_metadata.is_some() {
                        ui.separator();
                        ui.label("Ports (services managed by this role):");
                        
                        let mut to_remove = None;
                        for (idx, port) in self.ansible_ports.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.text_edit_singleline(port);
                                if ui.button("❌").clicked() {
                                    to_remove = Some(idx);
                                }
                            });
                        }
                        
                        if let Some(idx) = to_remove {
                            self.ansible_ports.remove(idx);
                        }

                        if ui.button("➕ Add Port").clicked() {
                            self.ansible_ports.push(String::new());
                        }
                    }

                    ui.separator();

                    ui.horizontal(|ui| {
                        let can_install = self.ansible_metadata.is_some() 
                            && !self.ansible_instance_name.is_empty()
                            && !self.ansible_validating;

                        if ui.add_enabled(can_install, MaterialButton::filled("Install")).clicked() {
                            if let Some(vm) = vm {
                                if let Some(host_idx) = self.ansible_install_host_idx {
                                    let host_name = self.rows[host_idx].host.clone();
                                    
                                    // Parse ports
                                    let ports: Vec<u16> = self.ansible_ports
                                        .iter()
                                        .filter_map(|p| p.parse::<u16>().ok())
                                        .collect();

                                    vm.install_ansible_role(
                                        host_name,
                                        self.ansible_instance_name.clone(),
                                        self.ansible_role_input.clone(),
                                        self.ansible_variables.clone(),
                                        ports,
                                    );
                                }
                            }
                        }

                        if ui.button("Cancel").clicked() {
                            open = false;
                        }
                    });
                });
            });

        if !open {
            self.show_ansible_install_dialog = false;
            self.ansible_role_input.clear();
            self.ansible_instance_name.clear();
            self.ansible_metadata = None;
            self.ansible_variables.clear();
            self.ansible_ports.clear();
            self.ansible_validating = false;
            self.ansible_validation_error = None;
        }
    }
```

- [ ] **Step 5: Add "Install Ansible Role" button to drawer**

```rust
            if ui.add(MaterialButton::outlined("Install Ansible Role").small()).clicked() {
                self.show_ansible_install_dialog = true;
                self.ansible_install_host_idx = Some(row_idx);
            }
```

- [ ] **Step 6: Call render_ansible_install_dialog in ui()**

```rust
        self.render_ansible_install_dialog(ui.ctx(), vm.as_deref_mut());
```

- [ ] **Step 7: Verify compilation**

Run: `cargo check --lib`  
Expected: No errors

- [ ] **Step 8: Commit Task 6**

```bash
git add mobile/src/ui_tabs/ssh.rs
git commit -m "feat(ssh-ui): add Ansible role install dialog

- Add Ansible dialog state fields to SshTab
- Implement render_ansible_install_dialog with validation
- Add role validation on blur (fetches Galaxy metadata)
- Add variables and ports configuration forms
- Add event handling for AnsibleRoleValidated, AnsibleRoleInstalled
- Add "Install Ansible Role" button to drawer

Ansible UI complete and functional.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 7: Dure-WSS UI Dialog

**Files:**
- Modify: `mobile/src/ui_tabs/ssh.rs`

**Interfaces:**
- Consumes:
  - `ViewModel` Dure-WSS methods from Task 4
  - `SshEvent` Dure-WSS variants from Task 3
- Produces:
  - `render_dure_wss_install_dialog()` method
  - Dure-WSS dialog state fields in `SshTab` struct
  - Start/Stop/Restart buttons in drawer

- [ ] **Step 1: Add Dure-WSS dialog state to SshTab struct**

```rust
    // Dure-WSS Install Dialog
    #[cfg_attr(feature = "serde", serde(skip))]
    show_dure_wss_install_dialog: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    dure_wss_host_idx: Option<usize>,
    #[cfg_attr(feature = "serde", serde(skip))]
    dure_wss_domain: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    dure_wss_email: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    dure_wss_channel: String,  // "stable", "dev", "beta"
    #[cfg_attr(feature = "serde", serde(skip))]
    dure_wss_variant: String,  // "headless", "gui"
```

- [ ] **Step 2: Update SshTab Default impl for Dure-WSS**

```rust
            show_dure_wss_install_dialog: false,
            dure_wss_host_idx: None,
            dure_wss_domain: String::new(),
            dure_wss_email: String::new(),
            dure_wss_channel: "stable".to_string(),
            dure_wss_variant: "headless".to_string(),
```

- [ ] **Step 3: Add Dure-WSS event handling**

```rust
                SshEvent::DureWssServiceInstalled { host_name, domain } => {
                    self.show_dure_wss_install_dialog = false;
                    self.load_rows();
                }
                SshEvent::DureWssStarted { host_name } => {
                    self.load_rows();
                }
                SshEvent::DureWssStopped { host_name } => {
                    self.load_rows();
                }
                SshEvent::DureWssUninstalled { host_name } => {
                    self.load_rows();
                }
```

- [ ] **Step 4: Implement render_dure_wss_install_dialog**

```rust
    fn render_dure_wss_install_dialog(&mut self, ctx: &egui::Context, vm: Option<&mut ViewModel>) {
        if !self.show_dure_wss_install_dialog {
            return;
        }

        let mut open = true;
        egui::Window::new("Install Dure-WSS Service")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 8.0;

                    // Domain
                    ui.label("Domain (for TLS certificate):");
                    ui.text_edit_singleline(&mut self.dure_wss_domain);
                    ui.label("Example: api.dure.one");

                    // Email
                    ui.label("Email (for ACME notifications):");
                    ui.text_edit_singleline(&mut self.dure_wss_email);
                    ui.label("Example: admin@example.com");

                    ui.separator();

                    // Channel
                    ui.label("Release Channel:");
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut self.dure_wss_channel, "stable".to_string(), "Stable");
                        ui.radio_value(&mut self.dure_wss_channel, "dev".to_string(), "Dev");
                        ui.radio_value(&mut self.dure_wss_channel, "beta".to_string(), "Beta");
                    });

                    // Variant
                    ui.label("Binary Variant:");
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut self.dure_wss_variant, "headless".to_string(), "Headless");
                        ui.radio_value(&mut self.dure_wss_variant, "gui".to_string(), "GUI");
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        let can_install = !self.dure_wss_domain.is_empty() 
                            && !self.dure_wss_email.is_empty()
                            && self.dure_wss_domain.contains('.')
                            && self.dure_wss_email.contains('@');

                        if ui.add_enabled(can_install, MaterialButton::filled("Install")).clicked() {
                            if let Some(vm) = vm {
                                if let Some(host_idx) = self.dure_wss_host_idx {
                                    let host_name = self.rows[host_idx].host.clone();
                                    
                                    vm.install_dure_wss(
                                        host_name,
                                        self.dure_wss_domain.clone(),
                                        self.dure_wss_email.clone(),
                                        self.dure_wss_channel.clone(),
                                        self.dure_wss_variant.clone(),
                                    );
                                }
                            }
                        }

                        if ui.button("Cancel").clicked() {
                            open = false;
                        }
                    });
                });
            });

        if !open {
            self.show_dure_wss_install_dialog = false;
            self.dure_wss_domain.clear();
            self.dure_wss_email.clear();
            self.dure_wss_channel = "stable".to_string();
            self.dure_wss_variant = "headless".to_string();
        }
    }
```

- [ ] **Step 5: Add Dure-WSS buttons to drawer**

In drawer operations section, add conditional buttons based on install status:

```rust
            // Dure-WSS operations
            if !row.dure_wss_enabled {
                if ui.add(MaterialButton::outlined("Install Dure-WSS").small()).clicked() {
                    self.show_dure_wss_install_dialog = true;
                    self.dure_wss_host_idx = Some(row_idx);
                }
            } else {
                if ui.add(MaterialButton::outlined("Start").small()).clicked() {
                    if let Some(vm) = vm {
                        vm.start_dure_wss(row.host.clone());
                    }
                }
                if ui.add(MaterialButton::outlined("Stop").small()).clicked() {
                    if let Some(vm) = vm {
                        vm.stop_dure_wss(row.host.clone());
                    }
                }
                if ui.add(MaterialButton::outlined("Restart").small()).clicked() {
                    if let Some(vm) = vm {
                        vm.restart_dure_wss(row.host.clone());
                    }
                }
                if ui.add(MaterialButton::outlined("Uninstall").small()).clicked() {
                    if let Some(vm) = vm {
                        vm.uninstall_dure_wss(row.host.clone());
                    }
                }
            }
```

- [ ] **Step 6: Update load_rows to set dure_wss_enabled flag**

In `load_rows()` method, when creating `SshRowData`:

```rust
                           let dure_wss_enabled = host_config.dure_wss_config.is_some();
                           
                           self.rows.push(SshRowData {
                               host: host_config.host.clone(),
                               port: host_config.port,
                               platform_name,
                               platform_type,
                               linux_detected: false,
                               linux_os: None,
                               ansible_enabled: false,
                               docker_enabled: false,
                               dure_wss_enabled,
                               // ... other fields
                           });
```

- [ ] **Step 7: Call render_dure_wss_install_dialog in ui()**

```rust
        self.render_dure_wss_install_dialog(ui.ctx(), vm.as_deref_mut());
```

- [ ] **Step 8: Verify compilation**

Run: `cargo check --lib`  
Expected: No errors

- [ ] **Step 9: Commit Task 7**

```bash
git add mobile/src/ui_tabs/ssh.rs
git commit -m "feat(ssh-ui): add Dure-WSS service install dialog

- Add Dure-WSS dialog state fields to SshTab
- Implement render_dure_wss_install_dialog with domain/email validation
- Add channel and variant selection (radio buttons)
- Add Start/Stop/Restart/Uninstall buttons to drawer
- Update load_rows to detect dure_wss_enabled status
- Add event handling for Dure-WSS lifecycle events

Dure-WSS UI complete and functional.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 8: Drawer Enhancements - Show Installed Services

**Files:**
- Modify: `mobile/src/ui_tabs/ssh.rs`

**Interfaces:**
- Consumes:
  - Config data from `load_rows()` (containers, roles, dure_wss_config)
  - Drawer rendering infrastructure (already exists)
- Produces:
  - Enhanced drawer showing installed containers, roles, and Dure-WSS status
  - Service-specific operation buttons

- [ ] **Step 1: Update SshRowData to include service details**

Add fields to `SshRowData` struct:

```rust
    // Installed services
    docker_containers: Vec<DockerContainerConfig>,
    ansible_roles: Vec<AnsibleRoleConfig>,
    dure_wss_status: Option<String>,
```

Update `SshRowData` Default impl:

```rust
            docker_containers: Vec::new(),
            ansible_roles: Vec::new(),
            dure_wss_status: None,
```

- [ ] **Step 2: Update load_rows to populate service data**

In `load_rows()` method:

```rust
                           self.rows.push(SshRowData {
                               host: host_config.host.clone(),
                               port: host_config.port,
                               platform_name,
                               platform_type,
                               linux_detected: false,
                               linux_os: None,
                               ansible_enabled: !host_config.ansible_roles.is_empty(),
                               docker_enabled: !host_config.docker_containers.is_empty(),
                               dure_wss_enabled: host_config.dure_wss_config.is_some(),
                               linux_status,
                               connection_status: ConnectionStatus::Unknown,
                               docker_containers: host_config.docker_containers.clone(),
                               ansible_roles: host_config.ansible_roles.clone(),
                               dure_wss_status: host_config.dure_wss_config.as_ref().map(|c| c.status.clone()),
                           });
```

- [ ] **Step 3: Add Docker containers section to drawer**

In the drawer content rendering, add after Linux status section:

```rust
                        // Docker Containers
                        if !row.docker_containers.is_empty() {
                            ui.separator();
                            ui.heading("Docker Containers");
                            
                            for container in &row.docker_containers {
                                ui.group(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.strong(&container.name);
                                        ui.label(format!("({}:{})", container.image, container.tag));
                                    });
                                    
                                    ui.label(format!("Status: {}", container.status));
                                    
                                    if !container.ports.is_empty() {
                                        ui.label("Ports:");
                                        for (host_port, container_port) in &container.ports {
                                            ui.label(format!("  {}:{}", host_port, container_port));
                                        }
                                    }
                                    
                                    ui.horizontal(|ui| {
                                        if ui.add(MaterialButton::outlined("Remove").small()).clicked() {
                                            if let Some(vm) = vm {
                                                vm.remove_docker_container(
                                                    row.host.clone(),
                                                    container.name.clone(),
                                                );
                                            }
                                        }
                                    });
                                });
                            }
                        }
```

- [ ] **Step 4: Add Ansible roles section to drawer**

```rust
                        // Ansible Roles
                        if !row.ansible_roles.is_empty() {
                            ui.separator();
                            ui.heading("Ansible Roles");
                            
                            for role in &row.ansible_roles {
                                ui.group(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.strong(&role.name);
                                        ui.label(format!("({})", role.galaxy_name));
                                    });
                                    
                                    ui.label(format!("Installed: {}", role.installed));
                                    
                                    if !role.ports.is_empty() {
                                        ui.label(format!("Ports: {}", role.ports.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", ")));
                                    }
                                    
                                    if !role.variables.is_empty() {
                                        ui.label("Variables:");
                                        for (key, value) in &role.variables {
                                            ui.label(format!("  {} = {}", key, value));
                                        }
                                    }
                                    
                                    ui.horizontal(|ui| {
                                        if ui.add(MaterialButton::outlined("Remove").small()).clicked() {
                                            if let Some(vm) = vm {
                                                vm.remove_ansible_role(
                                                    row.host.clone(),
                                                    role.name.clone(),
                                                );
                                            }
                                        }
                                    });
                                });
                            }
                        }
```

- [ ] **Step 5: Add Dure-WSS status section to drawer**

```rust
                        // Dure-WSS Service
                        if let Some(status) = &row.dure_wss_status {
                            ui.separator();
                            ui.heading("Dure-WSS Service");
                            
                            ui.label(format!("Status: {}", status));
                            
                            // Control buttons already added in Task 7
                        }
```

- [ ] **Step 6: Verify compilation**

Run: `cargo check --lib`  
Expected: No errors

- [ ] **Step 7: Test UI manually**

Run: `cargo run`  
Test:
1. Add SSH host
2. Click to expand drawer
3. Verify sections show correctly
4. Install Docker image
5. Verify container appears in drawer
6. Remove container
7. Verify container removed from drawer

- [ ] **Step 8: Commit Task 8**

```bash
git add mobile/src/ui_tabs/ssh.rs
git commit -m "feat(ssh-ui): enhance drawer to show installed services

- Update SshRowData with docker_containers, ansible_roles, dure_wss_status
- Update load_rows to populate service data from config
- Add Docker containers section to drawer with remove buttons
- Add Ansible roles section to drawer with remove buttons
- Add Dure-WSS status section to drawer
- Show ports, env vars, and variables for each service

Drawer enhancements complete - full service visibility.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 9: Documentation and Final Polish

**Files:**
- Modify: `docs/MVVM_MIGRATION_STATUS.md`
- Create: `docs/SSH_SERVICE_MANAGEMENT.md` (user documentation)

**Interfaces:**
- Consumes: All implemented features from Tasks 1-8
- Produces: Updated documentation, user guide

- [ ] **Step 1: Update MVVM_MIGRATION_STATUS.md**

Edit `docs/MVVM_MIGRATION_STATUS.md`, update SSH Tab section:

```markdown
- ✅ **Task 11**: SSH Tab ViewModel Integration
  - **Status**: Complete - full service lifecycle management
  - **File**: `mobile/src/ui_tabs/ssh.rs`
  - ui() accepts `Option<&mut ViewModel>` ✅
  - DureApp passes viewmodel.as_mut() ✅
  - Event processing pattern implemented ✅
  - **Completed Operations** (22 total):
    - ✅ SSH host add/delete/init/test
    - ✅ Linux status retrieval
    - ✅ Docker: validate image, install daemon, install image, remove container, list containers
    - ✅ Ansible: validate role, install daemon, install role, remove role, list roles
    - ✅ Dure-WSS: install service, start, stop, restart, uninstall
  - **UI Features**:
    - ✅ Modal dialogs for Docker/Ansible/Dure-WSS installation
    - ✅ API validation with metadata fetching (Docker Hub, Ansible Galaxy)
    - ✅ Port conflict detection across all services
    - ✅ Dependency auto-install with user confirmation
    - ✅ Drawer shows installed containers, roles, and Dure-WSS status
    - ✅ Service-specific operation buttons (remove, start, stop, restart)
  - **Calc Layer**:
    - ✅ calc::docker - Docker Hub API, container management
    - ✅ calc::ansible - Ansible Galaxy API, role management
    - ✅ calc::dure_wss - Dure-WSS service management
  - **Config Persistence**:
    - ✅ DockerContainerConfig, AnsibleRoleConfig, DureWssConfig
    - ✅ SshHostConfig extended with service fields
```

Update overall stats:

```markdown
**Status:** 
- ✅ **Actor Layer**: 100% complete (50+ operations including service lifecycle)
- ✅ **UI Migration**: 75% complete (22/30 operations - full Docker/Ansible/Dure-WSS)
```

- [ ] **Step 2: Create user documentation**

Create `docs/SSH_SERVICE_MANAGEMENT.md`:

```markdown
# SSH Service Management

Complete guide for managing Docker, Ansible, and Dure-WSS services on SSH hosts.

## Overview

The SSH tab provides full lifecycle management for:
- **Docker** - Install daemon, run containers from Docker Hub
- **Ansible** - Install daemon, deploy roles from Ansible Galaxy
- **Dure-WSS** - Install and manage Dure WebSocket Secure service

## Adding an SSH Host

1. Click **"Add Host"** in SSH tab
2. Enter SSH connection details (host, port, credentials)
3. Click **"Add"** to save
4. Host appears in table

## Installing Docker Containers

### Prerequisites
- SSH host added
- Internet access on remote host

### Steps

1. Click host row to expand drawer
2. Click **"Install Docker Image"**
3. Enter Docker Hub image name (e.g., `linuxserver/wireguard`)
4. Wait for validation (metadata fetched from Docker Hub)
5. Configure:
   - **Container Name**: Unique identifier (e.g., `wireguard-vpn`)
   - **Tag**: Select from available tags (default: `latest`)
   - **Port Mappings**: Map host ports to container ports
   - **Environment Variables**: Set required env vars
6. Click **"Install"**
7. If Docker not installed:
   - Confirmation dialog appears
   - Click **"Install Docker"** to auto-install daemon (~2 min)
8. Container installs and appears in drawer

### Port Conflict Detection

- Port conflicts detected automatically
- Error shown if port already in use
- Solution: Choose different host port or remove conflicting service

## Installing Ansible Roles

### Prerequisites
- SSH host added
- Debian/Ubuntu-based Linux (for Ansible PPA)

### Steps

1. Click host row to expand drawer
2. Click **"Install Ansible Role"**
3. Enter Galaxy role (e.g., `serhii9132.wireguard`)
4. Wait for validation (metadata fetched from Ansible Galaxy)
5. Configure:
   - **Instance Name**: Unique identifier (e.g., `wireguard-main`)
   - **Variables**: Set role variables (auto-populated from metadata)
   - **Ports**: Specify ports used by services (for conflict detection)
6. Click **"Install"**
7. If Ansible not installed:
   - Auto-installs Ansible via PPA (~1 min)
8. Role installs, playbook runs, appears in drawer

## Installing Dure-WSS Service

### Prerequisites
- SSH host added
- Debian/Ubuntu-based Linux
- Valid domain pointing to host IP

### Steps

1. Click host row to expand drawer
2. Click **"Install Dure-WSS"**
3. Configure:
   - **Domain**: FQDN for TLS cert (e.g., `api.dure.one`)
   - **Email**: For ACME notifications (e.g., `admin@example.com`)
   - **Channel**: Release channel (stable/dev/beta)
   - **Variant**: Binary type (headless/gui)
4. Click **"Install"**
5. Service downloads via curl script
6. Automatically configures domain and email
7. Starts service
8. Status appears in drawer

### Managing Dure-WSS

After installation, drawer shows control buttons:
- **Start**: Start service
- **Stop**: Stop service
- **Restart**: Restart service
- **Uninstall**: Remove service completely

## Viewing Installed Services

Expand drawer to see:

### Docker Containers
- Container name and image
- Status (running/stopped/error)
- Port mappings
- **Remove** button

### Ansible Roles
- Role name and Galaxy namespace
- Installation status
- Variables and ports
- **Remove** button

### Dure-WSS Service
- Service status (running/stopped/not_installed)
- Control buttons (start/stop/restart/uninstall)

## Troubleshooting

### Docker Image Not Found
- **Error**: "Image 'linuxserver/invalid' not found on Docker Hub"
- **Solution**: Check spelling, visit hub.docker.com to search

### Port Already in Use
- **Error**: "Port 8080 already in use by container 'nginx'"
- **Solution**: Remove conflicting container or choose different port

### Ansible Role Not Found
- **Error**: "Role 'invalid.role' not found on Ansible Galaxy"
- **Solution**: Check spelling, visit galaxy.ansible.com to search

### Docker Daemon Install Failed
- **Error**: "Failed to start Docker service: permission denied"
- **Solution**: SSH into host, run `sudo systemctl start docker`, check logs with `sudo journalctl -u docker`

### Network Timeout
- **Error**: "Connection timeout after 30s"
- **Solution**: Check network connectivity, retry when stable

## Best Practices

1. **Unique Names**: Use descriptive, unique names for containers and role instances
2. **Port Planning**: Document port allocations to avoid conflicts
3. **Testing**: Test containers with small images (nginx, alpine) before complex deployments
4. **Backups**: Export container volumes before removing containers
5. **Updates**: Monitor Docker Hub / Ansible Galaxy for role updates
6. **Security**: Use official images when possible, review role source code

## API Integration

### Docker Hub API
- Validates images exist
- Fetches available tags
- Shows image description

### Ansible Galaxy API
- Validates roles exist
- Fetches role metadata (variables, dependencies)
- Shows role description

Metadata cached for 5 minutes to reduce API calls.
```

- [ ] **Step 3: Run final tests**

Run: `cargo test --lib`  
Expected: All calc layer tests pass

Run: `cargo check --all-features`  
Expected: No errors

Run: `cargo clippy --all-features`  
Expected: No critical warnings

- [ ] **Step 4: Manual testing checklist**

Test all workflows:
- [ ] Install Docker image (new host, Docker not installed)
- [ ] Install Docker image (Docker already installed)
- [ ] Port conflict detection (try same port twice)
- [ ] Remove Docker container
- [ ] Install Ansible role (Ansible not installed)
- [ ] Install Ansible role (Ansible already installed)
- [ ] Remove Ansible role
- [ ] Install Dure-WSS
- [ ] Start/Stop/Restart Dure-WSS
- [ ] Uninstall Dure-WSS
- [ ] Drawer shows all services correctly
- [ ] Config persists across app restarts

- [ ] **Step 5: Commit Task 9**

```bash
git add docs/MVVM_MIGRATION_STATUS.md docs/SSH_SERVICE_MANAGEMENT.md
git commit -m "docs: update MVVM status and add SSH service management guide

- Update MVVM_MIGRATION_STATUS.md with full SSH service lifecycle
- Add comprehensive user documentation for Docker/Ansible/Dure-WSS
- Document installation procedures, troubleshooting, best practices
- Update completion stats: 75% UI migration (22/30 operations)

SSH service lifecycle feature complete and documented.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Plan Complete

This comprehensive implementation plan covers all 9 tasks for full SSH service lifecycle management:

1. ✅ Config Model + Docker Calc Layer (TDD)
2. ✅ Ansible + Dure-WSS Calc Layer (TDD)
3. ✅ Actor Layer - Commands, Events, Handlers
4. ✅ ViewModel Public API
5. ✅ Docker UI Dialog
6. ✅ Ansible UI Dialog
7. ✅ Dure-WSS UI Dialog
8. ✅ Drawer Enhancements
9. ✅ Documentation and Final Polish

**Total Steps**: ~140 bite-sized steps across 9 tasks  
**Estimated Time**: 8-12 hours of focused implementation  
**Test Coverage**: Unit tests for calc layer, manual testing for UI

**Next Steps**: Execute this plan using one of the recommended approaches below.

---
