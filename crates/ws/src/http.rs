use crate::error::Result;
use crate::static_files::StaticFileHandler;

/// HTTP response
pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    pub fn ok(body: Vec<u8>, content_type: &str) -> Self {
        Self {
            status: 200,
            headers: vec![("Content-Type".into(), content_type.into())],
            body,
        }
    }

    pub fn not_found() -> Self {
        Self {
            status: 404,
            headers: vec![("Content-Type".into(), "text/plain".into())],
            body: b"Not Found".to_vec(),
        }
    }

    pub fn internal_error() -> Self {
        Self {
            status: 500,
            headers: vec![("Content-Type".into(), "text/plain".into())],
            body: b"Internal Server Error".to_vec(),
        }
    }
}

/// Handle HTTP request
pub async fn handle_http_request(
    path: &str,
    static_handler: &StaticFileHandler,
) -> Result<HttpResponse> {
    match path {
        "/health" => Ok(HttpResponse::ok(
            b"{\"status\":\"ok\"}".to_vec(),
            "application/json",
        )),
        _ => {
            // Try to serve static file
            match static_handler.serve(path).await {
                Ok(content) => {
                    let mime_type = static_handler.mime_type(path);
                    Ok(HttpResponse::ok(content, mime_type))
                }
                Err(_) => Ok(HttpResponse::not_found()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_http_response_creation() {
        let resp = HttpResponse::ok(b"test".to_vec(), "text/plain");
        assert_eq!(resp.status, 200);
        assert_eq!(resp.body, b"test");
    }

    #[smol_potat::test]
    async fn test_health_endpoint() {
        let handler = StaticFileHandler::new(PathBuf::from("."));
        let resp = handle_http_request("/health", &handler).await.unwrap();

        assert_eq!(resp.status, 200);
        assert_eq!(resp.body, b"{\"status\":\"ok\"}");
    }
}
