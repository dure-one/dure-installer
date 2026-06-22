//! Combined HTTPS/WSS server with ACME TLS certificates
//!
//! Features:
//! - Serves static files from downloaded dure-wasm repository
//! - WebSocket Secure (WSS) server on same port
//! - Webhook POST request logging with pattern matching
//! - Session tracking in database
//! - TLS certificates from ACME database
//! - Swagger UI at /swagger-ui and OpenAPI JSON at /api-docs/openapi.json
//! - AsyncAPI docs at /asyncapi-docs/
//!
//! Run with:
//! ```sh
//! cargo run --bin wss-server -- --domain example.com
//! ```

pub mod api;
pub mod asyncapi_docs;
pub mod db;
pub mod handlers;
pub mod http_get;
pub mod http_post;
pub mod https;
pub mod tls;
#[cfg(not(target_arch = "wasm32"))]
pub mod webauthn;
pub mod ws;

use async_net::{TcpListener, TcpStream};
use async_tls::TlsAcceptor;
use std::io;
use std::net::ToSocketAddrs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use utoipa::{
    Modify, OpenApi,
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
};
use utoipa_swagger_ui::Config as SwaggerConfig;

use db::DbConn;
use https::{handle_https_request, read_http_request};
use tls::create_acceptor;
use ws::{handle_websocket, perform_websocket_handshake};

/// Helper function to generate self-signed certificate
fn generate_self_signed_cert(domain: &str) -> (String, String) {
    let proj_dirs = directories::ProjectDirs::from("pe", "nikescar", "dure")
        .expect("Failed to get project dirs");
    let cert_dir = proj_dirs.config_dir().join("certs");
    std::fs::create_dir_all(&cert_dir).expect("Failed to create cert directory");

    let ss_cert = cert_dir.join("self-signed.crt");
    let ss_key = cert_dir.join("self-signed.key");

    tls::generate_self_signed(domain, &ss_cert, &ss_key)
        .expect("Failed to generate self-signed certificate");

    eprintln!("✓ Self-signed certificate generated");

    (
        ss_cert.to_string_lossy().to_string(),
        ss_key.to_string_lossy().to_string(),
    )
}

/// Check if DNS provider credentials are configured
fn has_dns_credentials(domain_config: &crate::config::DomainConfig) -> bool {
    match domain_config.dns_provider.to_lowercase().as_str() {
        "cloudflare" => {
            domain_config.cloudflare.api_token.is_some()
                || (domain_config.cloudflare.email.is_some()
                    && domain_config.cloudflare.api_key.is_some())
        }
        "duckdns" => domain_config.duckdns.token.is_some(),
        "gcloud" => domain_config.gcloud.project.is_some(),
        "porkbun" => {
            domain_config.porkbun.api_key.is_some()
                && domain_config.porkbun.secret_api_key.is_some()
        }
        _ => false,
    }
}

