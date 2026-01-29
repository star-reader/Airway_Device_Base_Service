use crate::db::Database;
use crate::encryption::{EncryptedData, HybridEncryptor, RsaKeyPair};
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureDevice {
    pub id: String,
    pub encrypted_fingerprint: EncryptedData,
    pub encrypted_hardware_info: Option<EncryptedData>,
    pub encrypted_aes_key: String,
    pub public_key_pem: String,
    pub created_at: i64,
    pub last_seen: i64,
}

pub struct SecureDeviceManager {
    db: Arc<Database>,
    encryptor: HybridEncryptor,
}

impl SecureDeviceManager {
    pub fn new(db: Arc<Database>) -> Result<Self> {
        let encryptor = HybridEncryptor::new()?;
        Ok(Self { db, encryptor })
    }

    /// 使用指定的RSA密钥对创建
    pub fn with_keypair(db: Arc<Database>, key_pair: RsaKeyPair) -> Result<Self> {
        let encryptor = HybridEncryptor::with_rsa_keypair(key_pair)?;
        Ok(Self { db, encryptor })
    }

    pub fn create_secure_device(
        &self,
        id: String,
        fingerprint: &str,
        hardware_info: Option<&str>,
    ) -> Result<SecureDevice> {
        let (encrypted_fingerprint, encrypted_key) = 
            self.encryptor.encrypt_string(fingerprint)?;

        let encrypted_hardware_info = if let Some(hw_info) = hardware_info {
            let (encrypted_hw, _) = self.encryptor.encrypt_string(hw_info)?;
            Some(encrypted_hw)
        } else {
            None
        };

        let public_key_pem = self.encryptor.public_key_pem()?;

        let device = SecureDevice {
            id: id.clone(),
            encrypted_fingerprint,
            encrypted_hardware_info,
            encrypted_aes_key: encrypted_key,
            public_key_pem,
            created_at: chrono::Utc::now().timestamp(),
            last_seen: chrono::Utc::now().timestamp(),
        };

        self.save_to_db(&device)?;

        Ok(device)
    }

    pub fn decrypt_fingerprint(&self, device: &SecureDevice) -> Result<String> {
        self.encryptor.decrypt_string(
            &device.encrypted_fingerprint,
            &device.encrypted_aes_key,
        )
    }

    pub fn decrypt_hardware_info(&self, device: &SecureDevice) -> Result<Option<String>> {
        match &device.encrypted_hardware_info {
            Some(encrypted) => {
                let decrypted = self.encryptor.decrypt_string(
                    encrypted,
                    &device.encrypted_aes_key,
                )?;
                Ok(Some(decrypted))
            }
            None => Ok(None),
        }
    }

    fn save_to_db(&self, device: &SecureDevice) -> Result<()> {
        let conn = self.db.get_conn()?;

        let fingerprint_json = serde_json::to_string(&device.encrypted_fingerprint)?;
        let hardware_info_json = device
            .encrypted_hardware_info
            .as_ref()
            .map(|hw| serde_json::to_string(hw))
            .transpose()?;

        conn.execute(
            "INSERT OR REPLACE INTO secure_devices 
             (id, encrypted_fingerprint, encrypted_hardware_info, encrypted_aes_key, 
              public_key_pem, created_at, last_seen)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                &device.id,
                fingerprint_json,
                hardware_info_json,
                &device.encrypted_aes_key,
                &device.public_key_pem,
                device.created_at,
                device.last_seen,
            ],
        )?;

