//! ACME command implementation for SSL certificate management using lego

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use crate::calc::lego::{self, DnsProvider};
use crate::storage::models::acme::{
    get_certificate, get_certificates_needing_renewal, list_certificates, store_certificate,
};
use crate::{Config, calc::db, config::AppConfig};
use anyhow::{Context, Result};
use std::path::PathBuf;

/// Get config file path
fn get_config_path() -> Result<PathBuf> {
    let proj_dirs = directories::ProjectDirs::from("pe", "nikescar", "dure")
        .context("Failed to get project directories")?;
    Ok(proj_dirs.config_dir().join("config.yml"))
}

/// Load application config
fn load_config() -> Result<(AppConfig, PathBuf)> {
    let config_path = get_config_path()?;
    let app_config = AppConfig::load_or_default(&config_path);
    Ok((app_config, config_path))
}

/// Get DNS provider from config
fn get_dns_provider(domain_config: &crate::config::DomainConfig) -> Result<DnsProvider> {
    match domain_config.dns_provider.to_uppercase().as_str() {
        "CLOUDFLARE_DNS" | "CLOUDFLARE" => Ok(DnsProvider::Cloudflare),
        "DUCKDNS" => Ok(DnsProvider::DuckDns),
        "GCP_CLOUDDNS" | "GCLOUD" => Ok(DnsProvider::GoogleCloud),
        "PORKBUN" => Ok(DnsProvider::Porkbun),
        "" => anyhow::bail!("DNS provider not configured in config.yml"),
        other => anyhow::bail!("Unsupported DNS provider: {}", other),
    }
}

/// Build environment variables for DNS provider
fn build_dns_env_vars(
    provider: &DnsProvider,
    domain_config: &crate::config::DomainConfig,
) -> Result<Vec<(String, String)>> {
    let mut env_vars = Vec::new();

    match provider {
        DnsProvider::Cloudflare => {
            // Prefer API token over email+key
            if let Some(token) = &domain_config.cloudflare.api_token {
                env_vars.push(("CLOUDFLARE_DNS_API_TOKEN".to_string(), token.clone()));
            } else if let (Some(email), Some(key)) = (
                &domain_config.cloudflare.email,
                &domain_config.cloudflare.api_key,
            ) {
                env_vars.push(("CLOUDFLARE_EMAIL".to_string(), email.clone()));
                env_vars.push(("CLOUDFLARE_API_KEY".to_string(), key.clone()));
            } else {
                return Err(anyhow::anyhow!(
                    "Cloudflare requires either 'api_token' or 'email' + 'api_key' in domain.cloudflare config"
                ));
            }
        }
        DnsProvider::DuckDns => {
            if let Some(token) = &domain_config.duckdns.token {
                env_vars.push(("DUCKDNS_TOKEN".to_string(), token.clone()));
            } else {
                return Err(anyhow::anyhow!(
                    "DuckDNS requires 'token' in domain.duckdns config"
                ));
            }
        }
        DnsProvider::GoogleCloud => {
            if let Some(project) = &domain_config.gcloud.project {
                env_vars.push(("GCE_PROJECT".to_string(), project.clone()));
            } else {
                return Err(anyhow::anyhow!(
                    "Google Cloud requires 'project' in domain.gcloud config"
                ));
            }

            if let Some(sa_file) = &domain_config.gcloud.service_account_file {
                env_vars.push(("GCE_SERVICE_ACCOUNT_FILE".to_string(), sa_file.clone()));
            }

            if let Some(impersonate) = &domain_config.gcloud.impersonate_service_account {
                env_vars.push((
                    "GCE_IMPERSONATE_SERVICE_ACCOUNT".to_string(),
                    impersonate.clone(),
                ));
            }
        }
        DnsProvider::Porkbun => {
            if let (Some(api_key), Some(secret)) = (
                &domain_config.porkbun.api_key,
                &domain_config.porkbun.secret_api_key,
            ) {
                env_vars.push(("PORKBUN_API_KEY".to_string(), api_key.clone()));
                env_vars.push(("PORKBUN_SECRET_API_KEY".to_string(), secret.clone()));
            } else {
                return Err(anyhow::anyhow!(
                    "Porkbun requires 'api_key' and 'secret_api_key' in domain.porkbun config"
                ));
            }
        }
    }

    Ok(env_vars)
}

