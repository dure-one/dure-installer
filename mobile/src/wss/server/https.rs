//! HTTPS request parsing and dispatch for the HTTPS/WSS server.

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use futures::io::{AsyncReadExt, AsyncWriteExt};
use std::collections::HashMap;
use std::io;

use super::{ServerSettings, Stats, generate_session_id};

/// HTTP request structure
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl HttpRequest {
    pub fn is_websocket_upgrade(&self) -> bool {
        self.headers
            .get("upgrade")
            .map(|v| v.to_lowercase() == "websocket")
            .unwrap_or(false)
            && self
                .headers
                .get("connection")
                .map(|v| v.to_lowercase().contains("upgrade"))
                .unwrap_or(false)
    }

    pub fn get_session_id(&self) -> Option<String> {
        if let Some(session_id) = self.headers.get("x-session-id") {
            return Some(session_id.clone());
        }

        if let Some(cookie) = self.headers.get("cookie") {
            for part in cookie.split(';') {
                let part = part.trim();
                if let Some(value) = part.strip_prefix("session_id=") {
                    return Some(value.to_string());
                }
            }
        }

        None
    }
}

/// Parse HTTP request from stream
pub async fn read_http_request<S: AsyncReadExt + Unpin>(stream: &mut S) -> io::Result<HttpRequest> {
    // Check if debug logging is enabled
    let debug = std::env::var("DURE_DEBUG_HTTP").is_ok();

    let mut request_line = String::new();
    let mut buf = [0u8; 1];

    // Read request line
    loop {
        stream.read_exact(&mut buf).await?;
        if buf[0] == b'\n' {
            break;
        }
        if buf[0] != b'\r' {
            request_line.push(buf[0] as char);
        }
    }

    // Debug: Print raw request line
    if debug {
        dure_debug!("Request line: {:?}", request_line);
    }

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 3 {
        if debug {
            dure_debug!("Invalid request line - parts: {:?}", parts);
        }
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid request line: {:?}", request_line),
        ));
    }

    let method = parts[0].to_string();
    let path = parts[1].to_string();
    let version = parts[2].to_string();

    if debug {
        dure_debug!(
            "Method={}, Path={}, Version={}",
            method, path, version
        );
    }

    let mut headers = HashMap::new();
    loop {
        let mut line = String::new();
        loop {
            stream.read_exact(&mut buf).await?;
            if buf[0] == b'\n' {
                break;
            }
            if buf[0] != b'\r' {
                line.push(buf[0] as char);
            }
        }

        if line.is_empty() {
            break;
        }

        if let Some(colon_pos) = line.find(':') {
            let key = line[..colon_pos].trim().to_lowercase();
            let value = line[colon_pos + 1..].trim().to_string();
            if debug {
                dure_debug!("Header: {} = {}", key, value);
            }
            headers.insert(key, value);
        }
    }

    let body = if let Some(content_length) = headers.get("content-length") {
        if let Ok(len) = content_length.parse::<usize>() {
            if debug {
                dure_debug!("Reading body of {} bytes", len);
            }
            let mut body = vec![0u8; len];
            stream.read_exact(&mut body).await?;
            body
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    if debug {
        dure_debug!("Request parsed successfully");
    }

    Ok(HttpRequest {
        method,
        path,
        version,
        headers,
        body,
    })
}

pub fn build_http_response(
    status: u16,
    status_text: &str,
    headers: Vec<(&str, &str)>,
    body: &[u8],
) -> Vec<u8> {
    let debug = std::env::var("DURE_DEBUG_HTTP").is_ok();

    let mut response = format!("HTTP/1.1 {} {}\r\n", status, status_text);
    for (key, value) in headers {
        response.push_str(&format!("{}: {}\r\n", key, value));
    }
    response.push_str(&format!("Content-Length: {}\r\n", body.len()));
    response.push_str("Connection: close\r\n\r\n");

    if debug {
        dure_debug!(
            "Response status: {} {}, body size: {} bytes",
            status,
            status_text,
            body.len()
        );
    }

    let mut response_bytes = response.into_bytes();
    response_bytes.extend_from_slice(body);
    response_bytes
}

pub async fn handle_https_request<S>(
    mut stream: S,
    request: HttpRequest,
    settings: ServerSettings,
    stats: Stats,
    peer_addr: std::net::SocketAddr,
) -> io::Result<()>
where
    S: AsyncReadExt + AsyncWriteExt + Unpin,
{
    use crate::storage::models::session;

    stats.http_request();
    dure_debug!("{} {} from {}", request.method, request.path, peer_addr);

    let session_id = request.get_session_id().unwrap_or_else(generate_session_id);

    if let Ok(mut db) = settings.db.lock() {
        let sess = session::Session::new(
            session_id.clone(),
            settings.domain.clone(),
            "http".to_string(),
            peer_addr.to_string(),
        );
        if let Err(e) = session::store_session(&mut db, &sess) {
            dure_debug!("Failed to store session: {}", e);
        }
    }

    match request.method.as_str() {
        "GET" => super::http_get::handle_http_get(&mut stream, &request, &settings).await,
        "POST" => {
            super::http_post::handle_http_post(&mut stream, &request, &settings, &stats, peer_addr)
                .await
        }
        "DELETE" => handle_delete(&mut stream, &request, &settings).await,
        "PUT" => handle_put(&mut stream, &request, &settings).await,
        _ => {
            let body = b"405 Method Not Allowed";
            let response = build_http_response(
                405,
                "Method Not Allowed",
                vec![("Content-Type", "text/plain")],
                body,
            );
            stream.write_all(&response).await?;
            stream.flush().await?;
            Ok(())
        }
    }
}

async fn handle_delete<S: AsyncWriteExt + Unpin>(
    stream: &mut S,
    request: &HttpRequest,
    settings: &ServerSettings,
) -> io::Result<()> {
    // DELETE /api/todo/{id}
    if request.path.starts_with("/api/todo/") {
        let id_str = request
            .path
            .trim_end_matches('/')
            .rsplit('/')
            .next()
            .unwrap_or("");
        let api_key = request
            .headers
            .get("todo_apikey")
            .map(String::as_str)
            .unwrap_or("");
        let (status, body) = match id_str.parse::<i32>() {
            Ok(id) => super::api::todo::delete_todo(&settings.todo_store, id, api_key),
            Err(_) => (400, "{\"error\":\"invalid id\"}".to_string()),
        };
        let status_str = match status {
            200 => "OK",
            401 => "Unauthorized",
            404 => "Not Found",
            _ => "Bad Request",
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

    // DELETE /api/webhook/patterns/{id}
    if request.path.starts_with("/api/webhook/patterns/") {
        let id_str = request
            .path
            .trim_end_matches('/')
            .rsplit('/')
            .next()
            .unwrap_or("");
        let response = match id_str.parse::<i64>() {
            Ok(id) => {
                let result = settings.db.lock().ok().and_then(|mut db| {
                    crate::storage::models::webhook::delete_webhook_pattern(&mut db, id).ok()
                });
                if result.is_some() {
                    build_http_response(
                        200,
                        "OK",
                        vec![("Content-Type", "application/json")],
                        b"{}",
                    )
                } else {
                    build_http_response(
                        404,
                        "Not Found",
                        vec![("Content-Type", "application/json")],
                        b"{\"error\":\"not found\"}",
                    )
                }
            }
            Err(_) => build_http_response(
                400,
                "Bad Request",
                vec![("Content-Type", "application/json")],
                b"{\"error\":\"invalid id\"}",
            ),
        };
        stream.write_all(&response).await?;
        stream.flush().await?;
        return Ok(());
    }

    let response = build_http_response(
        404,
        "Not Found",
        vec![("Content-Type", "text/plain")],
        b"404 Not Found",
    );
    stream.write_all(&response).await?;
    stream.flush().await?;
    Ok(())
}

async fn handle_put<S: AsyncWriteExt + Unpin>(
    stream: &mut S,
    request: &HttpRequest,
    settings: &ServerSettings,
) -> io::Result<()> {
    // PUT /api/todo/{id}
    if request.path.starts_with("/api/todo/") {
        let id_str = request
            .path
            .trim_end_matches('/')
            .rsplit('/')
            .next()
            .unwrap_or("");
        let (status, body) = match id_str.parse::<i32>() {
            Ok(id) => super::api::todo::mark_done(&settings.todo_store, id),
            Err(_) => (400, "{\"error\":\"invalid id\"}".to_string()),
        };
        let status_str = match status {
            200 => "OK",
            404 => "Not Found",
            _ => "Bad Request",
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

    // PUT /api/webhook/config
    if request.path == "/api/webhook/config" {
        use crate::storage::models::webhook::{
            WebhookConfig, get_webhook_config, update_webhook_config,
        };
        let req: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);
        let response = match req {
            Ok(v) => {
                let enabled = v
                    .get("logging_enabled")
                    .and_then(|b| b.as_bool())
                    .unwrap_or(false);
                let cfg = WebhookConfig {
                    logging_enabled: enabled,
                };
                let result = settings.db.lock().ok().and_then(|mut db| {
                    update_webhook_config(&mut db, &cfg).ok()?;
                    get_webhook_config(&mut db).ok()
                });
                match result {
                    Some(c) => {
                        let body = format!("{{\"logging_enabled\":{}}}", c.logging_enabled);
                        build_http_response(
                            200,
                            "OK",
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
            Err(e) => {
                let body = format!("{{\"error\":\"{}\"}}", e);
                build_http_response(
                    400,
                    "Bad Request",
                    vec![("Content-Type", "application/json")],
                    body.as_bytes(),
                )
            }
        };
        stream.write_all(&response).await?;
        stream.flush().await?;
        return Ok(());
    }

    let response = build_http_response(
        404,
        "Not Found",
        vec![("Content-Type", "text/plain")],
        b"404 Not Found",
    );
    stream.write_all(&response).await?;
    stream.flush().await?;
    Ok(())
}
