//! Hosting command implementation
//!
//! CLI commands for managing hosting infrastructure including domain registration,
//! DNS configuration, VM creation, and service deployment.

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use crate::calc::audit;
use crate::calc::db;
use anyhow::Result;
use diesel::prelude::*;

use crate::calc::hosting::{
    check_dns_records, check_domain_registration, check_ssh_connection, check_vm_status, create_vm,
    delete_vm, install_dure_service, register_domain, update_dns_records, update_nameservers,
    validate_hosting_config,
};
use crate::storage::models::hosting::{
    Hosting, HostingStatus, delete_hosting as db_delete_hosting, get_hosting_by_domain,
    init_hosting_table, list_hostings, update_hosting_status, upsert_hosting,
};

/// Get database connection
fn get_db_connection() -> Result<SqliteConnection> {
    let _db_path = get_db_path()?;
    let mut conn = db::establish_connection();

    // Initialize table if needed
    init_hosting_table(&mut conn)?;

    Ok(conn)
}

fn get_db_path() -> Result<std::path::PathBuf> {
    let db_path = crate::calc::db::get_db_path();
    Ok(std::path::PathBuf::from(db_path))
}

/// Execute hosting check command
pub fn execute_hosting_check(domain: Option<String>) -> Result<()> {
    let mut conn = get_db_connection()?;

    if let Some(domain_name) = domain {
        // Check specific hosting
        let hosting = get_hosting_by_domain(&mut conn, &domain_name)?
            .ok_or_else(|| anyhow::anyhow!("Hosting not found: {}", domain_name))?;

        check_single_hosting(&hosting)?;
    } else {
        // Check all hostings
        let hostings = list_hostings(&mut conn)?;

        if hostings.is_empty() {
            println!("No hostings configured");
            return Ok(());
        }

        println!("Checking {} hosting(s)...\n", hostings.len());

        for hosting in hostings {
            check_single_hosting(&hosting)?;
            println!();
        }
    }

    Ok(())
}

fn check_single_hosting(hosting: &Hosting) -> Result<()> {
    println!("=== Hosting: {} ===", hosting.domain);

    // Validate configuration
    let validation = validate_hosting_config(hosting);

    if !validation.valid {
        println!("❌ Configuration errors:");
        for error in &validation.errors {
            println!("   - {}", error);
        }
    } else {
        println!("✅ Configuration valid");
    }

    if !validation.warnings.is_empty() {
        println!("⚠️  Warnings:");
        for warning in &validation.warnings {
            println!("   - {}", warning);
        }
    }

    // Check DNS records
    if let Ok(dns_check) = check_dns_records(&hosting.domain) {
        if dns_check.configured {
            println!("✅ DNS configured:");
            println!("   A records: {}", dns_check.a_records.join(", "));
            if !dns_check.txt_records.is_empty() {
                println!("   TXT records: {}", dns_check.txt_records.len());
            }
        } else {
            println!("❌ DNS not configured");
        }
    }

    // Check VM status if applicable
    if hosting.vm_provider != "none" && hosting.vm_created {
        if let Some(instance_id) = &hosting.vm_instance_id {
            if let Some(token) = &hosting.vm_provider_token {
                if let Ok(vm_status) = check_vm_status(&hosting.vm_provider, token, instance_id) {
                    if vm_status.exists && vm_status.running {
                        println!(
                            "✅ VM running: {}",
                            vm_status
                                .ip_address
                                .as_ref()
                                .unwrap_or(&"unknown".to_string())
                        );
                    } else if vm_status.exists {
                        println!("⚠️  VM exists but not running");
                    } else {
                        println!("❌ VM not found");
                    }
                }
            }
        }
    }

    Ok(())
}

