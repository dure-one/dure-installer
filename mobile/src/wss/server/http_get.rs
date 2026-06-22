//! Static file serving (GET request handling) for the HTTPS/WSS server.
//! Also handles Swagger UI and OpenAPI JSON routes.
//!
//! ## Compression Support
//!
//! This module supports serving pre-compressed static files (.gz and .br) based on
//! the client's `Accept-Encoding` header:
//!
//! - **Brotli (.br)**: Preferred compression (15-20% better than gzip)
//! - **Gzip (.gz)**: Fallback compression (widely supported)
//! - **None**: Uncompressed file if no compressed version exists
//!
//! The build process (`build.wasm.sh`) creates compressed versions of WASM and JS files.
//! The server automatically selects the best available compression based on client support.

use async_fs as fs;
use futures::io::AsyncWriteExt;
use std::io;
use std::path::Path;
use std::sync::Arc;

use super::ServerSettings;
use super::https::{HttpRequest, build_http_response};

/// Download and extract dure-wasm static files
pub async fn download_static_files(dir: &Path) -> io::Result<()> {
    eprintln!("Downloading static files from GitHub...");

    fs::create_dir_all(dir).await?;

    let zip_path = dir.join("dure-wasm.zip");
    let url = "https://github.com/nikescar/dure-wasm/archive/refs/heads/main.zip";
    let zip_path_clone = zip_path.clone();

    std::thread::spawn(move || -> io::Result<()> {
        let response = ureq::get(url)
            .call()
            .map_err(|e| io::Error::other(format!("Download failed: {}", e)))?;

        let mut file = std::fs::File::create(&zip_path_clone)?;
        std::io::copy(&mut response.into_reader(), &mut file)?;
        Ok(())
    })
    .join()
    .map_err(|_| io::Error::other("Thread panicked"))??;

    eprintln!("Downloaded to {:?}, extracting...", zip_path);

    let dir_clone = dir.to_path_buf();
    std::thread::spawn(move || -> io::Result<()> {
        let file = std::fs::File::open(&zip_path)?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| io::Error::other(format!("Unzip failed: {}", e)))?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| io::Error::other(e))?;

            let outpath = match file.enclosed_name() {
                Some(path) => dir_clone.join(path),
                None => continue,
            };

            if file.name().ends_with('/') {
                std::fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    std::fs::create_dir_all(p)?;
                }
                let mut outfile = std::fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }

        std::fs::remove_file(&zip_path)?;
        Ok(())
    })
    .join()
    .map_err(|_| io::Error::other("Thread panicked"))??;

    let extracted_dir = dir.join("dure-wasm-main");
    if fs::metadata(&extracted_dir).await.is_ok() {
        let mut entries = fs::read_dir(&extracted_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let dest = dir.join(entry.file_name());
            fs::rename(entry.path(), dest).await?;
        }
        fs::remove_dir(&extracted_dir).await?;
    }

    eprintln!("✓ Static files ready at {:?}", dir);
    Ok(())
}

pub async fn static_files_exist(dir: &Path) -> bool {
    dir.join("index.html").exists()
}

/// Compression encoding preference
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompressionEncoding {
    Brotli,
    Gzip,
    None,
}

/// Parse Accept-Encoding header to determine preferred compression
pub fn parse_accept_encoding(accept_encoding: Option<&str>) -> CompressionEncoding {
    let accept_encoding = match accept_encoding {
        Some(val) => val,
        None => return CompressionEncoding::None,
    };

    // Check for brotli first (typically better compression)
    if accept_encoding.contains("br") {
        CompressionEncoding::Brotli
    } else if accept_encoding.contains("gzip") {
        CompressionEncoding::Gzip
    } else {
        CompressionEncoding::None
    }
}

