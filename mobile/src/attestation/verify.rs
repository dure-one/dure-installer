//! Verify attestations against binaries

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use super::download::{compute_file_hash, download_attestations};
use super::types::{AttestationPayload, AttestationResult, VerificationError};
use std::path::Path;

/// Verify a binary against its GitHub attestations
///
/// # Arguments
/// * `binary_path` - Path to the binary to verify
/// * `owner` - GitHub repository owner
/// * `repo` - GitHub repository name
///
/// # Returns
/// * `Ok(AttestationResult)` if verification succeeds
/// * `Err(VerificationError)` if verification fails
pub fn verify_binary(
    binary_path: &str,
    owner: &str,
    repo: &str,
) -> Result<AttestationResult, VerificationError> {
    dure_info!("Verifying binary: {}", binary_path);
    dure_info!("Repository: {}/{}", owner, repo);

    let path = Path::new(binary_path);

    // Step 1: Compute the hash of the binary
    let digest = compute_file_hash(path)?;
    dure_info!("Binary SHA256: {}", digest);

    // Step 2: Download attestations from GitHub
    let attestations = download_attestations(binary_path, owner, repo)?;

    if attestations.is_empty() {
        dure_warn!("No attestations found for this binary");
        return Err(VerificationError::NoAttestationsFound);
    }

    // Step 3: Verify each attestation
    let mut verified_count = 0;
    let mut last_predicate_type = String::new();
    let mut last_workflow = String::new();

    for attestation in &attestations {
        // Check if signature was verified by GitHub
        if let Some(summary) = &attestation.verification_summary {
            if !summary.signature_verified {
                dure_warn!("Attestation signature not verified");
                continue;
            }

            // Extract certificate info
            if let Some(cert) = &summary.certificate {
                if let Some(san) = &cert.subject_alternative_name {
                    last_workflow = san.clone();
                }
            }
        }

        // Decode and verify the payload
        match verify_attestation_payload(&attestation.attestation.payload, &digest) {
            Ok(payload) => {
                last_predicate_type = payload.predicate_type.clone();
                verified_count += 1;
                dure_info!(
                    "Verified attestation with predicate: {}",
                    payload.predicate_type
                );
            }
            Err(e) => {
                dure_warn!("Failed to verify attestation payload: {}", e);
            }
        }
    }

    if verified_count == 0 {
        return Err(VerificationError::SignatureVerificationFailed(
            "No attestations could be verified".to_string(),
        ));
    }

    Ok(AttestationResult {
        verified: true,
        digest,
        predicate_type: last_predicate_type,
        repository: format!("{}/{}", owner, repo),
        workflow: last_workflow,
        attestation_count: verified_count,
    })
}

/// Verify the payload of an attestation
fn verify_attestation_payload(
    payload_base64: &str,
    expected_digest: &str,
) -> Result<AttestationPayload, VerificationError> {
    // Decode base64 payload
    let payload_bytes = base64_decode(payload_base64)?;
    let payload_str = String::from_utf8(payload_bytes)
        .map_err(|e| VerificationError::InvalidAttestation(format!("Invalid UTF-8: {}", e)))?;

    // Parse JSON payload
    let payload: AttestationPayload = serde_json::from_str(&payload_str)
        .map_err(|e| VerificationError::JsonError(format!("Failed to parse payload: {}", e)))?;

    // Verify that the digest matches
    let mut digest_matches = false;
    for subject in &payload.subject {
        if subject.digest.sha256 == expected_digest {
            digest_matches = true;
            break;
        }
    }

    if !digest_matches {
        return Err(VerificationError::InvalidAttestation(
            "Digest does not match any subject in attestation".to_string(),
        ));
    }

    Ok(payload)
}

/// Base64 decode helper
fn base64_decode(input: &str) -> Result<Vec<u8>, VerificationError> {
    use base64::{Engine, engine::general_purpose::STANDARD};
    STANDARD
        .decode(input)
        .map_err(|e| VerificationError::InvalidAttestation(format!("Base64 decode error: {}", e)))
}

/// Verify WASM binary (special handling for web environment)
#[cfg(target_arch = "wasm32")]
pub fn verify_wasm(
    wasm_bytes: &[u8],
    owner: &str,
    repo: &str,
) -> Result<AttestationResult, VerificationError> {
    use sha2::{Digest, Sha256};

    dure_info!("Verifying WASM binary");
    dure_info!("Repository: {}/{}", owner, repo);

    // Compute SHA256 of WASM bytes
    let mut hasher = Sha256::new();
    hasher.update(wasm_bytes);
    let digest_bytes = hasher.finalize();
    let digest = format!("{:x}", digest_bytes);

    dure_info!("WASM SHA256: {}", digest);

    // For WASM, we'd need to fetch attestations via ehttp
    // This is a simplified implementation
    dure_warn!("WASM attestation verification not fully implemented");

    // Return a result indicating the hash was computed but not verified
    Ok(AttestationResult {
        verified: false,
        digest,
        predicate_type: "https://dure.io/attestation/wasm/v1".to_string(),
        repository: format!("{}/{}", owner, repo),
        workflow: "unknown".to_string(),
        attestation_count: 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_decode() {
        let input = "SGVsbG8gV29ybGQ=";
        let result = base64_decode(input).unwrap();
        assert_eq!(result, b"Hello World");
    }
}
