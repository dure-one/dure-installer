# Dure Installer - Complete Code Structure (SudoLang)

> Full structure and interface documentation for all mobile/src/**/*.rs files.
> See docs/ARCH.sudolang.md for high-level architecture summary.

**Total Files**: 187 Rust files  
**Purpose**: Code-level reference for managing and understanding implementation details

---

## Entry Points

```sudo
interface EntryPoints {
  constraint: Platform-specific entry points with shared core via lib.rs
  
  lib.rs {
    // Library exports and core module declarations
    modules: [api, attestation, calc, i18n, logging, site, storage]
    platform_modules: [android, config, config_migration]
    gui_modules: [ui_components, ui_dlg, ui_tabs, viewmodel, dure]
    constraint: Feature-gated GUI modules (#[cfg(feature = "gui")])
  }
  
  main.rs {
    // Desktop entry point (Linux/macOS/Windows)
    fn main() => eframe::run_native(DureApp)
    constraint: Loads config, theme, spawns GUI
  }
  
  main_android.rs {
    // Android entry point (native-activity)
    fn android_main(app: AndroidApp) => eframe::run_native(DureApp)
    constraint: Android-specific initialization (JNI, ndk-context)
  }
  
  main_wasm.rs {
    // WASM entry point (browser)
    fn main() => eframe::start_web(DureApp)
    constraint: WASM-specific initialization
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
    constraint: Profile-scoped config paths
  }
  
  config_migration.rs {
    // Migrate legacy config formats
    fn migrate_config_v1_to_v2()
    constraint: Desktop only (#[cfg(not(target_arch = "wasm32"))])
  }
  
  logging.rs {
    // Logging setup with platform-specific backends
    fn setup_logger(level: LevelFilter)
    constraint: Android uses android_logger, desktop uses env_logger
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
    constraint: Loads translations from embedded resources
  }
  
  install.rs {
    // Installation utilities
    fn install_server()
    fn setup_systemd_service()
    constraint: Desktop only
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
    
    constraint: Material3 theme, actor-based updates
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
  constraint: Platform-specific Android utilities (#[cfg(target_os = "android")])
  
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
  
  packagemanager.rs {
    // Android PackageManager utilities
    fn get_package_name() -> String
    fn get_app_version() -> String
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
    constraint: Used for attestation verification
  }
  
  mycart.rs {
    // Mycart backend API client
    fn create_store(name: &str) -> Result<Store>
    fn list_products() -> Result<Vec<Product>>
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
    constraint: Downloads from GitHub releases
  }
  
  verify.rs {
    fn verify_attestation(binary: &[u8], attestation: Attestation) -> Result<VerificationResult>
    constraint: Uses Sigstore verification, checks certificate chain
  }
}
```

---

## Calc Module (19 files)

Business logic layer - profile-scoped operations.

