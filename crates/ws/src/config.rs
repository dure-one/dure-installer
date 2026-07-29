use std::net::SocketAddr;
use std::path::PathBuf;

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Bind address (e.g., "0.0.0.0:8443")
    pub bind_addr: SocketAddr,

    /// Server domain name
    pub domain: String,

    /// TLS certificate path
    pub tls_cert: PathBuf,

    /// TLS private key path
    pub tls_key: PathBuf,

    /// Static files directory
    pub static_dir: PathBuf,

    /// Maximum concurrent connections
    pub max_connections: usize,

    /// Enable fail2ban-rs DDoS protection
    pub enable_ddos_protection: bool,

    /// Database path for sessions/webhooks
    pub db_path: PathBuf,
}

impl ServerConfig {
    pub fn new(domain: impl Into<String>) -> Self {
        Self {
            bind_addr: "0.0.0.0:8443".parse().unwrap(),
            domain: domain.into(),
            tls_cert: PathBuf::from("./cert.pem"),
            tls_key: PathBuf::from("./key.pem"),
            static_dir: PathBuf::from("./serv"),
            max_connections: 10_000,
            enable_ddos_protection: true,
            db_path: PathBuf::from("./ws.db"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = ServerConfig::new("example.com");
        assert_eq!(config.domain, "example.com");
        assert_eq!(config.max_connections, 10_000);
        assert!(config.enable_ddos_protection);
    }

    #[test]
    fn test_config_customization() {
        let mut config = ServerConfig::new("test.com");
        config.max_connections = 5_000;
        config.bind_addr = "127.0.0.1:9443".parse().unwrap();

        assert_eq!(config.max_connections, 5_000);
        assert_eq!(config.bind_addr.port(), 9443);
    }
}
