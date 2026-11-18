//! Dure-mycart service management functionality
//!
//! Provides git-based installation and lifecycle management for dure-mycart

use crate::calc::keyring::DatabaseHandle;
use crate::calc::ssh;
use crate::config::SshHostConfig;
use crate::{dure_debug, dure_error, dure_info};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::Arc;

/// Container status from docker compose ps
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DureContainer {
    pub name: String,
    pub status: String,
    pub image: String,
}

/// Check dure-mycart installation and running containers
pub async fn check_dure_mycart(
    host_config: &SshHostConfig,
    profile_keyring: Option<&Arc<DatabaseHandle>>,
) -> Result<(bool, Vec<DureContainer>)> {
    // Check if directory exists
    let check_dir = "test -d /srv/dure-mycart/prosody-mycart-stack && echo exists || echo missing";
    let dir_result = ssh::execute_command(host_config, check_dir, profile_keyring)
        .await
        .context("Failed to check dure-mycart directory")?;

    if dir_result.trim() != "exists" {
        return Ok((false, Vec::new()));
    }

    // Get docker compose ps output
    let compose_ps = "cd /srv/dure-mycart/prosody-mycart-stack && docker compose ps --format '{{.Name}}|{{.Status}}|{{.Image}}' 2>/dev/null || echo ''";
    let ps_output = ssh::execute_command(host_config, compose_ps, profile_keyring)
        .await
        .unwrap_or_default();

    let mut containers = Vec::new();
    for line in ps_output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 3 {
            containers.push(DureContainer {
                name: parts[0].to_string(),
                status: parts[1].to_string(),
                image: parts[2].to_string(),
            });
        }
    }

    let installed = !containers.is_empty();
    Ok((installed, containers))
}

/// Get docker logs for a specific container
pub async fn get_container_logs(
    host_config: &SshHostConfig,
    container_name: &str,
    tail_lines: usize,
    profile_keyring: Option<&Arc<DatabaseHandle>>,
) -> Result<Vec<String>> {
    let logs_cmd = format!("docker logs {} --tail {} 2>&1", container_name, tail_lines);
    let logs = ssh::execute_command(host_config, &logs_cmd, profile_keyring)
        .await
        .unwrap_or_else(|e| format!("Failed to get logs: {}", e));

    Ok(logs.lines().map(|s| s.to_string()).collect())
}

