//! GCP Service Usage API module
//!
//! Handles enabling and checking GCP service APIs.

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use anyhow::Result;

use super::{GcpRestClient, GCP_SERVICE_USAGE_API_BASE};

// ============================================================================
// API Methods on GcpRestClient
// ============================================================================

impl GcpRestClient {
    /// Enable a GCP service API
    ///
    /// # Arguments
    /// * `project_id` - The GCP project ID
    /// * `service` - The service name (e.g., "bigquery.googleapis.com")
    ///
    /// # Example
    /// ```no_run
    /// # use dure::api::gcp::GcpRestClient;
    /// let client = GcpRestClient::new("token".to_string());
    /// client.enable_service("my-project", "bigquery.googleapis.com")?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn enable_service(&self, project_id: &str, service: &str) -> Result<()> {
        let url = format!(
            "{}/projects/{}/services/{}:enable",
            GCP_SERVICE_USAGE_API_BASE, project_id, service
        );

        let response = self.post(&url, "{}")?;
        let status = response.status();

        if status == 200 || status == 201 {
            Ok(())
        } else {
            let body = response.into_string().unwrap_or_default();
            Err(anyhow::anyhow!(
                "Failed to enable service {}: HTTP {} - {}",
                service,
                status,
                body
            ))
        }
    }

    /// Check if a GCP service API is enabled
    ///
    /// # Arguments
    /// * `project_id` - The GCP project ID
    /// * `service` - The service name (e.g., "bigquery.googleapis.com")
    pub fn is_service_enabled(&self, project_id: &str, service: &str) -> Result<bool> {
        let url = format!(
            "{}/projects/{}/services/{}",
            GCP_SERVICE_USAGE_API_BASE, project_id, service
        );

        let response = self.get(&url)?;
        let body: serde_json::Value = response.into_json()?;

        // Check if state is "ENABLED"
        Ok(body["state"].as_str() == Some("ENABLED"))
    }
}
