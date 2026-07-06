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