        Ok(())
    }

    pub fn load_from_db(&self, device_id: &str) -> Result<Option<SecureDevice>> {
        let conn = self.db.get_conn()?;

        let result = conn.query_row(
            "SELECT id, encrypted_fingerprint, encrypted_hardware_info, 
                    encrypted_aes_key, public_key_pem, created_at, last_seen
             FROM secure_devices WHERE id = ?1",
            [device_id],
            |row| {
                let fingerprint_json: String = row.get(1)?;
                let hardware_info_json: Option<String> = row.get(2)?;

                let encrypted_fingerprint: EncryptedData =
                    serde_json::from_str(&fingerprint_json)
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            1,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        ))?;

                let encrypted_hardware_info = hardware_info_json
                    .map(|json| serde_json::from_str(&json))
                    .transpose()
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        2,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    ))?;

                Ok(SecureDevice {
                    id: row.get(0)?,
                    encrypted_fingerprint,
                    encrypted_hardware_info,
                    encrypted_aes_key: row.get(3)?,
                    public_key_pem: row.get(4)?,
                    created_at: row.get(5)?,
                    last_seen: row.get(6)?,
                })
            },
        );

        match result {
            Ok(device) => Ok(Some(device)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn update_last_seen(&self, device_id: &str) -> Result<()> {
        let conn = self.db.get_conn()?;
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "UPDATE secure_devices SET last_seen = ?1 WHERE id = ?2",
            rusqlite::params![now, device_id],
        )?;

        Ok(())
    }

    pub fn list_secure_devices(&self) -> Result<Vec<SecureDevice>> {
        let conn = self.db.get_conn()?;

        let mut stmt = conn.prepare(
            "SELECT id, encrypted_fingerprint, encrypted_hardware_info, 
                    encrypted_aes_key, public_key_pem, created_at, last_seen
             FROM secure_devices ORDER BY last_seen DESC",
        )?;

        let devices = stmt
            .query_map([], |row| {
                let fingerprint_json: String = row.get(1)?;
                let hardware_info_json: Option<String> = row.get(2)?;

                let encrypted_fingerprint: EncryptedData =
                    serde_json::from_str(&fingerprint_json)
                        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                            1,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        ))?;

                let encrypted_hardware_info = hardware_info_json
                    .map(|json| serde_json::from_str(&json))
                    .transpose()
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        2,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    ))?;

                Ok(SecureDevice {
                    id: row.get(0)?,
                    encrypted_fingerprint,
                    encrypted_hardware_info,
                    encrypted_aes_key: row.get(3)?,
                    public_key_pem: row.get(4)?,
                    created_at: row.get(5)?,
                    last_seen: row.get(6)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(devices)
    }

    pub fn delete_device(&self, device_id: &str) -> Result<()> {
        let conn = self.db.get_conn()?;
        conn.execute("DELETE FROM secure_devices WHERE id = ?1", [device_id])?;
        Ok(())
    }

    pub fn export_private_key(&self) -> Result<String> {
        self.encryptor.private_key_pem()
    }

    pub fn export_public_key(&self) -> Result<String> {
        self.encryptor.public_key_pem()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;
    use tempfile::NamedTempFile;

    fn setup_test_db() -> Arc<Database> {
        let temp_file = NamedTempFile::new().unwrap();
        let config = Config {
            db_path: temp_file.path().to_path_buf(),
            enable_wal: false,
            pool_size: 1,
        };

        let db = Arc::new(Database::new(&config).unwrap());
        db.migrate().unwrap();

        // 创建安全设备表
        let conn = db.get_conn().unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS secure_devices (
                id TEXT PRIMARY KEY,
                encrypted_fingerprint TEXT NOT NULL,
                encrypted_hardware_info TEXT,
                encrypted_aes_key TEXT NOT NULL,
                public_key_pem TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                last_seen INTEGER NOT NULL
            )",
            [],
        )
        .unwrap();

        db
    }

    #[test]
    fn test_secure_device_creation() {
        let db = setup_test_db();
        let manager = SecureDeviceManager::new(db).unwrap();

        let device = manager
            .create_secure_device(
                "test-device-1".to_string(),
                "fingerprint123",
                Some("hardware info"),
            )
            .unwrap();

        assert_eq!(device.id, "test-device-1");
        assert!(!device.encrypted_aes_key.is_empty());
        assert!(!device.public_key_pem.is_empty());
    }

    #[test]
    fn test_encrypt_decrypt_fingerprint() {
        let db = setup_test_db();
        let manager = SecureDeviceManager::new(db).unwrap();

        let original_fingerprint = "my-secret-fingerprint";
        let device = manager
            .create_secure_device(
                "test-device-2".to_string(),
                original_fingerprint,
                None,
            )
            .unwrap();

        let decrypted = manager.decrypt_fingerprint(&device).unwrap();
        assert_eq!(decrypted, original_fingerprint);
    }

    #[test]
    fn test_load_from_db() {
        let db = setup_test_db();
        let manager = SecureDeviceManager::new(db.clone()).unwrap();

        let device = manager
            .create_secure_device(
                "test-device-3".to_string(),
                "fingerprint456",
                Some("hw info"),
            )
            .unwrap();

        let loaded = manager.load_from_db("test-device-3").unwrap().unwrap();
        assert_eq!(loaded.id, device.id);

        let decrypted_fp = manager.decrypt_fingerprint(&loaded).unwrap();
        assert_eq!(decrypted_fp, "fingerprint456");
    }

    #[test]
    fn test_list_devices() {
        let db = setup_test_db();
        let manager = SecureDeviceManager::new(db).unwrap();

        manager
            .create_secure_device("device-1".to_string(), "fp1", None)
            .unwrap();
        manager
            .create_secure_device("device-2".to_string(), "fp2", None)
            .unwrap();

        let devices = manager.list_secure_devices().unwrap();
        assert_eq!(devices.len(), 2);
    }
}
