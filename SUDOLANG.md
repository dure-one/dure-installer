# Dure Installer - Complete Code Structure (SudoLang)

> Full structure and interface documentation for all mobile/src/**/*.rs files.
> See docs/ARCH.sudolang.md for high-level architecture summary.
> See docs/ACTOR_IMPLEMENTATION_PLAN.md for actor implementation details.

**Total Files**: 179 Rust files  
**Purpose**: Code-level reference for managing and understanding implementation details

**Architecture**: Layered with actor-based ViewModel  
- UI → ViewModel (actors) → Calc/Api  
- Actors: Platform (✓), SSH (✓), Logs (✓), Mycart (✓), DNS (planned)  
- No direct UI → Calc/Api calls (enforced by actor layer)

---

## Recent Changes (2026-11-21)

```sudo
RecentFixes {
  ssh_host_resolution {
    issue: "Service checks failed: 'no SSH host found for site=dure.co'"
    root_cause: "ui_tabs/mycart.rs used naive substring matching"
    
    before {
      // Simple contains() check - fails when formats don't match
      let ssh_host = config.ssh_hosts.iter()
        .find(|h| h.host.contains(&site));
    }
    
    after {
      // Proper resolution with explicit binding + fallback
      let site_config = config.mycart_sites.iter()
        .find(|s| s.site_address == site);
      let ssh_host = site_config
        .and_then(|s| mycart::resolve_ssh_host_for_site(&config, s));
    }
    
    affected_operations: [CheckDureMycart, CheckXmppProxy, CheckFail2BanRS]
    files_changed: [ui_tabs/mycart.rs (3 locations)]
    added: Debug logging for resolution failures
  }
  
  crontab_json_parsing {
    issue: "LoadCrontab failed: 'invalid type: map, expected a sequence'"
    root_cause: "API returns map format, code expected array"
    
    before {
      // Expected array format only
      let api_response: ApiResponse<Vec<CrontabJob>> = resp.into_json()?;
    }
    
    after {
      // Flexible parsing supports both formats
      let raw_json: serde_json::Value = resp.into_json()?;
      let jobs = match result {
        Array => serde_json::from_value(result),
        Map => map.values().map(from_value).collect(),
        _ => error
      };
    }
    
    location: api/mycart.rs::get_crontab()
    backward_compatible: true
    added: Format detection logging
  }
  
  tests_added {
    file: mobile/src/calc/mycart_tests.rs (NEW)
    tests: [
      test_resolve_ssh_host_explicit_binding,
      test_resolve_ssh_host_fallback_domain_match,
      test_resolve_ssh_host_fallback_ip_match,
      test_resolve_ssh_host_no_match,
      test_resolve_ssh_host_explicit_takes_precedence,
      test_crontab_response_array_format,
      test_crontab_response_map_format
    ]
    coverage: SSH resolution + API response parsing
  }
}
```

---

## Entry Points

```sudo
interface EntryPoints {
  constraint {
    Platform-specific entry points with shared core via lib.rs
  }
  
  lib.rs {
    // Library exports and core module declarations
    modules: [api, attestation, calc, i18n, logging, site, storage]
    platform_modules: [android, config, config_migration]
    gui_modules: [ui_components, ui_dlg, ui_tabs, viewmodel, dure]
    
    constraint {
      Feature-gated GUI modules (#[cfg(feature = "gui")])
    }
  }
  
  main.rs {
    // Desktop entry point (Linux/macOS/Windows)
    fn main() => eframe::run_native(DureApp)
    
    constraint {
      Loads config, theme, spawns GUI
    }
  }
  
  main_android.rs {
    // Android entry point (native-activity)
    fn android_main(app: AndroidApp) => eframe::run_native(DureApp)
    
    constraint {
      Android-specific initialization (JNI, ndk-context)
    }
  }
  
  main_wasm.rs {
    // WASM entry point (browser)
    fn main() => eframe::start_web(DureApp)
    
    constraint {
      WASM-specific initialization
    }
  }
}
```

---

## Core Modules

```sudo
interface CoreModules {
  config.rs {
    // Configuration management with profiles
    Config { profile: String, settings: Settings }
    Settings { theme, language, platform_config, dns_config }
    
    fn load_config(profile: &str) -> Config
    fn save_config(config: &Config)
    
    constraint {
      Profile-scoped config paths
    }
  }
  
  config_migration.rs {
    // Migrate legacy config formats
    fn migrate_config_v1_to_v2()
    
    constraint {
      Desktop only (#[cfg(not(target_arch = "wasm32"))])
    }
  }
  
  logging.rs {
    // Logging setup with platform-specific backends
    fn setup_logger(level: LevelFilter)
    
    constraint {
      Android uses android_logger, desktop uses env_logger
    }
  }
  
  logging_test.rs {
    // Logging tests
  }
  
  log_capture.rs {
    // Capture logs for UI display
    struct LogCapture { buffer: Vec<LogRecord> }
  }
  
  i18n.rs {
    // Internationalization setup
    fn setup_i18n() -> I18n
    
    constraint {
      Loads translations from embedded resources
    }
  }
  
  install.rs {
    // Installation utilities
    fn install_server()
    fn setup_systemd_service()
    
    constraint {
      Desktop only
    }
  }
  
  install_stt.rs {
    // STT (Speech-to-Text) installation
    fn install_stt_models()
  }
  
  dure.rs {
    // Main GUI application (eframe)
    struct DureApp {
      config: Config
      viewmodel: ViewModel
      current_profile: Option<ProfileContext>
      ui_state: tabs, dialogs, drawer
    }
    
    impl eframe::App for DureApp {
      fn update(&mut self, ctx: &Context, frame: &mut Frame)
    }
    
    constraint {
      Material3 theme, actor-based updates
    }
  }
  
  dure_stt.rs {
    // STT (Speech-to-Text) GUI variant
  }
  
  http_server.rs {
    // HTTP server for WASM/mobile
    fn start_http_server(port: u16)
  }
  
  tray.rs {
    // System tray icon (desktop only)
    fn create_tray_icon() -> TrayIcon
  }
  
  site/mod.rs {
    // Site management utilities
    fn register_site(domain: String, pubkey: PublicKey)
  }
}
```