/// Copy certificate files to config directory
fn copy_cert_to_config_dir(domain: &str, config_dir: &PathBuf) -> Result<(String, String, String)> {
    let cert_dir = config_dir.join("certs");
    std::fs::create_dir_all(&cert_dir)?;

    let lego_dir = lego::get_lego_dir();
    let lego_cert_dir = lego_dir.join("certificates");

    let src_cert = lego_cert_dir.join(format!("{}.crt", domain));
    let src_key = lego_cert_dir.join(format!("{}.key", domain));
    let src_issuer = lego_cert_dir.join(format!("{}.issuer.crt", domain));

    let dest_cert = cert_dir.join(format!("{}.crt", domain));
    let dest_key = cert_dir.join(format!("{}.key", domain));
    let dest_issuer = cert_dir.join(format!("{}.issuer.crt", domain));

    std::fs::copy(&src_cert, &dest_cert).context("Failed to copy certificate file")?;
    std::fs::copy(&src_key, &dest_key).context("Failed to copy private key file")?;
    std::fs::copy(&src_issuer, &dest_issuer).context("Failed to copy issuer certificate file")?;

    Ok((
        dest_cert.to_string_lossy().to_string(),
        dest_key.to_string_lossy().to_string(),
        dest_issuer.to_string_lossy().to_string(),
    ))
}

/// Execute ACME install command (lego auto-downloads when needed)
pub fn execute_acme_install() -> Result<()> {
    dure_info!("Checking lego installation...");

    let config = Config::new()?;
    let config_dir = config.config_dir;

    if lego::check_lego_installed(&config_dir) {
        dure_info!(" lego is already installed");
        return Ok(());
    }

    dure_info!("Downloading lego...");
    lego::download_lego(&config_dir)?;

    dure_info!(" lego installed successfully");
    dure_info!("");
    dure_info!("Next steps:");
    dure_info!("  1. Configure DNS provider in config.yml");
    dure_info!("  2. Issue a certificate: dure acme issue");

    Ok(())
}

/// Execute ACME issue command
pub fn execute_acme_issue(domains: Vec<String>) -> Result<()> {
    let config = Config::new()?;
    let (mut app_config, config_path) = load_config()?;

    // Use provided domains or fall back to config
    let target_domain = if !domains.is_empty() {
        domains[0].clone()
    } else if !app_config.domain.name.is_empty() {
        app_config.domain.name.clone()
    } else {
        anyhow::bail!(
            "No domain specified. Use: dure acme issue <domain> or configure domain.name in config.yml"
        );
    };

    // Get email from config or use default
    let email = if let Some(ref e) = app_config.domain.cert.email {
        e.clone()
    } else {
        format!("admin@{}", target_domain)
    };

    dure_info!("Issuing certificate for: {}", target_domain);
    dure_info!("Email: {}", email);

    // Get DNS provider from config
    let dns_provider = get_dns_provider(&app_config.domain)?;
    dure_info!("DNS Provider: {:?}", dns_provider);

    // Build environment variables
    let env_vars = build_dns_env_vars(&dns_provider, &app_config.domain)?;
    let env_refs: Vec<(&str, &str)> = env_vars
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();

    if env_refs.is_empty() {
        dure_warn!(" Warning: No DNS provider credentials found in config.yml");
        dure_info!("Please configure domain.cloudflare/duckdns/gcloud/porkbun in config.yml");
    }

    // Issue certificate using lego
    let cert = lego::issue_certificate(
        &config.config_dir,
        &email,
        &target_domain,
        dns_provider,
        &env_refs,
    )?;

    dure_info!(" Certificate issued successfully");

    // Copy certificates to config directory
    dure_info!("Copying certificates to config directory...");
    let (cert_path, key_path, issuer_path) =
        copy_cert_to_config_dir(&target_domain, &config.config_dir)?;

    dure_info!(" Certificates copied");
    dure_info!("");
    dure_info!("Certificate details:");
    dure_info!("  Certificate: {}", cert_path);
    dure_info!("  Private key: {}", key_path);
    dure_info!("  Issuer: {}", issuer_path);

    // Store in database
    let mut conn = db::establish_connection();
    let db_cert = lego::Certificate::new(
        target_domain.clone(),
        cert_path.clone(),
        key_path.clone(),
        issuer_path.clone(),
        cert.issued_at,
        cert.expires_at,
    );
    store_certificate(&mut conn, &db_cert)?;

    // Update config.yml with certificate paths
    app_config.domain.cert.cert_path = Some(cert_path);
    app_config.domain.cert.key_path = Some(key_path);
    app_config.domain.cert.issuer_path = Some(issuer_path);
    app_config.domain.cert.email = Some(email);

    app_config.save(&config_path)?;
    dure_info!(" Configuration updated");

    dure_info!("");
    eprintln!(
        "Certificate will expire in {} days",
        (cert.expires_at - cert.issued_at) / 86400
    );
    dure_info!("Remember to renew before expiry:");
    dure_info!("  dure acme renew");

    Ok(())
}

