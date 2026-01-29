/// Sync client for remote data synchronization
pub struct SyncClient {
    endpoint: String,
}

impl SyncClient {
    /// Create a new sync client
    pub fn new(endpoint: String) -> Self {
        Self { endpoint }
    }

    /// Get endpoint URL
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_client_creation() {
        let client = SyncClient::new("https://api.example.com".to_string());
        assert_eq!(client.endpoint(), "https://api.example.com");
    }
}