/// Helper function to issue certificate with lego
fn issue_cert_with_lego(
    domain: &str,
    app_config: &crate::config::AppConfig,
) -> Result<(String, String, String), Box<dyn std::error::Error>> {
    use crate::calc::lego;

    let proj_dirs = directories::ProjectDirs::from("pe", "nikescar", "dure")
        .ok_or("Failed to get project dirs")?;
    let config_dir = proj_dirs.config_dir().to_path_buf();

    // Get DNS provider
    let dns_provider = match app_config.domain.dns_provider.to_lowercase().as_str() {
        "cloudflare" => lego::DnsProvider::Cloudflare,
        "duckdns" => lego::DnsProvider::DuckDns,
        "gcloud" => lego::DnsProvider::GoogleCloud,
        "porkbun" => lego::DnsProvider::Porkbun,
        _ => return Err("Unsupported DNS provider".into()),
    };

    // Build environment variables
    let mut env_vars = Vec::new();
    match dns_provider {
        lego::DnsProvider::Cloudflare => {
            if let Some(token) = &app_config.domain.cloudflare.api_token {
                env_vars.push(("CLOUDFLARE_DNS_API_TOKEN", token.as_str()));
            } else if let (Some(email), Some(key)) = (
                &app_config.domain.cloudflare.email,
                &app_config.domain.cloudflare.api_key,
            ) {
                env_vars.push(("CLOUDFLARE_EMAIL", email.as_str()));
                env_vars.push(("CLOUDFLARE_API_KEY", key.as_str()));
            }
        }
        lego::DnsProvider::DuckDns => {
            if let Some(token) = &app_config.domain.duckdns.token {
                env_vars.push(("DUCKDNS_TOKEN", token.as_str()));
            }
        }
        lego::DnsProvider::GoogleCloud => {
            if let Some(project) = &app_config.domain.gcloud.project {
                env_vars.push(("GCE_PROJECT", project.as_str()));
            }
            if let Some(sa_file) = &app_config.domain.gcloud.service_account_file {
                env_vars.push(("GCE_SERVICE_ACCOUNT_FILE", sa_file.as_str()));
            }
        }
        lego::DnsProvider::Porkbun => {
            if let (Some(api_key), Some(secret)) = (
                &app_config.domain.porkbun.api_key,
                &app_config.domain.porkbun.secret_api_key,
            ) {
                env_vars.push(("PORKBUN_API_KEY", api_key.as_str()));
                env_vars.push(("PORKBUN_SECRET_API_KEY", secret.as_str()));
            }
        }
    }

    let email = app_config
        .domain
        .cert
        .email
        .as_deref()
        .map(|e| e.to_string())
        .unwrap_or_else(|| format!("admin@{}", domain));

    // Issue certificate
    let _cert = lego::issue_certificate(&config_dir, &email, domain, dns_provider, &env_vars)?;

    // Copy to config directory
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

    std::fs::copy(&src_cert, &dest_cert)?;
    std::fs::copy(&src_key, &dest_key)?;
    std::fs::copy(&src_issuer, &dest_issuer)?;

    Ok((
        dest_cert.to_string_lossy().to_string(),
        dest_key.to_string_lossy().to_string(),
        dest_issuer.to_string_lossy().to_string(),
    ))
}

/// Arguments for running the WSS server
pub struct RunArgs {
    pub domain: String,
    /// Bind address in `host:port` format, e.g. `"0.0.0.0:443"`
    pub addr: String,
    pub db_path: String,
    pub stats_interval: u64,
    pub download_static: bool,
}

// Conditionally include WebAuthn API based on target architecture
#[cfg(not(target_arch = "wasm32"))]
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Dure Server API",
        description = "REST API for the Dure HTTPS/WSS server.\n\n\
            **AsyncAPI (WebSocket) docs:** [/asyncapi-docs/](/asyncapi-docs/)",
    ),
    nest(
        (path = "/api/todo",    api = api::todo::TodoApi),
        (path = "/api/webhook", api = api::webhook::WebhookApi),
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "todo",    description = "Todo items management endpoints."),
        (name = "webhook", description = "Webhook management endpoints."),
    )
)]
struct ApiDoc;

#[cfg(target_arch = "wasm32")]
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Dure Server API",
        description = "REST API for the Dure HTTPS/WSS server.\n\n\
            **AsyncAPI (WebSocket) docs:** [/asyncapi-docs/](/asyncapi-docs/)",
    ),
    nest(
        (path = "/api/todo",    api = api::todo::TodoApi),
        (path = "/api/webhook", api = api::webhook::WebhookApi),
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "todo",    description = "Todo items management endpoints."),
        (name = "webhook", description = "Webhook management endpoints."),
    )
)]
struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "api_key",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("todo_apikey"))),
        );
    }
}