```sudo
interface CalcModule {
  constraint: All operations scoped to ProfileContext
  
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
    
    constraint: Scopes all DB/file operations to profile
  }
  
  db.rs {
    // Profile-scoped database operations
    fn get_connection(profile: &ProfileContext) -> SqliteConnection
    fn run_migrations(conn: &SqliteConnection)
    fn query_vms(profile: &ProfileContext) -> Result<Vec<Vm>>
    fn insert_vm(vm: Vm, profile: &ProfileContext) -> Result<()>
    
    constraint: All queries filtered by profile.store_id
  }
  
  crypt.rs {
    // Cryptographic operations
    fn generate_keypair() -> (SecretKey, PublicKey)
    fn encrypt(data: &[u8], recipient_pk: PublicKey) -> Result<Vec<u8>>
    fn decrypt(encrypted: &[u8], sk: SecretKey) -> Result<Vec<u8>>
    
    constraint: Uses Ed25519/X25519 key pairs
  }
  
  keyring.rs {
    // Secure key storage
    fn store_key(domain: &str, username: &str, password: &str) -> Result<()>
    fn retrieve_key(domain: &str) -> Result<(String, String)>
    fn delete_key(domain: &str) -> Result<()>
    
    constraint: Platform-specific (keychain/secret-service/DPAPI)
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
    
    constraint: Caches results, uses multiple resolvers
  }
  
  ns.rs {
    // Nameserver management orchestration
    fn register_domain(domain: &str, provider: Provider, token: &str, profile: &ProfileContext) -> Result<()>
    fn update_records(domain: &str, records: Vec<DnsRecord>, profile: &ProfileContext) -> Result<()>
    fn delete_domain(domain: &str, profile: &ProfileContext) -> Result<()>
    
    constraint: Routes to appropriate API module based on provider
  }
  
  ssh.rs {
    // SSH operations
    fn connect(host: &str, port: u16, auth: SshAuth) -> Result<SshSession>
    fn execute_command(session: &SshSession, cmd: &str) -> Result<String>
    fn upload_file(session: &SshSession, local: &Path, remote: &str) -> Result<()>
    
    constraint: Uses russh (pure Rust SSH)
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
    // GCP orchestration
    fn provision_vm(spec: VmSpec, profile: &ProfileContext) -> Result<Vm>
    fn destroy_vm(vm_id: &str, profile: &ProfileContext) -> Result<()>
    fn get_vm_status(vm_id: &str) -> Result<VmStatus>
    
    constraint: Calls api::gcp, stores state in DB
  }
  
  platform_gcp.rs {
    // GCP platform-level operations
    fn setup_project(name: &str, profile: &ProfileContext) -> Result<Project>
    fn enable_apis(project: &str) -> Result<()>
  }
  
  hosting_gcp.rs {
    // GCP hosting utilities
    fn setup_hosting(domain: &str, profile: &ProfileContext) -> Result<()>
  }
  
  site.rs {
    // Site management
    fn register_site(domain: &str, pubkey: PublicKey, profile: &ProfileContext) -> Result<()>
    fn list_sites(profile: &ProfileContext) -> Result<Vec<Site>>
  }
  
  mycart.rs {
    // Mycart integration
    fn create_store(name: &str, profile: &ProfileContext) -> Result<Store>
    fn sync_products(profile: &ProfileContext) -> Result<()>
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
  constraint: Reuses ViewModel actors for headless operation
  
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
    
    ns.rs {
      fn execute_ns_status(profile: &ProfileContext, domain: Option<&str>) -> Result<()>
      fn execute_ns_add(domain: &str, provider: &str, token: &str, profile: &ProfileContext) -> Result<()>
      fn execute_ns_del(domain: &str, profile: &ProfileContext) -> Result<()>
      fn execute_ns_insert(type: &str, domain: &str, value: &str, apply: bool) -> Result<()>
      fn execute_ns_remove(type: &str, domain: &str, value: &str) -> Result<()>
    }
    
    site.rs {
      fn execute_site_status(profile: &ProfileContext) -> Result<()>
      fn execute_site_add(domain: &str, pubkey: &str, profile: &ProfileContext) -> Result<()>
      fn execute_site_del(domain: &str, profile: &ProfileContext) -> Result<()>
    }
    
    ssh.rs {
      fn execute_ssh_status(profile: &ProfileContext) -> Result<()>
      fn execute_ssh_add(host: &str, auth: SshAuth, profile: &ProfileContext) -> Result<()>
      fn execute_ssh_del(host: &str, profile: &ProfileContext) -> Result<()>
      fn execute_ssh_init(host: &str, profile: &ProfileContext) -> Result<()>
    }
    
    platform_vm.rs {
      fn execute_platform_add(name: &str, platform_type: &str, profile: &ProfileContext) -> Result<()>
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
  constraint: Diesel ORM with SQLite (PostgreSQL support)
  
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
  constraint: Feature-gated (#[cfg(feature = "gui")])
  
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
  constraint: Feature-gated (#[cfg(feature = "gui")])
  
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
  constraint: Feature-gated (#[cfg(feature = "gui")])
  
  mod.rs {
    // Tab exports
  }
  
  platform.rs {
    // Platform tab (GCP VMs, billing)
    struct PlatformTab { vms: Vec<Vm>, selected: Option<String> }
    fn show(ui: &mut Ui, viewmodel: &ViewModel, profile: &ProfileContext)
    
    constraint: Sends PlatformCommand to actor, receives PlatformEvent
  }
  
  platform_stt.rs {
    // STT variant
  }
  
  ssh.rs {
    // SSH tab (host management)
    struct SshTab { hosts: Vec<SshHost>, selected: Option<String> }
    fn show(ui: &mut Ui, viewmodel: &ViewModel, profile: &ProfileContext)
    
    constraint: Sends SshCommand to actor, receives SshEvent
  }
  
  ssh_stt.rs {
    // STT variant
  }
  
  ns.rs {
    // DNS nameserver tab
    struct NsTab { domains: Vec<Domain>, records: HashMap<String, Vec<DnsRecord>> }
    fn show(ui: &mut Ui, viewmodel: &ViewModel, profile: &ProfileContext)
    
    constraint: Sends NsCommand to actor, receives NsEvent
  }
  
  ns_stt.rs {
    // STT variant
  }
  
  site.rs {
    // Site management tab
    struct SiteTab { sites: Vec<Site>, selected: Option<String> }
    fn show(ui: &mut Ui, viewmodel: &ViewModel, profile: &ProfileContext)
  }
  
  products.rs {
    // Product management tab (mycart)
    struct ProductsTab { products: Vec<Product>, filter: String }
    fn show(ui: &mut Ui, viewmodel: &ViewModel, profile: &ProfileContext)
  }
  
  products_stt.rs {
    // STT variant
  }
  
  orders.rs {
    // Order management tab (mycart)
    struct OrdersTab { orders: Vec<Order>, status_filter: OrderStatus }
    fn show(ui: &mut Ui, viewmodel: &ViewModel, profile: &ProfileContext)
  }
  
  orders_stt.rs {
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

## ViewModel Module (29 files)

Actor-based MVVM layer coordinating business logic and UI.

```sudo
interface ViewModelModule {
  constraint: Actor-based with smol async runtime
  constraint: Profile-scoped operations
  