/// Execute hosting init command
pub fn execute_hosting_init(
    domain: String,
    dns_provider: String,
    dns_token: Option<String>,
    vm_provider: Option<String>,
    vm_token: Option<String>,
    registrar: Option<String>,
    registrar_token: Option<String>,
) -> Result<()> {
    let mut conn = get_db_connection()?;

    println!("Initializing hosting for domain: {}", domain);

    // Create or load hosting configuration
    let mut hosting = if let Some(existing) = get_hosting_by_domain(&mut conn, &domain)? {
        println!("Loading existing configuration...");
        existing
    } else {
        println!("Creating new configuration...");
        Hosting::new(
            domain.clone(),
            dns_provider.clone(),
            vm_provider.clone().unwrap_or_else(|| "none".to_string()),
        )
    };

    // Update configuration
    hosting.dns_provider = dns_provider;
    hosting.dns_provider_token = dns_token;
    hosting.vm_provider = vm_provider.unwrap_or_else(|| "none".to_string());
    hosting.vm_provider_token = vm_token;
    hosting.domain_registrar = registrar;
    hosting.domain_registrar_token = registrar_token;

    // Validate configuration
    let validation = validate_hosting_config(&hosting);
    if !validation.valid {
        dure_info!("Configuration errors:");
        for error in &validation.errors {
            dure_info!("  - {}", error);
        }
        anyhow::bail!("Invalid hosting configuration");
    }

    // Save initial configuration
    upsert_hosting(&mut conn, &hosting)?;
    update_hosting_status(&mut conn, &domain, HostingStatus::Initializing, None)?;

    // Record audit event
    let _ = audit::push_cli("system", "cli", "hosting init", &domain);

    println!("\n--- Phase A: Domain Registration ---");

    // A-1: Check domain registration
    if let Some(ref registrar) = hosting.domain_registrar {
        if let Some(ref reg_token) = hosting.domain_registrar_token {
            println!("Checking domain registration...");

            match check_domain_registration(&domain, registrar, Some(reg_token)) {
                Ok(check) => {
                    if check.registered {
                        println!("✅ Domain already registered");
                        hosting.domain_registered = true;
                    } else {
                        println!("❌ Domain not registered, attempting registration...");

                        match register_domain(&domain, registrar, reg_token) {
                            Ok(()) => {
                                println!("✅ Domain registered successfully");
                                hosting.domain_registered = true;
                            }
                            Err(e) => {
                                dure_error!(" Failed to register domain: {}", e);
                            }
                        }
                    }

                    // A-2 & A-3: Update and check nameservers
                    if !check.nameservers.is_empty() {
                        println!("Updating nameservers: {:?}", check.nameservers);

                        match update_nameservers(
                            &domain,
                            registrar,
                            reg_token,
                            check.nameservers.clone(),
                        ) {
                            Ok(()) => {
                                println!("✅ Nameservers updated");
                                hosting.ns_addresses =
                                    Some(serde_json::to_string(&check.nameservers)?);
                            }
                            Err(e) => {
                                dure_error!(" Failed to update nameservers: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    dure_error!(" Failed to check domain registration: {}", e);
                }
            }
        } else {
            println!("⚠️  Skipping domain registration (no registrar token)");
        }
    } else {
        println!("⚠️  Skipping domain registration (no registrar configured)");
    }

    upsert_hosting(&mut conn, &hosting)?;

    println!("\n--- Phase B: DNS Configuration ---");

    // B-1 & B-2: Update and check DNS records
    if let Some(ref dns_token) = hosting.dns_provider_token {
        println!("Updating DNS records...");

        let mut dns_records = std::collections::HashMap::new();
        // TODO: Get IP address from VM or user input
        dns_records.insert("A".to_string(), "127.0.0.1".to_string());

        match update_dns_records(&domain, &hosting.dns_provider, dns_token, dns_records) {
            Ok(()) => {
                println!("✅ DNS records updated");

                // Verify DNS records
                match check_dns_records(&domain) {
                    Ok(check) => {
                        if check.configured {
                            println!("✅ DNS records verified");
                            hosting.dns_configured = true;
                        } else {
                            dure_warn!("️  DNS records not yet propagated");
                        }
                    }
                    Err(e) => {
                        dure_error!(" Failed to verify DNS records: {}", e);
                    }
                }
            }
            Err(e) => {
                dure_error!(" Failed to update DNS records: {}", e);
            }
        }
    } else {
        println!("⚠️  Skipping DNS update (no DNS provider token)");
    }

    upsert_hosting(&mut conn, &hosting)?;

    println!("\n--- Phase C: VM Creation ---");

    // C-1 & C-2: Create VM and check SSH connection
    if hosting.vm_provider != "none" {
        if let Some(ref vm_token) = hosting.vm_provider_token {
            println!("Checking VM status...");

            let instance_name = format!("dure-{}", domain.replace('.', "-"));

            match create_vm(&hosting.vm_provider, vm_token, &instance_name, None) {
                Ok(vm_status) => {
                    if vm_status.exists {
                        println!("✅ VM created/exists: {:?}", vm_status.ip_address);
                        hosting.vm_created = true;
                        hosting.vm_instance_id = Some(instance_name.clone());
                        hosting.vm_ip_address = vm_status.ip_address;

                        // Check SSH connection
                        if let Some(ref ip) = hosting.vm_ip_address {
                            if let Some(ref user) = hosting.vm_ssh_user {
                                println!("Checking SSH connection...");

                                match check_ssh_connection(
                                    ip,
                                    user,
                                    hosting.vm_ssh_key_path.as_deref(),
                                ) {
                                    Ok(ssh_check) => {
                                        if ssh_check.reachable {
                                            println!("✅ SSH connection successful");
                                        } else {
                                            dure_error!(" SSH connection failed");
                                        }
                                    }
                                    Err(e) => {
                                        dure_error!(" SSH check error: {}", e);
                                    }
                                }
                            }
                        }
                    } else {
                        dure_error!(" Failed to create VM");
                    }
                }
                Err(e) => {
                    dure_error!(" VM creation error: {}", e);
                }
            }
        } else {
            println!("⚠️  Skipping VM creation (no VM provider token)");
        }
    } else {
        println!("⚠️  Skipping VM creation (provider: none)");
    }

    upsert_hosting(&mut conn, &hosting)?;

    println!("\n--- Phase D: Service Installation ---");

    // D-1 & D-2: Install and run dure service
    if hosting.vm_created {
        if let (Some(ip), Some(user)) = (&hosting.vm_ip_address, &hosting.vm_ssh_user) {
            println!("Installing dure service...");

            match install_dure_service(ip, user, hosting.vm_ssh_key_path.as_deref()) {
                Ok(()) => {
                    println!("✅ Dure service installed");
                    hosting.service_installed = true;
                    hosting.service_running = true;
                }
                Err(e) => {
                    dure_error!(" Service installation error: {}", e);
                }
            }
        }
    } else {
        println!("⚠️  Skipping service installation (no VM)");
    }

    // Final save and status update
    upsert_hosting(&mut conn, &hosting)?;

    let final_status = if hosting.service_running {
        HostingStatus::Active
    } else if hosting.dns_configured || hosting.vm_created {
        HostingStatus::Configured
    } else {
        HostingStatus::Configured
    };

    update_hosting_status(&mut conn, &domain, final_status, None)?;

    println!("\n=== Initialization Complete ===");
    println!("Domain: {}", domain);
    println!("Status: {}", final_status.as_str());

    Ok(())
}

/// Execute hosting show command
pub fn execute_hosting_show(domain: Option<String>) -> Result<()> {
    let mut conn = get_db_connection()?;

    if let Some(domain_name) = domain {
        // Show specific hosting
        let hosting = get_hosting_by_domain(&mut conn, &domain_name)?
            .ok_or_else(|| anyhow::anyhow!("Hosting not found: {}", domain_name))?;

        print_hosting_details(&hosting);
    } else {
        // Show all hostings
        let hostings = list_hostings(&mut conn)?;

        if hostings.is_empty() {
            println!("No hostings configured");
            return Ok(());
        }

        for hosting in hostings {
            print_hosting_details(&hosting);
            println!();
        }
    }

    Ok(())
}

fn print_hosting_details(hosting: &Hosting) {
    println!("=== Hosting: {} ===", hosting.domain);
    println!("Status: {}", hosting.status.as_str());

    println!("\nDomain Registration:");
    println!(
        "  Registrar: {}",
        hosting.domain_registrar.as_deref().unwrap_or("none")
    );
    println!(
        "  Registered: {}",
        if hosting.domain_registered {
            "yes"
        } else {
            "no"
        }
    );

    println!("\nDNS Configuration:");
    println!("  Provider: {}", hosting.dns_provider);
    println!(
        "  Configured: {}",
        if hosting.dns_configured { "yes" } else { "no" }
    );
    if let Some(ref ns) = hosting.ns_addresses {
        println!("  Nameservers: {}", ns);
    }

    println!("\nVM Configuration:");
    println!("  Provider: {}", hosting.vm_provider);
    println!(
        "  Created: {}",
        if hosting.vm_created { "yes" } else { "no" }
    );
    if let Some(ref instance_id) = hosting.vm_instance_id {
        println!("  Instance ID: {}", instance_id);
    }
    if let Some(ref ip) = hosting.vm_ip_address {
        println!("  IP Address: {}", ip);
    }
    if let Some(ref user) = hosting.vm_ssh_user {
        println!("  SSH User: {}", user);
    }

    println!("\nService Status:");
    println!(
        "  Installed: {}",
        if hosting.service_installed {
            "yes"
        } else {
            "no"
        }
    );
    println!(
        "  Running: {}",
        if hosting.service_running { "yes" } else { "no" }
    );

    if let Some(ref error) = hosting.error_message {
        println!("\n❌ Error: {}", error);
    }

    println!("\nTimestamps:");
    println!(
        "  Created: {}",
        chrono::DateTime::from_timestamp(hosting.created_at, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "unknown".to_string())
    );
    println!(
        "  Updated: {}",
        chrono::DateTime::from_timestamp(hosting.updated_at, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "unknown".to_string())
    );
}

/// Execute hosting select command
pub fn execute_hosting_select(domain: String) -> Result<()> {
    let mut conn = get_db_connection()?;

    // Verify hosting exists
    let _hosting = get_hosting_by_domain(&mut conn, &domain)?
        .ok_or_else(|| anyhow::anyhow!("Hosting not found: {}", domain))?;

    update_hosting_status(&mut conn, &domain, HostingStatus::Selected, None)?;

    println!("✅ Selected hosting: {}", domain);

    Ok(())
}

/// Execute hosting deselect command
pub fn execute_hosting_deselect(domain: String) -> Result<()> {
    let mut conn = get_db_connection()?;

    let hosting = get_hosting_by_domain(&mut conn, &domain)?
        .ok_or_else(|| anyhow::anyhow!("Hosting not found: {}", domain))?;

    // Revert to previous status (default to Configured)
    let new_status = if hosting.service_running {
        HostingStatus::Active
    } else {
        HostingStatus::Configured
    };

    update_hosting_status(&mut conn, &domain, new_status, None)?;

    println!("✅ Deselected hosting: {}", domain);

    Ok(())
}

/// Execute hosting close command (block with iptables)
pub fn execute_hosting_close(domain: String) -> Result<()> {
    let mut conn = get_db_connection()?;

    let hosting = get_hosting_by_domain(&mut conn, &domain)?
        .ok_or_else(|| anyhow::anyhow!("Hosting not found: {}", domain))?;

    if hosting.vm_provider == "none" {
        anyhow::bail!("Cannot close hosting without VM");
    }

    // TODO: Implement iptables blocking via SSH
    println!("⚠️  Iptables blocking not yet implemented");

    update_hosting_status(&mut conn, &domain, HostingStatus::Closed, None)?;

    println!("✅ Closed hosting: {}", domain);

    Ok(())
}

/// Execute hosting reopen command (unblock with iptables)
pub fn execute_hosting_reopen(domain: String) -> Result<()> {
    let mut conn = get_db_connection()?;

    let _hosting = get_hosting_by_domain(&mut conn, &domain)?
        .ok_or_else(|| anyhow::anyhow!("Hosting not found: {}", domain))?;

    // TODO: Implement iptables unblocking via SSH
    println!("⚠️  Iptables unblocking not yet implemented");

    update_hosting_status(&mut conn, &domain, HostingStatus::Active, None)?;

    println!("✅ Reopened hosting: {}", domain);

    Ok(())
}

/// Execute hosting delete command
pub fn execute_hosting_delete(domain: String, force: bool) -> Result<()> {
    let mut conn = get_db_connection()?;

    let hosting = get_hosting_by_domain(&mut conn, &domain)?
        .ok_or_else(|| anyhow::anyhow!("Hosting not found: {}", domain))?;

    // Check if VM should be deleted
    if hosting.vm_created && hosting.vm_provider != "none" {
        if !force {
            println!("⚠️  VM exists. Use --force to delete VM as well.");
            println!("   Only removing hosting configuration...");
        } else {
            println!("Deleting VM instance...");

            if let (Some(token), Some(instance_id)) =
                (&hosting.vm_provider_token, &hosting.vm_instance_id)
            {
                match delete_vm(&hosting.vm_provider, token, instance_id) {
                    Ok(()) => {
                        println!("✅ VM deleted");
                    }
                    Err(e) => {
                        dure_error!(" Failed to delete VM: {}", e);
                        println!("Continuing with configuration deletion...");
                    }
                }
            }
        }
    }

    // Delete from database
    db_delete_hosting(&mut conn, &domain)?;

    // Record audit event
    let _ = audit::push_cli("system", "cli", "hosting delete", &domain);

    println!("✅ Deleted hosting configuration: {}", domain);

    Ok(())
}

/// Execute hosting list command
pub fn execute_hosting_list() -> Result<()> {
    let mut conn = get_db_connection()?;

    let hostings = list_hostings(&mut conn)?;

    if hostings.is_empty() {
        println!("No hostings configured");
        return Ok(());
    }

    println!("Hostings ({})", hostings.len());
    println!("{:-<80}", "");
    println!(
        "{:<30} {:<15} {:<15} {:<20}",
        "Domain", "Status", "VM Provider", "Service"
    );
    println!("{:-<80}", "");

    for hosting in hostings {
        let service_status = if hosting.service_running {
            "running"
        } else if hosting.service_installed {
            "installed"
        } else {
            "not installed"
        };

        println!(
            "{:<30} {:<15} {:<15} {:<20}",
            hosting.domain,
            hosting.status.as_str(),
            hosting.vm_provider,
            service_status
        );
    }

    Ok(())
}
