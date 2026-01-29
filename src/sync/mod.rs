pub mod client;
pub mod strategy;

use crate::error::Result;

/// Sync status
#[derive(Debug, Clone)]
pub enum SyncStatus {
    Idle,
    InProgress,
    Completed,
    Failed(String),
}

/// Sync manager (placeholder for future implementation)
pub struct SyncManager;

impl SyncManager {
    /// Create a new sync manager
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    /// Get sync status
    pub fn status(&self) -> SyncStatus {
        SyncStatus::Idle
    }
}

impl Default for SyncManager {
    fn default() -> Self {
        Self
    }
}
