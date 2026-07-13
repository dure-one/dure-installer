use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::error::{WsError, Result};

/// Load certificates from PEM file
pub fn load_certs(path: &Path) -> Result<Vec<CertificateDer<'static>>> {
    let file = File::open(path)
        .map_err(|e| WsError::Certificate(format!("Cannot open cert file: {}", e)))?;

    let mut reader = BufReader::new(file);

    certs(&mut reader)
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| WsError::Certificate(format!("Cannot parse certs: {}", e)))
}

/// Load private key from PEM file
pub fn load_private_key(path: &Path) -> Result<PrivateKeyDer<'static>> {
    let file = File::open(path)
        .map_err(|e| WsError::Certificate(format!("Cannot open key file: {}", e)))?;

    let mut reader = BufReader::new(file);

    pkcs8_private_keys(&mut reader)
        .next()
        .ok_or_else(|| WsError::Certificate("No private key found".into()))?
        .map(PrivateKeyDer::Pkcs8)
        .map_err(|e| WsError::Certificate(format!("Cannot parse key: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    const TEST_CERT: &str = "-----BEGIN CERTIFICATE-----
MIIBkTCB+wIJAKHHCgVZU7POMA0GCSqGSIb3DQEBBQUAMA0xCzAJBgNVBAYTAlVT
MB4XDTEwMDUyODIyNDYyNVoXDTIwMDUyNTIyNDYyNVowDTELMAkGA1UEBhMCVVMw
gZ8wDQYJKoZIhvcNAQEBBQADgY0AMIGJAoGBANDICgiZb+BiVFyxH3DQq7pxhHcW
6B7Zy2MQ8Vvx...truncated...
-----END CERTIFICATE-----";

    const TEST_KEY: &str = "-----BEGIN PRIVATE KEY-----
MIIBVQIBADANBgkqhkiG9w0BAQEFAASCAT8wggE7AgEAAkEAwmKgPsJuqH/Qf+kM
...truncated...
-----END PRIVATE KEY-----";

    #[test]
    fn test_load_certs_invalid_path() {
        let result = load_certs(Path::new("/nonexistent/cert.pem"));
        assert!(matches!(result, Err(WsError::Certificate(_))));
    }

    #[test]
    fn test_load_private_key_invalid_path() {
        let result = load_private_key(Path::new("/nonexistent/key.pem"));
        assert!(matches!(result, Err(WsError::Certificate(_))));
    }

    // TODO: Add tests with actual cert/key files once tempfile is added to dev-dependencies
}
