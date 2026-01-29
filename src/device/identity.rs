use serde::{Deserialize, Serialize};

/// Device identity information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub device_id: String,
    pub fingerprint: String,
    pub is_authorized: bool,
}

impl Identity {
    /// Create a new identity
    pub fn new(device_id: String, fingerprint: String) -> Self {
        Self {
            device_id,
            fingerprint,
            is_authorized: true,
        }
    }

    /// Check if identity is valid
    pub fn is_valid(&self) -> bool {
        !self.device_id.is_empty() && !self.fingerprint.is_empty() && self.is_authorized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_creation() {
        let identity = Identity::new(
            "device-123".to_string(),
            "fingerprint-456".to_string(),
        );
        
        assert!(identity.is_valid());
        assert_eq!(identity.device_id, "device-123");
    }

    #[test]
    fn test_invalid_identity() {
        let mut identity = Identity::new(
            "device-123".to_string(),
            "fingerprint-456".to_string(),
        );
        
        identity.is_authorized = false;
        assert!(!identity.is_valid());
    }
}
