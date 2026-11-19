//! Minimal CLI module for Dure DNS functionality

use clap::{Parser, Subcommand};

pub mod commands;

#[derive(Parser)]
#[command(name = "dure")]
#[command(
    about = "Dure - Distributed E-commerce Platform",
    long_about = "Dure - Distributed E-commerce Platform\n\nUse 'dure info' to see commands organized by category"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    // ==================== Hosting Control Commands ====================
    /// Platform management (GCP, Firebase, Supabase)
    Platform {
        #[command(subcommand)]
        command: Option<PlatformCommands>,
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

    // ==================== Common/Utility Commands ====================
    /// Cryptographic operations (encrypt/decrypt)
    Crypt {
        #[command(subcommand)]
        command: CryptCommands,
    },
    /// Show diagnostic metadata about the workspace
    Info,
    /// Initialize a workspace
    Init {
        /// Issue ID prefix (e.g., "bd")
        #[arg(long)]
        prefix: Option<String>,
        /// Overwrite existing DB
        #[arg(long)]
        force: bool,
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
pub enum CryptCommands {
    /// Show base pubkey for system
    Status,
    /// Encrypt data for a recipient
    Enc {
        /// Recipient's public key (base64 or hex)
        recipient_pubkey: String,
        /// Data to encrypt
        data: String,
        /// Output as hex instead of base64
        #[arg(long)]
        hex: bool,
    },
    /// Decrypt data
    Dec {
        /// Encrypted data (base64 or hex)
        encrypted_data: String,
        /// Output raw bytes instead of UTF-8 text
        #[arg(long)]
        raw: bool,
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

/// Run CLI mode - parse and execute CLI commands
#[cfg(not(target_arch = "wasm32"))]
pub fn run_cli_mode() -> anyhow::Result<()> {
    use crate::calc::profile::ProfileContext;

    let cli = Cli::parse();

    // Load default profile for CLI operations
    let profile = ProfileContext::new("default")?;

    match cli.command {
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
        Commands::Crypt { command } => match command {
            CryptCommands::Status => {
                commands::crypt::execute_crypt_status(&profile)?;
            }
            CryptCommands::Enc {
                recipient_pubkey,
                data,
                hex,
            } => {
                commands::crypt::execute_crypt_enc(&profile, recipient_pubkey, data, hex)?;
            }
            CryptCommands::Dec {
                encrypted_data,
                raw,
            } => {
                commands::crypt::execute_crypt_dec(&profile, encrypted_data, raw)?;
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
        Commands::Info => {
            use clap::CommandFactory;

            println!("Dure CLI Info:");
            println!("  Version: {}", env!("CARGO_PKG_VERSION"));
            println!("  Mode: CLI");
            println!();

            // Define command categories
            let categories = vec![
                ("Hosting Control Commands", vec!["ns", "platform"]),
                ("Client Commands", vec!["dns", "key", "ssh", "audit"]),
                (
                    "Common/Utility Commands",
                    vec!["crypt", "site", "info", "init"],
                ),
            ];

            let mut cmd = Cli::command();

            for (category, command_names) in categories {
                println!("{}:", category);
                for name in command_names {
                    if let Some(subcmd) = cmd.find_subcommand_mut(name) {
                        let about = subcmd
                            .get_about()
                            .map(|s| s.to_string())
                            .unwrap_or_default();
                        println!("  {:<8} - {}", name, about);

                        // Print subcommands if any
                        for sub in subcmd.get_subcommands() {
                            if sub.get_name() == "help" {
                                continue;
                            }
                            let sub_about =
                                sub.get_about().map(|s| s.to_string()).unwrap_or_default();
                            println!("    {:<10} - {}", sub.get_name(), sub_about);
                        }
                    }
                }
                println!();
            }
        }
        Commands::Init { prefix, force } => {
            println!("Initializing Dure workspace...");
            if let Some(p) = prefix {
                println!("  Prefix: {}", p);
            }
            if force {
                println!("  Force: true");
            }
            println!("Note: Full initialization not yet implemented in CLI mode");
        }
    }

    Ok(())
}
