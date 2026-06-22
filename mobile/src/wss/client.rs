//! Combined HTTPS/WSS client for testing
//!
//! Features:
//! - HTTPS GET requests (static files)
//! - HTTPS POST requests (webhooks)
//! - WebSocket Secure (WSS) connections
//! - Session ID persistence across reconnections
//!
//! Run with:
//! ```sh
//! # WebSocket mode
//! cargo run --bin wss-client -- --url wss://example.com --mode ws
//!
//! # HTTP GET mode
//! cargo run --bin wss-client -- --url https://example.com --mode get --path /index.html
//!
//! # HTTP POST mode (webhook)
//! cargo run --bin wss-client -- --url https://example.com --mode post --path /webhook/test
//! ```

use async_net::TcpStream;
use async_tls::TlsConnector;
use async_tungstenite::{client_async, tungstenite::Message};
use futures::io::{AsyncReadExt, AsyncWriteExt};
use futures::{FutureExt, SinkExt, StreamExt};
use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;
use url::Url;

/// Build a TLS connector that skips certificate verification (for self-signed certs).
fn create_tls_connector_insecure() -> TlsConnector {
    use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
    use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
    use rustls::{ClientConfig, DigitallySignedStruct, Error, SignatureScheme};

    #[derive(Debug)]
    struct AcceptAnyCert;

    impl ServerCertVerifier for AcceptAnyCert {
        fn verify_server_cert(
            &self,
            _end_entity: &CertificateDer,
            _intermediates: &[CertificateDer],
            _server_name: &ServerName,
            _ocsp: &[u8],
            _now: UnixTime,
        ) -> Result<ServerCertVerified, Error> {
            Ok(ServerCertVerified::assertion())
        }
        fn verify_tls12_signature(
            &self,
            _msg: &[u8],
            _cert: &CertificateDer,
            _sig: &DigitallySignedStruct,
        ) -> Result<HandshakeSignatureValid, Error> {
            Ok(HandshakeSignatureValid::assertion())
        }
        fn verify_tls13_signature(
            &self,
            _msg: &[u8],
            _cert: &CertificateDer,
            _sig: &DigitallySignedStruct,
        ) -> Result<HandshakeSignatureValid, Error> {
            Ok(HandshakeSignatureValid::assertion())
        }
        fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
            vec![
                SignatureScheme::RSA_PKCS1_SHA256,
                SignatureScheme::RSA_PKCS1_SHA384,
                SignatureScheme::RSA_PKCS1_SHA512,
                SignatureScheme::ECDSA_NISTP256_SHA256,
                SignatureScheme::ECDSA_NISTP384_SHA384,
                SignatureScheme::ECDSA_NISTP521_SHA512,
                SignatureScheme::RSA_PSS_SHA256,
                SignatureScheme::RSA_PSS_SHA384,
                SignatureScheme::RSA_PSS_SHA512,
                SignatureScheme::ED25519,
            ]
        }
    }

    // Install the ring crypto provider
    let _ = rustls::crypto::ring::default_provider().install_default();

    let config = ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(AcceptAnyCert))
        .with_no_client_auth();

    TlsConnector::from(Arc::new(config))
}

type WsStream = async_tungstenite::WebSocketStream<async_tls::client::TlsStream<TcpStream>>;

/// Client mode
#[derive(Debug, Clone, PartialEq)]
enum ClientMode {
    WebSocket,
    HttpGet,
    HttpPost,
}

/// Client settings
#[derive(Clone)]
struct ClientSettings {
    mode: ClientMode,
    url: String,
    path: String,
    body: String,
}

impl ClientSettings {
    fn new(mode: ClientMode, url: String) -> Self {
        Self {
            mode,
            url,
            path: "/".to_string(),
            body: r#"{"test":"data"}"#.to_string(),
        }
    }
}

/// Connection statistics
struct Stats {
    total_connections: AtomicU64,
    total_http_requests: AtomicU64,
    total_wss_messages_sent: AtomicU64,
    total_wss_messages_received: AtomicU64,
}