---

## Android Module (8 files)

```sudo
interface AndroidModule {
  constraint {
    Platform-specific Android utilities (#[cfg(target_os = "android")])
  }
  
  mod.rs {
    // Module exports
  }
  
  activity.rs {
    // Android Activity utilities
    fn get_activity() -> Activity
    fn get_context() -> Context
  }
  
  clipboard.rs {
    // Android clipboard access
    fn set_clipboard(text: String)
    fn get_clipboard() -> String
  }
  
  contexttheme.rs {
    // Android theme/style utilities
    fn get_theme_attr(attr: i32) -> i32
  }
  
  inputmethod.rs {
    // Android IME (keyboard) control
    fn show_keyboard()
    fn hide_keyboard()
  }
  
  log.rs {
    // Android logging integration
    fn android_log(level: Level, msg: &str)
  }
  
  screensize.rs {
    // Android screen dimensions
    fn get_screen_size() -> (u32, u32)
    fn get_dpi() -> f32
  }
}
```

---

## API Module (24 files)

External API integrations for GCP, DNS providers, GitHub, mycart.

```sudo
interface ApiModule {
  mod.rs {
    // API module exports
  }
  
  desktop.rs {
    // Desktop-specific API utilities
    fn check_user_mismatch() -> Result<()>
  }
  
  ehttp_cache.rs {
    // HTTP client with caching
    struct CachedClient { cache: HashMap<Url, Response> }
    fn fetch_cached(url: &str) -> Result<Response>
  }
  
  github.rs {
    // GitHub API integration
    fn get_latest_release() -> Result<Release>
    fn download_artifact(url: &str) -> Result<Vec<u8>>
    
    constraint {
      Used for attestation verification
    }
  }
  
  mycart.rs {
    // Mycart backend API client
    fn login_and_get_token(site: &str, email: &str, password: &str) -> Result<Option<String>>
    fn verify_admin_access(site: &str, email: &str, password: &str) -> Result<Option<String>>
    fn get_mycart_stats(site: &str, jwt_token: &str) -> Result<MycartStats>
    fn get_seller_info(site: &str, jwt_token: &str) -> Result<SellerInfo>
    fn get_crontab(site: &str, jwt_token: &str) -> Result<Vec<CrontabJob>>
    fn patch_crontab(site: &str, jwt_token: &str, job_id: &str, patch: CrontabPatch) -> Result<CrontabJob>
    fn read_log_file(host_config: &SshHostConfig, log_name: &str, tail_lines: usize) -> Result<String>
    
    // Recent fix (2026-11-21): Flexible JSON parsing
    constraint {
      get_crontab() handles both array and map response formats
      Backward compatible with multiple API versions
      Logs response format (array/map/other) for debugging
    }
  }
  
  // DNS provider APIs
  ns_cloudflare.rs {
    // Cloudflare DNS API
    fn update_records(domain: &str, records: Vec<DnsRecord>) -> Result<()>
  }
  
  ns_duckdns.rs {
    // DuckDNS API
    fn update_ip(domain: &str, ip: IpAddr) -> Result<()>
  }
  
  ns_porkbun.rs {
    // Porkbun DNS API
    fn update_records(domain: &str, records: Vec<DnsRecord>) -> Result<()>
  }
  
  ns_gcp.rs {
    // GCP Cloud DNS API
    fn update_records(domain: &str, records: Vec<DnsRecord>) -> Result<()>
  }
  
  // Cloudflare sub-module
  cloudflare/ {
    mod.rs { /* exports */ }
    
    dns.rs {
      struct CloudflareDns { zone_id: String, token: String }
      fn list_records() -> Result<Vec<Record>>
      fn create_record(record: Record) -> Result<()>
    }
    
    oauth.rs {
      fn get_oauth_url() -> String
      fn exchange_code(code: &str) -> Result<Token>
    }
    
    oauth_test.rs {
      // OAuth flow tests
    }
    
    registrar.rs {
      fn list_domains() -> Result<Vec<Domain>>
      fn register_domain(domain: &str) -> Result<()>
    }
  }
  
  // GCP sub-module
  gcp/ {
    mod.rs { /* exports */ }
    
    compute.rs {
      struct GcpCompute { project_id: String, token: String }
      fn list_instances() -> Result<Vec<Instance>>
      fn create_instance(spec: InstanceSpec) -> Result<Instance>
      fn delete_instance(name: &str) -> Result<()>
    }
    
    compute_addresses.rs {
      fn list_addresses() -> Result<Vec<Address>>
      fn reserve_address(region: &str) -> Result<Address>
    }
    
    billing.rs {
      fn get_billing_account() -> Result<BillingAccount>
      fn get_project_costs(project_id: &str) -> Result<Vec<Cost>>
    }
    
    dns.rs {
      fn list_managed_zones() -> Result<Vec<ManagedZone>>
      fn update_record_sets(zone: &str, changes: Changes) -> Result<()>
    }
    
    oauth.rs {
      fn get_oauth_url(scopes: Vec<&str>) -> String
      fn exchange_code(code: &str) -> Result<Token>
      fn refresh_token(refresh: &str) -> Result<Token>
    }
    
    publicca.rs {
      fn create_certificate(domain: &str) -> Result<Certificate>
    }
    
    resourcemanager.rs {
      fn list_projects() -> Result<Vec<Project>>
      fn create_project(name: &str) -> Result<Project>
    }
    
    serviceusage.rs {
      fn enable_api(project: &str, api: &str) -> Result<()>
      fn list_enabled_services(project: &str) -> Result<Vec<String>>
    }
    
    bigquery.rs {
      fn query(sql: &str) -> Result<QueryResult>
      fn create_dataset(name: &str) -> Result<Dataset>
    }
  }
}
```