/// Serve static file with optional pre-compressed support
///
/// Checks for pre-compressed versions (.br, .gz) based on Accept-Encoding header.
/// Returns (content, content_type, optional_content_encoding)
pub async fn serve_static_file_compressed(
    base_dir: &Path,
    path: &str,
    accept_encoding: Option<&str>,
) -> io::Result<(Vec<u8>, String, Option<&'static str>)> {
    let path = path.trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };
    let file_path = base_dir.join(path);

    let canonical_base = fs::canonicalize(base_dir).await?;
    let canonical_file = fs::canonicalize(&file_path)
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "File not found"))?;

    if !canonical_file.starts_with(&canonical_base) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Access denied",
        ));
    }

    // Determine compression preference
    let compression = parse_accept_encoding(accept_encoding);

    // Try to serve pre-compressed version
    match compression {
        CompressionEncoding::Brotli => {
            // Append .br to the full filename (e.g., app.wasm -> app.wasm.br)
            let mut br_path = file_path.as_os_str().to_os_string();
            br_path.push(".br");
            let br_path = std::path::PathBuf::from(br_path);

            if fs::metadata(&br_path).await.is_ok() {
                let content = fs::read(&br_path).await?;
                let content_type = get_content_type(&file_path);
                return Ok((content, content_type, Some("br")));
            }
            // Fall through to try gzip
        }
        CompressionEncoding::Gzip => {
            // Append .gz to the full filename (e.g., app.wasm -> app.wasm.gz)
            let mut gz_path = file_path.as_os_str().to_os_string();
            gz_path.push(".gz");
            let gz_path = std::path::PathBuf::from(gz_path);

            if fs::metadata(&gz_path).await.is_ok() {
                let content = fs::read(&gz_path).await?;
                let content_type = get_content_type(&file_path);
                return Ok((content, content_type, Some("gzip")));
            }
            // Fall through to uncompressed
        }
        CompressionEncoding::None => {
            // No compression requested, serve uncompressed
        }
    }

    // If brotli was preferred but not available, try gzip
    if compression == CompressionEncoding::Brotli {
        let mut gz_path = file_path.as_os_str().to_os_string();
        gz_path.push(".gz");
        let gz_path = std::path::PathBuf::from(gz_path);

        if fs::metadata(&gz_path).await.is_ok() {
            let content = fs::read(&gz_path).await?;
            let content_type = get_content_type(&file_path);
            return Ok((content, content_type, Some("gzip")));
        }
    }

    // Serve uncompressed
    let content = fs::read(&file_path).await?;
    let content_type = get_content_type(&file_path);
    Ok((content, content_type, None))
}

pub async fn serve_static_file(base_dir: &Path, path: &str) -> io::Result<(Vec<u8>, String)> {
    let (content, content_type, _) = serve_static_file_compressed(base_dir, path, None).await?;
    Ok((content, content_type))
}

pub fn get_content_type(path: &Path) -> String {
    match path.extension().and_then(|s| s.to_str()) {
        Some("html") | Some("htm") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("wasm") => "application/wasm",
        Some("ico") => "image/x-icon",
        _ => "application/octet-stream",
    }
    .to_string()
}

