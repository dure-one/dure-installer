use std::fs;
use std::path::{Path, PathBuf};
use crate::error::{WsError, Result};

/// Static file handler
pub struct StaticFileHandler {
    static_dir: PathBuf,
}

impl StaticFileHandler {
    /// Create new static file handler
    pub fn new(static_dir: PathBuf) -> Self {
        Self { static_dir }
    }

    /// Serve a file by path
    pub async fn serve(&self, path: &str) -> Result<Vec<u8>> {
        // Security: prevent directory traversal
        let safe_path = path.trim_start_matches('/');
        if safe_path.contains("..") {
            return Err(WsError::Http("Invalid path".into()));
        }

        let file_path = self.static_dir.join(safe_path);

        // Default to index.html for directories
        let file_path = if file_path.is_dir() {
            file_path.join("index.html")
        } else {
            file_path
        };

        smol::unblock(move || {
            fs::read(&file_path)
                .map_err(|e| WsError::NotFound(format!("{}: {}", file_path.display(), e)))
        }).await
    }

    /// Get MIME type for file extension
    pub fn mime_type(&self, path: &str) -> &'static str {
        match Path::new(path).extension().and_then(|s| s.to_str()) {
            Some("html") => "text/html",
            Some("css") => "text/css",
            Some("js") => "application/javascript",
            Some("json") => "application/json",
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("svg") => "image/svg+xml",
            Some("wasm") => "application/wasm",
            _ => "application/octet-stream",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[smol_potat::test]
    async fn test_serve_existing_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, b"Hello, World!").unwrap();

        let handler = StaticFileHandler::new(temp_dir.path().to_path_buf());
        let content = handler.serve("test.txt").await.unwrap();

        assert_eq!(content, b"Hello, World!");
    }

    #[smol_potat::test]
    async fn test_serve_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let handler = StaticFileHandler::new(temp_dir.path().to_path_buf());

        let result = handler.serve("nonexistent.txt").await;
        assert!(matches!(result, Err(WsError::NotFound(_))));
    }

    #[smol_potat::test]
    async fn test_directory_traversal_protection() {
        let temp_dir = TempDir::new().unwrap();
        let handler = StaticFileHandler::new(temp_dir.path().to_path_buf());

        let result = handler.serve("../../../etc/passwd").await;
        assert!(matches!(result, Err(WsError::Http(_))));
    }

    #[test]
    fn test_mime_types() {
        let handler = StaticFileHandler::new(PathBuf::from("."));

        assert_eq!(handler.mime_type("test.html"), "text/html");
        assert_eq!(handler.mime_type("style.css"), "text/css");
        assert_eq!(handler.mime_type("app.js"), "application/javascript");
        assert_eq!(handler.mime_type("data.json"), "application/json");
        assert_eq!(handler.mime_type("image.png"), "image/png");
        assert_eq!(handler.mime_type("unknown.xyz"), "application/octet-stream");
    }
}
