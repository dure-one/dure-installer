use thiserror::Error;

#[derive(Debug, Error)]
pub enum WsError {
    #[error("TLS error: {0}")]
    Tls(String),

    #[error("WebSocket protocol error: {0}")]
    WebSocket(String),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, WsError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = WsError::Tls("cert expired".into());
        assert_eq!(err.to_string(), "TLS error: cert expired");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let ws_err: WsError = io_err.into();
        assert!(matches!(ws_err, WsError::Io(_)));
    }
}
