//! Minimal CLI module for Dure DNS functionality

use clap::{Parser, Subcommand};

pub mod commands;

#[derive(Parser)]
#[command(name = "dure")]
#[command(
    about = "Dure - Distributed E-commerce Platform",
    long_about = "Dure - Distributed E-commerce Platform\n\n\
                  Use 'dure --gui' to launch graphical interface\n\
                  Use 'dure profile login <name>' to load a profile\n\
                  Use 'dure help' to see all commands"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    // ==================== Profile Management ====================
    /// Profile management (create, delete, login, list)
    Profile {
        #[command(subcommand)]
        command: ProfileCommands,
    },

    // ==================== Hosting Control Commands ====================
    /// Platform management (GCP, Firebase, Supabase)
    Platform {
        #[command(subcommand)]
        command: Option<PlatformCommands>,
    },

    // ==================== Store Management ====================
    /// Mycart site management (requires loaded profile)
    Mycart {
        #[command(subcommand)]
        command: MycartCommands,
    },

    // ==================== Client Commands ====================
    /// DNS lookup with caching (A, AAAA, TXT records)
    Dns {
        #[command(subcommand)]
        command: DnsCommands,
    },
    /// Key management (password manager with KeePass)
    Key {
        #[command(subcommand)]
        command: KeyCommands,
    },
    /// SSH host management
    Ssh {
        #[command(subcommand)]
        command: SshCommands,
    },
    /// Audit trail management (show action history, clear logs)
    Audit {
        #[command(subcommand)]
        command: AuditCommands,
    },
}

#[derive(Subcommand)]
pub enum AuditCommands {
    /// Review action history (most recent 50 records)
    Status,
    /// Wipe all audit records (requires confirmation)
    Clear,
}

#[derive(Subcommand)]
pub enum ProfileCommands {
    /// List all available profiles
    List,
    /// Create a new profile
    Create {
        /// Profile name
        name: String,
        /// Profile password
        password: String,
    },
    /// Delete a profile
    Delete {
        /// Profile name to delete
        name: String,
    },
    /// Login to a profile (loads it for subsequent commands)
    Login {
        /// Profile name
        name: String,
        /// Profile password
        password: String,
    },
}

#[derive(Subcommand)]
pub enum MycartCommands {
    /// Add a new mycart site
    Add {
        /// Site address (domain)
        address: String,
        /// Admin email
        email: String,
        /// Admin password
        password: String,
        /// Platform ID (optional)
        #[arg(long)]
        platform: Option<String>,
        /// SSH host (optional)
        #[arg(long)]
        ssh_host: Option<String>,
    },
    /// Delete a mycart site
    Del {
        /// Site address to delete
        address: String,
    },
    /// List all mycart sites
    List,
    /// Get site statistics
    Stats {
        /// Site address
        address: String,
    },
    /// Get site logs (requires SSH)
    Logs {
        /// Site address
        address: String,
    },
    /// Get site crontab
    Crontab {
        /// Site address
        address: String,
    },
}

#[derive(Subcommand)]
pub enum DnsCommands {
    /// Query A records for a domain
    A {
        /// Domain name to query
        domain: String,
    },
    /// Query AAAA records for a domain
    Aaaa {
        /// Domain name to query
        domain: String,
    },
    /// Query NS records for a domain
    Ns {
        /// Domain name to query
        domain: String,
    },
    /// Query TXT records for a domain
    Txt {
        /// Domain name to query
        domain: String,
    },
    /// Add TXT record for bastion IP address
    Bastion {
        /// IP address to add to bastion allow list
        ip: String,
    },
}

#[derive(Subcommand)]
pub enum KeyCommands {
    /// Save keyring to KeePass file (export)
    Save {
        /// Output file path (default: ./exported_keys.kdbx)
        output: Option<String>,
    },
    /// Load keyring from KeePass file (import/replace)
    Load {
        /// Input KeePass file path (.kdbx)
        input: String,
    },
    /// List all keys in the current keyring
    Status,
    /// Add a new key to the keyring
    Add {
        /// Domain/URL for the key (e.g., www.dure.app)
        domain: String,
        /// Username/email (e.g., admin@dure.co)
        username: String,
        /// Password/credential
        password: String,
    },
    /// Delete a key from the keyring
    Del {
        /// Domain/URL of the key to delete
        domain: String,
    },
}

#[derive(Subcommand)]
pub enum PlatformCommands {
    /// Add a new platform
    Add {
        /// Platform name
        name: String,
        /// Platform type (gcp, firebase, supabase)
        #[arg(long, short = 't', default_value = "gcp")]
        platform_type: String,
    },

    // Platform-specific actions using external subcommand for dynamic routing
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Subcommand)]
pub enum SshCommands {
    /// Show list and status of SSH hosts
    Status,
    /// Add SSH host to configuration
    Add {
        /// SSH connection string (username@hostname)
        host: String,
        /// SSH password
        #[arg(long)]
        pass: Option<String>,
        /// Path to private key file
        #[arg(long)]
        prvkey: Option<String>,
        /// SSH port (default: 22)
        #[arg(long, default_value = "22")]
        port: u16,
    },
    /// Delete SSH host from configuration
    Del {
        /// SSH connection string (username@hostname)
        host: String,
    },
    /// Initialize SSH host (install swap, nftables, dure server)
    Init {
        /// SSH connection string (username@hostname)
        host: String,
    },
}

