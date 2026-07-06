# SSH Service Lifecycle Management - Design Specification

**Date:** 2026-07-05  
**Status:** Approved  
**Goal:** Implement full lifecycle management for Docker, Ansible, and Dure-WSS services on SSH hosts with form-based configuration, API validation, port mapping, and automated dependency installation.

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Data Model](#data-model)
3. [API Integration](#api-integration)
4. [Component Design](#component-design)
5. [Data Flow](#data-flow)
6. [Error Handling](#error-handling)
7. [Testing Strategy](#testing-strategy)
8. [Implementation Phases](#implementation-phases)

---

## Architecture Overview

### Layer Structure

```
┌─────────────────────────────────────────────────────┐
│ UI Layer (ssh.rs)                                   │
│  ├── Modal Dialogs (InstallDockerDialog, etc.)     │
│  ├── Drawer (service status, operations buttons)   │
│  └── Event Handlers (poll ViewModel events)        │
└────────────────┬────────────────────────────────────┘
                 │ ViewModel API calls
┌────────────────▼────────────────────────────────────┐
│ ViewModel (viewmodel/mod.rs)                        │
│  └── Public methods:                                │
│      - install_docker_image(host, image, config)    │
│      - install_ansible_role(host, role, vars)       │
│      - install_dure_wss(host, domain, email, ...)   │
└────────────────┬────────────────────────────────────┘
                 │ Send commands
┌────────────────▼────────────────────────────────────┐
│ SshActor (viewmodel/ssh/actor.rs)                   │
│  └── Command handlers route to calc layer           │
└────────────────┬────────────────────────────────────┘
                 │ Call service managers
┌────────────────▼────────────────────────────────────┐
│ Calc Layer (service-specific modules)               │
│  ├── calc::docker (Docker Hub API, containers)      │
│  ├── calc::ansible (Galaxy API, roles)              │
│  ├── calc::dure_wss (install script, systemd)       │
│  └── calc::ssh (SSH command execution)              │
└────────────────┬────────────────────────────────────┘
                 │ SSH commands
┌────────────────▼────────────────────────────────────┐
│ Remote Host                                          │
│  ├── Docker daemon + containers                     │
│  ├── Ansible daemon + roles                         │
│  └── Dure-WSS service                               │
└──────────────────────────────────────────────────────┘
```

### Key Principles

1. **Single Responsibility**: Each calc module handles one service type (docker, ansible, dure_wss)
2. **API-First Validation**: Fetch metadata from Docker Hub / Ansible Galaxy before installation
3. **Dependency Chaining**: Auto-install daemon if missing (Docker, Ansible) with user confirmation
4. **Port Conflict Prevention**: Validate ports against all services before executing remote commands
5. **Config Persistence**: All installed services saved to `config.yaml` (extends `SshHostConfig`)

### Design Decisions

**Chosen Approach:** Hybrid - SSH Actor + Service Manager Modules

- **SSH Actor** handles async command orchestration
- **Calc modules** contain service-specific business logic
- **No separate service actors** (pragmatic, matches existing Platform/NS pattern)
- **Benefits:** Manageable complexity, clear boundaries, testable calc layer

---

## Data Model

### Config Extensions

**Extend `SshHostConfig` in `mobile/src/config.rs`:**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshHostConfig {
    // ... existing fields (host, port, password, private_key_path, etc.) ...
    
    /// Docker containers installed on this host
    #[serde(default)]
    pub docker_containers: Vec<DockerContainerConfig>,
    
    /// Ansible roles installed on this host
    #[serde(default)]
    pub ansible_roles: Vec<AnsibleRoleConfig>,
    
    /// Dure-WSS service configuration
    #[serde(default)]
    pub dure_wss_config: Option<DureWssConfig>,
}

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

### Port Conflict Detection

**Port Allocation Tracking:**
- All allocated ports stored in config (Docker containers + Ansible roles)
- Before installation, validate: `new_port` not in `existing_ports`
- Error if conflict detected with service name

**Port Collection Logic:**
```rust
fn get_allocated_ports(host_config: &SshHostConfig) -> HashSet<u16> {
    let mut ports = HashSet::new();
    
    // Docker container ports
    for container in &host_config.docker_containers {
        for (host_port, _) in &container.ports {
            ports.insert(*host_port);
        }
    }
    
    // Ansible role ports
    for role in &host_config.ansible_roles {
        for port in &role.ports {
            ports.insert(*port);
        }
    }
    
    ports
}
```

---

## API Integration

### Docker Hub API Client

**New file: `mobile/src/calc/docker.rs`**

```rust
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
fn parse_docker_image(image: &str) -> Result<(&str, &str)> {
    if let Some((namespace, name)) = image.split_once('/') {
        Ok((namespace, name))
    } else {
        // No slash = official image from library
        Ok(("library", image))
    }
}

/// Fetch image metadata from Docker Hub API
pub async fn fetch_docker_image_metadata(image: &str) -> Result<DockerImageMetadata> {
    let (namespace, name) = parse_docker_image(image)?;
    
    // Call Docker Hub API v2
    let url = format!(
        "https://hub.docker.com/v2/repositories/{}/{}",
        namespace, name
    );
    
    let response: serde_json::Value = ureq::get(&url)
        .call()
        .context("Failed to fetch from Docker Hub")?
        .into_json()?;
    
    // Extract tags (limit to 10 most recent)
    let tags = response["results"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .take(10)
                .filter_map(|t| t["name"].as_str().map(String::from))
                .collect()
        })
        .unwrap_or_else(|| vec!["latest".to_string()]);
    
    // Parse Dockerfile for EXPOSE directives (if available)
    // For now, return empty - requires fetching Dockerfile or layer inspection
    let exposed_ports = vec![];
    
    // Parse ENV directives
    let env_vars = vec![];
    
    Ok(DockerImageMetadata {
        name: format!("{}/{}", namespace, name),
        description: response["description"]
            .as_str()
            .unwrap_or("")
            .to_string(),
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
    
    // Build docker run command
    let mut run_cmd = format!(
        "docker run -d --name {} --restart unless-stopped",
        config.name
    );
    
    // Add port mappings
    for (host_port, container_port) in &config.ports {
        run_cmd.push_str(&format!(" -p {}:{}", host_port, container_port));
    }
    
    // Add environment variables
    for (key, value) in &config.env {
        run_cmd.push_str(&format!(" -e {}={}", key, value));
    }
    
    // Add image
    run_cmd.push_str(&format!(" {}:{}", config.image, config.tag));
    
    // Execute
    ssh::execute_command(host_config, &run_cmd).await
}

/// Stop and remove Docker container
pub async fn remove_docker_container(
    host_config: &SshHostConfig,
    container_name: &str,
) -> Result<String> {
    // Stop container
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

/// Build docker run command (for testing)
pub fn build_docker_run_command(config: &DockerContainerConfig) -> String {
    let mut cmd = format!("docker run -d --name {} --restart unless-stopped", config.name);
    
    for (host_port, container_port) in &config.ports {
        cmd.push_str(&format!(" -p {}:{}", host_port, container_port));
    }
    
    for (key, value) in &config.env {
        cmd.push_str(&format!(" -e {}={}", key, value));
    }
    
    cmd.push_str(&format!(" {}:{}", config.image, config.tag));
    cmd
}
```

### Ansible Galaxy API Client

**New file: `mobile/src/calc/ansible.rs`**

```rust
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
fn parse_galaxy_role(role: &str) -> Result<(&str, &str)> {
    role.split_once('.')
        .ok_or_else(|| anyhow::anyhow!("Invalid Galaxy role format. Expected 'namespace.role'"))
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
    
    // Parse variables from role metadata (if available)
    // For now, return empty - would need to fetch role files
    let variables = vec![];
    
    // Parse dependencies
    let dependencies = role_data["summary_fields"]["dependencies"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|d| d.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    
    Ok(AnsibleRoleMetadata {
        name: format!("{}.{}", namespace, name),
        description: role_data["description"]
            .as_str()
            .unwrap_or("")
            .to_string(),
        variables,
        dependencies,
        suggested_ports: vec![],
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

### Dure-WSS Service Manager

**New file: `mobile/src/calc/dure_wss.rs`**

```rust
use anyhow::{Context, Result};
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
    // Stop service
    let _ = ssh::execute_command(host_config, "dure wss stop").await;
    
    // Remove binary and config
    ssh::execute_command(host_config, "sudo rm -f /usr/local/bin/dure").await?;
    ssh::execute_command(host_config, "sudo rm -rf ~/.config/dure").await?;
    
    Ok("Dure-WSS uninstalled".to_string())
}
```

### API Response Caching

**In-memory cache to reduce API calls:**

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

struct MetadataCache<T> {
    cache: HashMap<String, (T, Instant)>,
    ttl: Duration,
}

impl<T: Clone> MetadataCache<T> {
    fn new() -> Self {
        Self {
            cache: HashMap::new(),
            ttl: Duration::from_secs(300), // 5 minutes
        }
    }
    
    fn get(&self, key: &str) -> Option<T> {
        if let Some((value, timestamp)) = self.cache.get(key) {
            if timestamp.elapsed() < self.ttl {
                return Some(value.clone());
            }
        }
        None
    }
    
    fn insert(&mut self, key: String, value: T) {
        self.cache.insert(key, (value, Instant::now()));
    }
}
```

---

## Component Design

### SSH Actor - New Commands

**File: `mobile/src/viewmodel/ssh/commands.rs`**

```rust
#[derive(Debug, Clone)]
pub enum SshCommand {
    // ... existing commands (AddHost, DeleteHost, etc.) ...
    
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
}
```

### SSH Actor - New Events

**File: `mobile/src/viewmodel/ssh/events.rs`**

```rust
use crate::calc::docker::DockerImageMetadata;
use crate::calc::ansible::AnsibleRoleMetadata;
use crate::config::{DockerContainerConfig, AnsibleRoleConfig};

#[derive(Debug, Clone)]
pub enum SshEvent {
    // ... existing events ...
    
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
    
    // Enhanced Progress + Error
    Progress {
        operation: String,
        progress: f32,    // 0.0 - 1.0
        status: String,
    },
    Error {
        operation: String,
        stage: String,
        error: String,
        recovered_state: String,
        suggested_action: String,
    },
}
```

### UI Dialog State

**File: `mobile/src/ui_tabs/ssh.rs`**

Add to `SshTab` struct:

```rust
pub struct SshTab {
    // ... existing fields ...
    
    // Docker Install Dialog
    show_docker_install_dialog: bool,
    docker_install_host_idx: Option<usize>,
    docker_image_input: String,
    docker_container_name: String,
    docker_tag: String,
    docker_metadata: Option<DockerImageMetadata>,
    docker_port_mappings: Vec<(String, String)>, // (host_port, container_port)
    docker_env_vars: Vec<(String, String)>,
    docker_validating: bool,
    docker_validation_error: Option<String>,
    
    // Ansible Install Dialog
    show_ansible_install_dialog: bool,
    ansible_install_host_idx: Option<usize>,
    ansible_role_input: String,
    ansible_instance_name: String,
    ansible_metadata: Option<AnsibleRoleMetadata>,
    ansible_variables: Vec<(String, String)>,
    ansible_ports: Vec<String>,
    ansible_validating: bool,
    ansible_validation_error: Option<String>,
    
    // Dure-WSS Install Dialog
    show_dure_wss_install_dialog: bool,
    dure_wss_host_idx: Option<usize>,
    dure_wss_domain: String,
    dure_wss_email: String,
    dure_wss_channel: String,  // "stable", "dev", "beta"
    dure_wss_variant: String,  // "headless", "gui"
    
    // Dependency Install Confirmation
    show_dependency_confirm_dialog: bool,
    dependency_service: String, // "Docker", "Ansible"
    dependency_host_idx: Option<usize>,
    pending_install_action: Option<PendingInstall>,
}

#[derive(Clone)]
enum PendingInstall {
    DockerImage {
        container_name: String,
        image: String,
        tag: String,
        ports: Vec<(u16, u16)>,
        env: Vec<(String, String)>,
    },
    AnsibleRole {
        instance_name: String,
        galaxy_name: String,
        variables: Vec<(String, String)>,
        ports: Vec<u16>,
    },
}
```

---

## Data Flow

### Flow 1: Install Docker Image

```
1. User Opens Dialog
   └─> Clicks "Install Docker Image" in drawer operations
       └─> Sets show_docker_install_dialog = true
       └─> Captures docker_install_host_idx

2. Image Validation
   └─> User types "linuxserver/wireguard" in image field
       └─> On blur event:
           ├─> Set docker_validating = true
           └─> vm.validate_docker_image("linuxserver/wireguard")
               └─> SshActor receives: ValidateDockerImage { image }
                   └─> calc::docker::fetch_docker_image_metadata()
                       └─> HTTP GET hub.docker.com/v2/repositories/linuxserver/wireguard
                           ├─> Success: metadata { tags, exposed_ports [51820], env_vars [PEERS] }
                           │   └─> Event: DockerImageValidated { image, metadata }
                           │       └─> UI receives event:
                           │           ├─> Set docker_metadata = Some(metadata)
                           │           ├─> Set docker_validating = false
                           │           ├─> Pre-populate docker_port_mappings with exposed_ports
                           │           └─> Pre-populate docker_env_vars with env_vars
                           └─> Error: image not found
                               └─> Event: Error { operation: "ValidateDockerImage", error }
                                   └─> UI: docker_validation_error = Some(error)

3. Port Mapping Configuration
   └─> Form shows detected ports: 51820 (UDP)
       ├─> User can add/remove/modify port mappings
       └─> On change, validate against existing services:
           └─> get_allocated_ports(host_config)
               ├─> Conflict detected → show error, disable submit
               └─> No conflict → enable submit

4. Environment Variables Configuration
   └─> Form shows detected env vars: PEERS (default: 1)
       └─> User can modify values or add new vars

5. Dependency Check
   └─> User clicks "Install" button
       └─> Check: is Docker installed on host?
           ├─> docker_enabled flag in SshRowData
           └─> If false:
               ├─> Set show_dependency_confirm_dialog = true
               ├─> Set dependency_service = "Docker"
               ├─> Set pending_install_action = DockerImage { ... }
               └─> Show: "Docker not installed. Install Docker first? (~2 min)"
                   ├─> User clicks "Cancel" → close all dialogs
                   └─> User clicks "Install Docker" →
                       └─> vm.install_docker(host_name)
                           └─> SshActor: InstallDocker { host_name }
                               └─> calc::docker::install_docker_daemon()
                                   ├─> SSH: curl -fsSL https://get.docker.com -o get-docker.sh
                                   │   └─> Progress: 25%
                                   ├─> SSH: sudo sh get-docker.sh
                                   │   └─> Progress: 50%
                                   ├─> SSH: sudo systemctl enable docker
                                   │   └─> Progress: 75%
                                   └─> SSH: sudo systemctl start docker
                                       └─> Progress: 100%
                                           └─> Event: DockerDaemonInstalled { host_name }
                                               └─> UI updates docker_enabled = true
                                                   └─> Proceed to image installation

6. Image Installation
   └─> vm.install_docker_image(host_name, container_name, image, tag, ports, env)
       └─> SshActor: InstallDockerImage { ... }
           └─> Validate ports again (server-side check)
               ├─> Conflict → Event: Error { operation, error: "Port conflict" }
               └─> No conflict → proceed
                   └─> calc::docker::run_docker_container()
                       ├─> SSH: docker pull linuxserver/wireguard:latest
                       │   └─> Progress: "Pulling image... 45%"
                       ├─> SSH: docker run -d --name wireguard-main -p 51820:51820/udp -e PEERS=1 ...
                       │   └─> Progress: "Starting container..."
                       └─> Event: DockerImageInstalled { host_name, container_name }
                           └─> UI:
                               ├─> Close dialogs
                               ├─> Update config: host.docker_containers.push(container)
                               ├─> Refresh drawer to show new container
                               └─> Show success notification

7. Error Handling (Example: Pull Fails)
   └─> If docker pull times out:
       └─> Event: Error {
               operation: "InstallDockerImage",
               stage: "PullingImage",
               error: "Connection timeout after 30s",
               recovered_state: "DockerDaemonReady",
               suggested_action: "Check network and retry",
           }
           └─> UI shows error with [Retry] button
               └─> Retry calls same vm.install_docker_image() again
```

### Flow 2: Install Ansible Role

```
1. User Opens Dialog
   └─> Clicks "Install Ansible Role"
       └─> show_ansible_install_dialog = true

2. Role Validation
   └─> User types "serhii9132.wireguard"
       └─> On blur: vm.validate_ansible_role("serhii9132.wireguard")
           └─> calc::ansible::fetch_ansible_role_metadata()
               └─> HTTP GET galaxy.ansible.com/api/v2/roles/?name=wireguard&namespace=serhii9132
                   ├─> Success: metadata { variables, dependencies, suggested_ports }
                   │   └─> Event: AnsibleRoleValidated { role, metadata }
                   │       └─> UI: populate ansible_variables, ansible_ports
                   └─> Error: role not found
                       └─> Event: Error { ... }

3. Variables Configuration
   └─> Form shows role variables:
       ├─> wireguard_port (default: 51820)
       ├─> wireguard_peers (default: 1)
       └─> User modifies as needed

4. Port Configuration
   └─> Form shows suggested_ports from metadata
       └─> User can add/remove ports
           └─> Port conflict validation on change

5. Dependency Check + Installation
   └─> User clicks "Install"
       └─> Check: is Ansible installed?
           ├─> No: show confirmation, auto-install Ansible
           └─> Yes: proceed
               └─> vm.install_ansible_role(host, name, galaxy_name, vars, ports)
                   └─> calc::ansible::install_ansible_role()
                       ├─> SSH: ansible-galaxy install serhii9132.wireguard
                       ├─> Generate playbook with variables
                       ├─> SSH: write playbook to /tmp/wireguard-main.yml
                       └─> SSH: ansible-playbook /tmp/wireguard-main.yml
                           └─> Event: AnsibleRoleInstalled { host_name, instance_name }

6. Status Update
   └─> Config updated: host.ansible_roles.push(role)
   └─> Drawer shows: "wireguard-main (serhii9132.wireguard) - Installed"
```

### Flow 3: Install Dure-WSS

```
1. User Opens Dialog
   └─> Clicks "Install Dure-WSS"
       └─> show_dure_wss_install_dialog = true

2. Configuration Form
   └─> Fields:
       ├─> Domain (e.g., "api.dure.one") - validated as FQDN
       ├─> Email (e.g., "user@example.com") - validated as email
       ├─> Channel (dropdown: stable, dev, beta)
       └─> Variant (dropdown: headless, gui)

3. Installation (No Dependency Check)
   └─> User clicks "Install"
       └─> vm.install_dure_wss(host, domain, email, channel, variant)
           └─> calc::dure_wss::install_dure_wss()
               ├─> SSH: curl https://run.dure.one | DURE_CHANNEL=dev DURE_VARIANT=headless sh
               │   └─> Progress: "Downloading installer..."
               ├─> SSH: dure wss config --domain api.dure.one --email user@example.com
               │   └─> Progress: "Configuring service..."
               └─> SSH: dure wss start
                   └─> Event: DureWssServiceInstalled { host_name, domain }

4. Status Update
   └─> Config: host.dure_wss_config = Some(config)
   └─> Drawer shows: "Dure-WSS (api.dure.one) - Running"
       └─> Operations: [Stop] [Restart] [Status] [Remove]
```

---

## Error Handling

### Error Categories

#### 1. Validation Errors (Pre-Installation)

**Docker image not found:**
```
Error: Image 'linuxserver/wireguardd' not found on Docker Hub.

Suggestions:
- Check spelling (did you mean 'linuxserver/wireguard'?)
- Visit hub.docker.com to search for images
```

**Ansible role not found:**
```
Error: Role 'serhii9132.wireguardd' not found on Ansible Galaxy.

Suggestions:
- Check spelling
- Visit galaxy.ansible.com to search for roles
```

**Port conflict:**
```
Error: Port 51820 already in use by container 'wireguard-vpn'.

Actions:
- Remove 'wireguard-vpn' container first
- Choose a different host port (e.g., 51821)
```

**Invalid domain:**
```
Error: Invalid domain 'api..dure.one' (consecutive dots not allowed).
```

#### 2. Dependency Installation Errors

**Docker install fails:**
```
Installing Docker:
├─ Downloaded installer... ✓
├─ Installed Docker daemon... ✓
├─ Enabled Docker service... ✓
└─ Started Docker service... ✗ Failed: permission denied

Docker is installed but not running.

Manual Steps:
1. SSH into host: ssh user@host
2. Start Docker: sudo systemctl start docker
3. Check logs: sudo journalctl -u docker

[Retry] [Cancel]
```

**Ansible install fails:**
```
Installing Ansible:
├─ Updated package list... ✓
├─ Installed prerequisites... ✓
├─ Added Ansible PPA... ✗ Failed: PPA not found

Ansible installation incomplete.

Manual Steps:
1. SSH into host
2. Add PPA manually: sudo add-apt-repository ppa:ansible/ansible
3. Retry installation

[Retry] [Cancel]
```

#### 3. Installation Errors (After Dependencies Met)

**Docker pull fails (network):**
```
Installing Docker Image:
├─ Checking Docker daemon... ✓
├─ Pulling image... ✗ Connection timeout after 30s
└─ Starting container... (not started)

Docker is ready but image pull failed.

Suggestions:
- Check network connectivity
- Try again when network is stable
- Use a different Docker registry/mirror

[Retry] [Cancel]
```

**Docker run fails (port conflict):**
```
Installing Docker Image:
├─ Checking Docker daemon... ✓
├─ Pulled image... ✓ linuxserver/wireguard:latest
└─ Starting container... ✗ Port 51820 already bound

Image is ready but container failed to start.

Conflict:
- Port 51820 is already in use on the host
- Check: sudo netstat -tulpn | grep 51820

Actions:
- Remove conflicting service
- Choose different port

[Edit Ports] [Cancel]
```

**Ansible role install fails (missing dependency):**
```
Installing Ansible Role:
├─ Checking Ansible daemon... ✓
├─ Installing role... ✗ Dependency 'geerlingguy.security' not found
└─ Running playbook... (not started)

Role installation failed.

Missing Dependencies:
- geerlingguy.security (required by serhii9132.wireguard)

Actions:
- Install dependency first: ansible-galaxy install geerlingguy.security
- Or choose a different role

[Install Dependency] [Cancel]
```

**Ansible playbook fails (variable error):**
```
Installing Ansible Role:
├─ Checking Ansible daemon... ✓
├─ Installed role... ✓ serhii9132.wireguard
└─ Running playbook... ✗ Variable 'wireguard_port' undefined

Role is installed but playbook execution failed.

Configuration Error:
- Required variable 'wireguard_port' not set

Actions:
- Set variable in form and retry
- Check role documentation for required variables

[Edit Variables] [Retry]
```

#### 4. Runtime Errors (After Installation)

**Container stopped unexpectedly:**
```
Container Status: wireguard-main
Status: Exited (1) 2 minutes ago

Last Error: "Failed to create network interface"

Actions:
- View logs: docker logs wireguard-main
- Restart: docker restart wireguard-main
- Check host capabilities (NET_ADMIN required for VPN)

[View Logs] [Restart] [Remove]
```

### Error Event Structure

```rust
pub enum SshEvent {
    Error {
        operation: String,         // "InstallDockerImage", "InstallAnsibleRole"
        stage: String,             // "PullingImage", "RunningPlaybook"
        error: String,             // "Connection timeout after 30s"
        recovered_state: String,   // "DockerDaemonReady", "RoleInstalled"
        suggested_action: String,  // "Check network and retry", "Set required variables"
    },
}
```

### UI Error Display

**During Installation (Progress Dialog):**
```
┌─────────────────────────────────────────┐
│ Installing Docker Image: wireguard-main │
├─────────────────────────────────────────┤
│ ✓ Checking Docker daemon                │
│ ⏳ Pulling image (45%)                   │
│ ⋯ Starting container                    │
│                                         │
│ [Cancel]                                │
└─────────────────────────────────────────┘
```

**On Error:**
```
┌─────────────────────────────────────────┐
│ Error: Installing Docker Image          │
├─────────────────────────────────────────┤
│ ✓ Checking Docker daemon                │
│ ✗ Pulling image                         │
│   Connection timeout after 30s          │
│ ⋯ Starting container (not started)      │
│                                         │
│ Recovered State: Docker daemon ready    │
│                                         │
│ Suggestion: Check network and retry     │
│                                         │
│ [Retry] [View Details] [Cancel]        │
└─────────────────────────────────────────┘
```

### Error Recovery Actions

**Retry Button:**
- For transient failures (network errors)
- Re-executes the same command
- Preserves user's configuration

**View Logs Button:**
- SSH into host
- Fetch service/container logs
- Display in scrollable text area

**Edit Configuration Button:**
- Return to form with current values
- Allow user to adjust (ports, variables)
- Re-validate and retry

**Remove Partial Install Button:**
- Clean up failed state
- Remove partially installed services
- Return to initial state

---

## Testing Strategy

### Unit Tests (Calc Layer)

**`mobile/src/calc/docker.rs`:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_docker_image_with_namespace() {
        assert_eq!(
            parse_docker_image("linuxserver/wireguard").unwrap(),
            ("linuxserver", "wireguard")
        );
    }

    #[test]
    fn test_parse_docker_image_official() {
        assert_eq!(
            parse_docker_image("nginx").unwrap(),
            ("library", "nginx")
        );
    }

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

    #[test]
    fn test_get_allocated_ports() {
        let mut host_config = SshHostConfig::default();
        
        host_config.docker_containers.push(DockerContainerConfig {
            name: "nginx".into(),
            image: "nginx".into(),
            tag: "latest".into(),
            ports: vec![(8080, 80), (8443, 443)],
            env: vec![],
            status: "running".into(),
        });
        
        host_config.ansible_roles.push(AnsibleRoleConfig {
            name: "wireguard".into(),
            galaxy_name: "serhii9132.wireguard".into(),
            variables: vec![],
            ports: vec![51820],
            installed: true,
        });
        
        let allocated = get_allocated_ports(&host_config);
        
        assert!(allocated.contains(&8080));
        assert!(allocated.contains(&8443));
        assert!(allocated.contains(&51820));
        assert_eq!(allocated.len(), 3);
    }
}
```

**`mobile/src/calc/ansible.rs`:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_galaxy_role_valid() {
        assert_eq!(
            parse_galaxy_role("serhii9132.wireguard").unwrap(),
            ("serhii9132", "wireguard")
        );
    }

    #[test]
    fn test_parse_galaxy_role_invalid() {
        assert!(parse_galaxy_role("invalid-role").is_err());
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
```

### Integration Tests (Actor Layer)

**`mobile/src/viewmodel/ssh/actor.rs`:**

```rust
#[cfg(test)]
mod actor_tests {
    use super::*;
    use crate::config::*;

    #[async_test]
    async fn test_install_docker_with_dependency() {
        let mut actor = SshActor::new_test();
        
        // Mock: Docker not installed
        actor.mock_ssh_response("which docker", "", 1); // exit code 1
        
        // Mock: Docker installation succeeds
        actor.mock_ssh_responses(vec![
            ("curl -fsSL https://get.docker.com", "OK"),
            ("sudo sh get-docker.sh", "OK"),
            ("sudo systemctl enable docker", "OK"),
            ("sudo systemctl start docker", "OK"),
        ]);
        
        // Mock: Docker pull succeeds
        actor.mock_ssh_response("docker pull nginx:latest", "Pull complete");
        
        // Mock: Docker run succeeds
        actor.mock_ssh_response("docker run", "container_id_123");
        
        let cmd = SshCommand::InstallDockerImage {
            host_name: "test-host".to_string(),
            container_name: "nginx-test".to_string(),
            image: "nginx".to_string(),
            tag: "latest".to_string(),
            ports: vec![(8080, 80)],
            env: vec![],
        };
        
        actor.handle_command(cmd).await;
        
        // Verify events
        assert_eq!(actor.events.len(), 3);
        assert!(matches!(actor.events[0], SshEvent::DockerDaemonInstallRequired { .. }));
        assert!(matches!(actor.events[1], SshEvent::DockerDaemonInstalled { .. }));
        assert!(matches!(actor.events[2], SshEvent::DockerImageInstalled { .. }));
        
        // Verify config updated
        assert_eq!(actor.config.ssh_hosts[0].docker_containers.len(), 1);
        assert_eq!(actor.config.ssh_hosts[0].docker_containers[0].name, "nginx-test");
    }

    #[async_test]
    async fn test_port_conflict_detection() {
        let mut actor = SshActor::new_test();
        
        // Add existing container on port 8080
        actor.config.ssh_hosts[0].docker_containers.push(DockerContainerConfig {
            name: "existing-nginx".to_string(),
            image: "nginx".to_string(),
            tag: "latest".to_string(),
            ports: vec![(8080, 80)],
            env: vec![],
            status: "running".to_string(),
        });
        
        // Try to install new container on same port
        let cmd = SshCommand::InstallDockerImage {
            host_name: "test-host".to_string(),
            container_name: "new-nginx".to_string(),
            image: "nginx".to_string(),
            tag: "alpine".to_string(),
            ports: vec![(8080, 80)], // Conflict!
            env: vec![],
        };
        
        actor.handle_command(cmd).await;
        
        // Verify error event emitted
        assert_eq!(actor.events.len(), 1);
        assert!(matches!(
            &actor.events[0],
            SshEvent::Error { error, .. } if error.contains("Port 8080 already in use")
        ));
        
        // Verify no container added
        assert_eq!(actor.config.ssh_hosts[0].docker_containers.len(), 1);
    }

    #[async_test]
    async fn test_install_ansible_role_with_variables() {
        let mut actor = SshActor::new_test();
        
        // Mock: Ansible installed
        actor.mock_ssh_response("which ansible", "/usr/bin/ansible", 0);
        
        // Mock: Role installation
        actor.mock_ssh_response("ansible-galaxy install", "installed");
        actor.mock_ssh_response("cat >", "");
        actor.mock_ssh_response("ansible-playbook", "PLAY RECAP");
        
        let cmd = SshCommand::InstallAnsibleRole {
            host_name: "test-host".to_string(),
            instance_name: "wg-main".to_string(),
            galaxy_name: "serhii9132.wireguard".to_string(),
            variables: vec![("wireguard_port".to_string(), "51820".to_string())],
            ports: vec![51820],
        };
        
        actor.handle_command(cmd).await;
        
        // Verify events
        assert!(matches!(
            &actor.events.last().unwrap(),
            SshEvent::AnsibleRoleInstalled { instance_name, .. } if instance_name == "wg-main"
        ));
        
        // Verify config
        assert_eq!(actor.config.ssh_hosts[0].ansible_roles.len(), 1);
        assert_eq!(actor.config.ssh_hosts[0].ansible_roles[0].name, "wg-main");
    }
}
```

### Manual Testing Checklist

#### Docker Lifecycle
- [ ] Validate image: valid name (linuxserver/wireguard) → metadata loads
- [ ] Validate image: invalid name (linuxserver/invalid123) → error shown
- [ ] Install Docker daemon from scratch on fresh host
- [ ] Install Docker image with Docker already installed
- [ ] Port conflict: try to use same port twice → blocked with clear error
- [ ] Port mapping: custom host:container ports work correctly
- [ ] Environment variables: passed to container and visible in `docker inspect`
- [ ] Multiple instances: same image (nginx), different names (nginx-1, nginx-2)
- [ ] Remove container: config updated, container removed from host
- [ ] Container status: verify running/stopped/error states shown correctly

#### Ansible Lifecycle
- [ ] Validate role: valid name (serhii9132.wireguard) → metadata loads
- [ ] Validate role: invalid name (invalid.role123) → error shown
- [ ] Install Ansible from scratch on fresh host
- [ ] Install role with variables: variables passed correctly to playbook
- [ ] Port allocation: role's ports tracked and conflict detection works
- [ ] Multiple roles: different namespaces, no conflicts
- [ ] Remove role: `ansible-galaxy remove` executed, config updated
- [ ] Role status: shows installed/not_installed correctly

#### Dure-WSS Lifecycle
- [ ] Install: curl script executed with correct DURE_CHANNEL and DURE_VARIANT
- [ ] Configure: domain and email passed to `dure wss config`
- [ ] Start service: `dure wss start` succeeds, status shows "running"
- [ ] Stop service: `dure wss stop` succeeds, status shows "stopped"
- [ ] Restart service: `dure wss restart` succeeds
- [ ] Status check: displays correct service state
- [ ] Uninstall: binary and config removed, status shows "not_installed"

#### Error Scenarios
- [ ] Network timeout during docker pull → partial state preserved, retry works
- [ ] Invalid port mapping (port 0, port > 65535) → validation error before SSH
- [ ] Dependency install fails (no internet) → clear error, manual steps shown
- [ ] Container crashes after start → status updates to "exited", logs viewable
- [ ] Ansible playbook syntax error → shows Ansible error output
- [ ] Conflicting port between Docker and Ansible → detected and blocked

#### UI/UX
- [ ] Modal dialogs open/close correctly
- [ ] Form validation provides immediate feedback
- [ ] Progress indicators show during operations
- [ ] Success notifications appear after installation
- [ ] Error messages are clear and actionable
- [ ] Drawer updates after installation without page refresh
- [ ] Multiple operations can be performed in sequence

### TDD Workflow

For each feature implementation:

1. **Write failing calc layer test** (e.g., `test_parse_docker_image`)
2. **Implement calc function** to make test pass
3. **Test passes** (run `cargo test calc::docker`)
4. **Write failing actor test** with mocked SSH responses
5. **Implement actor command handler**
6. **Test passes** (run `cargo test viewmodel::ssh::actor`)
7. **Wire up ViewModel public API** method
8. **Wire up UI dialog** and event handling
9. **Manual test** full flow in GUI

**Example TDD Cycle for Docker Image Installation:**

```bash
# 1. Write test (fails)
# mobile/src/calc/docker.rs
#[test]
fn test_build_docker_run_command() { ... }

# 2. Implement function
pub fn build_docker_run_command(config: &DockerContainerConfig) -> String { ... }

# 3. Test passes
cargo test test_build_docker_run_command

# 4. Write actor test (fails)
# mobile/src/viewmodel/ssh/actor.rs
#[async_test]
async fn test_install_docker_with_dependency() { ... }

# 5. Implement actor handler
async fn handle_install_docker_image(&mut self, cmd: InstallDockerImage) { ... }

# 6. Test passes
cargo test test_install_docker_with_dependency

# 7. Wire up ViewModel API
pub fn install_docker_image(&self, host: String, ...) { ... }

# 8. Wire up UI
fn render_docker_install_dialog(&mut self, ui: &mut Ui, vm: Option<&mut ViewModel>) { ... }

# 9. Manual test in GUI
cargo run
# Click "Install Docker Image", fill form, verify success
```

---

## Implementation Phases

### Phase 1: Foundation (Config + Calc Layer)

**Tasks:**
1. Extend `SshHostConfig` with new structs
2. Implement `calc::docker` module with API client and functions
3. Implement `calc::ansible` module with API client and functions
4. Implement `calc::dure_wss` module with functions
5. Write unit tests for all calc functions
6. Verify: `cargo test calc::` passes

**Deliverable:** Calc layer complete and tested

### Phase 2: Actor Layer (Commands + Events)

**Tasks:**
1. Add new commands to `ssh/commands.rs`
2. Add new events to `ssh/events.rs`
3. Implement actor command handlers in `ssh/actor.rs`
4. Add dependency detection logic
5. Add port conflict detection logic
6. Write integration tests with mocked SSH
7. Verify: `cargo test viewmodel::ssh::` passes

**Deliverable:** Actor layer complete and tested

### Phase 3: ViewModel API

**Tasks:**
1. Add public methods to `viewmodel/mod.rs`:
   - `validate_docker_image()`
   - `install_docker_image()`
   - `remove_docker_container()`
   - `validate_ansible_role()`
   - `install_ansible_role()`
   - `remove_ansible_role()`
   - `install_dure_wss()`
   - `start/stop/restart_dure_wss()`
   - `uninstall_dure_wss()`
2. Wire commands to SSH actor
3. Verify API compiles

**Deliverable:** ViewModel API complete

### Phase 4: UI Dialogs (Docker)

**Tasks:**
1. Add dialog state to `SshTab` struct
2. Implement `render_docker_install_dialog()`
3. Add image validation on blur
4. Add port mapping form
5. Add env vars form
6. Add dependency confirmation dialog
7. Wire up event handling for Docker events
8. Manual test: validate → install → success

**Deliverable:** Docker UI complete

### Phase 5: UI Dialogs (Ansible)

**Tasks:**
1. Implement `render_ansible_install_dialog()`
2. Add role validation
3. Add variables form
4. Add ports form
5. Wire up event handling
6. Manual test: validate → install → success

**Deliverable:** Ansible UI complete

### Phase 6: UI Dialogs (Dure-WSS)

**Tasks:**
1. Implement `render_dure_wss_install_dialog()`
2. Add domain/email validation
3. Add channel/variant dropdowns
4. Wire up event handling
5. Add start/stop/restart buttons to drawer
6. Manual test: install → start → stop → uninstall

**Deliverable:** Dure-WSS UI complete

### Phase 7: Drawer Enhancements

**Tasks:**
1. Update drawer to show installed containers
2. Update drawer to show installed roles
3. Update drawer to show Dure-WSS status
4. Add operation buttons for each service
5. Add status indicators (running, stopped, error)
6. Add "View Logs" functionality

**Deliverable:** Full service management in drawer

### Phase 8: Error Handling & Polish

**Tasks:**
1. Implement detailed error messages
2. Add retry functionality
3. Add progress indicators
4. Add success notifications
5. Add help links for errors
6. Manual test all error scenarios

**Deliverable:** Production-ready error handling

### Phase 9: Documentation & Testing

**Tasks:**
1. Update MVVM_MIGRATION_STATUS.md
2. Write user documentation
3. Complete manual testing checklist
4. Performance testing (API caching)
5. Final review and polish

**Deliverable:** Feature complete and documented

---

## Global Constraints

- **Rust Edition:** 2021
- **Async Runtime:** smol 2.0 (already in use)
- **HTTP Client:** ureq 2.12 with rustls (already in use)
- **MVVM Pattern:** Follow existing pattern (Platform/NS actors)
- **No OpenSSL:** Use pure Rust libraries (russh, rustls)
- **Cross-platform:** Desktop (Linux/macOS/Windows), Android, WASM
- **TDD:** Write tests before implementation
- **Config Backward Compatibility:** Use `#[serde(default)]` for new fields
- **Error Messages:** User-friendly, actionable, no technical jargon

---

## Success Criteria

1. ✅ User can install Docker images with custom port/env configuration
2. ✅ User can install Ansible roles with custom variables
3. ✅ User can install/manage Dure-WSS service
4. ✅ Dependency auto-installation works (Docker, Ansible)
5. ✅ Port conflict detection prevents errors
6. ✅ API validation provides immediate feedback
7. ✅ Error messages are clear and actionable
8. ✅ All services persist to config.yaml
9. ✅ Unit tests pass: `cargo test calc::`
10. ✅ Integration tests pass: `cargo test viewmodel::ssh::`
11. ✅ Manual testing checklist complete
12. ✅ Documentation updated

---

## Future Enhancements (Out of Scope)

- Docker Compose support (multi-container deployments)
- Ansible playbook editor (inline YAML editing)
- Service monitoring (CPU/memory usage graphs)
- Log streaming (real-time container/service logs)
- Automatic updates (detect new image versions)
- Service dependencies (start order for containers)
- Backup/restore for container volumes
- Network configuration (custom Docker networks)

---

**End of Design Specification**