/// Server settings (cheap to clone — all inner data is Arc-wrapped)
#[derive(Clone)]
pub struct ServerSettings {
    pub domain: String,
    pub server_id: String,
    pub ping_interval: u64,
    pub idle_timeout: u64,
    pub max_connections: usize,
    pub static_dir: PathBuf,
    pub asyncapi_docs_dir: PathBuf,
    pub db: DbConn,
    pub todo_store: api::todo::Store,
    pub swagger_config: Arc<SwaggerConfig<'static>>,
    pub openapi_json: Arc<String>,
    pub debug_http: bool,
    #[cfg(not(target_arch = "wasm32"))]
    pub webauthn: Option<webauthn::WebAuthnState>,
}

impl ServerSettings {
    pub fn new(domain: String, db: DbConn, debug_http: bool) -> Self {
        let openapi_json = Arc::new(
            ApiDoc::openapi()
                .to_json()
                .expect("Failed to serialize OpenAPI spec"),
        );
        let swagger_config = Arc::new(SwaggerConfig::from("/api-docs/openapi.json"));
        Self {
            domain,
            server_id: format!("dure-server-{}", std::process::id()),
            ping_interval: 30,
            idle_timeout: 300,
            max_connections: 10000,
            static_dir: PathBuf::from("serv"),
            asyncapi_docs_dir: PathBuf::from("asyncapi-gen/docs/api-docs"),
            db,
            todo_store: api::todo::Store::default(),
            swagger_config,
            openapi_json,
            debug_http,
            #[cfg(not(target_arch = "wasm32"))]
            webauthn: None,
        }
    }
}

/// Connection statistics
#[derive(Clone)]
pub struct Stats {
    pub active_connections: Arc<AtomicUsize>,
    pub total_connections: Arc<AtomicU64>,
    pub total_http_requests: Arc<AtomicU64>,
    pub total_wss_messages: Arc<AtomicU64>,
    pub total_webhook_posts: Arc<AtomicU64>,
}

impl Default for Stats {
    fn default() -> Self {
        Self::new()
    }
}

