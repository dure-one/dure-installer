//! Download attestations from GitHub

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use super::types::{AttestationBundle, AttestationsResponse, VerificationError};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::Path;

/// Compute SHA256 hash of a file
pub fn compute_file_hash(path: &Path) -> Result<String, VerificationError> {
    let mut file = fs::File::open(path)
        .map_err(|e| VerificationError::IoError(format!("Failed to open file: {}", e)))?;

    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file
            .read(&mut buffer)
            .map_err(|e| VerificationError::IoError(format!("Failed to read file: {}", e)))?;

        if bytes_read == 0 {
            break;
        }

        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

/// Download attestations for a binary from GitHub
///
/// # Arguments
/// * `binary_path` - Path to the binary file
/// * `owner` - GitHub repository owner
/// * `repo` - GitHub repository name
///
/// # Returns
/// * List of attestation bundles
pub fn download_attestations(
    binary_path: &str,
    owner: &str,
    repo: &str,
) -> Result<Vec<AttestationBundle>, VerificationError> {
    let path = Path::new(binary_path);

    // Compute the digest of the binary
    let digest = compute_file_hash(path)?;

    // GitHub API endpoint for attestations
    let api_url = format!(
        "https://api.github.com/repos/{}/{}/attestations/sha256:{}",
        owner, repo, digest
    );

    dure_info!("Fetching attestations from: {}", api_url);

    // Make HTTP request to GitHub API
    #[cfg(not(target_arch = "wasm32"))]
    {
        download_attestations_native(&api_url)
    }

    #[cfg(target_arch = "wasm32")]
    {
        download_attestations_wasm(&api_url)
    }
}

/// Download attestations using native HTTP client (for desktop)
#[cfg(not(target_arch = "wasm32"))]
fn download_attestations_native(
    api_url: &str,
) -> Result<Vec<AttestationBundle>, VerificationError> {
    use ureq;

    let response = ureq::get(api_url)
        .set("Accept", "application/vnd.github+json")
        .set("X-GitHub-Api-Version", "2022-11-28")
        .set("User-Agent", "dure-attestation-verifier")
        .call();

    match response {
        Ok(resp) => {
            let json_str = resp.into_string().map_err(|e| {
                VerificationError::HttpError(format!("Failed to read response: {}", e))
            })?;

            let attestations_response: AttestationsResponse = serde_json::from_str(&json_str)
                .map_err(|e| {
                    VerificationError::JsonError(format!("Failed to parse JSON: {}", e))
                })?;

            dure_info!(
                "Downloaded {} attestations",
                attestations_response.attestations.len()
            );
            Ok(attestations_response.attestations)
        }
        Err(ureq::Error::Status(code, _)) => {
            if code == 404 {
                dure_warn!("No attestations found (404)");
                Ok(Vec::new())
            } else {
                Err(VerificationError::HttpError(format!(
                    "HTTP error: {}",
                    code
                )))
            }
        }
        Err(e) => Err(VerificationError::HttpError(format!(
            "Request failed: {}",
            e
        ))),
    }
}

/// Download attestations using WASM-compatible HTTP (for web)
#[cfg(target_arch = "wasm32")]
fn download_attestations_wasm(api_url: &str) -> Result<Vec<AttestationBundle>, VerificationError> {
    // For WASM, we need to use ehttp which is async-compatible with egui
    // This is a simplified version - in practice, you'd want to handle this asynchronously
    dure_warn!("WASM attestation download not fully implemented - returning empty list");
    Ok(Vec::new())
}

/// Save attestations to a file (JSONL format)
pub fn save_attestations_to_file(
    attestations: &[AttestationBundle],
    output_path: &str,
) -> Result<(), VerificationError> {
    use std::io::Write;

    let mut file = fs::File::create(output_path)
        .map_err(|e| VerificationError::IoError(format!("Failed to create file: {}", e)))?;

    for attestation in attestations {
        let json = serde_json::to_string(attestation)
            .map_err(|e| VerificationError::JsonError(format!("Failed to serialize: {}", e)))?;
        writeln!(file, "{}", json)
            .map_err(|e| VerificationError::IoError(format!("Failed to write: {}", e)))?;
    }

    dure_info!(
        "Saved {} attestations to {}",
        attestations.len(),
        output_path
    );
    Ok(())
}

/// Load attestations from a file (JSONL format)
pub fn load_attestations_from_file(
    input_path: &str,
) -> Result<Vec<AttestationBundle>, VerificationError> {
    let content = fs::read_to_string(input_path)
        .map_err(|e| VerificationError::IoError(format!("Failed to read file: {}", e)))?;

    let mut attestations = Vec::new();
    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let attestation: AttestationBundle = serde_json::from_str(line)
            .map_err(|e| VerificationError::JsonError(format!("Failed to parse line: {}", e)))?;
        attestations.push(attestation);
    }

    dure_info!(
        "Loaded {} attestations from {}",
        attestations.len(),
        input_path
    );
    Ok(attestations)
}
