/// Sync strategy
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SyncStrategy {
    /// Full synchronization - download all data
    Full,
    /// Incremental synchronization - only changes since last sync
    Incremental,
    /// On-demand synchronization - sync specific data only
    OnDemand,
}

impl SyncStrategy {
    /// Get strategy name
    pub fn as_str(&self) -> &str {
        match self {
            SyncStrategy::Full => "full",
            SyncStrategy::Incremental => "incremental",
            SyncStrategy::OnDemand => "on-demand",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_strategy() {
        assert_eq!(SyncStrategy::Full.as_str(), "full");
        assert_eq!(SyncStrategy::Incremental.as_str(), "incremental");
        assert_eq!(SyncStrategy::OnDemand.as_str(), "on-demand");
    }
}