impl Stats {
    pub fn new() -> Self {
        Self {
            active_connections: Arc::new(AtomicUsize::new(0)),
            total_connections: Arc::new(AtomicU64::new(0)),
            total_http_requests: Arc::new(AtomicU64::new(0)),
            total_wss_messages: Arc::new(AtomicU64::new(0)),
            total_webhook_posts: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn connection_started(&self) {
        self.active_connections.fetch_add(1, Ordering::Relaxed);
        self.total_connections.fetch_add(1, Ordering::Relaxed);
    }

    pub fn connection_ended(&self) {
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn http_request(&self) {
        self.total_http_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn wss_message(&self) {
        self.total_wss_messages.fetch_add(1, Ordering::Relaxed);
    }

    pub fn webhook_post(&self) {
        self.total_webhook_posts.fetch_add(1, Ordering::Relaxed);
    }

    pub fn print_stats(&self) {
        eprintln!(
            "Stats: active={}, total={}, http={}, wss={}, webhooks={}",
            self.active_connections.load(Ordering::Relaxed),
            self.total_connections.load(Ordering::Relaxed),
            self.total_http_requests.load(Ordering::Relaxed),
            self.total_wss_messages.load(Ordering::Relaxed),
            self.total_webhook_posts.load(Ordering::Relaxed),
        );
    }
}

pub fn generate_session_id() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let random = std::process::id();
    format!("session-{}-{}", timestamp, random)
}

async fn stats_reporter(stats: Stats, interval_secs: u64) {
    loop {
        smol::Timer::after(Duration::from_secs(interval_secs)).await;
        stats.print_stats();
    }
}

async fn handle_connection(
    stream: TcpStream,
    acceptor: Option<TlsAcceptor>,
    settings: ServerSettings,
    stats: Stats,
) -> io::Result<()> {
    use async_tungstenite::WebSocketStream;
    use async_tungstenite::tungstenite::protocol::Role;

    let peer_addr = stream.peer_addr()?;
    let debug = std::env::var("DURE_DEBUG_HTTP").is_ok() || settings.debug_http;

    if debug {
        eprintln!("DEBUG: New connection from {}", peer_addr);
    }
    stats.connection_started();

    let result = async {
        if let Some(acceptor) = acceptor {
            // TLS mode: HTTPS and WSS
            if debug {
                eprintln!("DEBUG: Accepting TLS connection from {}", peer_addr);
            }
            let mut tls_stream = acceptor
                .accept(stream)
                .await
                .map_err(|e| io::Error::other(format!("TLS error: {}", e)))?;
            if debug {
                eprintln!("DEBUG: TLS handshake completed for {}", peer_addr);
            }
            let request = read_http_request(&mut tls_stream).await?;

            if request.is_websocket_upgrade() {
                let session_id =
                    perform_websocket_handshake(&mut tls_stream, &request, &settings.server_id)
                        .await?;
                let ws_stream = WebSocketStream::from_raw_socket(
                    tls_stream,
                    Role::Server,
                    None,
                )
                .await;
                handle_websocket(
                    ws_stream,
                    peer_addr,
                    session_id,
                    settings,
                    stats.clone(),
                )
                .await
            } else {
                handle_https_request( tls_stream, request, settings, stats.clone(), peer_addr)
                    .await
            }
        } else {
            // Plain mode: HTTP and WS (no TLS)
            if debug {
                eprintln!("DEBUG: Plain connection (no TLS) from {}", peer_addr);
            }
            let mut plain_stream = stream;
            let request = read_http_request(&mut plain_stream).await?;

            if request.is_websocket_upgrade() {
                let session_id =
                    perform_websocket_handshake(&mut plain_stream, &request, &settings.server_id)
                        .await?;
                let ws_stream = WebSocketStream::from_raw_socket(
                    AsupersyncAdapter::new(plain_stream),
                    Role::Server,
                    None,
                )
                .await;
                handle_websocket(
                    &cx,
                    ws_stream,
                    peer_addr,
                    session_id,
                    settings,
                    stats.clone(),
                )
                .await
            } else {
                handle_https_request(
                    &cx,
                    plain_stream,
                    request,
                    settings,
                    stats.clone(),
                    peer_addr,
                )
                .await
            }
        }
    }
    .await;

    stats.connection_ended();
    result
}

/// Run the WSS server with explicit arguments.
///
/// This is the primary entry point — both `main()` and the CLI `wss server` command
/// funnel through here.
pub fn run_with_args(args: RunArgs) -> io::Result<()> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let _ = env_logger::builder().format_timestamp(None).try_init();

    let RunArgs {
        domain,
        addr,
        db_path,
        stats_interval,
        download_static,
    } = args;

    // Write PID file so `wss status` can detect this in-process server.
    let pid = std::process::id();
    let started_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let pid_path =
        crate::calc::wss::pid_file_path(&domain).map_err(|e| io::Error::other(e.to_string()))?;
    std::fs::write(&pid_path, format!("{}\n{}\n{}", pid, started_at, db_path))
        .map_err(|e| io::Error::other(format!("PID file: {}", e)))?;

    let result = smol::block_on(run_server_async(
        domain.clone(),
        addr,
        db_path,
        stats_interval,
        download_static,
    ));

    // Remove PID file on clean exit or error.
    let _ = std::fs::remove_file(&pid_path);

    result
}

async fn run_server_async(
    domain: String,
    addr: String,
    db_path: String,
    stats_interval: u64,
    download_static: bool,
) -> io::Result<()> {

    let db = db::open_db(&db_path)?;
    eprintln!("✓ Database opened: {}", db_path);

    // Placeholder for now - will be updated after loading config
    let server_settings = ServerSettings::new(domain.clone(), db, false);

    if download_static && !http_get::static_files_exist(&server_settings.static_dir).await {
        if let Err(e) = http_get::download_static_files(&server_settings.static_dir).await {
            eprintln!("Warning: Failed to download static files: {}", e);
        }
    } else if http_get::static_files_exist(&server_settings.static_dir).await {
        eprintln!("✓ Static files present at {:?}", server_settings.static_dir);
    }

    // Try to load certificate from config.yml first
    let config_path = {
        let proj_dirs = directories::ProjectDirs::from("pe", "nikescar", "dure")
            .ok_or_else(|| io::Error::other("Failed to get project dirs"))?;
        proj_dirs.config_dir().join("config.yml")
    };

    let mut app_config = crate::config::AppConfig::load_or_default(&config_path);

    // Update server settings with config values
    let mut server_settings = server_settings;
    server_settings.debug_http = app_config.server.debug_http;

    let (cert_path, key_path) = 
        // First try: certificates from config.yml
        if let (Some(cert), Some(key)) = (
            app_config.domain.cert.cert_path.as_ref(),
            app_config.domain.cert.key_path.as_ref(),
        ) {
            if std::path::Path::new(cert).exists() && std::path::Path::new(key).exists() {
                eprintln!("✓ Using certificates from config.yml");
                Some((cert.clone(), key.clone()))
            } else {
                eprintln!("⚠ Certificate paths in config.yml are invalid");
                None
            }
        } else {
            None
        }
        // Second try: check for lego certificates in config directory
        .or_else(|| {
            directories::ProjectDirs::from("pe", "nikescar", "dure")
                .and_then(|proj_dirs| {
                    let cert_dir = proj_dirs.config_dir().join("certs");
                    let cert_path = cert_dir.join(format!("{}.crt", domain));
                    let key_path = cert_dir.join(format!("{}.key", domain));
                    
                    if cert_path.exists() && key_path.exists() {
                        eprintln!("✓ Found existing lego certificates");
                        Some((
                            cert_path.to_string_lossy().to_string(),
                            key_path.to_string_lossy().to_string(),
                        ))
                    } else {
                        None
                    }
                })
        })
        // Last resort: generate certificate
        .unwrap_or_else(|| {
            eprintln!("⚠ No valid SSL certificate found");
            
            // Check if we have DNS provider configuration
            if !app_config.domain.dns_provider.is_empty() 
                && has_dns_credentials(&app_config.domain) 
            {
                eprintln!("  Attempting to issue certificate with lego...");
                
                // Try to issue certificate automatically
                match issue_cert_with_lego(&domain, &app_config) {
                    Ok((cert, key, issuer)) => {
                        eprintln!("✓ Certificate issued successfully with lego");
                        
                        // Update config.yml
                        app_config.domain.cert.cert_path = Some(cert.clone());
                        app_config.domain.cert.key_path = Some(key.clone());
                        app_config.domain.cert.issuer_path = Some(issuer);
                        let _ = app_config.save(&config_path);
                        
                        (cert, key)
                    }
                    Err(e) => {
                        eprintln!("  Failed to auto-issue certificate: {}", e);
                        eprintln!("  Falling back to self-signed certificate");
                        generate_self_signed_cert(&domain)
                    }
                }
            } else {
                eprintln!("  No DNS provider configured in config.yml");
                eprintln!("  Configure domain.dns_provider and DNS provider credentials");
                eprintln!("  or run: dure acme issue");
                eprintln!("  Falling back to self-signed certificate");
                generate_self_signed_cert(&domain)
            }
        });

    // Load TLS configuration if enabled
    let use_tls = app_config.server.use_tls;
    let acceptor = if use_tls {
        eprintln!("Loading TLS configuration...");
        eprintln!("  Certificate: {}", cert_path);
        eprintln!("  Private key: {}", key_path);

        Some(create_acceptor(
            std::path::Path::new(&cert_path),
            std::path::Path::new(&key_path),
        )?)
    } else {
        eprintln!("⚠ TLS disabled - running in plain HTTP/WS mode (insecure!)");
        None
    };

    // Initialize WebAuthn if TLS is enabled
    #[cfg(not(target_arch = "wasm32"))]
    {
        if use_tls {
            let rp_origin = format!("https://{}:443", domain);
            match webauthn::WebAuthnState::new(&domain, &rp_origin, Some("Dure")) {
                Ok(webauthn_state) => {
                    eprintln!("✓ WebAuthn initialized for domain: {}", domain);
                    server_settings.webauthn = Some(webauthn_state);
                }
                Err(e) => {
                    eprintln!("⚠ Failed to initialize WebAuthn: {:?}", e);
                    eprintln!("  WebAuthn authentication will not be available");
                }
            }
        } else {
            eprintln!("⚠ WebAuthn disabled (requires TLS)");
        }
    }

    let stats = Stats::new();

    // Spawn stats reporter as a background task.
    let stats_clone = stats.clone();
    smol::spawn(async move {
        stats_reporter(stats_clone, stats_interval).await;
    })
    .detach();

    let socket_addr = addr
        .to_socket_addrs()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::AddrNotAvailable, "No address"))?;

