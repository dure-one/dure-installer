//! Webhook API: manage allow-patterns, view logs, configure logging.
//!
//! Endpoints (all under `/api/webhook`):
//! - `GET  /api/webhook/config`          – get logging config
//! - `PUT  /api/webhook/config`          – update logging config
//! - `GET  /api/webhook/patterns`        – list allow-patterns
//! - `POST /api/webhook/patterns`        – add an allow-pattern
//! - `DELETE /api/webhook/patterns/{id}` – remove an allow-pattern
//! - `GET  /api/webhook/logs`            – list recent webhook request logs

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

#[derive(OpenApi)]
#[openapi(
    paths(
        get_config, update_config,
        list_patterns, add_pattern, delete_pattern,
        list_logs,
    ),
    components(schemas(
        WebhookConfigDto, WebhookPatternDto, WebhookLogDto,
        AddPatternRequest, UpdateConfigRequest, WebhookError,
    )),
    tags((name = "webhook", description = "Webhook management endpoints."))
)]
pub struct WebhookApi;

// ── DTOs ──────────────────────────────────────────────────────────────────────

/// Webhook logging configuration
#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct WebhookConfigDto {
    /// Whether incoming webhook request bodies are persisted to the database
    pub logging_enabled: bool,
}

/// An allow-pattern entry (path prefix that is accepted as a webhook target)
#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct WebhookPatternDto {
    /// Row id
    pub id: i64,
    /// Path prefix, e.g. `/webhook/`
    pub pattern: String,
    /// Unix timestamp (seconds) when the pattern was added
    pub created_at: u64,
}

/// A logged webhook request
#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct WebhookLogDto {
    pub id: i64,
    /// Matched allow-pattern
    pub pattern: String,
    /// Actual request path
    pub path: String,
    pub method: String,
    /// Request headers as a JSON object string
    pub headers: String,
    pub body: String,
    pub remote_addr: String,
    /// Unix timestamp of receipt
    pub received_at: u64,
}

/// Request body for adding a new allow-pattern
#[derive(Serialize, Deserialize, ToSchema)]
pub struct AddPatternRequest {
    /// Path prefix to allow, e.g. `/webhook/github`
    pub pattern: String,
}

/// Request body for updating logging configuration
#[derive(Serialize, Deserialize, ToSchema)]
pub struct UpdateConfigRequest {
    pub logging_enabled: bool,
}

/// Error response
#[derive(Serialize, Deserialize, ToSchema)]
pub struct WebhookError {
    pub error: String,
}

// ── Stub handler annotations (real logic is in https.rs / http_post.rs) ──────

/// Get webhook logging configuration
#[utoipa::path(
    get, path = "/config",
    responses(
        (status = 200, description = "Current config", body = WebhookConfigDto),
        (status = 500, description = "Database error",  body = WebhookError),
    )
)]
pub fn get_config() {}

/// Update webhook logging configuration
#[utoipa::path(
    put, path = "/config",
    request_body = UpdateConfigRequest,
    responses(
        (status = 200, description = "Config updated",  body = WebhookConfigDto),
        (status = 400, description = "Invalid request", body = WebhookError),
    )
)]
pub fn update_config() {}

/// List webhook allow-patterns
#[utoipa::path(
    get, path = "/patterns",
    responses(
        (status = 200, description = "List of allow-patterns", body = [WebhookPatternDto]),
    )
)]
pub fn list_patterns() {}

/// Add a webhook allow-pattern
#[utoipa::path(
    post, path = "/patterns",
    request_body = AddPatternRequest,
    responses(
        (status = 201, description = "Pattern added",    body = WebhookPatternDto),
        (status = 400, description = "Invalid request",  body = WebhookError),
    )
)]
pub fn add_pattern() {}

/// Delete a webhook allow-pattern
#[utoipa::path(
    delete, path = "/patterns/{id}",
    params(("id" = i64, Path, description = "Pattern id to delete")),
    responses(
        (status = 200, description = "Pattern deleted"),
        (status = 404, description = "Not found", body = WebhookError),
    )
)]
pub fn delete_pattern() {}

/// List recent webhook request logs
#[utoipa::path(
    get, path = "/logs",
    params(
        ("limit" = Option<i64>, Query, description = "Max results (default 100)")
    ),
    responses(
        (status = 200, description = "List of logged requests", body = [WebhookLogDto]),
    )
)]
pub fn list_logs() {}