  mod.rs {
    // ViewModel coordinator
    struct ViewModel {
      platform_tx: Sender<PlatformCommand>
      drawer_tx: Sender<DrawerCommand>
      ssh_tx: Sender<SshCommand>
      ns_tx: Sender<NsCommand>
      logs_tx: Sender<LogCommand>
      event_rx: Receiver<ViewModelEvent>
      state: ViewModelState
      profile: Option<ProfileContext>
      #[cfg(feature = "gui")] egui_ctx: Option<egui::Context>
    }
    
    fn new(ctx: egui::Context, profile: Option<ProfileContext>) -> Self
    fn send_command(&self, cmd: Command)
    fn poll_events(&mut self) -> Vec<ViewModelEvent>
    
    constraint: Spawns actor threads, routes commands, aggregates events
  }
  
  common.rs {
    // Shared types
    enum ViewModelEvent { Platform, Drawer, Ssh, Ns, Logs }
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
  
  // Actor modules
  platform/ {
    mod.rs { /* platform actor exports */ }
    
    actor.rs {
      // Platform actor (GCP VM management)
      async fn platform_actor(
        cmd_rx: Receiver<PlatformCommand>,
        event_tx: Sender<ViewModelEvent>,
        profile: ProfileContext
      ) {
        loop {
          match cmd_rx.recv().await {
            PlatformCommand::ListVms => {
              let vms = calc::gcp::list_vms(&profile).await;
              event_tx.send(PlatformEvent::VmsListed(vms));
            }
            PlatformCommand::CreateVm(spec) => {
              let vm = calc::gcp::provision_vm(spec, &profile).await;
              event_tx.send(PlatformEvent::VmCreated(vm));
            }
            // ... more commands
          }
        }
      }
    }
    
    commands.rs {
      enum PlatformCommand { ListVms, CreateVm(VmSpec), DeleteVm(String) }
    }
    
    events.rs {
      enum PlatformEvent { VmsListed(Vec<Vm>), VmCreated(Vm), VmDeleted(String) }
    }
    
    drawer_actor.rs {
      // Billing drawer actor
      async fn drawer_actor(
        cmd_rx: Receiver<DrawerCommand>,
        event_tx: Sender<ViewModelEvent>,
        profile: ProfileContext
      )
    }
    
    state.rs {
      struct PlatformState { vms: Vec<Vm>, selected: Option<String> }
    }
    
    tests.rs {
      // Platform actor tests
    }
  }
  
  ssh/ {
    mod.rs { /* SSH actor exports */ }
    
    actor.rs {
      // SSH actor (host management, connections)
      async fn ssh_actor(
        cmd_rx: Receiver<SshCommand>,
        event_tx: Sender<ViewModelEvent>,
        profile: ProfileContext
      ) {
        loop {
          match cmd_rx.recv().await {
            SshCommand::ListHosts => {
              let hosts = calc::ssh::list_hosts(&profile).await;
              event_tx.send(SshEvent::HostsListed(hosts));
            }
            SshCommand::Connect(host) => {
              let session = calc::ssh::connect(&host).await;
              event_tx.send(SshEvent::Connected(session));
            }
            // ... more commands
          }
        }
      }
    }
    
    commands.rs {
      enum SshCommand { ListHosts, Connect(String), Disconnect, Execute(String) }
    }
    
    events.rs {
      enum SshEvent { HostsListed(Vec<SshHost>), Connected(SshSession), Output(String) }
    }
    
    state.rs {
      struct SshState { hosts: Vec<SshHost>, sessions: HashMap<String, SshSession> }
    }
    
    tests.rs {
      // SSH actor tests
    }
  }
  
  ns/ {
    mod.rs { /* NS actor exports */ }
    
    actor.rs {
      // DNS nameserver actor
      async fn ns_actor(
        cmd_rx: Receiver<NsCommand>,
        event_tx: Sender<ViewModelEvent>,
        profile: ProfileContext
      )
    }
    
    commands.rs {
      enum NsCommand { ListDomains, AddDomain, UpdateRecords, DeleteDomain }
    }
    
    events.rs {
      enum NsEvent { DomainsListed(Vec<Domain>), DomainAdded, RecordsUpdated }
    }
    
    state.rs {
      struct NsState { domains: Vec<Domain>, records: HashMap<String, Vec<DnsRecord>> }
    }
    
    tests.rs {
      // NS actor tests
    }
  }
  
  logs/ {
    mod.rs { /* logs actor exports */ }
    
    actor.rs {
      // Logs aggregation actor
      async fn logs_actor(
        cmd_rx: Receiver<LogCommand>,
        event_tx: Sender<ViewModelEvent>
      )
    }
    
    commands.rs {
      enum LogCommand { Subscribe, GetRecent, Clear }
    }
    
    events.rs {
      enum LogEvent { NewLog(LogRecord), HistoryLoaded(Vec<LogRecord>) }
    }
    
    state.rs {
      struct LogsState { buffer: VecDeque<LogRecord> }
    }
  }
  
  mycart/ {
    mod.rs { /* mycart actor exports */ }
    
    actor.rs {
      // Mycart integration actor
      async fn mycart_actor(
        cmd_rx: Receiver<MycartCommand>,
        event_tx: Sender<ViewModelEvent>,
        profile: ProfileContext
      )
    }
    
    commands.rs {
      enum MycartCommand { SyncProducts, CreateProduct, UpdateOrder }
    }
    
    events.rs {
      enum MycartEvent { ProductsSynced(Vec<Product>), OrderUpdated(Order) }
    }
    
    state.rs {
      struct MycartState { products: Vec<Product>, orders: Vec<Order> }
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
  
  constraint: All files profile-scoped except entry points and platform utilities
}
```

---

## Usage for Code Management

```sudo
OnCodeChange {
  constraint: Update this document when adding/removing/renaming files
  
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
  constraint: ARCH.sudolang.md = high-level architecture
  constraint: SUDOLANG.md (this) = code-level structure
  
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
