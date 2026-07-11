use async_compat::Compat;
use std::path::PathBuf;
use std::sync::Arc;

use crate::error::{ServiceError, Result};
use crate::protocol::{Chat, ChatEvent, Message};

/// Bridge between smol and deltachat-core (tokio)
pub struct DeltachatBridge {
    _db_path: PathBuf,
}

impl DeltachatBridge {
    /// Initialize deltachat context
    pub async fn init(db_path: PathBuf) -> Result<Self> {
        // Verify database path directory exists
        if let Some(parent) = db_path.parent() {
            if !parent.exists() {
                return Err(ServiceError::Database(
                    format!("Directory does not exist: {}", parent.display())
                ));
            }
        }

        Ok(Self { _db_path: db_path })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[smol_potat::test]
    async fn test_bridge_init_success() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("chat.db");

        let bridge = DeltachatBridge::init(db_path).await;
        assert!(bridge.is_ok());
    }

    #[smol_potat::test]
    async fn test_bridge_init_invalid_path() {
        let db_path = PathBuf::from("/nonexistent/directory/chat.db");

        let result = DeltachatBridge::init(db_path).await;
        assert!(matches!(result, Err(ServiceError::Database(_))));
    }
}
