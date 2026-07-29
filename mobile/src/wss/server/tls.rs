//! TLS certificate and key loading for the HTTPS/WSS server.

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use futures_rustls::TlsAcceptor;
use std::io;
use std::path::Path;
use std::sync::Arc;

fn tls_err(e: impl std::fmt::Display) -> io::Error {
    io::Error::other(e.to_string())
}

/// Build a `TlsAcceptor` from PEM certificate + key files.
pub fn create_acceptor(cert_path: &Path, key_path: &Path) -> io::Result<TlsAcceptor> {
    // Install the ring crypto provider
    let _ = rustls::crypto::ring::default_provider().install_default();

    // Load certificate chain
    let cert_file = std::fs::File::open(cert_path)?;
    let mut cert_reader = std::io::BufReader::new(cert_file);
    let certs: Vec<_> = rustls_pemfile::certs(&mut cert_reader)
        .collect::<Result<_, _>>()
        .map_err(tls_err)?;

    // Load private key
    let key_file = std::fs::File::open(key_path)?;
    let mut key_reader = std::io::BufReader::new(key_file);
    let key = rustls_pemfile::private_key(&mut key_reader)
        .map_err(tls_err)?
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "No private key found"))?;

    // Build server config
    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(tls_err)?;

    Ok(TlsAcceptor::from(Arc::new(config)))
}

/// Generate a self-signed certificate + key for `domain` and write them to
/// `cert_path` / `key_path` as PEM files.  Existing files are overwritten.
pub fn generate_self_signed(domain: &str, cert_path: &Path, key_path: &Path) -> io::Result<()> {
    use rcgen::{CertifiedKey, generate_simple_self_signed};
    let subject_alt_names = vec![domain.to_string(), "localhost".to_string()];
    let CertifiedKey { cert, key_pair } =
        generate_simple_self_signed(subject_alt_names).map_err(tls_err)?;
    std::fs::write(cert_path, cert.pem())?;
    std::fs::write(key_path, key_pair.serialize_pem())?;
    Ok(())
}
