//! Types for GitHub Artifact Attestations

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Error types for attestation verification
#[derive(Debug)]
pub enum VerificationError {
    /// Failed to download attestations
    DownloadFailed(String),
    /// Failed to compute hash of binary
    HashError(String),
    /// No attestations found for the binary
    NoAttestationsFound,
    /// Attestation signature verification failed
    SignatureVerificationFailed(String),
    /// Attestation content is invalid
    InvalidAttestation(String),
    /// I/O error
    IoError(String),
    /// JSON parsing error
    JsonError(String),
    /// HTTP request error
    HttpError(String),
}

impl fmt::Display for VerificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DownloadFailed(msg) => write!(f, "Failed to download attestations: {}", msg),
            Self::HashError(msg) => write!(f, "Failed to compute hash: {}", msg),
            Self::NoAttestationsFound => write!(f, "No attestations found for this binary"),
            Self::SignatureVerificationFailed(msg) => {
                write!(f, "Signature verification failed: {}", msg)
            }
            Self::InvalidAttestation(msg) => write!(f, "Invalid attestation: {}", msg),
            Self::IoError(msg) => write!(f, "I/O error: {}", msg),
            Self::JsonError(msg) => write!(f, "JSON error: {}", msg),
            Self::HttpError(msg) => write!(f, "HTTP error: {}", msg),
        }
    }
}

impl std::error::Error for VerificationError {}

/// Result of attestation verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationResult {
    /// Whether the attestation was successfully verified
    pub verified: bool,
    /// The digest (SHA256 hash) of the binary
    pub digest: String,
    /// The predicate type of the attestation
    pub predicate_type: String,
    /// The repository that created the attestation
    pub repository: String,
    /// The workflow that created the attestation
    pub workflow: String,
    /// Number of attestations verified
    pub attestation_count: usize,
}

/// GitHub Attestation Bundle (JSONL format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationBundle {
    /// The attestation data in DSSE format
    pub attestation: DsseEnvelope,
    /// Verification summary
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_summary: Option<VerificationSummary>,
}

/// DSSE (Dead Simple Signing Envelope) format
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DsseEnvelope {
    /// Payload (base64 encoded)
    pub payload: String,
    /// Payload type
    pub payload_type: String,
    /// Signatures
    pub signatures: Vec<Signature>,
}

/// Signature in DSSE envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    /// Key ID hint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyid: Option<String>,
    /// Signature bytes (base64 encoded)
    pub sig: String,
}

/// Verification summary from GitHub
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationSummary {
    /// Whether the signature was verified
    pub signature_verified: bool,
    /// Certificate information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate: Option<CertificateInfo>,
}

/// Certificate information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CertificateInfo {
    /// Source repository owner
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_repository_owner: Option<String>,
    /// Source repository
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_repository: Option<String>,
    /// Subject alternative name (workflow)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_alternative_name: Option<String>,
}

/// Attestation payload (decoded from base64)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttestationPayload {
    /// Type field
    #[serde(rename = "_type")]
    pub type_field: String,
    /// Subject of the attestation
    pub subject: Vec<Subject>,
    /// Predicate type
    pub predicate_type: String,
    /// Predicate (build details)
    pub predicate: serde_json::Value,
}

/// Subject in attestation payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subject {
    /// Name/path of the artifact
    pub name: String,
    /// Digest of the artifact
    pub digest: DigestInfo,
}

/// Digest information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestInfo {
    /// SHA256 hash
    pub sha256: String,
}

/// GitHub API response for attestations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationsResponse {
    /// List of attestation bundles
    pub attestations: Vec<AttestationBundle>,
}