---

## Attestation Module (4 files)

Binary verification via GitHub Sigstore.

```sudo
interface AttestationModule {
  mod.rs {
    // Public exports
  }
  
  types.rs {
    struct Attestation { signature: Vec<u8>, cert: Certificate }
    struct VerificationResult { valid: bool, trusted: bool }
  }
  
  download.rs {
    fn download_attestation(artifact_url: &str) -> Result<Attestation>
    
    constraint {
      Downloads from GitHub releases
    }
  }
  
  verify.rs {
    fn verify_attestation(binary: &[u8], attestation: Attestation) -> Result<VerificationResult>
    
    constraint {
      Uses Sigstore verification, checks certificate chain
    }
  }
}
```

---

## Calc Module (19 files)

Business logic layer - profile-scoped operations.

```sudo
interface CalcModule {
  constraint {
    All operations scoped to ProfileContext
  }
  
  mod.rs {
    // Module exports
  }
  
  profile.rs {
    // Profile management
    struct ProfileContext {
      store_id: String
      user_email: String
      db_conn: SqliteConnection
    }
    
    fn create_profile(store_id: &str, email: &str) -> Result<ProfileContext>
    fn load_profile(store_id: &str) -> Result<ProfileContext>
    fn list_profiles() -> Result<Vec<Profile>>
    
    constraint {
      Scopes all DB/file operations to profile
    }
  }
  
  db.rs {
    // Profile-scoped database operations
    fn get_connection(profile: &ProfileContext) -> SqliteConnection
    fn run_migrations(conn: &SqliteConnection)
    fn query_vms(profile: &ProfileContext) -> Result<Vec<Vm>>
    fn insert_vm(vm: Vm, profile: &ProfileContext) -> Result<()>
    
    constraint {
      All queries filtered by profile.store_id
    }
  }
  
  crypt.rs {
    // Cryptographic operations
    fn generate_keypair() -> (SecretKey, PublicKey)
    fn encrypt(data: &[u8], recipient_pk: PublicKey) -> Result<Vec<u8>>
    fn decrypt(encrypted: &[u8], sk: SecretKey) -> Result<Vec<u8>>
    
    constraint {
      Uses Ed25519/X25519 key pairs
    }
  }
  
  keyring.rs {
    // Secure key storage
    fn store_key(domain: &str, username: &str, password: &str) -> Result<()>
    fn retrieve_key(domain: &str) -> Result<(String, String)>
    fn delete_key(domain: &str) -> Result<()>
    
    constraint {
      Platform-specific (keychain/secret-service/DPAPI)
    }
  }
  
  audit.rs {
    // Audit log management
    fn log_action(action: &str, profile: &ProfileContext) -> Result<()>
    fn get_audit_trail(profile: &ProfileContext) -> Result<Vec<AuditRecord>>
    fn clear_audit_logs(profile: &ProfileContext) -> Result<()>
  }
  
  session.rs {
    // Session management
    fn create_session(profile: &ProfileContext) -> Result<Session>
    fn validate_session(token: &str) -> Result<bool>
  }
  
  dns.rs {
    // DNS lookup with caching
    fn lookup_a(domain: &str) -> Result<Vec<Ipv4Addr>>
    fn lookup_aaaa(domain: &str) -> Result<Vec<Ipv6Addr>>
    fn lookup_txt(domain: &str) -> Result<Vec<String>>
    
    constraint {
      Caches results, uses multiple resolvers
    }
  }
  
  ssh.rs {
    // SSH operations
    fn connect(host: &str, port: u16, auth: SshAuth) -> Result<SshSession>
    fn execute_command(session: &SshSession, cmd: &str) -> Result<String>
    fn upload_file(session: &SshSession, local: &Path, remote: &str) -> Result<()>
    
    constraint {
      Uses russh (pure Rust SSH)
    }
  }
  
  docker.rs {
    // Docker operations via SSH
    fn install_docker(session: &SshSession) -> Result<()>
    fn deploy_container(session: &SshSession, image: &str) -> Result<String>
    fn list_containers(session: &SshSession) -> Result<Vec<Container>>
  }
  
  acme.rs {
    // ACME protocol (Let's Encrypt)
    fn create_account() -> Result<Account>
    fn order_certificate(domain: &str) -> Result<Certificate>
    fn install_certificate(domain: &str, cert: Certificate) -> Result<()>
  }
  
  gcp.rs {
    // GCP Compute Engine client and operations (merged from platform_gcp + hosting_gcp)
    // Core GCP client
    fn provision_vm(spec: VmSpec, profile: &ProfileContext) -> Result<Vm>
    fn destroy_vm(vm_id: &str, profile: &ProfileContext) -> Result<()>
    fn get_vm_status(vm_id: &str) -> Result<VmStatus>
    
    // Module: gcp::hosting
    mod hosting {
      fn generate_ssh_keypair() -> Result<(String, Vec<u8>)>
      fn delete_vm(client: &GcpRestClient, vm: &VmInstance) -> Result<String>
      fn restart_vm(client: &GcpRestClient, vm: &VmInstance) -> Result<String>
      fn regenerate_vm(client: &GcpRestClient, platform: &mut CloudPlatformConfig, zone: &str) -> Result<String>
    }
    
    // Module: gcp::platform
    mod platform {
      fn compute_platform_status(platform: &CloudPlatformConfig, total_projects: usize) -> String
      fn compute_project_status(platform: &CloudPlatformConfig, current_ip: &str, whitelisted: bool) -> String
    }
    
    constraint {
      Calls api::gcp, stores state in DB
    }
  }
  
  mycart.rs {
    // Mycart business logic layer
    fn add_mycart_site(config_path: &Path, site_address: &str, email: &str, password: &str) -> Result<()>
    fn list_mycart_sites(config_path: &Path) -> Result<Vec<MycartSiteConfig>>
    fn delete_mycart_site(config_path: &Path, site_address: &str) -> Result<()>
    fn get_site_stats(config_path: &Path, site_address: &str) -> Result<MycartStats>
    fn get_seller_info(config_path: &Path, site_address: &str) -> Result<SellerInfo>
    fn get_site_crontab(config_path: &Path, site_address: &str) -> Result<Vec<CrontabJob>>
    fn list_crontab(site_address: &str, jwt_token: &str) -> Result<Vec<CrontabJob>>
    fn set_crontab_enabled(site_address: &str, jwt_token: &str, job_id: &str, enabled: bool) -> Result<CrontabJob>
    fn set_crontab_interval(site_address: &str, jwt_token: &str, job_id: &str, interval: &str) -> Result<CrontabJob>
    fn get_duremycart_logs(host_config: &SshHostConfig, profile_keyring: Option<&Arc<DatabaseHandle>>) -> Result<Vec<String>>
    fn get_xmppproxy_logs(host_config: &SshHostConfig, profile_keyring: Option<&Arc<DatabaseHandle>>) -> Result<Vec<String>>
    fn get_fail2banrs_logs(host_config: &SshHostConfig, profile_keyring: Option<&Arc<DatabaseHandle>>) -> Result<Vec<String>>
    fn resolve_platform_for_site(config: &AppConfig, site: &MycartSiteConfig) -> Option<&CloudPlatformConfig>
    fn resolve_ssh_host_for_site(config: &AppConfig, site: &MycartSiteConfig) -> Option<&SshHostConfig>
    fn load_site_operations(site_address: &str, limit: i64) -> Result<Vec<OperationLog>>
    
    // Recent fix (2026-11-21): SSH host resolution
    constraint {
      resolve_ssh_host_for_site() supports explicit binding + domain/IP fallback
      Used by CheckDureMycart, CheckXmppProxy, CheckFail2BanRS in ui_tabs/mycart.rs
    }
  }
  
  mycart_tests.rs {
    // Unit tests for mycart business logic (NEW 2026-11-21)
    test_resolve_ssh_host_explicit_binding()
    test_resolve_ssh_host_fallback_domain_match()
    test_resolve_ssh_host_fallback_ip_match()
    test_resolve_ssh_host_no_match()
    test_resolve_ssh_host_explicit_takes_precedence()
    test_crontab_response_array_format()
    test_crontab_response_map_format()
    
    constraint {
      Tests SSH resolution logic and API response parsing
    }
  }
  
  dure_mycart.rs {
    // Dure-specific mycart operations
  }
  
  env_parser.rs {
    // Environment variable parsing
    fn parse_env_file(path: &Path) -> Result<HashMap<String, String>>
  }
}
```