/// Execute ACME renew command
pub fn execute_acme_renew(domain: String, force: bool) -> Result<()> {
    let config = Config::new()?;
    let (mut app_config, config_path) = load_config()?;

    // Use provided domain or fall back to config
    let target_domain = if !domain.is_empty() {
        domain
    } else if !app_config.domain.name.is_empty() {
        app_config.domain.name.clone()
    } else {
        anyhow::bail!("No domain specified");
    };

    dure_info!("Renewing certificate for: {}", target_domain);

    // Check if renewal is needed
    if !force {
        let mut conn = db::establish_connection();
        if let Some(existing_cert) = get_certificate(&mut conn, &target_domain)? {
            if !existing_cert.needs_renewal() {
                let days_until_expiry =
                    (existing_cert.expires_at as i64 - chrono::Utc::now().timestamp()) / 86400;
                eprintln!(
                    "Certificate does not need renewal yet ({} days until expiry)",
                    days_until_expiry
                );
                dure_info!("Use --force to renew anyway");
                return Ok(());
            }
        }
    }

    // Get DNS provider from config
    let dns_provider = get_dns_provider(&app_config.domain)?;

    // Build environment variables
    let env_vars = build_dns_env_vars(&dns_provider, &app_config.domain)?;
    let env_refs: Vec<(&str, &str)> = env_vars
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();

    // Renew certificate
    let cert =
        lego::renew_certificate(&config.config_dir, &target_domain, dns_provider, &env_refs)?;

    dure_info!(" Certificate renewed successfully");

    // Copy certificates to config directory
    let (cert_path, key_path, issuer_path) =
        copy_cert_to_config_dir(&target_domain, &config.config_dir)?;

    // Update database
    let mut conn = db::establish_connection();
    let db_cert = lego::Certificate::new(
        target_domain,
        cert_path.clone(),
        key_path.clone(),
        issuer_path.clone(),
        cert.issued_at,
        cert.expires_at,
    );
    store_certificate(&mut conn, &db_cert)?;

    // Update config
    app_config.domain.cert.cert_path = Some(cert_path);
    app_config.domain.cert.key_path = Some(key_path);
    app_config.domain.cert.issuer_path = Some(issuer_path);
    app_config.save(&config_path)?;

    dure_info!(" Certificate and configuration updated");

    Ok(())
}

/// Execute ACME list command
pub fn execute_acme_list() -> Result<()> {
    let mut conn = db::establish_connection();

    let certs = list_certificates(&mut conn)?;

    if certs.is_empty() {
        dure_info!("No certificates found.");
        dure_info!("");
        dure_info!("Run 'dure acme issue' to create a certificate");
        return Ok(());
    }

    dure_info!("SSL Certificates:");
    dure_info!("");

    for cert in certs {
        dure_info!("Domain: {}", cert.domain);
        dure_info!("  Certificate: {}", cert.cert_path);
        dure_info!("  Key: {}", cert.key_path);
        dure_info!("  Issuer: {}", cert.issuer_path);
        dure_info!("  Valid: {}", if cert.is_valid { "Yes" } else { "No" });

        if cert.needs_renewal() {
            dure_info!("  ⚠ Needs renewal (expires soon)");
        }

        let days_left = (cert.expires_at as i64 - chrono::Utc::now().timestamp()) / 86400;
        dure_info!("  Expires in: {} days", days_left);
        dure_info!("");
    }

    Ok(())
}

/// Execute ACME status command
pub fn execute_acme_status() -> Result<()> {
    let mut conn = db::establish_connection();

    let renewal_certs = get_certificates_needing_renewal(&mut conn)?;

    if renewal_certs.is_empty() {
        dure_info!(" All certificates are up to date");
        return Ok(());
    }

    dure_info!("Certificates needing renewal ({}):", renewal_certs.len());
    dure_info!("");

    for cert in renewal_certs {
        let days_left = (cert.expires_at as i64 - chrono::Utc::now().timestamp()) / 86400;
        dure_info!("  {} - expires in {} days", cert.domain, days_left);
    }

    dure_info!("");
    dure_info!("Run 'dure acme renew' to renew certificates");

    Ok(())
}

/// Execute ACME sync command (deprecated)
pub fn execute_acme_sync() -> Result<()> {
    dure_info!("ACME sync command is deprecated.");
    dure_info!("Certificates are tracked automatically in the database.");
    Ok(())
}
