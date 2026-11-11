//! Dure-mycart service management functionality
//!
//! Provides git-based installation and lifecycle management for dure-mycart

use crate::{dure_debug, dure_error, dure_info};
use anyhow::{Context, Result};
use crate::calc::keyring::DatabaseHandle;
use crate::calc::ssh;
use crate::config::SshHostConfig;
use std::sync::Arc;

/// Install dure-mycart via git clone and docker compose
pub async fn install_dure_mycart(
    host_config: &SshHostConfig,
    domain: &str,
    profile_keyring: Option<&Arc<DatabaseHandle>>,
) -> Result<Vec<String>> {
    let mut progress = Vec::new();

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

    // Step 3: Copy .env.example to .env
    progress.push("Configuring environment...".to_string());
    let copy_env = "cd /srv/dure-mycart/prosody-mycart-stack && cp .env.example .env";
    ssh::execute_command(host_config, copy_env, profile_keyring)
        .await
        .context("Failed to copy .env.example")?;

    // Step 4: Replace domain placeholders
    let sed_admin = format!(
        "cd /srv/dure-mycart/prosody-mycart-stack && sed -i 's/admin@example\\.com/admin@{}/g' .env",
        escape_sed_replacement(domain)
    );
    ssh::execute_command(host_config, &sed_admin, profile_keyring)
        .await
        .context("Failed to replace admin email")?;

    let sed_domain = format!(
        "cd /srv/dure-mycart/prosody-mycart-stack && sed -i 's/example\\.com/{}/g' .env",
        escape_sed_replacement(domain)
    );
    ssh::execute_command(host_config, &sed_domain, profile_keyring)
        .await
        .context("Failed to replace domain")?;

    // Step 5: Version-specific fixes
    if tag == "v0.0.15" {
        progress.push("Applying v0.0.15 hotfix...".to_string());
        let fix_sql = "cd /srv/dure-mycart/prosody-mycart-stack && \
            sed -i 's/^PROSODY_SQL_DRIVER=/#PROSODY_SQL_DRIVER=/g' .env && \
            sed -i 's/^PROSODY_SQL_DB=/#PROSODY_SQL_DB=/g' .env && \
            sed -i 's/^PROSODY_SQL_USERNAME=/#PROSODY_SQL_USERNAME=/g' .env && \
            sed -i 's/^PROSODY_SQL_PASSWORD=/#PROSODY_SQL_PASSWORD=/g' .env && \
            sed -i 's/^PROSODY_SQL_HOST=/#PROSODY_SQL_HOST=/g' .env";
        ssh::execute_command(host_config, fix_sql, profile_keyring)
            .await
            .context("Failed to apply v0.0.15 hotfix")?;
        dure_info!("Applied v0.0.15 SQL configuration hotfix");
    }

    progress.push(format!("Environment configured for domain: {}", domain));

    // Step 6: Start docker compose
    progress.push("Starting services with docker compose...".to_string());
    let compose_up = "cd /srv/dure-mycart/prosody-mycart-stack && docker compose up -d";
    ssh::execute_command(host_config, compose_up, profile_keyring)
        .await
        .context("Failed to start docker compose services")?;

    progress.push("✓ Dure-mycart installed and started".to_string());

    Ok(progress)
}

/// Uninstall dure-mycart by stopping docker compose
pub async fn uninstall_dure_mycart(
    host_config: &SshHostConfig,
    profile_keyring: Option<&Arc<DatabaseHandle>>,
) -> Result<Vec<String>> {
    let mut progress = Vec::new();

    progress.push("Stopping dure-mycart services...".to_string());

    // Stop docker compose (ignore errors if already stopped)
    let compose_down = "cd /srv/dure-mycart/prosody-mycart-stack && docker compose down 2>/dev/null || true";
    match ssh::execute_command(host_config, compose_down, profile_keyring).await {
        Ok(_) => {
            progress.push("✓ Services stopped".to_string());
        }
        Err(e) => {
            dure_debug!("Compose down returned error (might be already stopped): {}", e);
            progress.push("Services stopped (or were not running)".to_string());
        }
    }

    progress.push("✓ Dure-mycart uninstalled".to_string());
    progress.push("Note: Repository files remain in /srv/dure-mycart for data preservation".to_string());

    Ok(progress)
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
        assert_eq!(escape_sed_replacement("sub.domain.com/path"), "sub.domain.com\\/path");
    }

    #[test]
    fn test_escape_sed_replacement_with_ampersand() {
        assert_eq!(escape_sed_replacement("user&domain.com"), "user\\&domain.com");
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
}