---

## CLI Module (19 files)

Command-line interface implementation.

```sudo
interface CliModule {
  constraint {
    Reuses ViewModel actors for headless operation
  }
  
  mod.rs {
    // CLI definition (clap)
    enum Commands {
      Ns(NsCommands)
      Platform(PlatformCommands)
      Dns(DnsCommands)
      Key(KeyCommands)
      Ssh(SshCommands)
      Audit(AuditCommands)
      Crypt(CryptCommands)
      Site(SiteCommands)
      Info
      Init
    }
    
    fn run_cli_mode() -> Result<()>
  }
  
  output/components/stats.rs {
    // CLI output formatting
    fn format_table(data: Vec<Vec<String>>)
    fn format_json(value: serde_json::Value)
  }
  
  commands/ {
    mod.rs { /* command implementations */ }
    
    audit.rs {
      fn execute_audit_status(profile: &ProfileContext) -> Result<()>
      fn execute_audit_clear(profile: &ProfileContext) -> Result<()>
    }
    
    crypt.rs {
      fn execute_crypt_status() -> Result<()>
      fn execute_crypt_enc(recipient: &str, data: &str) -> Result<()>
      fn execute_crypt_dec(encrypted: &str) -> Result<()>
    }
    
    dns.rs {
      fn execute_dns_a(domain: &str) -> Result<()>
      fn execute_dns_aaaa(domain: &str) -> Result<()>
      fn execute_dns_txt(domain: &str) -> Result<()>
      fn execute_dns_bastion(ip: &str) -> Result<()>
    }
    
    keyring.rs {
      fn execute_key_status(profile: &ProfileContext) -> Result<()>
      fn execute_key_add(domain: &str, user: &str, pass: &str) -> Result<()>
      fn execute_key_del(domain: &str) -> Result<()>
      fn execute_key_save(output: Option<&str>) -> Result<()>
      fn execute_key_load(input: &str) -> Result<()>
    }
    
    ssh.rs {
      fn execute_ssh_status(profile: &ProfileContext) -> Result<()>
      fn execute_ssh_add(host: &str, auth: SshAuth, profile: &ProfileContext) -> Result<()>
      fn execute_ssh_del(host: &str, profile: &ProfileContext) -> Result<()>
      fn execute_ssh_init(host: &str, profile: &ProfileContext) -> Result<()>
    }
    
    platform/ {
      mod.rs { /* platform command router */ }
      
      helpers.rs {
        fn format_instance_status(instance: &Instance) -> String
      }
      
      runner.rs {
        fn run_platform_command(args: Vec<String>, profile: &ProfileContext) -> Result<()>
      }
      
      vm.rs {
        fn list_vms(profile: &ProfileContext) -> Result<()>
        fn create_vm(spec: VmSpec, profile: &ProfileContext) -> Result<()>
        fn delete_vm(id: &str, profile: &ProfileContext) -> Result<()>
      }
      
      billing.rs {
        fn show_billing(profile: &ProfileContext) -> Result<()>
      }
      
      firewall.rs {
        fn list_firewall_rules(profile: &ProfileContext) -> Result<()>
        fn add_firewall_rule(rule: FirewallRule, profile: &ProfileContext) -> Result<()>
      }
      
      list.rs {
        fn list_platforms(profile: &ProfileContext) -> Result<()>
      }
      
      tests.rs {
        // CLI command tests
      }
    }
  }
}
```

