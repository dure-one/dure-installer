//! GCP Cloud Billing API module
//!
//! Handles GCP billing account and project billing operations.

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::{GcpRestClient, GCP_BILLING_API_BASE};

// ============================================================================
// Billing Types
// ============================================================================

/// Billing account list response
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BillingAccountList {
    #[serde(default)]
    pub billing_accounts: Vec<BillingAccount>,
    #[serde(default)]
    pub next_page_token: Option<String>,
}

/// Billing account details
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BillingAccount {
    pub name: String, // e.g., "billingAccounts/012345-ABCDEF-678901"
    pub display_name: String,
    pub open: bool,
    #[serde(default)]
    pub master_billing_account: Option<String>,
}

/// Project billing info
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectBillingInfo {
    pub name: String, // e.g., "projects/my-project/billingInfo"
    pub project_id: String,
    #[serde(default)]
    pub billing_account_name: Option<String>, // e.g., "billingAccounts/012345-ABCDEF-678901"
    pub billing_enabled: bool,
}

impl BillingAccount {
    /// Extract billing account ID from name
    /// e.g., "billingAccounts/012345-ABCDEF-678901" -> "012345-ABCDEF-678901"
    pub fn id(&self) -> Option<&str> {
        self.name.strip_prefix("billingAccounts/")
    }
}

// ============================================================================
// API Methods on GcpRestClient
// ============================================================================

impl GcpRestClient {
    /// List billing accounts
    ///
    /// API: GET /v1/billingAccounts
    pub fn list_billing_accounts(&self) -> Result<BillingAccountList> {
        let url = format!("{}/billingAccounts", GCP_BILLING_API_BASE);

        let response = self.get(&url)?;

        if response.status() != 200 {
            let error_text = response.into_string().unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Failed to list billing accounts: {}",
                error_text
            ));
        }

        let list: BillingAccountList = response.into_json()?;
        Ok(list)
    }

    /// Get billing info for a project
    ///
    /// API: GET /v1/projects/{projectId}/billingInfo
    pub fn get_project_billing_info(&self, project_id: &str) -> Result<ProjectBillingInfo> {
        let url = format!(
            "{}/projects/{}/billingInfo",
            GCP_BILLING_API_BASE, project_id
        );

        let response = self.get(&url)?;

        if response.status() != 200 {
            let error_text = response.into_string().unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Failed to get billing info: {}",
                error_text
            ));
        }

        let info: ProjectBillingInfo = response.into_json()?;
        Ok(info)
    }

    /// Enable billing for a project by associating it with a billing account
    ///
    /// API: PUT /v1/projects/{projectId}/billingInfo
    pub fn enable_project_billing(
        &self,
        project_id: &str,
        billing_account_name: &str,
    ) -> Result<ProjectBillingInfo> {
        let url = format!(
            "{}/projects/{}/billingInfo",
            GCP_BILLING_API_BASE, project_id
        );

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct BillingInfoUpdate {
            billing_account_name: String,
        }

        let body = serde_json::to_string(&BillingInfoUpdate {
            billing_account_name: billing_account_name.to_string(),
        })?;

        let response = match ureq::put(&url)
            .set("Authorization", &format!("Bearer {}", self.access_token))
            .set("Content-Type", "application/json")
            .send_string(&body)
        {
            Ok(response) => response,
            Err(ureq::Error::Status(code, response)) => {
                let body = response.into_string().unwrap_or_default();
                return Err(anyhow::anyhow!("HTTP {} error for {}: {}", code, url, body));
            }
            Err(ureq::Error::Transport(transport)) => {
                return Err(anyhow::anyhow!("Network error for {}: {}", url, transport));
            }
        };

        if response.status() != 200 {
            let error_text = response.into_string().unwrap_or_default();
            return Err(anyhow::anyhow!("Failed to enable billing: {}", error_text));
        }

        let info: ProjectBillingInfo = response.into_json()?;
        Ok(info)
    }
}
