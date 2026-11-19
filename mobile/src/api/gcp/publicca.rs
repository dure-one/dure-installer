//! GCP Public Certificate Authority API module
//!
//! Handles External Account Key creation for ACME External Account Binding (EAB).
//! API Documentation: https://cloud.google.com/certificate-manager/docs/public-ca

use crate::{dure_debug, dure_error, dure_info};
use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::GcpRestClient;

/// API base URL for Public CA
pub const GCP_PUBLIC_CA_API_BASE: &str = "https://publicca.googleapis.com/v1";

// ============================================================================
// External Account Key Types
// ============================================================================

/// External Account Key for ACME EAB
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalAccountKey {
    /// Resource name: projects/{project}/locations/{location}/externalAccountKeys/{keyId}
    pub name: String,
    /// Key ID for ACME EAB
    pub key_id: String,
    /// Base64-URL-encoded HS256 key for ACME EAB
    pub b64_mac_key: String,
}

// ============================================================================
// API Methods on GcpRestClient
// ============================================================================

impl GcpRestClient {
    /// Create External Account Key for ACME EAB
    ///
    /// API: POST /v1/projects/{project}/locations/{location}/externalAccountKeys
    ///
    /// # Arguments
    /// - `project_id`: GCP project ID
    /// - `location`: Location (currently only "global" is supported)
    ///
    /// # Returns
    /// ExternalAccountKey with keyId and b64MacKey
    ///
    /// # Errors
    /// - API not enabled (403)
    /// - Missing IAM permission `publicca.externalAccountKeys.create` (403)
    /// - Network error
    pub fn create_external_account_key(
        &self,
        project_id: &str,
        location: &str,
    ) -> Result<ExternalAccountKey> {
        dure_debug!(
            "Creating external account key: project_id={}, location={}",
            project_id,
            location
        );

        let url = format!(
            "{}/projects/{}/locations/{}/externalAccountKeys",
            GCP_PUBLIC_CA_API_BASE, project_id, location
        );

        // POST with empty body (all fields are output-only)
        let response = self.post(&url, "{}")?;

        if response.status() != 200 {
            let error_text = response.into_string().unwrap_or_default();

            // Check for specific error cases
            if error_text.contains("has not been used in project")
                || error_text.contains("it is disabled")
            {
                return Err(anyhow::anyhow!(
                    "Public Certificate Authority API is not enabled. \
                    Please enable it in the GCP Console:\n{}",
                    error_text
                ));
            }

            if error_text.contains("ACCESS_TOKEN_SCOPE_INSUFFICIENT")
                || error_text.contains("insufficient authentication scopes")
            {
                return Err(anyhow::anyhow!(
                    "OAuth token missing required scope. \
                    Please re-authenticate in Platform tab to grant Public CA API access. \
                    (Disconnect → Connect GCP)"
                ));
            }

            if error_text.contains("publicca.externalAccountKeys.create") {
                return Err(anyhow::anyhow!(
                    "Missing IAM permission 'publicca.externalAccountKeys.create'. \
                    Please grant the permission in IAM:\n{}",
                    error_text
                ));
            }

            return Err(anyhow::anyhow!(
                "Failed to create external account key: {}",
                error_text
            ));
        }

        let key: ExternalAccountKey = response.into_json()?;
        dure_info!(
            "Successfully created external account key: key_id={}",
            key.key_id
        );
        Ok(key)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_external_account_key_deserialize() {
        let json = r#"{
            "name": "projects/test-project/locations/global/externalAccountKeys/abc123",
            "keyId": "test-key-id-12345",
            "b64MacKey": "dGVzdC1obWFjLWtleS1kYXRh"
        }"#;

        let key: ExternalAccountKey = serde_json::from_str(json).unwrap();
        assert_eq!(key.key_id, "test-key-id-12345");
        assert_eq!(key.b64_mac_key, "dGVzdC1obWFjLWtleS1kYXRh");
        assert!(key.name.contains("externalAccountKeys/abc123"));
    }

    #[test]
    fn test_external_account_key_serialize() {
        let key = ExternalAccountKey {
            name: "projects/test/locations/global/externalAccountKeys/xyz".to_string(),
            key_id: "key-123".to_string(),
            b64_mac_key: "aG1hYy1rZXk=".to_string(),
        };

        let json = serde_json::to_string(&key).unwrap();
        assert!(json.contains("key-123"));
        assert!(json.contains("aG1hYy1rZXk="));
    }

    #[test]
    fn test_api_base_url_format() {
        assert_eq!(
            GCP_PUBLIC_CA_API_BASE,
            "https://publicca.googleapis.com/v1"
        );
    }
}