---

## Storage Module (12 files)

Database schema, models, and migrations.

```sudo
interface StorageModule {
  constraint {
    Diesel ORM with SQLite (PostgreSQL support)
  }
  
  mod.rs {
    // Storage module exports
    fn establish_connection(db_path: &Path) -> SqliteConnection
  }
  
  diesel_schema.rs {
    // Generated Diesel schema
    table! {
      profiles { id, store_id, user_email, created_at }
      vms { id, profile_id, name, ip, status }
      dns_records { id, profile_id, domain, type, value }
      ssh_hosts { id, profile_id, host, port, auth }
      audit_logs { id, profile_id, action, timestamp }
      sessions { id, profile_id, token, expires_at }
      sites { id, profile_id, domain, pubkey }
      devices { id, profile_id, device_id, name }
      crypt_keys { id, profile_id, pubkey, privkey }
      opslog { id, profile_id, operation, status }
      mycart_sites { id, profile_id, store_id, domain }
    }
  }
  
  migrations/mod.rs {
    // Diesel migrations
    fn run_pending_migrations(conn: &SqliteConnection)
  }
  
  models/ {
    mod.rs { /* model exports */ }
    
    audit.rs {
      #[derive(Queryable, Insertable)]
      struct AuditLog { id, profile_id, action, timestamp }
    }
    
    crypt.rs {
      #[derive(Queryable, Insertable)]
      struct CryptKey { id, profile_id, pubkey, privkey }
    }
    
    device.rs {
      #[derive(Queryable, Insertable)]
      struct Device { id, profile_id, device_id, name }
    }
    
    dns.rs {
      #[derive(Queryable, Insertable)]
      struct DnsRecord { id, profile_id, domain, type, value }
    }
    
    mod.rs {
      // Profile model
      #[derive(Queryable, Insertable)]
      struct Profile { id, store_id, user_email, created_at }
    }
    
    mycart_site.rs {
      #[derive(Queryable, Insertable)]
      struct MycartSite { id, profile_id, store_id, domain }
    }
    
    opslog.rs {
      #[derive(Queryable, Insertable)]
      struct OpsLog { id, profile_id, operation, status }
    }
    
    session.rs {
      #[derive(Queryable, Insertable)]
      struct Session { id, profile_id, token, expires_at }
    }
    
    site.rs {
      #[derive(Queryable, Insertable)]
      struct Site { id, profile_id, domain, pubkey }
    }
  }
}
```

---

## UI Components Module (10 files)

Reusable GUI components.

```sudo
interface UiComponentsModule {
  constraint {
    Feature-gated (#[cfg(feature = "gui")])
  }
  
  mod.rs {
    // Component exports
  }
  
  action_menu.rs {
    // Context menu component
    struct ActionMenu { items: Vec<MenuItem> }
    fn show(ui: &mut Ui, pos: Pos2)
  }
  
  emoji_loader.rs {
    // Emoji texture loader
    fn load_emoji(ctx: &Context, emoji: &str) -> TextureHandle
  }
  
  emoji_progress.rs {
    // Progress indicator with emoji
    struct EmojiProgress { emoji: String, progress: f32 }
    fn show(ui: &mut Ui)
  }
  
  status_grid.rs {
    // Status grid display
    fn show_status_grid(ui: &mut Ui, items: Vec<StatusItem>)
  }
  
  drawer/ {
    mod.rs { /* drawer exports */ }
    
    logs_renderer.rs {
      // Render log messages in drawer
      fn render_logs(ui: &mut Ui, logs: &[LogRecord])
    }
    
    operations_renderer.rs {
      // Render active operations
      fn render_operations(ui: &mut Ui, ops: &HashMap<String, Operation>)
    }
    
    status_utils.rs {
      // Status formatting utilities
      fn format_status(status: Status) -> String
    }
    
    tab_bar.rs {
      // Drawer tab bar
      fn render_tab_bar(ui: &mut Ui, selected: &mut usize)
    }
  }
}
```

---

## UI Dialogs Module (27 files)

Modal dialogs for GUI.

```sudo
interface UiDialogsModule {
  constraint {
    Feature-gated (#[cfg(feature = "gui")])
  }
  
  mod.rs {
    // Dialog exports
  }
  
  about.rs {
    struct DlgAbout { version: String, open: bool }
    fn show(ctx: &Context)
  }
  
  about_stt.rs {
    // STT variant of about dialog
  }
  
  clipboard_popup.rs {
    struct ClipboardPopup { text: String, pos: Pos2 }
    fn show(ctx: &Context)
  }
  
  db_debugger.rs {
    struct DlgDbDebugger { sql: String, results: Vec<Row> }
    fn show(ctx: &Context, conn: &SqliteConnection)
  }
  
  hosting_gcp_wizard.rs {
    // GCP hosting setup wizard
    struct GcpWizard { step: usize, project: String, region: String }
    fn show(ctx: &Context, profile: &ProfileContext)
  }
  
  hosting_pricing.rs {
    // Hosting pricing calculator
    fn show(ctx: &Context)
  }
  
  identity_delete_confirm.rs {
    // Confirm identity deletion
    fn show(ctx: &Context, identity_id: &str)
  }
  
  identity_passphrase_generate.rs {
    // Generate secure passphrase
    fn show(ctx: &Context) -> Option<String>
  }
  
  mycart_add_menu_category.rs {
    // Add menu category dialog
    fn show(ctx: &Context, profile: &ProfileContext)
  }
  
  mycart_add_product_variant.rs {
    // Add product variant
    fn show(ctx: &Context, product_id: &str)
  }
  
  mycart_product_detail.rs {
    // Product detail view/edit
    struct ProductDetailDialog { product: Product, dirty: bool }
    fn show(ctx: &Context, profile: &ProfileContext)
  }
  
  mycart_product_editor.rs {
    // Product editor
    fn show(ctx: &Context, product: &mut Product)
  }
  
  mycart_product_editor_stt.rs {
    // STT variant
  }
  
  platform_gcp.rs {
    // GCP platform configuration dialog
    fn show(ctx: &Context, profile: &ProfileContext)
  }
  
  platform_gcp_stt.rs {
    // STT variant
  }
  
  profile_create.rs {
    struct DlgProfileCreate { store_id: String, email: String }
    fn show(ctx: &Context) -> Option<ProfileContext>
  }
  
  profile_delete.rs {
    struct DlgProfileDelete { profile_name: String }
    fn show(ctx: &Context, profile: &ProfileContext)
  }
  
  profile_login.rs {
    struct DlgProfileLogin { profiles: Vec<Profile>, selected: usize }
    fn show(ctx: &Context) -> Option<ProfileContext>
  }
  
  settings.rs {
    struct DlgSettings { settings: Settings, dirty: bool }
    fn show(ctx: &Context, config: &mut Config)
  }
  
  settings_stt.rs {
    // STT variant
  }
  
  ssh_add_host.rs {
    struct SshAddHostDialog { host: String, port: u16, auth: SshAuth }
    fn show(ctx: &Context, profile: &ProfileContext)
  }
  
  ssh_add_host_stt.rs {
    // STT variant
  }
  
  ssh_remote_dialog.rs {
    // SSH remote operations dialog
    fn show(ctx: &Context, session: &SshSession)
  }
  
  ssh_remote_dialog_stt.rs {
    // STT variant
  }
}
```

