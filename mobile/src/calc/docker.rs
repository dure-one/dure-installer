//! Docker management functionality
//!
//! Provides Docker Hub API integration and container lifecycle management

use crate::calc::ssh;
use crate::config::{DockerContainerConfig, SshHostConfig};
use crate::{dure_debug, dure_error, dure_info, dure_warn};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

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
    let description = response["description"].as_str().unwrap_or("").to_string();

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

/// Install Docker daemon on remote host with extrepo
pub async fn install_docker_daemon(
    host_config: &SshHostConfig,
    profile_keyring: Option<&std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
) -> Result<Vec<String>> {
    let mut progress = Vec::new();

    // 1. Install extrepo if not exists
    progress.push("Installing extrepo...".to_string());
    ssh::execute_command(host_config, "apt install -y extrepo", profile_keyring).await?;

    // 2. Enable docker-ce in extrepo
    progress.push("Enabling docker-ce in extrepo...".to_string());
    ssh::execute_command(host_config, "extrepo enable docker-ce", profile_keyring).await?;

    // 3. Update apt
    progress.push("Updating package lists...".to_string());
    ssh::execute_command(host_config, "apt update -y", profile_keyring).await?;

    // 4. Install docker-ce
    progress.push("Installing docker-ce...".to_string());
    ssh::execute_command(host_config, "apt install -y docker-ce", profile_keyring).await?;

    // 5. Enable services
    progress.push("Enabling containerd and docker...".to_string());
    ssh::execute_command(
        host_config,
        "systemctl enable containerd docker",
        profile_keyring,
    )
    .await?;

    // 6. Start services
    progress.push("Starting containerd and docker...".to_string());
    ssh::execute_command(
        host_config,
        "systemctl start containerd docker",
        profile_keyring,
    )
    .await?;

    progress.push("Docker installed successfully".to_string());

    Ok(progress)
}

/// Detect package manager on remote host (Debian 13 only - always apt)
async fn detect_package_manager(_host_config: &SshHostConfig) -> Result<String> {
    use crate::dure_info;
    // All servers are Debian 13 - always use apt
    dure_info!("📦 Using apt package manager (Debian 13)");
    Ok("apt".to_string())
}

/// Detect which Docker package is installed (docker.io or docker-ce)
async fn detect_docker_package(
    host_config: &SshHostConfig,
    profile_keyring: Option<&std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
) -> Result<String> {
    use crate::dure_info;

    dure_info!("🔍 Detecting Docker package type...");

    // Check for docker-ce first (official Docker)
    dure_info!("  Checking for docker-ce...");
    match ssh::execute_command(host_config, "dpkg -l | grep docker-ce", profile_keyring).await {
        Ok(output) => {
            dure_info!("  ✅ Found docker-ce: {}", output.trim());
            return Ok("docker-ce".to_string());
        }
        Err(e) => {
            dure_info!("  ❌ docker-ce not found: {}", e);
        }
    }

    // Check for docker.io (Ubuntu/Debian package)
    dure_info!("  Checking for docker.io...");
    match ssh::execute_command(host_config, "dpkg -l | grep docker.io", profile_keyring).await {
        Ok(output) => {
            dure_info!("  ✅ Found docker.io: {}", output.trim());
            return Ok("docker.io".to_string());
        }
        Err(e) => {
            dure_info!("  ❌ docker.io not found: {}", e);
        }
    }

    // Check for any docker package (generic)
    dure_info!("  Checking for generic docker package...");
    match ssh::execute_command(host_config, "dpkg -l | grep docker", profile_keyring).await {
        Ok(output) => {
            dure_info!("  ✅ Found docker packages: {}", output.trim());
            // Return the first package name found
            if output.contains("docker-ce") {
                return Ok("docker-ce".to_string());
            } else if output.contains("docker.io") {
                return Ok("docker.io".to_string());
            } else {
                return Ok("docker".to_string());
            }
        }
        Err(e) => {
            dure_info!("  ❌ No docker packages found: {}", e);
        }
    }

    anyhow::bail!("No Docker package found (checked docker-ce, docker.io, docker)")
}

/// Remove Docker daemon from remote host
pub async fn remove_docker_daemon(
    host_config: &SshHostConfig,
    profile_keyring: Option<&std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
) -> Result<Vec<String>> {
    let mut progress = Vec::new();

    progress.push("Stopping Docker service...".to_string());
    let _ = ssh::execute_command(host_config, "sudo systemctl stop docker", profile_keyring).await;

    progress.push("Disabling Docker service...".to_string());
    let _ = ssh::execute_command(
        host_config,
        "sudo systemctl disable docker",
        profile_keyring,
    )
    .await;

    // Detect Docker package type (Debian 13 always has apt)
    progress.push("Detecting Docker package...".to_string());
    let docker_pkg = detect_docker_package(host_config, profile_keyring).await?;
    progress.push(format!("Found Docker package: {}", docker_pkg));

    // Build package list based on detected Docker type
    let packages = match docker_pkg.as_str() {
        "docker-ce" => "docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin docker-ce-rootless-extras",
        "docker.io" => "docker.io containerd runc",
        _ => "docker docker-engine docker.io containerd runc",
    };

    progress.push("Removing Docker packages...".to_string());
    let purge_cmd = format!("sudo apt-get purge -y {}", packages);
    progress.push(format!("Running: {}", purge_cmd));

    match ssh::execute_command(host_config, &purge_cmd, profile_keyring).await {
        Ok(output) => {
            progress.push(format!("Purge output: {}", output.trim()));
        }
        Err(e) => {
            progress.push(format!("Purge failed: {}", e));
            return Err(e);
        }
    }

    progress.push("Running autoremove...".to_string());
    match ssh::execute_command(host_config, "sudo apt-get autoremove -y", profile_keyring).await {
        Ok(output) => progress.push(format!("Autoremove output: {}", output.trim())),
        Err(e) => progress.push(format!("Autoremove failed (non-fatal): {}", e)),
    }

    progress.push("Removing Docker data directories...".to_string());
    let _ = ssh::execute_command(host_config, "sudo rm -rf /var/lib/docker", profile_keyring).await;
    let _ = ssh::execute_command(
        host_config,
        "sudo rm -rf /var/lib/containerd",
        profile_keyring,
    )
    .await;

    progress.push("Docker removed successfully".to_string());

    Ok(progress)
}

/// Check if Docker is installed and running
pub async fn is_docker_installed(host_config: &SshHostConfig) -> Result<bool> {
    match ssh::execute_command(host_config, "which docker", None).await {
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
    ssh::execute_command(host_config, &pull_cmd, None).await?;

    // Build and execute docker run command
    let run_cmd = build_docker_run_command(config);
    ssh::execute_command(host_config, &run_cmd, None).await
}

/// Stop and remove Docker container
pub async fn remove_docker_container(
    host_config: &SshHostConfig,
    container_name: &str,
) -> Result<String> {
    // Stop container (ignore errors if already stopped)
    let stop_cmd = format!("docker stop {}", container_name);
    let _ = ssh::execute_command(host_config, &stop_cmd, None).await;

    // Remove container
    let rm_cmd = format!("docker rm {}", container_name);
    ssh::execute_command(host_config, &rm_cmd, None).await
}

/// List running containers
pub async fn list_docker_containers(
    host_config: &SshHostConfig,
) -> Result<Vec<DockerContainerConfig>> {
    let output = ssh::execute_command(
        host_config,
        "docker ps -a --format '{{.Names}}|{{.Image}}|{{.Status}}'",
        None,
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
}