/// Load active profile from environment or .active file
#[cfg(not(target_arch = "wasm32"))]
fn load_active_profile() -> anyhow::Result<crate::calc::profile::ProfileContext> {
    use crate::calc::profile::ProfileManager;

    // Check environment variable first
    if let Ok(profile_name) = std::env::var("DURE_PROFILE") {
        return crate::calc::profile::ProfileContext::new(profile_name);
    }

    // Check .active file
    if let Some(profile_name) = ProfileManager::get_active_profile()? {
        return crate::calc::profile::ProfileContext::new(profile_name);
    }

    // No profile loaded
    anyhow::bail!(
        "No profile loaded. Please run 'dure profile login <name>' first.\n\
         Or set DURE_PROFILE environment variable."
    )
}

/// Run CLI mode - parse and execute CLI commands
#[cfg(not(target_arch = "wasm32"))]
pub fn run_cli_mode() -> anyhow::Result<()> {
    use crate::calc::profile::{ProfileContext, ProfileManager};

    let cli = Cli::parse();

    match cli.command {
        // Profile commands don't need loaded profile
        Commands::Profile { command } => match command {
            ProfileCommands::List => {
                commands::profile::execute_profile_list()?;
            }
            ProfileCommands::Create { name, password } => {
                commands::profile::execute_profile_create(name.clone(), password)?;
            }
            ProfileCommands::Delete { name } => {
                commands::profile::execute_profile_delete(name)?;
            }
            ProfileCommands::Login { name, password } => {
                let profile = commands::profile::execute_profile_login(name.clone(), password)?;
                ProfileManager::set_active_profile(&profile.name)?;
                println!("✓ Profile '{}' loaded", profile.name);
                println!();
                println!("You can now run commands like:");
                println!("  dure ssh status");
                println!("  dure platform");
                println!("  dure mycart list");
            }
        },

        // All other commands require loaded profile
        cmd => {
            let profile = load_active_profile()?;
            match cmd {
                Commands::Profile { .. } => unreachable!(),
                Commands::Audit { command } => match command {
            AuditCommands::Status => {
                commands::audit::execute_audit_status(&profile)?;
            }
            AuditCommands::Clear => {
                commands::audit::execute_audit_clear(&profile)?;
            }
        },
        Commands::Dns { command } => match command {
            DnsCommands::A { domain } => {
                commands::dns::execute_dns(&profile, "a", &domain)?;
            }
            DnsCommands::Aaaa { domain } => {
                commands::dns::execute_dns(&profile, "aaaa", &domain)?;
            }
            DnsCommands::Ns { domain } => {
                commands::dns::execute_dns(&profile, "ns", &domain)?;
            }
            DnsCommands::Txt { domain } => {
                commands::dns::execute_dns(&profile, "txt", &domain)?;
            }
            DnsCommands::Bastion { ip } => {
                commands::dns::execute_dns_bastion(&profile, &ip)?;
            }
        },
                Commands::Mycart { command } => match command {
                    MycartCommands::Add {
                        address,
                        email,
                        password,
                        platform,
                        ssh_host,
                    } => {
                        smol::block_on(commands::mycart::add_mycart_site(
                            &profile, address, email, password, platform, ssh_host,
                        ))?;
                    }
                    MycartCommands::Del { address } => {
                        commands::mycart::delete_mycart_site(&profile, address)?;
                    }
                    MycartCommands::List => {
                        commands::mycart::list_mycart_sites(&profile)?;
                    }
                    MycartCommands::Stats { address } => {
                        smol::block_on(commands::mycart::get_site_stats(&profile, address))?;
                    }
                    MycartCommands::Logs { address } => {
                        smol::block_on(commands::mycart::get_site_logs(&profile, address))?;
                    }
                    MycartCommands::Crontab { address } => {
                        smol::block_on(commands::mycart::get_site_crontab(&profile, address))?;
                    }
                },
        Commands::Key { command } => match command {
            KeyCommands::Save { output } => {
                commands::keyring::execute_key_save(&profile, output.clone())?;
            }
            KeyCommands::Load { input } => {
                commands::keyring::execute_key_load(&profile, input.clone())?;
            }
            KeyCommands::Status => {
                commands::keyring::execute_key_status(&profile)?;
            }
            KeyCommands::Add {
                domain,
                username,
                password,
            } => {
                commands::keyring::execute_key_add(
                    &profile,
                    domain.clone(),
                    username.clone(),
                    password.clone(),
                )?;
            }
            KeyCommands::Del { domain } => {
                commands::keyring::execute_key_del(&profile, domain.clone())?;
            }
        },
        Commands::Platform { command } => match command {
            None => {
                // Default: show combined list + show for all platforms
                commands::platform::list::execute_platform_combined(&profile)?;
            }
            Some(PlatformCommands::Add {
                name,
                platform_type,
            }) => {
                commands::platform::execute_platform_add(&profile, name, platform_type)?;
            }
            Some(PlatformCommands::External(args)) => {
                commands::platform::execute_platform_external(&profile, args)?;
            }
        },
        Commands::Ssh { command } => match command {
            SshCommands::Status => {
                commands::ssh::execute_ssh_status(&profile)?;
            }
            SshCommands::Add {
                host,
                pass,
                prvkey,
                port,
            } => {
                commands::ssh::execute_ssh_add(&profile, host, pass, prvkey, port)?;
            }
            SshCommands::Del { host } => {
                commands::ssh::execute_ssh_del(&profile, host)?;
            }
            SshCommands::Init { host } => {
                commands::ssh::execute_ssh_init(&profile, host)?;
            }
        },
            }
        }
    }

    Ok(())
}