    let listener = TcpListener::bind(socket_addr).await?;

    if use_tls {
        eprintln!("🚀 HTTPS/WSS server (TLS enabled)");
        eprintln!("   Domain:        {}", domain);
        eprintln!("   Address:       https://{}", socket_addr);
        eprintln!("   Static:        {:?}", server_settings.static_dir);
        eprintln!("   Database:      {}", db_path);
        eprintln!("   Swagger UI:    https://{}/swagger-ui", domain);
        eprintln!("   AsyncAPI docs: https://{}/asyncapi-docs/", domain);
    } else {
        eprintln!("🚀 HTTP/WS server (TLS disabled - development mode)");
        eprintln!("   Domain:        {}", domain);
        eprintln!("   Address:       http://{}", socket_addr);
        eprintln!("   Static:        {:?}", server_settings.static_dir);
        eprintln!("   Database:      {}", db_path);
        eprintln!("   Swagger UI:    http://{}/swagger-ui", domain);
        eprintln!("   AsyncAPI docs: http://{}/asyncapi-docs/", domain);
    }
    eprintln!("\nPress Ctrl+C to stop");

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let current = stats.active_connections.load(Ordering::Relaxed);
                if current >= server_settings.max_connections {
                    drop(stream);
                    continue;
                }

                let acceptor = acceptor.clone();
                let settings = server_settings.clone();
                let stats = stats.clone();