/// Install dure-mycart with specific version and image tag
pub async fn install_dure_with_config(
    host_config: &SshHostConfig,
    version: &str,
    env_config: &HashMap<String, String>,
    image_tag: &str,
    profile_keyring: Option<&Arc<DatabaseHandle>>,
) -> Result<Vec<String>> {
    let mut progress = Vec::new();

    // Check if already exists
    let check_dir = "test -d /srv/dure-mycart && echo exists || echo missing";
    let dir_result = ssh::execute_command(host_config, check_dir, profile_keyring)
        .await
        .unwrap_or_else(|_| "missing".to_string());

    if dir_result.trim() == "exists" {
        progress.push(format!("Updating existing installation to {}...", version));

        // Checkout version
        let git_checkout = format!(
            "cd /srv/dure-mycart && git fetch && git checkout {}",
            version
        );
        ssh::execute_command(host_config, &git_checkout, profile_keyring)
            .await
            .context("Failed to checkout version")?;

        progress.push(format!("Checked out version: {}", version));
    } else {
        progress.push("Checking prerequisites...".to_string());

        // Check/create /srv directory (root user, no sudo needed)
        let check_srv = "test -d /srv && echo exists || (mkdir -p /srv && echo created)";
        let srv_result = ssh::execute_command(host_config, check_srv, profile_keyring)
            .await
            .context("Failed to check/create /srv directory")?;
        progress.push(format!("Directory /srv: {}", srv_result.trim()));

        // Check git
        let check_git = "which git > /dev/null && git --version || echo 'Git not installed'";
        let git_result = ssh::execute_command(host_config, check_git, profile_keyring)
            .await
            .context("Failed to check git")?;

        if git_result.contains("not installed") {
            return Err(anyhow::anyhow!(
                "Git is not installed on remote host. Install with: apt install git"
            ));
        }
        progress.push(format!("Git: {}", git_result.trim()));

        // Test GitHub connectivity
        let test_github =
            "timeout 5 git ls-remote https://github.com/dure-one/dure-mycart.git HEAD 2>&1 || echo 'timeout'";
        let github_result = ssh::execute_command(host_config, test_github, profile_keyring)
            .await
            .context("Failed to test GitHub connection")?;

        if github_result.contains("timeout")
            || github_result.contains("fatal")
            || github_result.contains("Connection failed")
        {
            return Err(anyhow::anyhow!(
                "Cannot reach GitHub from remote host: {}",
                github_result.trim()
            ));
        }
        progress.push("GitHub connectivity verified".to_string());

        progress.push(format!("Cloning dure-mycart {}...", version));

        let clone_cmd = if version == "main" || version == "test" {
            format!(
                "cd /srv && git clone --branch {} https://github.com/dure-one/dure-mycart.git",
                version
            )
        } else {
            format!(
                "cd /srv && git clone --branch {} --depth 1 https://github.com/dure-one/dure-mycart.git",
                version
            )
        };

        let clone_result = ssh::execute_command(host_config, &clone_cmd, profile_keyring).await;

        match clone_result {
            Ok(output) => {
                progress.push("Repository cloned successfully".to_string());
                if !output.trim().is_empty() {
                    progress.push(output);
                }
            }
            Err(e) => {
                let error_msg = format!(
                    "Failed to clone repository: {}. Command was: {}",
                    e, clone_cmd
                );
                dure_error!("{}", error_msg);
                return Err(anyhow::anyhow!(error_msg));
            }
        }
    }

    // Write .env file
    progress.push("Writing configuration...".to_string());
    let env_content: Vec<String> = env_config
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect();
    let env_text = env_content.join("\n");

    let write_env = format!(
        "cat > /srv/dure-mycart/prosody-mycart-stack/.env << 'ENVEOF'\n{}\nENVEOF",
        env_text
    );
    ssh::execute_command(host_config, &write_env, profile_keyring)
        .await
        .context("Failed to write .env file")?;

    progress.push(".env file created".to_string());

    // Run docker compose with appropriate flags
    progress.push(format!("Starting services with {} image...", image_tag));
    let compose_cmd = match image_tag {
        "latest" => "cd /srv/dure-mycart/prosody-mycart-stack && docker compose up -d",
        "test" => "cd /srv/dure-mycart/prosody-mycart-stack && docker compose -f docker-compose.yml -f docker-compose.test.yml up -d",
        "dev" => "cd /srv/dure-mycart/prosody-mycart-stack && docker compose -f docker-compose.yml -f docker-compose.dev.yml up -d",
        _ => return Err(anyhow::anyhow!("Invalid image tag: {}", image_tag)),
    };

    let compose_output = ssh::execute_command(host_config, compose_cmd, profile_keyring)
        .await
        .context("Failed to start docker compose")?;

    progress.push("Docker compose started".to_string());
    progress.push(compose_output);

    Ok(progress)
}