impl Stats {
    fn new() -> Self {
        Self {
            total_connections: AtomicU64::new(0),
            total_http_requests: AtomicU64::new(0),
            total_wss_messages_sent: AtomicU64::new(0),
            total_wss_messages_received: AtomicU64::new(0),
        }
    }

    fn print_stats(&self) {
        eprintln!(
            "Stats: connections={}, http={}, ws_sent={}, ws_received={}",
            self.total_connections.load(Ordering::Relaxed),
            self.total_http_requests.load(Ordering::Relaxed),
            self.total_wss_messages_sent.load(Ordering::Relaxed),
            self.total_wss_messages_received.load(Ordering::Relaxed),
        );
    }
}

fn create_tls_connector() -> io::Result<TlsConnector> {
    // Install the ring crypto provider
    let _ = rustls::crypto::ring::default_provider().install_default();

    let mut root_store = rustls::RootCertStore::empty();
    for cert in rustls_native_certs::load_native_certs()
        .map_err(|e| io::Error::other(format!("Load native certs: {}", e)))? {
        root_store.add(cert)
            .map_err(|e| io::Error::other(format!("Add cert: {}", e)))?;
    }

    let config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    Ok(TlsConnector::from(Arc::new(config)))
}

async fn https_get_request(
    url: &str,
    path: &str,
    insecure: bool,
    stats: &Stats,
) -> io::Result<String> {
    let url = Url::parse(url).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let host = url
        .host_str()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "no host in URL"))?;
    let port = url.port().unwrap_or(443);
    let addr = format!("{}:{}", host, port);

    eprintln!("Connecting to {}...", addr);
    let tcp_stream = TcpStream::connect(addr).await?;
    let connector = if insecure {
        create_tls_connector_insecure()
    } else {
        create_tls_connector()?
    };
    let mut tls_stream = connector
        .connect(host, tcp_stream)
        .await?;
    eprintln!("TLS handshake completed");

    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: dure-test-client/1.0\r\nConnection: close\r\n\r\n",
        path, host
    );
    tls_stream.write_all(request.as_bytes()).await?;
    tls_stream.flush().await?;

    stats.total_connections.fetch_add(1, Ordering::Relaxed);
    stats.total_http_requests.fetch_add(1, Ordering::Relaxed);

    eprintln!("Request sent, reading response...");
    let mut response = Vec::new();
    if let Err(e) = tls_stream.read_to_end(&mut response).await {
        // Servers that close TCP without TLS close_notify trigger this error, but the
        // response body is already buffered — treat it as EOF.
        if response.is_empty() {
            return Err(e);
        }
    }
    Ok(String::from_utf8_lossy(&response).to_string())
}

async fn https_post_request(
    url: &str,
    path: &str,
    body: &str,
    insecure: bool,
    stats: &Stats,
) -> io::Result<String> {
    let url = Url::parse(url).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let host = url
        .host_str()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "no host in URL"))?;
    let port = url.port().unwrap_or(443);
    let addr = format!("{}:{}", host, port);

    eprintln!("Connecting to {}...", addr);
    let tcp_stream = TcpStream::connect(addr).await?;
    let connector = if insecure {
        create_tls_connector_insecure()
    } else {
        create_tls_connector()?
    };
    let mut tls_stream = connector
        .connect(host, tcp_stream)
        .await?;
    eprintln!("TLS handshake completed");

    let request = format!(
        "POST {} HTTP/1.1\r\nHost: {}\r\nUser-Agent: dure-test-client/1.0\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        path,
        host,
        body.len(),
        body
    );
    tls_stream.write_all(request.as_bytes()).await?;
    tls_stream.flush().await?;

    stats.total_connections.fetch_add(1, Ordering::Relaxed);
    stats.total_http_requests.fetch_add(1, Ordering::Relaxed);

    eprintln!("Request sent, reading response...");
    let mut response = Vec::new();
    if let Err(e) = tls_stream.read_to_end(&mut response).await {
        if response.is_empty() {
            return Err(e);
        }
    }
    Ok(String::from_utf8_lossy(&response).to_string())
}

