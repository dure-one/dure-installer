//! GCP Cloud Resource Manager API module
//!
//! Handles GCP project operations.

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::{GcpRestClient, GCP_RESOURCE_MANAGER_API_BASE};
use crate::api::gcp::compute::Operation;

// ============================================================================
// Project Types
// ============================================================================

/// Project list response
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectList {
    #[serde(default)]
    pub projects: Vec<Project>,
    #[serde(default)]
    pub next_page_token: Option<String>,
}

/// Project details
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    #[serde(default)]
    pub name: Option<String>, // e.g., "projects/my-project-123"
    pub project_id: String, // e.g., "my-project-123" (always present)
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default, rename = "lifecycleState")]
    pub state: Option<String>, // "ACTIVE", "DELETE_REQUESTED", etc.
    #[serde(default)]
    pub labels: std::collections::HashMap<String, String>,
}

impl Project {
    /// Extract project ID from name if needed
    pub fn id(&self) -> &str {
        &self.project_id
    }

    /// Get display name with fallback to project_id
    pub fn display_name(&self) -> &str {
        self.display_name.as_deref().unwrap_or(&self.project_id)
    }

    /// Get project state with fallback to "UNKNOWN"
    pub fn state(&self) -> &str {
        self.state.as_deref().unwrap_or("UNKNOWN")
    }

    /// Check if project is active/usable (not being deleted)
    pub fn is_active(&self) -> bool {
        match self.state.as_deref() {
            // Explicitly active
            Some("ACTIVE") => true,
            // No state field or unspecified - assume usable
            None | Some("LIFECYCLE_STATE_UNSPECIFIED") => true,
            // Being deleted - not usable
            Some("DELETE_REQUESTED") | Some("DELETE_IN_PROGRESS") => false,
            // Unknown state - assume usable to be safe
            _ => true,
        }
    }
}

// ============================================================================
// API Methods on GcpRestClient
// ============================================================================

impl GcpRestClient {
    /// List all projects the user has access to
    ///
    /// API: GET /v3/projects
    /// List GCP projects that the user has access to
    ///
    /// API: GET /v1/projects
    ///
    /// # Arguments
    /// * `filter` - Optional filter expression (e.g., "name:my-project-*", "labels.env:prod")
    ///   See: https://cloud.google.com/resource-manager/reference/rest/v1/projects/list#query-parameters
    ///
    /// # Examples
    /// ```no_run
    /// # use dure::api::gcp::GcpRestClient;
    /// let client = GcpRestClient::new("token".to_string());
    ///
    /// // List all projects
    /// let all_projects = client.list_projects(None)?;
    ///
    /// // List projects with specific filter
    /// let filtered = client.list_projects(Some("labels.env:prod"))?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn list_projects(&self, filter: Option<&str>) -> Result<ProjectList> {
        let mut url = format!("{}/projects", GCP_RESOURCE_MANAGER_API_BASE);

        // Add filter query parameter if provided
        if let Some(filter_value) = filter {
            url = format!("{}?filter={}", url, urlencoding::encode(filter_value));
        }

        let response = self.get(&url)?;

        if response.status() != 200 {
            let error_text = response.into_string().unwrap_or_default();
            return Err(anyhow::anyhow!("Failed to list projects: {}", error_text));
        }

        let list: ProjectList = response.into_json()?;
        Ok(list)
    }

    /// Get project details
    ///
    /// API: GET /v3/projects/{projectId}
    pub fn get_project(&self, project_id: &str) -> Result<Project> {
        let url = format!("{}/projects/{}", GCP_RESOURCE_MANAGER_API_BASE, project_id);

        let response = self.get(&url)?;
        let project: Project = response.into_json()?;
        Ok(project)
    }

    /// Create a new project
    ///
    /// API: POST /v1/projects
    pub fn create_project(&self, project_id: &str, display_name: &str) -> Result<Operation> {
        let url = format!("{}/projects", GCP_RESOURCE_MANAGER_API_BASE);

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct CreateProjectRequest {
            project_id: String,
            name: String,
            labels: std::collections::HashMap<String, String>,
        }

        let mut labels = std::collections::HashMap::new();
        labels.insert("dure".to_string(), "true".to_string());

        let body = serde_json::to_string(&CreateProjectRequest {
            project_id: project_id.to_string(),
            name: display_name.to_string(),
            labels,
        })?;

        let response = self.post(&url, &body)?;

        if response.status() != 200 {
            let error_text = response.into_string().unwrap_or_default();
            return Err(anyhow::anyhow!("Failed to create project: {}", error_text));
        }

        let operation: Operation = response.into_json()?;
        Ok(operation)
    }
}