/// Install dure-mycart via git clone and docker compose (legacy wrapper)
pub async fn install_dure_mycart(
    host_config: &SshHostConfig,
    domain: &str,
    env_config: &HashMap<String, String>,
    profile_keyring: Option<&Arc<DatabaseHandle>>,
) -> Result<Vec<String>> {
    let mut progress = Vec::new();

    // Check if already exists - update instead of clone
    let check_dir = "test -d /srv/dure-mycart && echo exists || echo missing";
    let dir_result = ssh::execute_command(host_config, check_dir, profile_keyring)
        .await
        .unwrap_or_else(|_| "missing".to_string());

    if dir_result.trim() == "exists" {
        progress.push("Existing installation detected, updating...".to_string());

        // Check for local changes
        let git_status = "cd /srv/dure-mycart && git status --porcelain";
        let status_output = ssh::execute_command(host_config, git_status, profile_keyring)
            .await
            .unwrap_or_default();

        if !status_output.trim().is_empty() {
            progress.push("Local changes detected, stashing before update...".to_string());
            let git_stash = "cd /srv/dure-mycart && git stash";
            ssh::execute_command(host_config, git_stash, profile_keyring)
                .await
                .context("Failed to stash local changes")?;
        }

        let git_pull = "cd /srv/dure-mycart && git pull";
        ssh::execute_command(host_config, git_pull, profile_keyring)
            .await
            .context("Failed to update repository")?;
        progress.push("Repository updated successfully".to_string());
    } else {
        // Step 1: Get latest tag
        progress.push("Detecting latest dure-mycart version...".to_string());
        let tag_cmd = "git ls-remote --tags --sort=\"v:refname\" https://github.com/dure-one/dure-mycart.git | tail -n1 | sed 's/.*\\///; s/\\^{}//'";
        let tag = ssh::execute_command(host_config, tag_cmd, profile_keyring)
            .await
            .context("Failed to detect latest tag")?
            .trim()
            .to_string();

        if tag.is_empty() {
            anyhow::bail!("No tags found in repository");
        }

        dure_info!("Detected version: {}", tag);
        progress.push(format!("Latest version: {}", tag));

        // Step 2: Clone repository
        progress.push(format!("Cloning dure-mycart {}...", tag));
        let clone_cmd = format!(
            "cd /srv && git clone --branch {} --single-branch --depth 1 https://github.com/dure-one/dure-mycart.git",
            tag
        );
        ssh::execute_command(host_config, &clone_cmd, profile_keyring)
            .await
            .context("Failed to clone repository")?;

        progress.push("Repository cloned successfully".to_string());
    }

    // Step 3: Copy .env.example to .env (only if .env doesn't exist)
    progress.push("Configuring environment...".to_string());
    let check_env =
        "test -f /srv/dure-mycart/prosody-mycart-stack/.env && echo exists || echo missing";
    let env_exists = ssh::execute_command(host_config, check_env, profile_keyring)
        .await
        .unwrap_or_else(|_| "missing".to_string());

    if env_exists.trim() == "missing" {
        let copy_env = "cd /srv/dure-mycart/prosody-mycart-stack && cp .env.example .env";
        ssh::execute_command(host_config, copy_env, profile_keyring)
            .await
            .context("Failed to copy .env.example")?;
        progress.push(".env created from template".to_string());
    } else {
        progress.push("Using existing .env file".to_string());
    }

    // Step 4: Apply environment configuration (append all, docker uses last value)
    dure_debug!(
        "Applying {} environment variables to .env",
        env_config.len()
    );
    for (key, value) in env_config {
        dure_debug!("Setting {}={}", key, value);
        let append_cmd = format!(
            "cd /srv/dure-mycart/prosody-mycart-stack && echo '{}={}' >> .env",
            key,
            value.replace('\'', "'\\''") // Escape single quotes for shell
        );
        ssh::execute_command(host_config, &append_cmd, profile_keyring)
            .await
            .context(format!("Failed to set {} in .env", key))?;
    }

    // Verify what was written
    let cat_env = "cd /srv/dure-mycart/prosody-mycart-stack && tail -20 .env";
    let env_content = ssh::execute_command(host_config, cat_env, profile_keyring)
        .await
        .unwrap_or_else(|_| "Could not read .env".to_string());
    dure_debug!(".env tail:\n{}", env_content);

    // Step 5: Replace domain placeholders (if not in env_config)
    if !env_config.contains_key("DOMAIN") {
        let sed_domain = format!(
            "cd /srv/dure-mycart/prosody-mycart-stack && sed -i 's/example\\.com/{}/g' .env",
            escape_sed_replacement(domain)
        );
        ssh::execute_command(host_config, &sed_domain, profile_keyring)
            .await
            .context("Failed to replace domain")?;
    }

    progress.push(format!("Environment configured for domain: {}", domain));

    // Step 6: Stop existing containers and clean up
    progress.push("Stopping existing containers...".to_string());

    // Force remove known dure-mycart containers by explicit names
    let force_rm = "docker rm -f prosody-config-init xmpp-proxy-config-init prosody-modules-init prosody-permissions-init prosody dure-mycart nginx xmpp-proxy 2>&1 || true";
    ssh::execute_command(host_config, force_rm, profile_keyring)
        .await
        .unwrap_or_default();

    // Force down with volumes from compose directory
    let compose_down = "cd /srv/dure-mycart/prosody-mycart-stack && docker compose down -v --remove-orphans 2>&1 || true";
    ssh::execute_command(host_config, compose_down, profile_keyring)
        .await
        .unwrap_or_default();

    // Prune unused networks
    let prune_networks = "docker network prune -f 2>&1 || true";
    ssh::execute_command(host_config, prune_networks, profile_keyring)
        .await
        .unwrap_or_default();

    progress.push("Cleanup complete".to_string());

    // Step 7: Validate .env has required vars
    let check_env = "cd /srv/dure-mycart/prosody-mycart-stack && grep -E '^(username|host)=' .env || echo 'WARNING: username or host not found'";
    let env_check = ssh::execute_command(host_config, check_env, profile_keyring)
        .await
        .unwrap_or_else(|_| "Could not check .env".to_string());
    progress.push(format!(".env check: {}", env_check));

    // Step 8: Start docker compose
    progress.push("Starting services with docker compose...".to_string());

    // Determine which compose file to use based on MYCART_IMAGE_TAG
    let use_dev = env_config
        .get("MYCART_IMAGE_TAG")
        .map(|v| v == "test")
        .unwrap_or(false);

    let compose_up = if use_dev {
        "cd /srv/dure-mycart/prosody-mycart-stack && docker compose -f docker-compose.yml -f docker-compose.dev.yml up -d"
    } else {
        "cd /srv/dure-mycart/prosody-mycart-stack && docker compose -f docker-compose.yml up -d"
    };

    match ssh::execute_command(host_config, compose_up, profile_keyring).await {
        Ok(compose_output) => {
            progress.push("Docker compose output:".to_string());
            for line in compose_output.lines() {
                progress.push(format!("  {}", line));
            }
            progress.push("Dure-mycart installed and started".to_string());
        }
        Err(e) => {
            // Provide more context about the failure
            let error_msg = e.to_string();
            let check_env_vars = "cd /srv/dure-mycart/prosody-mycart-stack && cat .env | head -20";
            let env_preview = ssh::execute_command(host_config, check_env_vars, profile_keyring)
                .await
                .unwrap_or_else(|_| "Could not read .env".to_string());

            progress.push("Docker compose failed. .env preview:".to_string());
            for line in env_preview.lines().take(10) {
                progress.push(format!("  {}", line));
            }

            return Err(e).context(format!("Failed to start docker compose: {}", error_msg));
        }
    }

    Ok(progress)
}