---

## UI Tabs Module (18 files)

Main application tabs for GUI.

```sudo
interface UiTabsModule {
  constraint {
    Feature-gated (#[cfg(feature = "gui")])
  }
  
  mod.rs {
    // Tab exports
  }
  
  platform.rs {
    // Platform tab (GCP VMs, billing)
    struct PlatformTab { vms: Vec<Vm>, selected: Option<String> }
    fn show(ui: &mut Ui, viewmodel: &ViewModel, profile: &ProfileContext)
    
    constraint {
      Sends PlatformCommand to actor, receives PlatformEvent
    }
  }
  
  platform_stt.rs {
    // STT variant
  }
  
  ssh.rs {
    // SSH tab (host management)
    struct SshTab { hosts: Vec<SshHost>, selected: Option<String> }
    fn show(ui: &mut Ui, viewmodel: &ViewModel, profile: &ProfileContext)
    
    constraint {
      Sends SshCommand to actor, receives SshEvent
    }
  }
  
  ssh_stt.rs {
    // STT variant
  }
  
  settings.rs {
    // Settings tab
    struct SettingsTab { settings: Settings }
    fn show(ui: &mut Ui, config: &mut Config)
  }
  
  settings_stt.rs {
    // STT variant
  }
  
  dashboard.rs {
    // Dashboard tab (overview)
    fn show(ui: &mut Ui, viewmodel: &ViewModel, profile: &ProfileContext)
  }
  
  identity.rs {
    // Identity management tab
    fn show(ui: &mut Ui, profile: &ProfileContext)
  }
}
```

---

## ViewModel Module (35 files, 3 actors implemented)

Actor-based MVVM layer coordinating business logic and UI.

**Status**: Platform ✓, SSH ✓, Logs ✓ | Mycart (planned), DNS (planned)

