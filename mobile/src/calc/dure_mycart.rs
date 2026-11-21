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

/// Parse PROSODY_ADMINS env var (comma-separated emails)
///
/// # Format
/// "admin@dure.co,user@example.com" → [("admin", "dure.co"), ("user", "example.com")]
///
/// # Returns
/// Vector of (username, domain) tuples
fn parse_prosody_admins(value: &str) -> Vec<(String, String)> {
    value
        .split(',')
        .filter_map(|email| {
            let email = email.trim();
            if email.is_empty() {
                return None;
            }

            // Split on @ to get username and domain
            if let Some((user, domain)) = email.split_once('@') {
                let user = user.trim();
                let domain = domain.trim();
                if !user.is_empty() && !domain.is_empty() {
                    return Some((user.to_string(), domain.to_string()));
                }
            }
            None
        })
        .collect()
}

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
    env_fields: &[crate::calc::env_parser::EnvField],
    image_tag: &str,
    profile_keyring: Option<&Arc<DatabaseHandle>>,
) -> Result<Vec<String>> {
    let mut progress = Vec::new();

    // Check if already exists
    let check_dir = "test -d /srv/dure-mycart && echo exists || echo missing";
    let dir_result = ssh::execute_command(host_config, check_dir, profile_keyring)
        .await
        .unwrap_or_else(|_| "missing".to_string());

    let mut need_clone = false;

    if dir_result.trim() == "exists" {
        progress.push(format!("Updating existing installation to {}...", version));

        // Try git update with proper branch/tag handling
        let update_cmd = if version == "main" || version == "test" {
            format!(
                "cd /srv/dure-mycart && git fetch origin && git checkout {} && git reset --hard origin/{}",
                version, version
            )
        } else {
            // For tags, fetch and checkout tag
            format!(
                "cd /srv/dure-mycart && git fetch --tags && git checkout {}",
                version
            )
        };

        match ssh::execute_command(host_config, &update_cmd, profile_keyring).await {
            Ok(output) => {
                progress.push(format!("✓ Updated to {}", version));
                dure_info!("Git update successful: {}", output.trim());
            }
            Err(e) => {
                progress.push(format!("Git update failed: {}, removing directory...", e));
                dure_info!("Git update failed, will clone fresh: {}", e);

                // Remove directory and clone fresh
                ssh::execute_command(host_config, "rm -rf /srv/dure-mycart", profile_keyring)
                    .await
                    .context("Failed to remove directory")?;

                need_clone = true;
            }
        }
    }

    if dir_result.trim() != "exists" || need_clone {
        progress.push("Checking prerequisites...".to_string());

        // Check/create /srv directory (root user, no sudo needed)
        let check_srv = "test -d /srv && echo exists || (mkdir -p /srv && echo created)";
        dure_debug!("SSH command: {}", check_srv);

        let srv_result = ssh::execute_command(host_config, check_srv, profile_keyring).await;
        dure_debug!("SSH result: {:?}", srv_result);

        match srv_result {
            Ok(output) => {
                progress.push(format!("✓ /srv directory: {}", output.trim()));
                dure_info!("/srv check: {}", output.trim());
            }
            Err(e) => {
                let err_msg = format!(
                    "✗ Failed to check/create /srv: {}\nCommand: {}",
                    e, check_srv
                );
                progress.push(err_msg.clone());
                dure_error!("{}", err_msg);
                return Err(anyhow::anyhow!(err_msg));
            }
        }

        // Check git
        let check_git = "which git > /dev/null && git --version || echo 'Git not installed'";
        dure_debug!("SSH command: {}", check_git);

        let git_result = ssh::execute_command(host_config, check_git, profile_keyring).await;
        dure_debug!("SSH result: {:?}", git_result);

        match git_result {
            Ok(output) => {
                if output.contains("not installed") {
                    let err_msg =
                        "✗ Git is not installed on remote host. Install with: apt install git"
                            .to_string();
                    progress.push(err_msg.clone());
                    dure_error!("{}", err_msg);
                    return Err(anyhow::anyhow!(err_msg));
                }
                progress.push(format!("✓ Git: {}", output.trim()));
                dure_info!("Git check: {}", output.trim());
            }
            Err(e) => {
                let err_msg = format!("✗ Failed to check git: {}\nCommand: {}", e, check_git);
                progress.push(err_msg.clone());
                dure_error!("{}", err_msg);
                return Err(anyhow::anyhow!(err_msg));
            }
        }

        // Test GitHub connectivity
        let test_github =
            "timeout 5 git ls-remote https://github.com/dure-one/dure-mycart.git HEAD 2>&1 || echo 'timeout'";
        dure_debug!("SSH command: {}", test_github);

        let github_result = ssh::execute_command(host_config, test_github, profile_keyring).await;
        dure_debug!("SSH result: {:?}", github_result);

        match github_result {
            Ok(output) => {
                if output.contains("timeout")
                    || output.contains("fatal")
                    || output.contains("Connection failed")
                {
                    let err_msg =
                        format!("✗ Cannot reach GitHub from remote host: {}", output.trim());
                    progress.push(err_msg.clone());
                    dure_error!("{}", err_msg);
                    return Err(anyhow::anyhow!(err_msg));
                }
                progress.push("✓ GitHub connectivity verified".to_string());
                dure_info!("GitHub connectivity: OK");
            }
            Err(e) => {
                let err_msg = format!(
                    "✗ Failed to test GitHub connection: {}\nCommand: {}",
                    e, test_github
                );
                progress.push(err_msg.clone());
                dure_error!("{}", err_msg);
                return Err(anyhow::anyhow!(err_msg));
            }
        }

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

        dure_debug!("SSH command: {}", clone_cmd);
        let clone_result = ssh::execute_command(host_config, &clone_cmd, profile_keyring).await;
        dure_debug!("SSH result: {:?}", clone_result);

        match clone_result {
            Ok(output) => {
                progress.push("✓ Repository cloned successfully".to_string());
                dure_info!("Clone successful");
                if !output.trim().is_empty() {
                    progress.push(output);
                }
            }
            Err(e) => {
                let error_msg = format!(
                    "✗ Failed to clone repository: {}\nCommand: {}",
                    e, clone_cmd
                );
                progress.push(error_msg.clone());
                dure_error!("{}", error_msg);
                return Err(anyhow::anyhow!(error_msg));
            }
        }
    }

    // Verify target directory exists
    let check_compose_dir =
        "test -d /srv/dure-mycart/prosody-mycart-stack && echo exists || echo missing";
    let dir_check = ssh::execute_command(host_config, check_compose_dir, profile_keyring)
        .await
        .context("Failed to check prosody-mycart-stack directory")?;

    if dir_check.trim() == "missing" {
        return Err(anyhow::anyhow!(
            "Directory /srv/dure-mycart/prosody-mycart-stack does not exist. Repository may be corrupt."
        ));
    }
    progress.push("✓ Target directory verified".to_string());
    dure_info!("prosody-mycart-stack directory exists");

    // Write .env file (preserves comments and order)
    progress.push("Writing configuration...".to_string());
    let env_text = crate::calc::env_parser::generate_env_file(env_fields);

    dure_info!(
        "Generated .env content: {} bytes, {} fields",
        env_text.len(),
        env_fields.len()
    );
    dure_debug!(
        ".env content preview (first 200 chars): {}",
        env_text.chars().take(200).collect::<String>()
    );

    let write_env = format!(
        "cat > /srv/dure-mycart/prosody-mycart-stack/.env << 'ENVEOF'\n{}\nENVEOF",
        env_text
    );

    dure_debug!(
        "Writing .env file with heredoc command ({} bytes)",
        write_env.len()
    );

    let write_result = ssh::execute_command(host_config, &write_env, profile_keyring).await;

    match write_result {
        Ok(output) => {
            dure_info!(".env file written successfully");
            if !output.trim().is_empty() {
                dure_debug!(".env write command output: {}", output.trim());
            }
            progress.push("✓ .env file created".to_string());
        }
        Err(e) => {
            let err_msg = format!("Failed to write .env file: {}", e);
            dure_error!("{}", err_msg);
            progress.push(format!("✗ {}", err_msg));
            return Err(anyhow::anyhow!(err_msg));
        }
    }

    // Run docker compose with appropriate flags (capture stderr)
    progress.push(format!("Starting services with {} image...", image_tag));
    let compose_cmd = match image_tag {
        "latest" => "cd /srv/dure-mycart/prosody-mycart-stack && docker compose up -d 2>&1",
        "test" => "cd /srv/dure-mycart/prosody-mycart-stack && MYCART_IMAGE_TAG=test docker compose -f docker-compose.yml up -d 2>&1",
        "dev" => "cd /srv/dure-mycart/prosody-mycart-stack && docker compose -f docker-compose.yml -f docker-compose.dev.yml up -d 2>&1",
        _ => return Err(anyhow::anyhow!("Invalid image tag: {}", image_tag)),
    };

    match ssh::execute_command(host_config, compose_cmd, profile_keyring).await {
        Ok(output) => {
            let output_trimmed = output.trim();
            // Check for common error indicators in output even if command succeeded
            if output_trimmed.contains("Error") || output_trimmed.contains("error") {
                let err_msg = format!("Docker compose failed: {}", output_trimmed);
                progress.push(format!("✗ {}", err_msg));
                Err(anyhow::anyhow!(err_msg))
            } else {
                progress.push("✓ Docker compose started".to_string());
                if !output_trimmed.is_empty() {
                    progress.push(output_trimmed.to_string());
                }

                // Post-install: Initialize dure-mycart and register prosody admins
                run_post_install_commands(host_config, env_fields, profile_keyring, &mut progress)
                    .await?;

                Ok(progress)
            }
        }
        Err(e) => {
            let err_msg = format!("Failed to start docker compose: {}", e);
            progress.push(format!("✗ {}", err_msg));
            Err(anyhow::anyhow!(err_msg))
        }
    }
}

