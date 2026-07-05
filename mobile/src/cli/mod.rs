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
    /// DNS nameserver record management
    Ns {
        #[command(subcommand)]
        command: NsCommands,
    },
    /// Platform management (GCP, Firebase, Supabase)
    Platform {
        #[command(subcommand)]
        command: PlatformCommands,
    },
    /// Hosting management (domain, DNS, VM, service)
    Hosting {
        #[command(subcommand)]
        command: HostingCommands,
    },

    // ==================== Server Control Commands ====================
    /// ACME SSL certificate management
    Acme {
        #[command(subcommand)]
        command: AcmeCommands,
    },
    /// NFTables firewall management (SSH whitelist)
    Nft {
        #[command(subcommand)]
        command: NftCommands,
    },
    /// WebSocket Secure (WSS) server management
    Wss {
        #[command(subcommand)]
        command: WssCommands,
    },
    /// Webhook management and monitoring
    Webhook {
        #[command(subcommand)]
        command: WebhookCommands,
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
    /// Site management for site-to-site communication
    Site {
        #[command(subcommand)]
        command: SiteCommands,
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
pub enum AcmeCommands {
    /// Install acme.sh to the system
    Install,
    /// Check acme.sh installation and sync certificate status with the database
    Status,
    /// Issue a new SSL certificate
    Issue {
        /// Domain names to include in the certificate
        domains: Vec<String>,
    },
    /// Renew an existing certificate
    Renew {
        /// Domain to renew
        domain: String,
        /// Force renewal even if not needed
        #[arg(long)]
        force: bool,
    },
    /// List all managed certificates
    List,
}

#[derive(Subcommand)]
pub enum NsCommands {
    /// List all registered domains and their records
    Status {
        /// Optional domain name to show records for specific domain
        domain: Option<String>,
    },
    /// Add a new domain to nameserver
    Add {
        /// Domain name (e.g., www.example.com)
        domain: String,
        /// DNS provider (cloudflare, gcloud, duckdns, porkbun)
        #[arg(long)]
        provider: String,
        /// API token for the DNS provider
        #[arg(long)]
        token: String,
    },
    /// Delete a domain from nameserver
    Del {
        /// Domain name to delete
        domain: String,
    },
    /// Insert a DNS record to a domain
    Insert {
        /// Record type (a, aaaa, txt, ns)
        record_type: String,
        /// Domain name
        domain: String,
        /// Record value (IP address for A/AAAA, nameserver for NS, text for TXT)
        value: String,
        /// Apply the change to DNS provider immediately
        #[arg(long)]
        apply: bool,
    },
    /// Remove a DNS record from a domain
    Remove {
        /// Record type (a, aaaa, txt, ns)
        record_type: String,
        /// Domain name
        domain: String,
        /// Record value to remove
        value: String,
    },
}

#[derive(Subcommand)]
pub enum NftCommands {
    /// Show current nftables ruleset
    Show,
    /// Add an IP to SSH whitelist
    Whitelist {
        /// IP address to whitelist
        ip: String,
        /// Optional description for the IP
        #[arg(long)]
        description: Option<String>,
    },
    /// Remove an IP from SSH whitelist
    Remove {
        /// IP address to remove
        ip: String,
    },
    /// List all whitelisted IPs
    List,
}

#[derive(Subcommand)]
pub enum WssCommands {
    /// Show WebSocket server status
    Status {
        /// Domain name (optional, shows all if not provided)
        domain: Option<String>,
    },
    /// Start HTTPS/WSS server
    Server {
        /// Domain name
        domain: String,
        /// Bind address (default: 0.0.0.0:443)
        #[arg(long)]
        addr: Option<String>,
        /// Skip downloading static files
        #[arg(long)]
        no_download: bool,
        /// Stats interval in seconds (default: 60)
        #[arg(long)]
        stats_interval: Option<u64>,
    },
    /// Test client for HTTPS/WSS
    Client {
        /// Server URL (https:// or wss://)
        url: String,
        /// Client mode: ws, get, or post (default: ws)
        #[arg(long, short)]
        mode: Option<String>,
        /// Request path (default: /)
        #[arg(long, short)]
        path: Option<String>,
        /// POST request body (default: {"test":"data"})
        #[arg(long, short)]
        body: Option<String>,
        /// Skip TLS certificate verification (for self-signed certs)
        #[arg(long, short = 'k')]
        insecure: bool,
    },
    /// Stop WebSocket server
    Stop {
        /// Domain name
        domain: String,
    },
}

#[derive(Subcommand)]
pub enum WebhookCommands {
    /// Show webhook configuration status
    Status {},
    /// Enable webhook request logging
    EnableLogging {},
    /// Disable webhook request logging
    DisableLogging {},
    /// Add webhook allow pattern
    AddPattern {
        /// Pattern to add (e.g., /webhook/*, /api/*, *)
        pattern: String,
    },
    /// List webhook allow patterns
    ListPatterns {},
    /// Delete webhook allow pattern
    DeletePattern {
        /// Pattern ID to delete
        id: i64,
    },
    /// List recent webhook requests
    ListRequests {
        /// Maximum number of requests to show (default: 10)
        #[arg(long)]
        limit: Option<usize>,
        /// Filter by pattern
        #[arg(long)]
        pattern: Option<String>,
    },
    /// List sessions
    ListSessions {
        /// Filter by session type (http, wss, or all)
        #[arg(long)]
        session_type: Option<String>,
    },
    /// Clean up old data
    Cleanup {
        /// Maximum age in seconds (default: 86400 = 24 hours)
        max_age: Option<u64>,
    },
}

#[derive(Subcommand)]
pub enum SiteCommands {
    /// List all configured sites
    Status,
    /// Add a new site for site-to-site communication
    Add {
        /// Domain name (e.g., example.com)
        domain: String,
        /// Public key for authentication
        #[arg(long)]
        public_key: String,
    },
    /// Delete a site
    Del {
        /// Domain name to delete
        domain: String,
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
        /// Username/email (e.g., nikescar@gmail.com)
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
pub enum HostingCommands {
    /// Check hosting configuration and status
    Check {
        /// Domain name (optional, checks all if not provided)
        domain: Option<String>,
    },
    /// Initialize hosting (domain registration, DNS, VM, service)
    Init {
        /// Domain name
        domain: String,
        /// DNS provider (porkbun, cloudflare, duckdns, gcp_clouddns)
        #[arg(long)]
        dns_provider: String,
        /// DNS provider API token
        #[arg(long)]
        dns_token: Option<String>,
        /// VM provider (gcp, cafe24vps, none)
        #[arg(long)]
        vm_provider: Option<String>,
        /// VM provider API token
        #[arg(long)]
        vm_token: Option<String>,
        /// Domain registrar (porkbun, cloudflare)
        #[arg(long)]
        registrar: Option<String>,
        /// Registrar API token
        #[arg(long)]
        registrar_token: Option<String>,
    },
    /// Show hosting details and configurations
    Show {
        /// Domain name (optional, shows all if not provided)
        domain: Option<String>,
    },
    /// Select hosting for operations
    Select {
        /// Domain name
        domain: String,
    },
    /// Deselect hosting from operations
    Deselect {
        /// Domain name
        domain: String,
    },
    /// Close hosting (block with iptables)
    Close {
        /// Domain name
        domain: String,
    },
    /// Reopen hosting (unblock with iptables)
    Reopen {
        /// Domain name
        domain: String,
    },
    /// Delete hosting (and optionally VM)
    Delete {
        /// Domain name
        domain: String,
        /// Force deletion of VM instance
        #[arg(long)]
        force: bool,
    },
    /// List all hostings
    List,
}

#[derive(Subcommand)]
pub enum PlatformCommands {
    /// List all platforms with status (default command)
    List,

    /// Show platform details and available actions
    Show {
        name: String
    },

    /// Refresh platform data
    Refresh {
        name: String
    },

    /// Add a new VM to the platform
    AddVm {
        name: String,
        #[arg(long)]
        vm_name: Option<String>,
        #[arg(long)]
        zone: Option<String>,
        #[arg(long)]
        machine_type: Option<String>,
    },

    /// Update firewall rules (whitelist current IP)
    Firewall {
        name: String,
        #[arg(long)]
        ip: Option<String>,
    },

    /// Restart VM
    Restart {
        name: String,
        #[arg(long)]
        vm: Option<String>,
    },

    /// Delete VM
    DelVm {
        name: String,
        #[arg(long)]
        vm: Option<String>,
    },

    /// Show billing information
    Billing {
        name: String
    },

    /// Delete platform configuration
    Delete {
        name: String
    },

    // Legacy commands (kept for backwards compatibility)
    /// [Deprecated] List all platforms (use 'list' instead)
    #[command(hide = true)]
    Status,

    /// [Deprecated] Add a new platform (use GUI or init)
    #[command(hide = true)]
    Add {
        name: String,
        platform_type: String,
    },

    /// [Deprecated] Delete a platform (use 'delete' instead)
    #[command(hide = true)]
    Del {
        name: String,
    },

    /// Initialize a platform (OAuth, project setup)
    Init {
        name: String,
    },
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
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
pub fn run_cli_mode() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Audit { command } => match command {
            AuditCommands::Status => {
                commands::audit::execute_audit_status()?;
            }
            AuditCommands::Clear => {
                commands::audit::execute_audit_clear()?;
            }
        },
        Commands::Dns { command } => match command {
            DnsCommands::A { domain } => {
                commands::dns::execute_dns("a", &domain)?;
            }
            DnsCommands::Aaaa { domain } => {
                commands::dns::execute_dns("aaaa", &domain)?;
            }
            DnsCommands::Ns { domain } => {
                commands::dns::execute_dns("ns", &domain)?;
            }
            DnsCommands::Txt { domain } => {
                commands::dns::execute_dns("txt", &domain)?;
            }
            DnsCommands::Bastion { ip } => {
                commands::dns::execute_dns_bastion(&ip)?;
            }
        },
        Commands::Acme { command } => match command {
            AcmeCommands::Install => {
                commands::acme::execute_acme_install()?;
            }
            AcmeCommands::Status => {
                commands::acme::execute_acme_status()?;
            }
            AcmeCommands::Issue { domains } => {
                commands::acme::execute_acme_issue(domains)?;
            }
            AcmeCommands::Renew { domain, force } => {
                commands::acme::execute_acme_renew(domain, force)?;
            }
            AcmeCommands::List => {
                commands::acme::execute_acme_list()?;
            }
        },
        Commands::Ns { command } => match command {
            NsCommands::Status { domain } => {
                commands::ns::execute_ns_status(&domain)?;
            }
            NsCommands::Add {
                domain,
                provider,
                token,
            } => {
                commands::ns::execute_ns_add(&domain, &provider, &token)?;
            }
            NsCommands::Del { domain } => {
                commands::ns::execute_ns_del(&domain)?;
            }
            NsCommands::Insert {
                record_type,
                domain,
                value,
                apply,
            } => {
                commands::ns::execute_ns_insert(&record_type, &domain, &value, apply)?;
            }
            NsCommands::Remove {
                record_type,
                domain,
                value,
            } => {
                commands::ns::execute_ns_remove(&record_type, &domain, &value)?;
            }
        },
        Commands::Nft { command } => match command {
            NftCommands::Show => {
                commands::nft::execute_nft_show()?;
            }
            NftCommands::Whitelist { ip, description } => {
                commands::nft::execute_nft_whitelist(ip, description)?;
            }
            NftCommands::Remove { ip } => {
                commands::nft::execute_nft_remove(ip)?;
            }
            NftCommands::List => {
                commands::nft::execute_nft_list()?;
            }
        },
        Commands::Wss { command } => match command {
            WssCommands::Status { domain } => {
                commands::wss::execute_wss_status(domain)?;
            }
            WssCommands::Server {
                domain,
                addr,
                no_download,
                stats_interval,
            } => {
                commands::wss::execute_wss_server(domain, addr, no_download, stats_interval)?;
            }
            WssCommands::Client {
                url,
                mode,
                path,
                body,
                insecure,
            } => {
                commands::wss::execute_wss_client(url, mode, path, body, insecure)?;
            }
            WssCommands::Stop { domain } => {
                commands::wss::execute_wss_stop(domain)?;
            }
        },
        Commands::Webhook { command } => match command {
            WebhookCommands::Status {} => {
                commands::webhook::execute_webhook_status()?;
            }
            WebhookCommands::EnableLogging {} => {
                commands::webhook::execute_webhook_enable_logging()?;
            }
            WebhookCommands::DisableLogging {} => {
                commands::webhook::execute_webhook_disable_logging()?;
            }
            WebhookCommands::AddPattern { pattern } => {
                commands::webhook::execute_webhook_add_pattern(pattern)?;
            }
            WebhookCommands::ListPatterns {} => {
                commands::webhook::execute_webhook_list_patterns()?;
            }
            WebhookCommands::DeletePattern { id } => {
                commands::webhook::execute_webhook_delete_pattern(id)?;
            }
            WebhookCommands::ListRequests { limit, pattern } => {
                commands::webhook::execute_webhook_list_requests(limit, pattern)?;
            }
            WebhookCommands::ListSessions { session_type } => {
                commands::webhook::execute_webhook_list_sessions(session_type)?;
            }
            WebhookCommands::Cleanup { max_age } => {
                commands::webhook::execute_webhook_cleanup(max_age)?;
            }
        },
        Commands::Crypt { command } => match command {
            CryptCommands::Status => {
                commands::crypt::execute_crypt_status()?;
            }
            CryptCommands::Enc {
                recipient_pubkey,
                data,
                hex,
            } => {
                commands::crypt::execute_crypt_enc(recipient_pubkey, data, hex)?;
            }
            CryptCommands::Dec {
                encrypted_data,
                raw,
            } => {
                commands::crypt::execute_crypt_dec(encrypted_data, raw)?;
            }
        },
        Commands::Key { command } => match command {
            KeyCommands::Save { output } => {
                commands::keyring::execute_key_save(output.clone())?;
            }
            KeyCommands::Load { input } => {
                commands::keyring::execute_key_load(input.clone())?;
            }
            KeyCommands::Status => {
                commands::keyring::execute_key_status()?;
            }
            KeyCommands::Add {
                domain,
                username,
                password,
            } => {
                commands::keyring::execute_key_add(
                    domain.clone(),
                    username.clone(),
                    password.clone(),
                )?;
            }
            KeyCommands::Del { domain } => {
                commands::keyring::execute_key_del(domain.clone())?;
            }
        },
        Commands::Hosting { command } => match command {
            HostingCommands::Check { domain } => {
                commands::hosting::execute_hosting_check(domain)?;
            }
            HostingCommands::Init {
                domain,
                dns_provider,
                dns_token,
                vm_provider,
                vm_token,
                registrar,
                registrar_token,
            } => {
                commands::hosting::execute_hosting_init(
                    domain,
                    dns_provider,
                    dns_token,
                    vm_provider,
                    vm_token,
                    registrar,
                    registrar_token,
                )?;
            }
            HostingCommands::Show { domain } => {
                commands::hosting::execute_hosting_show(domain)?;
            }
            HostingCommands::Select { domain } => {
                commands::hosting::execute_hosting_select(domain)?;
            }
            HostingCommands::Deselect { domain } => {
                commands::hosting::execute_hosting_deselect(domain)?;
            }
            HostingCommands::Close { domain } => {
                commands::hosting::execute_hosting_close(domain)?;
            }
            HostingCommands::Reopen { domain } => {
                commands::hosting::execute_hosting_reopen(domain)?;
            }
            HostingCommands::Delete { domain, force } => {
                commands::hosting::execute_hosting_delete(domain, force)?;
            }
            HostingCommands::List => {
                commands::hosting::execute_hosting_list()?;
            }
        },
        Commands::Platform { command } => match command {
            PlatformCommands::List => {
                commands::platform::list::execute_platform_list()?;
            }
            PlatformCommands::Show { name } => {
                commands::platform::list::execute_platform_show(name)?;
            }
            PlatformCommands::Refresh { name } => {
                commands::platform::execute_refresh_command(name)?;
            }
            PlatformCommands::AddVm { name, vm_name, zone, machine_type } => {
                commands::platform::vm::execute_addvm_command(name, vm_name, zone, machine_type)?;
            }
            PlatformCommands::Firewall { name, ip } => {
                commands::platform::firewall::execute_firewall_command(name, ip)?;
            }
            PlatformCommands::Restart { name, vm } => {
                commands::platform::vm::execute_restart_command(name, vm)?;
            }
            PlatformCommands::DelVm { name, vm } => {
                commands::platform::vm::execute_delvm_command(name, vm)?;
            }
            PlatformCommands::Billing { name } => {
                commands::platform::billing::execute_billing_command(name)?;
            }
            PlatformCommands::Delete { name } => {
                commands::platform::execute_delete_command(name)?;
            }
            // Legacy commands
            PlatformCommands::Status => {
                eprintln!("Warning: 'status' is deprecated, use 'list' instead");
                commands::platform::list::execute_platform_list()?;
            }
            PlatformCommands::Add { name, platform_type } => {
                eprintln!("Warning: 'add' is deprecated");
                commands::platform::execute_platform_add(name, platform_type)?;
            }
            PlatformCommands::Del { name } => {
                eprintln!("Warning: 'del' is deprecated, use 'delete' instead");
                commands::platform::execute_delete_command(name)?;
            }
            PlatformCommands::Init { name } => {
                commands::platform::execute_platform_init(name)?;
            }
        },
        Commands::Site { command } => match command {
            SiteCommands::Status => {
                commands::site::execute_site_status()?;
            }
            SiteCommands::Add { domain, public_key } => {
                commands::site::execute_site_add(domain, public_key)?;
            }
            SiteCommands::Del { domain } => {
                commands::site::execute_site_del(domain)?;
            }
        },
        Commands::Ssh { command } => match command {
            SshCommands::Status => {
                commands::ssh::execute_ssh_status()?;
            }
            SshCommands::Add {
                host,
                pass,
                prvkey,
                port,
            } => {
                commands::ssh::execute_ssh_add(host, pass, prvkey, port)?;
            }
            SshCommands::Del { host } => {
                commands::ssh::execute_ssh_del(host)?;
            }
            SshCommands::Init { host } => {
                commands::ssh::execute_ssh_init(host)?;
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
                ("Hosting Control Commands", vec!["ns", "platform", "hosting"]),
                ("Server Control Commands", vec!["acme", "nft", "wss", "webhook"]),
                ("Client Commands", vec!["dns", "key", "ssh", "audit"]),
                ("Common/Utility Commands", vec!["crypt", "site", "info", "init"]),
            ];

            let mut cmd = Cli::command();

            for (category, command_names) in categories {
                println!("{}:", category);
                for name in command_names {
                    if let Some(subcmd) = cmd.find_subcommand_mut(name) {
                        let about = subcmd.get_about().map(|s| s.to_string()).unwrap_or_default();
                        println!("  {:<8} - {}", name, about);

                        // Print subcommands if any
                        for sub in subcmd.get_subcommands() {
                            if sub.get_name() == "help" {
                                continue;
                            }
                            let sub_about = sub.get_about().map(|s| s.to_string()).unwrap_or_default();
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
