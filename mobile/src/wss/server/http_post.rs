//! Webhook POST request handling and todo API POST for the HTTPS/WSS server.

use futures::io::AsyncWriteExt;
use std::collections::HashMap;
use std::io;

use super::https::{HttpRequest, build_http_response};
use super::{ServerSettings, Stats};

pub fn headers_to_json(headers: &HashMap<String, String>) -> String {
    let parts: Vec<String> = headers
        .iter()
        .map(|(k, v)| {
            format!(
                "\"{}\":\"{}\"",
                k.replace('"', "\\\""),
                v.replace('"', "\\\"")
            )
        })
        .collect();
    format!("{{{}}}", parts.join(","))
}

/// Handle an HTTP POST request.
///
/// Routes:
/// - `/api/todo`              → create todo
/// - `/api/webhook/patterns`  → add webhook allow-pattern
/// - anything else            → webhook handler (log if pattern matches)
pub async fn handle_http_post<S: AsyncWriteExt + Unpin>(
    stream: &mut S,
    request: &HttpRequest,
    settings: &ServerSettings,
    stats: &Stats,
    peer_addr: std::net::SocketAddr,
) -> io::Result<()> {
    if request.path == "/api/todo" {
        let (status, body) = super::api::todo::create_todo(&settings.todo_store, &request.body);
        let status_str = if status == 201 {
            "Created"
        } else if status == 409 {
            "Conflict"
        } else {
            "Bad Request"
        };
        let response = build_http_response(
            status,
            status_str,
            vec![("Content-Type", "application/json")],
            body.as_bytes(),
        );
        stream.write_all(&response).await?;
        stream.flush().await?;
        return Ok(());
    }

    if request.path == "/api/webhook/patterns" {
        use crate::storage::models::webhook::add_webhook_pattern;
        let req: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);
        let response = match req.ok().and_then(|v| {
            v.get("pattern")
                .and_then(|p| p.as_str())
                .map(str::to_string)
        }) {
            Some(pattern) => {
                let result = settings
                    .db
                    .lock()
                    .ok()
                    .and_then(|mut db| add_webhook_pattern(&mut db, &pattern).ok());
                match result {
                    Some(id) => {
                        let body = format!(
                            "{{\"id\":{},\"pattern\":\"{}\",\"created_at\":0}}",
                            id, pattern
                        );
                        build_http_response(
                            201,
                            "Created",
                            vec![("Content-Type", "application/json")],
                            body.as_bytes(),
                        )
                    }
                    None => build_http_response(
                        500,
                        "Internal Server Error",
                        vec![("Content-Type", "application/json")],
                        b"{\"error\":\"db error\"}",
                    ),
                }
            }
            None => build_http_response(
                400,
                "Bad Request",
                vec![("Content-Type", "application/json")],
                b"{\"error\":\"missing pattern field\"}",
            ),
        };
        stream.write_all(&response).await?;
        stream.flush().await?;
        return Ok(());
    }

    handle_webhook_post(stream, request, settings, stats, peer_addr).await
}

async fn handle_webhook_post<S: AsyncWriteExt + Unpin>(
    stream: &mut S,
    request: &HttpRequest,
    settings: &ServerSettings,
    stats: &Stats,
    peer_addr: std::net::SocketAddr,
) -> io::Result<()> {
    use crate::storage::models::webhook;

    stats.webhook_post();

    let (pattern_match, logging_enabled) = if let Ok(mut db) = settings.db.lock() {
        let pattern = webhook::find_matching_pattern(&mut db, &request.path)
            .ok()
            .flatten();
        let logging = webhook::get_webhook_config(&mut db)
            .map(|c| c.logging_enabled)
            .unwrap_or(false);
        (pattern, logging)
    } else {
        (None, false)
    };

    let (status, response_body) = if let Some(pattern) = pattern_match {
        eprintln!("Webhook matched pattern: {}", pattern);

        if logging_enabled {
            let headers_json = headers_to_json(&request.headers);
            let body = String::from_utf8_lossy(&request.body).to_string();

            if let Ok(mut db) = settings.db.lock() {
                if let Err(e) = webhook::log_webhook_request(
                    &mut db,
                    &pattern,
                    &request.path,
                    "POST",
                    &headers_json,
                    &body,
                    &peer_addr.to_string(),
                ) {
                    eprintln!("Failed to log webhook: {}", e);
                }
            }

            eprintln!("Webhook request logged");
        }

        (200, b"{\"status\":\"received\"}".to_vec())
    } else {
        eprintln!("Webhook path not in allow list: {}", request.path);
        (404, b"{\"error\":\"not found\"}".to_vec())
    };

    let response = build_http_response(
        status,
        if status == 200 { "OK" } else { "Not Found" },
        vec![("Content-Type", "application/json")],
        &response_body,
    );

    stream.write_all(&response).await?;
    stream.flush().await?;
    Ok(())
}