async fn connect_websocket(url: &str, insecure: bool) -> io::Result<WsStream> {
    let url = Url::parse(url).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let host = url
        .host_str()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "no host in URL"))?;
    let port = url.port().unwrap_or(443);
    let addr = format!("{}:{}", host, port);

    eprintln!("Connecting to {}...", addr);
    let tcp_stream = TcpStream::connect(addr).await?;
    let connector = if insecure {
        create_tls_connector_insecure()
    } else {
        create_tls_connector()?
    };
    let tls_stream = connector
        .connect(host, tcp_stream)
        .await?;
    eprintln!("TLS handshake completed");

    let request = async_tungstenite::tungstenite::http::Request::builder()
        .method("GET")
        .uri(url.as_str())
        .header("Host", host)
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Version", "13")
        .header(
            "Sec-WebSocket-Key",
            async_tungstenite::tungstenite::handshake::client::generate_key(),
        )
        .header("User-Agent", "dure-test-client/1.0")
        .body(())
        .map_err(|e| io::Error::other(e))?;

    let (ws_stream, _response) = client_async(request, tls_stream)
        .await
        .map_err(|e| io::Error::other(e))?;

    eprintln!("WebSocket connection established");
    Ok(ws_stream)
}

async fn handle_websocket_connection(
    ws_stream: WsStream,
    stats: Arc<Stats>,
    should_exit: Arc<AtomicBool>,
) -> io::Result<()> {
    println!("\n✓ WebSocket Connected!");
    println!("Type messages to send (Ctrl+D to quit):");

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // stdin reading runs in a blocking thread; messages come via std::sync::mpsc
    let (stdin_tx, stdin_rx) = std::sync::mpsc::channel::<String>();
    let exit_flag = should_exit.clone();
    std::thread::spawn(move || {
        use std::io::BufRead;
        let stdin = std::io::stdin();
        for line in stdin.lock().lines() {
            match line {
                Ok(text) => {
                    let text = text.trim().to_string();
                    if !text.is_empty() && stdin_tx.send(text).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        exit_flag.store(true, Ordering::Relaxed);
    });

    loop {
        // Check for incoming stdin messages (non-blocking).
        while let Ok(text) = stdin_rx.try_recv() {
            if let Err(e) = ws_sender.send(Message::Text(text.into())).await {
                eprintln!("Send error: {}", e);
                return Ok(());
            }
            stats
                .total_wss_messages_sent
                .fetch_add(1, Ordering::Relaxed);
        }

        if should_exit.load(Ordering::Relaxed) {
            break;
        }

        // Wait for WS message or a 100 ms tick, whichever comes first.
        let mut recv_fut = ws_receiver.next().fuse();
        let mut tick = smol::Timer::after(Duration::from_millis(100)).fuse();

        futures::select! {
            msg = recv_fut => {
                match msg {
                    Some(Ok(msg)) => {
                        if msg.is_text() {
                            println!("< {}", msg.to_text().unwrap());
                            stats.total_wss_messages_received.fetch_add(1, Ordering::Relaxed);
                        } else if msg.is_binary() {
                            println!("< [binary: {} bytes]", msg.len());
                            stats.total_wss_messages_received.fetch_add(1, Ordering::Relaxed);
                        } else if msg.is_close() {
                            eprintln!("Server closed connection");
                            return Ok(());
                        }
                    }
                    Some(Err(e)) => {
                        eprintln!("Receive error: {}", e);
                        return Ok(());
                    }
                    None => {
                        eprintln!("Stream ended");
                        return Ok(());
                    }
                }
            }
            _ = tick => {}
        }
    }

    Ok(())
}

pub fn main() -> io::Result<()> {
    let _ = env_logger::builder().format_timestamp(None).try_init();

    smol::block_on(run_client())
}

async fn run_client() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let mut url = String::from("https://localhost:443");
    let mut mode = ClientMode::WebSocket;
    let mut path = "/".to_string();
    let mut body = r#"{"test":"data"}"#.to_string();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--url" | "-u" => {
                i += 1;
                if i < args.len() {
                    url = args[i].clone();
                }
            }
            "--mode" | "-m" => {
                i += 1;
                if i < args.len() {
                    mode = match args[i].as_str() {
                        "ws" | "websocket" => ClientMode::WebSocket,
                        "get" => ClientMode::HttpGet,
                        "post" => ClientMode::HttpPost,
                        _ => ClientMode::WebSocket,
                    };
                }
            }
            "--path" | "-p" => {
                i += 1;
                if i < args.len() {
                    path = args[i].clone();
                }
            }
            "--body" | "-b" => {
                i += 1;
                if i < args.len() {
                    body = args[i].clone();
                }
            }
            "--help" | "-h" => {
                println!("Usage: {} [OPTIONS]", args[0]);
                println!("\nOptions:");
                println!("  --url, -u <URL>              Server URL (required)");
                println!("  --mode, -m <MODE>            Client mode: ws, get, post (default: ws)");
                println!("  --path, -p <PATH>            Request path (default: /)");
                println!(
                    "  --body, -b <BODY>            POST body (default: {{\"test\":\"data\"}})"
                );
                println!("  --help, -h                   Show this help");
                return Ok(());
            }
            _ => {}
        }
        i += 1;
    }

    let stats = Arc::new(Stats::new());

    match mode {
        ClientMode::HttpGet => {
            eprintln!("🚀 HTTPS GET Client  URL: {}  Path: {}", url, path);
            match https_get_request(&url, &path, false, &stats).await {
                Ok(response) => println!("Response:\n{}", response),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        ClientMode::HttpPost => {
            eprintln!(
                "🚀 HTTPS POST Client  URL: {}  Path: {}  Body: {}",
                url, path, body
            );
            match https_post_request(&url, &path, &body, false, &stats).await {
                Ok(response) => println!("Response:\n{}", response),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        ClientMode::WebSocket => {
            eprintln!("🚀 WebSocket Client  URL: {}", url);
            let should_exit = Arc::new(AtomicBool::new(false));
            match connect_websocket(&url, false).await {
                Ok(ws_stream) => {
                    stats.total_connections.fetch_add(1, Ordering::Relaxed);
                    if let Err(e) =
                        handle_websocket_connection(ws_stream, stats.clone(), should_exit)
                            .await
                    {
                        eprintln!("WebSocket error: {}", e);
                    }
                }
                Err(e) => eprintln!("Connection failed: {}", e),
            }
        }
    }

    stats.print_stats();
    Ok(())
}

/// Public entry point for `dure wss client`.
pub fn run_with_args(
    url: String,
    mode: &str,
    path: String,
    body: String,
    insecure: bool,
) -> io::Result<()> {
    let _ = env_logger::builder().format_timestamp(None).try_init();

    let client_mode = match mode {
        "get" => ClientMode::HttpGet,
        "post" => ClientMode::HttpPost,
        _ => ClientMode::WebSocket,
    };

    smol::block_on(async move {
        let stats = Arc::new(Stats::new());
        let result = match client_mode {
            ClientMode::HttpGet => {
                eprintln!("🚀 HTTPS GET  {} {}", url, path);
                match https_get_request(&url, &path, insecure, &stats).await {
                    Ok(response) => {
                        println!("{}", response);
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            }
            ClientMode::HttpPost => {
                eprintln!("🚀 HTTPS POST  {} {}  body: {}", url, path, body);
                match https_post_request(&url, &path, &body, insecure, &stats).await {
                    Ok(response) => {
                        println!("{}", response);
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            }
            ClientMode::WebSocket => {
                eprintln!("🚀 WebSocket  {}", url);
                let should_exit = Arc::new(AtomicBool::new(false));
                match connect_websocket(&url, insecure).await {
                    Ok(ws_stream) => {
                        stats.total_connections.fetch_add(1, Ordering::Relaxed);
                        handle_websocket_connection(ws_stream, stats.clone(), should_exit)
                            .await
                    }
                    Err(e) => Err(e),
                }
            }
        };
        stats.print_stats();
        result
    })
}
