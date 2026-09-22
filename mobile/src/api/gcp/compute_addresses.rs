//! GCP Compute Engine Addresses API module
//!
//! Manages external IP addresses (static/reserved).

use anyhow::Result;
use serde::Deserialize;

use super::{GCP_COMPUTE_API_BASE, GcpRestClient};
use super::compute::Operation;

/// External IP address (static/reserved)
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Address {
    pub name: String,
    pub address: String,        // IP address (e.g., "35.1.2.3")
    pub status: String,         // "RESERVED", "IN_USE"
    pub address_type: String,   // "EXTERNAL", "INTERNAL"
    #[serde(default)]
    pub users: Vec<String>,     // VM instances using this IP (empty if RESERVED)
    pub region: String,         // e.g., "us-central1"
    #[serde(default)]
    pub network_tier: Option<String>, // "PREMIUM" or "STANDARD"
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddressList {
    #[serde(default)]
    pub items: Vec<Address>,
}

impl GcpRestClient {
    /// List all addresses in a region
    pub fn list_addresses(
        &self,
        project_id: &str,
        region: &str,
    ) -> Result<Vec<Address>> {
        let url = format!(
            "{}/projects/{}/regions/{}/addresses",
            GCP_COMPUTE_API_BASE, project_id, region
        );

        let response = self.get(&url)?;
        let list: AddressList = response.into_json()?;
        Ok(list.items)
    }

    /// Get a specific address
    pub fn get_address(
        &self,
        project_id: &str,
        region: &str,
        address_name: &str,
    ) -> Result<Address> {
        let url = format!(
            "{}/projects/{}/regions/{}/addresses/{}",
            GCP_COMPUTE_API_BASE, project_id, region, address_name
        );

        let response = self.get(&url)?;
        let address: Address = response.into_json()?;
        Ok(address)
    }

    /// Delete (release) a reserved address
    pub fn delete_address(
        &self,
        project_id: &str,
        region: &str,
        address_name: &str,
    ) -> Result<Operation> {
        let url = format!(
            "{}/projects/{}/regions/{}/addresses/{}",
            GCP_COMPUTE_API_BASE, project_id, region, address_name
        );

        let response = self.delete(&url)?;
        let operation: Operation = response.into_json()?;
        Ok(operation)
    }
}