/// Run post-installation commands after docker compose
///
/// Waits for containers to be ready, then:
/// 1. Runs dure-mycart install to initialize database and admin user
/// 2. Registers all prosody admins from PROSODY_ADMINS env var
async fn run_post_install_commands(
    host_config: &SshHostConfig,
    env_fields: &[crate::calc::env_parser::EnvField],
    profile_keyring: Option<&Arc<DatabaseHandle>>,
    progress: &mut Vec<String>,
) -> Result<()> {
    use crate::calc::env_parser::get_env_value;

    progress.push("Running post-install configuration...".to_string());

    // Extract required env vars
    let mycart_domain = get_env_value(env_fields, "MYCART_DOMAIN");
    let admin_email = get_env_value(env_fields, "MYCART_INITIAL_ADMIN_ID");
    let admin_password = get_env_value(env_fields, "MYCART_INITIAL_ADMIN_PW");
    let prosody_admin_pw = get_env_value(env_fields, "PROSODY_ADMIN_PW");
    let prosody_admins = get_env_value(env_fields, "PROSODY_ADMINS");

    // Validate required fields
    if mycart_domain.is_none() || admin_email.is_none() || admin_password.is_none() {
        progress.push("⚠ Skipping post-install: missing required env vars (MYCART_DOMAIN, MYCART_INITIAL_ADMIN_ID, MYCART_INITIAL_ADMIN_PW)".to_string());
        return Ok(());
    }

    let domain = mycart_domain.unwrap();
    let email = admin_email.unwrap();
    let password = admin_password.unwrap();

    // Wait for containers to be ready (max 30 seconds)
    progress.push("Waiting for containers to be ready...".to_string());
    let mut retries = 15; // 15 * 2s = 30s
    let mut containers_ready = false;

    while retries > 0 {
        let check_cmd = "cd /srv/dure-mycart/prosody-mycart-stack && docker compose ps --format '{{.State}}' 2>/dev/null";
        if let Ok(output) = ssh::execute_command(host_config, check_cmd, profile_keyring).await {
            // Check if all containers are "running"
            let all_running = output
                .lines()
                .filter(|line| !line.trim().is_empty())
                .all(|line| line.trim().to_lowercase().contains("running"));

            if all_running && !output.trim().is_empty() {
                containers_ready = true;
                break;
            }
        }

        // Wait 2 seconds before retry
        smol::Timer::after(std::time::Duration::from_secs(2)).await;
        retries -= 1;
    }

    if !containers_ready {
        progress.push("⚠ Containers not ready after 30s, skipping post-install".to_string());
        return Ok(());
    }

    progress.push("✓ Containers ready".to_string());

    // Run dure-mycart install command
    progress.push(format!("Initializing dure-mycart for {}...", domain));
    let install_cmd = format!(
        "docker exec dure-mycart /bin/busybox sh -c 'cd /app && /app/dure-mycart install --domain {} --email {} --password {} --db sqlite' 2>&1",
        domain, email, password
    );

    match ssh::execute_command(host_config, &install_cmd, profile_keyring).await {
        Ok(output) => {
            progress.push("✓ Dure-mycart initialized".to_string());
            dure_info!("dure-mycart install output: {}", output.trim());
            if !output.trim().is_empty() {
                // Log but don't add to progress (may be verbose)
                dure_debug!("Install output: {}", output.trim());
            }
        }
        Err(e) => {
            progress.push(format!("⚠ Failed to initialize dure-mycart: {}", e));
            dure_error!("dure-mycart install failed: {}", e);
            // Don't fail entire install, just log
        }
    }

    // Register prosody admins
    if let Some(admins_str) = prosody_admins {
        if let Some(admin_pw) = prosody_admin_pw {
            let admins = parse_prosody_admins(&admins_str);
            if !admins.is_empty() {
                progress.push(format!("Registering {} prosody admin(s)...", admins.len()));

                for (username, domain) in admins {
                    let register_cmd = format!(
                        "docker exec prosody /usr/bin/prosodyctl register {} {} {} 2>&1",
                        username, domain, admin_pw
                    );

                    match ssh::execute_command(host_config, &register_cmd, profile_keyring).await {
                        Ok(_) => {
                            progress.push(format!("✓ Registered admin: {}@{}", username, domain));
                            dure_info!("Registered prosody admin: {}@{}", username, domain);
                        }
                        Err(e) => {
                            progress.push(format!(
                                "⚠ Failed to register {}@{}: {}",
                                username, domain, e
                            ));
                            dure_error!("prosodyctl register failed: {}", e);
                            // Continue with next admin
                        }
                    }
                }
            } else {
                progress.push("⚠ PROSODY_ADMINS is empty or invalid format".to_string());
            }
        } else {
            progress.push(
                "⚠ Skipping prosody admin registration: PROSODY_ADMIN_PW not set".to_string(),
            );
        }
    } else {
        progress.push("⚠ Skipping prosody admin registration: PROSODY_ADMINS not set".to_string());
    }

    progress.push("✓ Post-install configuration complete".to_string());
    Ok(())
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

    #[test]
    fn test_parse_prosody_admins_single() {
        let result = parse_prosody_admins("admin@dure.co");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], ("admin".to_string(), "dure.co".to_string()));
    }

    #[test]
    fn test_parse_prosody_admins_multiple() {
        let result = parse_prosody_admins("admin@dure.co,user@example.com,test@test.org");
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], ("admin".to_string(), "dure.co".to_string()));
        assert_eq!(result[1], ("user".to_string(), "example.com".to_string()));
        assert_eq!(result[2], ("test".to_string(), "test.org".to_string()));
    }

    #[test]
    fn test_parse_prosody_admins_with_spaces() {
        let result = parse_prosody_admins(" admin@dure.co , user@example.com ");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], ("admin".to_string(), "dure.co".to_string()));
        assert_eq!(result[1], ("user".to_string(), "example.com".to_string()));
    }

    #[test]
    fn test_parse_prosody_admins_empty() {
        let result = parse_prosody_admins("");
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_parse_prosody_admins_invalid() {
        let result = parse_prosody_admins("invalid,no-at-sign,@nodomain,username@");
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_parse_prosody_admins_mixed_valid_invalid() {
        let result = parse_prosody_admins("admin@dure.co,invalid,user@test.com");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], ("admin".to_string(), "dure.co".to_string()));
        assert_eq!(result[1], ("user".to_string(), "test.com".to_string()));
    }

    #[test]
    fn test_env_fields_to_heredoc() {
        let env_fields = vec![
            crate::calc::env_parser::EnvField::new("MYCART_DOMAIN", "example.com"),
            crate::calc::env_parser::EnvField::new("ADMIN_EMAIL", "admin@example.com"),
            crate::calc::env_parser::EnvField::new("SPECIAL", "value$with$dollar"),
        ];

        let env_text = crate::calc::env_parser::generate_env_file(&env_fields);

        // Verify heredoc format
        let heredoc = format!(
            "cat > /srv/dure-mycart/prosody-mycart-stack/.env << 'ENVEOF'\n{}\nENVEOF",
            env_text
        );

        // Verify content is in heredoc
        assert!(heredoc.contains("MYCART_DOMAIN=example.com"));
        assert!(heredoc.contains("ADMIN_EMAIL=admin@example.com"));
        assert!(heredoc.contains("SPECIAL=value$with$dollar"));

        // Verify quoted delimiter prevents variable expansion
        assert!(heredoc.contains("'ENVEOF'"));

        // Verify heredoc structure
        assert!(
            heredoc.starts_with("cat > /srv/dure-mycart/prosody-mycart-stack/.env << 'ENVEOF'\n")
        );
        assert!(heredoc.ends_with("\nENVEOF"));
    }
}