                smol::spawn(async move {
                    if let Err(e) = handle_connection(stream, acceptor, settings, stats).await {
                        eprintln!("Connection error: {}", e);
                    }
                })
                .detach();
            }
            Err(e) => eprintln!("Accept error: {}", e),
        }
    }
}

/// Entry point when used as a standalone binary.
///
/// Parses arguments from `std::env::args()` and calls [`run_with_args`].
pub fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let mut domain: Option<String> = None;
    let mut addr = "0.0.0.0:8443".to_string();
    let mut stats_interval = 60u64;
    let mut download_static = true;
    let db_path = crate::calc::db::get_db_path();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--domain" | "-d" => {
                i += 1;
                if i < args.len() {
                    domain = Some(args[i].clone());
                }
            }
            "--addr" | "-a" => {
                i += 1;
                if i < args.len() {
                    addr = args[i].clone();
                }
            }
            "--no-download" => download_static = false,
            "--stats-interval" => {
                i += 1;
                if i < args.len() {
                    stats_interval = args[i].parse().unwrap_or(60);
                }
            }
            "--help" | "-h" => {
                println!("Usage: {} [OPTIONS]", args[0]);
                println!("\nOptions:");
                println!("  --domain, -d <DOMAIN>      Domain name (required)");
                println!("  --addr, -a <ADDR>          Server address (default: 0.0.0.0:443)");
                println!("  --no-download              Skip downloading static files");
                println!("  --stats-interval <SECS>    Stats interval (default: 60)");
                println!("  --help, -h                 Show this help");
                return Ok(());
            }
            _ => {}
        }
        i += 1;
    }

    let domain = domain.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Domain required (--domain <domain>)",
        )
    })?;

    run_with_args(RunArgs {
        domain,
        addr,
        db_path,
        stats_interval,
        download_static,
    })
}