```sudo
interface ViewModelModule {
  constraint {
    Actor-based with smol async runtime
    Profile-scoped operations
    UI → ViewModel (actors) → Calc/Api (no direct UI → Calc/Api calls)
  }
  
  mod.rs {
    // ViewModel coordinator
    struct ViewModel {
      platform_tx: Sender<PlatformCommand>
      ssh_tx: Sender<SshCommand>
      logs_tx: Sender<LogsCommand>
      mycart_tx: Sender<MycartCommand>  // planned
      dns_tx: Sender<DnsCommand>        // planned
      event_rx: Receiver<ViewModelEvent>
      state: ViewModelState
      profile: Option<ProfileContext>
      #[cfg(feature = "gui")] egui_ctx: Option<egui::Context>
    }
    
    fn new(ctx: egui::Context, profile: Option<ProfileContext>) -> Self
    fn send_command(&self, cmd: Command)
    fn poll_events(&mut self) -> Vec<ViewModelEvent>
    
    constraint {
      Spawns actor threads, routes commands, aggregates events
    }
  }
  
  common.rs {
    // Shared types
    enum ViewModelEvent { Platform, Ssh, Logs, Mycart, Dns }
    struct ViewModelState { active_operations, recent_errors, textures }
  }
  
  io.rs {
    // I/O operations
    fn read_file_async(path: &Path) -> Result<Vec<u8>>
    fn write_file_async(path: &Path, data: &[u8]) -> Result<()>
  }
  
  runtime.rs {
    // smol runtime management
    fn spawn_runtime() -> RuntimeHandle
    fn spawn_task<F: Future>(future: F)
  }
  
  tests.rs {
    // ViewModel tests
  }
  
  // ===== IMPLEMENTED ACTORS =====
  
  platform/ {
    // Platform actor: GCP VM management + billing drawer
    constraint {
      Drawer UI pattern (operations + logs tabs)
    }
    
    mod.rs {
      pub use actor::PlatformActor;
      pub use commands::{DeleteOptions, FirewallRuleData, PlatformCommand};
      pub use drawer_actor::DrawerActor;
      pub use drawer_repository::DrawerRepository;
      pub use drawer_types::{DrawerCommand, DrawerEvent, DrawerState, DrawerTab};
      pub use events::{
        FirewallRuleInfo, FirewallStatus, FullFirewallRule, PlatformEvent,
        SshStatus, VmInfo, VmStatus
      };
    }
    
    actor.rs {
      struct PlatformActor {
        cmd_rx: Receiver<PlatformCommand>
        event_tx: Sender<PlatformEvent>
        profile: ProfileContext
      }
      
      fn spawn() -> (Sender<PlatformCommand>, Receiver<PlatformEvent>)
      
      // Commands: ListVms, CreateVm, DeleteVm, StartVm, StopVm, RestartVm,
      //           GetFirewallRules, UpdateFirewallRule, GetSshStatus
      
      constraint {
        Calls api::gcp::* (compute, billing, dns)
        Stores state in calc::db
      }
    }
    
    commands.rs {
      enum PlatformCommand {
        ListVms, CreateVm(VmSpec), DeleteVm { name, options: DeleteOptions },
        StartVm(String), StopVm(String), RestartVm(String),
        GetFirewallRules, UpdateFirewallRule(FirewallRuleData),
        GetSshStatus, CheckProjectSetup
      }
      
      struct DeleteOptions { delete_disk, delete_ip }
      struct FirewallRuleData { name, allowed_ips }
    }
    
    events.rs {
      enum PlatformEvent {
        VmInfo(VmInfo), VmCreated(VmInfo), VmDeleted(String),
        VmStarted(String), VmStopped(String), VmRestarted(String),
        FirewallRules(Vec<FullFirewallRule>),
        FirewallUpdated(FirewallRuleInfo),
        SshStatus(SshStatus), Error(String), Progress(String)
      }
      
      struct VmInfo { name, status: VmStatus, zone, ip, machine_type }
      struct FullFirewallRule { name, status: FirewallStatus, allowed, target_tags }
      struct SshStatus { reachable, last_check, error }
    }
    
    drawer_actor.rs {
      // Drawer UI actor (billing/logs viewer)
      struct DrawerActor {
        cmd_rx: Receiver<DrawerCommand>
        event_tx: Sender<DrawerEvent>
        repo: DrawerRepository
      }
      
      fn spawn() -> (Sender<DrawerCommand>, Receiver<DrawerEvent>)
    }
    
    drawer_repository.rs {
      // Data access for drawer
      struct DrawerRepository { profile: ProfileContext }
      
      fn get_billing_data() -> Result<BillingData>
      fn get_logs(limit: usize) -> Result<Vec<OperationLog>>
    }
    
    drawer_types.rs {
      enum DrawerTab { Operations, Logs }
      enum DrawerCommand { SwitchTab(DrawerTab), RefreshLogs, RefreshBilling }
      enum DrawerEvent { LogsUpdated(Vec<OperationLog>), BillingUpdated(BillingData) }
      struct DrawerState { active_tab, logs, billing }
    }
    
    tests.rs {
      // Platform actor tests
    }
  }
  
  ssh/ {
    // SSH actor: host/container management + operations drawer
    constraint {
      Drawer UI pattern (operations + logs tabs)
    }
    
    mod.rs {
      pub use actor::SshActor;
      pub use commands::SshCommand;
      pub use drawer_types::{
        ContainerInfo, DockerStatus, DrawerCommand, DrawerEvent,
        DrawerState, DrawerTab, DureStatus, HostInfo
      };
      pub use events::{DockerContainer, OperationLog, SshEvent, SshHostInfo};
    }
    
    actor.rs {
      struct SshActor {
        cmd_rx: Receiver<SshCommand>
        event_tx: Sender<SshEvent>
        profile: ProfileContext
      }
      
      fn spawn() -> (Sender<SshCommand>, Receiver<SshEvent>)
      
      // Commands: ListHosts, AddHost, RemoveHost, TestConnection,
      //           ExecuteCommand, DeployContainer, ListContainers,
      //           StartContainer, StopContainer, GetLogs
      
      constraint {
        Calls calc::ssh::* (connect, execute, upload)
        Calls calc::docker::* (deploy, list, control)
      }
    }
    
    commands.rs {
      enum SshCommand {
        ListHosts, AddHost(SshHostInfo), RemoveHost(String),
        TestConnection(String), ExecuteCommand { host, command },
        DeployContainer { host, image, name }, ListContainers(String),
        StartContainer { host, container_id }, StopContainer { host, container_id },
        GetLogs { host, container_id, lines }
      }
    }
    
    events.rs {
      enum SshEvent {
        HostsListed(Vec<SshHostInfo>), HostAdded(SshHostInfo), HostRemoved(String),
        ConnectionTest { host, success, error },
        CommandOutput { host, output, error },
        ContainerDeployed { host, container_id },
        ContainersListed { host, containers: Vec<DockerContainer> },
        ContainerStarted { host, container_id }, ContainerStopped { host, container_id },
        Logs { host, container_id, logs: String },
        Error(String), Progress(String)
      }
      
      struct SshHostInfo { name, host, port, username, key_path }
      struct DockerContainer { id, name, image, status: DockerStatus, ports }
    }
    
    drawer_types.rs {
      enum DrawerTab { Operations, Logs }
      enum DrawerCommand { SwitchTab(DrawerTab), RefreshLogs, RefreshHosts }
      enum DrawerEvent { LogsUpdated(Vec<OperationLog>), HostsUpdated(Vec<HostInfo>) }
      struct DrawerState { active_tab, logs, hosts }
      
      struct HostInfo { name, host, status: DureStatus }
      struct ContainerInfo { id, name, image, status: DockerStatus }
      enum DureStatus { Running, Stopped, Error }
      enum DockerStatus { Running, Stopped, Restarting }
    }
    
    tests.rs {
      // SSH actor tests
    }
  }
  
  logs/ {
    // Logs actor: unified operation logging for all tabs
    constraint {
      Used by platform, ssh, mycart, dns for operation tracking
    }
    
    mod.rs {
      pub use actor::{LogsActor, LogsCommand, LogsEvent};
      pub use opslog_actor::OpsLogActor;
      pub use opslog_commands::OpsLogCommand;
      pub use opslog_events::OpsLogEvent;
      pub use types::{LogLevel, LogRecord, OperationLog};
    }
    
    actor.rs {
      // Main logs aggregation actor
      struct LogsActor {
        cmd_rx: Receiver<LogsCommand>
        event_tx: Sender<LogsEvent>
      }
      
      fn spawn() -> (Sender<LogsCommand>, Receiver<LogsEvent>)
      
      enum LogsCommand { Subscribe(String), GetRecent(usize), Clear }
      enum LogsEvent { NewLog(LogRecord), HistoryLoaded(Vec<LogRecord>) }
    }
    
    opslog_actor.rs {
      // Operation-specific logging actor
      struct OpsLogActor {
        cmd_rx: Receiver<OpsLogCommand>
        event_tx: Sender<OpsLogEvent>
        buffer: VecDeque<OperationLog>
      }
      
      fn spawn() -> (Sender<OpsLogCommand>, Receiver<OpsLogEvent>)
      
      constraint {
        Per-tab operation logging (platform, ssh, mycart, dns)
      }
    }
    
    opslog_commands.rs {
      enum OpsLogCommand {
        LogOperation { tab: String, operation: String, status: String },
        GetLogs { tab: Option<String>, limit: usize },
        ClearLogs(String)
      }
    }
    
    opslog_events.rs {
      enum OpsLogEvent {
        OperationLogged(OperationLog),
        LogsRetrieved(Vec<OperationLog>),
        LogsCleared(String)
      }
    }
    
    types.rs {
      struct LogRecord { timestamp, level: LogLevel, message, source }
      struct OperationLog { id, tab, operation, status, timestamp, details }
      enum LogLevel { Debug, Info, Warn, Error }
    }
  }
  
  // ===== PLANNED ACTORS (see docs/ACTOR_IMPLEMENTATION_PLAN.md) =====
  
  mycart/ {
    // Mycart actor: remote backend API + SSH log fetching
    // Status: PLANNED (not implemented)
    constraint {
      Drawer UI pattern (operations + logs tabs)
      Dual mode - REST API + SSH logs
    }
    
    mod.rs {
      // pub use actor::MycartActor;
      // pub use commands::{LogType, MycartCommand};
      // pub use drawer_types::{DrawerCommand, DrawerEvent, DrawerState, DrawerTab};
      // pub use events::{MycartEvent, Order, Product, StoreHealth};
    }
    
    // Commands: CheckStoreHealth, ListProducts, GetProduct, CreateProduct,
    //           UpdateProduct, DeleteProduct, ListOrders, GetOrder,
    //           UpdateOrderStatus, FetchLogs, StreamLogs, StopLogStream
    
    // Events: StoreHealthChecked(StoreHealth), ProductsListed(Vec<Product>),
    //         ProductCreated(Product), OrdersListed(Vec<Order>),
    //         LogsFetched(String), LogsStreaming(String)
    
    constraint {
      REST API - https://mycart.dure.app/api/_/*
      SSH logs - /var/log/mycart/app.log, nginx, syslog
    }
  }
  
  dns/ {
    // DNS actor: Cloudflare DNS management
    // Status: PLANNED (not implemented)
    constraint {
      Drawer UI pattern (operations + logs tabs)
      Cloudflare API only (extracted from ui_tabs/ssh)
    }
    
    mod.rs {
      // pub use actor::DnsActor;
      // pub use commands::DnsCommand;
      // pub use drawer_types::{DrawerCommand, DrawerEvent, DrawerState, DrawerTab};
      // pub use events::{DnsEvent, DnsRecord, DnsZone};
    }
    
    // Commands: ListZones, GetZone, ListRecords, CreateRecord,
    //           UpdateRecord, DeleteRecord, BulkUpdateRecords
    
    // Events: ZonesListed(Vec<DnsZone>), RecordsListed(Vec<DnsRecord>),
    //         RecordCreated(DnsRecord), RecordUpdated(DnsRecord),
    //         RecordDeleted(String)
    
    constraint {
      Uses api::cloudflare::dns (zone mgmt, record CRUD)
    }
  }
}
```

