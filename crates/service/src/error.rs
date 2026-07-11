use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Deltachat error: {0}")]
    Deltachat(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Runtime compatibility error: {0}")]
    Compat(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, ServiceError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ServiceError::Deltachat("connection failed".into());
        assert_eq!(err.to_string(), "Deltachat error: connection failed");
    }
}