/// Handle an HTTP GET request.
///
/// Routes (checked in order):
/// - `/api-docs/openapi.json`  → serve pre-generated OpenAPI JSON
/// - `/swagger-ui`             → redirect to `/swagger-ui/index.html`
/// - `/swagger-ui/*`           → serve embedded Swagger UI files
/// - `/asyncapi-docs`          → redirect to `/asyncapi-docs/index.html`
/// - `/asyncapi-docs/*`        → serve files from `settings.asyncapi_docs_dir`
/// - `/api/webhook/config`     → get webhook logging config
/// - `/api/webhook/patterns`   → list webhook allow-patterns
/// - `/api/webhook/logs`       → list recent webhook logs
/// - anything else             → serve static files from `settings.static_dir`
pub async fn handle_http_get<S: AsyncWriteExt + Unpin>(
    stream: &mut S,
    request: &HttpRequest,
    settings: &ServerSettings,
) -> io::Result<()> {
    let path = &request.path;

    if path == "/api-docs/openapi.json" {
        let body = settings.openapi_json.as_bytes();
        let response =
            build_http_response(200, "OK", vec![("Content-Type", "application/json")], body);
        stream.write_all(&response).await?;
        stream.flush().await?;
        return Ok(());
    }

    if path == "/swagger-ui" || path == "/swagger-ui/" {
        let response = build_http_response(
            301,
            "Moved Permanently",
            vec![("Location", "/swagger-ui/index.html")],
            b"",
        );
        stream.write_all(&response).await?;
        stream.flush().await?;
        return Ok(());
    }

    if let Some(tail) = path.strip_prefix("/swagger-ui/") {
        let config = Arc::clone(&settings.swagger_config);
        let (status, body, content_type) = match utoipa_swagger_ui::serve(tail, config) {
            Ok(Some(file)) => (200, file.bytes.into_owned(), file.content_type),
            Ok(None) => (404, b"404 Not Found".to_vec(), "text/plain".to_string()),
            Err(e) => (500, e.to_string().into_bytes(), "text/plain".to_string()),
        };
        let response = build_http_response(
            status,
            status_text(status),
            vec![("Content-Type", &content_type)],
            &body,
        );
        stream.write_all(&response).await?;
        stream.flush().await?;
        return Ok(());
    }

    // AsyncAPI docs
    if path == "/asyncapi-docs" || path == "/asyncapi-docs/" {
        let response = build_http_response(
            301,
            "Moved Permanently",
            vec![("Location", "/asyncapi-docs/index.html")],
            b"",
        );
        stream.write_all(&response).await?;
        stream.flush().await?;
        return Ok(());
    }

    if let Some(tail) = path.strip_prefix("/asyncapi-docs/") {
        let tail = if tail.is_empty() { "index.html" } else { tail };
        match super::asyncapi_docs::get(&settings.asyncapi_docs_dir, tail) {
            Some((content, content_type)) => {
                let response =
                    build_http_response(200, "OK", vec![("Content-Type", content_type)], &content);
                stream.write_all(&response).await?;
                stream.flush().await?;
            }
            None => {
                let response = build_http_response(
                    404,
                    "Not Found",
                    vec![("Content-Type", "text/plain")],
                    b"404 Not Found",
                );
                stream.write_all(&response).await?;
                stream.flush().await?;
            }
        }
        return Ok(());
    }

    // Webhook REST API — GET endpoints
    if path == "/api/webhook/config" {
        use crate::storage::models::webhook::get_webhook_config;
        let body = match settings
            .db
            .lock()
            .ok()
            .and_then(|mut db| get_webhook_config(&mut db).ok())
        {
            Some(c) => format!("{{\"logging_enabled\":{}}}", c.logging_enabled),
            None => "{\"logging_enabled\":false}".to_string(),
        };
        let response = build_http_response(
            200,
            "OK",
            vec![("Content-Type", "application/json")],
            body.as_bytes(),
        );
        stream.write_all(&response).await?;
        stream.flush().await?;
        return Ok(());
    }

    if path == "/api/webhook/patterns" {
        use crate::storage::models::webhook::list_webhook_patterns;
        let patterns = settings
            .db
            .lock()
            .ok()
            .and_then(|mut db| list_webhook_patterns(&mut db).ok())
            .unwrap_or_default();
        let items: Vec<String> = patterns
            .iter()
            .map(|p| {
                format!(
                    "{{\"id\":{},\"pattern\":\"{}\",\"created_at\":{}}}",
                    p.id, p.pattern, p.created_at
                )
            })
            .collect();
        let body = format!("[{}]", items.join(","));
        let response = build_http_response(
            200,
            "OK",
            vec![("Content-Type", "application/json")],
            body.as_bytes(),
        );
        stream.write_all(&response).await?;
        stream.flush().await?;
        return Ok(());
    }

    if path == "/api/webhook/logs" || path.starts_with("/api/webhook/logs?") {
        use crate::storage::models::webhook::list_webhook_requests;
        // Parse optional ?limit= query param
        let limit = path
            .find("limit=")
            .and_then(|i| path[i + 6..].split('&').next())
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(100);
        let logs = settings
            .db
            .lock()
            .ok()
            .and_then(|mut db| list_webhook_requests(&mut db, None, limit as usize).ok())
            .unwrap_or_default();
        let items: Vec<String> = logs.iter().map(|l| {
            format!(
                "{{\"id\":{},\"pattern\":\"{}\",\"path\":\"{}\",\"method\":\"{}\",\"remote_addr\":\"{}\",\"received_at\":{}}}",
                l.id, l.pattern, l.path, l.method, l.remote_addr, l.received_at
            )
        }).collect();
        let body = format!("[{}]", items.join(","));
        let response = build_http_response(
            200,
            "OK",
            vec![("Content-Type", "application/json")],
            body.as_bytes(),
        );
        stream.write_all(&response).await?;
        stream.flush().await?;
        return Ok(());
    }

    // Get Accept-Encoding header for compression support
    let accept_encoding = request.headers.get("accept-encoding").map(|s| s.as_str());

    match serve_static_file_compressed(&settings.static_dir, path, accept_encoding).await {
        Ok((content, content_type, content_encoding)) => {
            let mut headers = vec![
                ("Content-Type", content_type.as_str()),
                // Vary header tells caches that response depends on Accept-Encoding
                ("Vary", "Accept-Encoding"),
            ];

            // Add Content-Encoding header if serving compressed content
            let encoding_str;
            if let Some(encoding) = content_encoding {
                encoding_str = encoding.to_string();
                headers.push(("Content-Encoding", &encoding_str));
                eprintln!(
                    "Served {} - {} bytes ({} compressed)",
                    path,
                    content.len(),
                    encoding
                );
            } else {
                eprintln!("Served {} - {} bytes", path, content.len());
            }

            let response = build_http_response(200, "OK", headers, &content);
            stream.write_all(&response).await?;
            stream.flush().await?;
        }
        Err(_) => {
            let body = b"404 Not Found";
            let response =
                build_http_response(404, "Not Found", vec![("Content-Type", "text/plain")], body);
            stream.write_all(&response).await?;
            stream.flush().await?;
        }
    }
    Ok(())
}

fn status_text(code: u16) -> &'static str {
    match code {
        200 => "OK",
        301 => "Moved Permanently",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "Unknown",
    }
}