/// Uninstall dure-mycart by stopping docker compose and optionally deleting data
pub async fn remove_dure_mycart(
    host_config: &SshHostConfig,
    delete_data: bool,
    profile_keyring: Option<&Arc<DatabaseHandle>>,
) -> Result<Vec<String>> {
    let mut progress = Vec::new();

    progress.push("Stopping dure-mycart services...".to_string());

    // Stop docker compose (ignore errors if already stopped)
    let compose_down =
        "cd /srv/dure-mycart/prosody-mycart-stack && docker compose down 2>/dev/null || true";
    match ssh::execute_command(host_config, compose_down, profile_keyring).await {
        Ok(_) => {
            progress.push("Services stopped".to_string());
        }
        Err(e) => {
            dure_debug!(
                "Compose down returned error (might be already stopped): {}",
                e
            );
            progress.push("Services stopped (or were not running)".to_string());
        }
    }

    // Delete data directories if requested
    if delete_data {
        progress.push("Deleting data directories...".to_string());

        let rm_dure = "rm -rf /srv/dure-mycart";
        match ssh::execute_command(host_config, rm_dure, profile_keyring).await {
            Ok(_) => progress.push("Removed /srv/dure-mycart".to_string()),
            Err(e) => {
                dure_error!("Failed to remove /srv/dure-mycart: {}", e);
                progress.push(format!("Failed to remove /srv/dure-mycart: {}", e));
            }
        }

        let rm_data = "rm -rf /srv/data";
        match ssh::execute_command(host_config, rm_data, profile_keyring).await {
            Ok(_) => progress.push("Removed /srv/data".to_string()),
            Err(e) => {
                dure_error!("Failed to remove /srv/data: {}", e);
                progress.push(format!("Failed to remove /srv/data: {}", e));
            }
        }

        progress.push("Data directories deleted".to_string());
    } else {
        progress.push("Data directories preserved at /srv/dure-mycart and /srv/data".to_string());
    }

    progress.push("Dure-mycart uninstalled".to_string());

    Ok(progress)
}

/// Uninstall dure-mycart by stopping docker compose (deprecated - use remove_dure_mycart)
#[deprecated(note = "Use remove_dure_mycart instead")]
pub async fn uninstall_dure_mycart(
    host_config: &SshHostConfig,
    profile_keyring: Option<&Arc<DatabaseHandle>>,
) -> Result<Vec<String>> {
    remove_dure_mycart(host_config, false, profile_keyring).await
}

/// Escape special characters for sed replacement string
fn escape_sed_replacement(s: &str) -> String {
    s.replace('/', "\\/")
        .replace('&', "\\&")
        .replace('\\', "\\\\")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_sed_replacement_simple() {
        assert_eq!(escape_sed_replacement("example.com"), "example.com");
    }

    #[test]
    fn test_escape_sed_replacement_with_slash() {
        assert_eq!(
            escape_sed_replacement("sub.domain.com/path"),
            "sub.domain.com\\/path"
        );
    }

    #[test]
    fn test_escape_sed_replacement_with_ampersand() {
        assert_eq!(
            escape_sed_replacement("user&domain.com"),
            "user\\&domain.com"
        );
    }

    #[test]
    fn test_escape_sed_replacement_with_backslash() {
        assert_eq!(escape_sed_replacement("domain\\test"), "domain\\\\test");
    }

    #[test]
    fn test_escape_sed_replacement_complex() {
        assert_eq!(
            escape_sed_replacement("test/path&value\\data"),
            "test\\/path\\&value\\\\data"
        );
    }

    #[test]
    fn test_dure_container_creation() {
        let container = DureContainer {
            name: "mycart-web".to_string(),
            status: "Up 2 hours".to_string(),
            image: "dure/mycart:latest".to_string(),
        };

        assert_eq!(container.name, "mycart-web");
        assert_eq!(container.status, "Up 2 hours");
        assert_eq!(container.image, "dure/mycart:latest");
    }
}