---

## File Count Summary

```sudo
FileCounts {
  total: 187
  
  by_module: {
    viewmodel: 29
    ui_dlg: 27
    api: 24
    cli: 19
    calc: 19
    ui_tabs: 18
    storage: 12
    ui_components: 10
    android: 8
    attestation: 4
    root: 17
  }
  
  constraint {
    All files profile-scoped except entry points and platform utilities
  }
}
```

---

## Usage for Code Management

```sudo
OnCodeChange {
  constraint {
    Update this document when adding/removing/renaming files
  }
  
  AddNewFile {
    1. Create file in appropriate module
    2. Add to module's mod.rs exports
    3. Document in this SUDOLANG.md under correct section
    4. Add to relevant actor if part of ViewModel
  }
  
  RefactorModule {
    1. Update file structure
    2. Update mod.rs exports
    3. Update SUDOLANG.md sections
    4. Update ARCH.sudolang.md if architecture changes
  }
  
  FindImplementation(feature) {
    // Use this doc to locate files implementing feature
    match feature {
      "GCP VM" => [calc/gcp.rs, api/gcp/compute.rs, viewmodel/platform/actor.rs, ui_tabs/platform.rs]
      "DNS" => [calc/ns.rs, api/ns_*.rs, viewmodel/ns/actor.rs, ui_tabs/ns.rs]
      "SSH" => [calc/ssh.rs, viewmodel/ssh/actor.rs, ui_tabs/ssh.rs]
      "Profile" => [calc/profile.rs, storage/models/mod.rs, ui_dlg/profile_*.rs]
    }
  }
}
```

---

## Cross-Reference with ARCH.sudolang.md

```sudo
Relationship {
  constraint {
    ARCH.sudolang.md = high-level architecture
    SUDOLANG.md (this) = code-level structure
  }
  
  ARCH.sudolang.md {
    // Shows: layers, actors, data flow, patterns
    // Does not show: individual files, function signatures
  }
  
  SUDOLANG.md {
    // Shows: all files, key interfaces, module organization
    // Does not show: implementation details, algorithms
  }
  
  Usage {
    architecture_design => read ARCH.sudolang.md
    code_navigation => read SUDOLANG.md
    implementation => read actual .rs files
  }
}
```
