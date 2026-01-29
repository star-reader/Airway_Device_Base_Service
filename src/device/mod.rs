pub mod fingerprint;
pub mod identity;

use crate::db::Database;
use crate::error::{AeroBaseError, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// 设备信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub fingerprint: String,
    pub hardware_info: Option<String>,
    pub created_at: i64,
    pub last_seen: i64,
}

/// 设备管理器，用于处理设备指纹和身份识别
pub struct DeviceManager {
    db: Arc<Database>,
}

impl DeviceManager {
    /// 创建新的设备管理器
    pub fn new(db: Arc<Database>) -> Result<Self> {
        Ok(Self { db })
    }

    /// 获取或创建设备指纹
    pub fn get_or_create_fingerprint(&self) -> Result<Device> {
        // Generate fingerprint
        let fingerprint = fingerprint::generate_fingerprint()?;
        
        // Try to find existing device with this fingerprint
        let conn = self.db.get_conn()?;
        
        let existing: Option<Device> = conn
            .query_row(
                "SELECT id, fingerprint, hardware_info, created_at, last_seen 
                 FROM devices WHERE fingerprint = ?1",
                [&fingerprint],
                |row| {
                    Ok(Device {
                        id: row.get(0)?,
                        fingerprint: row.get(1)?,
                        hardware_info: row.get(2)?,
                        created_at: row.get(3)?,
                        last_seen: row.get(4)?,
                    })
                },
            )
            .optional()?;
        
        if let Some(mut device) = existing {
            // Update last_seen
            device.last_seen = Utc::now().timestamp();
            conn.execute(
                "UPDATE devices SET last_seen = ?1 WHERE id = ?2",
                rusqlite::params![device.last_seen, &device.id],
            )?;
            
            log::info!("Found existing device: {}", device.id);
            Ok(device)
        } else {
            // Create new device
            let device = Device {
                id: Uuid::new_v4().to_string(),
                fingerprint: fingerprint.clone(),
                hardware_info: Some(fingerprint::get_hardware_info()?),
                created_at: Utc::now().timestamp(),
                last_seen: Utc::now().timestamp(),
            };
            
            conn.execute(
                "INSERT INTO devices (id, fingerprint, hardware_info, created_at, last_seen) 
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![
                    &device.id,
                    &device.fingerprint,
                    &device.hardware_info,
                    device.created_at,
                    device.last_seen,
                ],
            )?;
            
            log::info!("Created new device: {}", device.id);
            Ok(device)
        }
    }

    /// 根据 ID 获取设备
    pub fn get_device(&self, id: &str) -> Result<Option<Device>> {
        let conn = self.db.get_conn()?;
        
        let device = conn
            .query_row(
                "SELECT id, fingerprint, hardware_info, created_at, last_seen 
                 FROM devices WHERE id = ?1",
                [id],
                |row| {
                    Ok(Device {
                        id: row.get(0)?,
                        fingerprint: row.get(1)?,
                        hardware_info: row.get(2)?,
                        created_at: row.get(3)?,
                        last_seen: row.get(4)?,
                    })
                },
            )
            .optional()?;
        
        Ok(device)
    }

    /// 列出所有设备
    pub fn list_devices(&self) -> Result<Vec<Device>> {
        let conn = self.db.get_conn()?;
        
        let mut stmt = conn.prepare(
            "SELECT id, fingerprint, hardware_info, created_at, last_seen 
             FROM devices ORDER BY last_seen DESC",
        )?;
        
        let devices = stmt
            .query_map([], |row| {
                Ok(Device {
                    id: row.get(0)?,
                    fingerprint: row.get(1)?,
                    hardware_info: row.get(2)?,
                    created_at: row.get(3)?,
                    last_seen: row.get(4)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        
        Ok(devices)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;
    use tempfile::NamedTempFile;

    #[test]
    fn test_device_manager() {
        let temp_file = NamedTempFile::new().unwrap();
        let config = Config {
            db_path: temp_file.path().to_path_buf(),
            enable_wal: false,
            pool_size: 1,
        };

        let db = Arc::new(Database::new(&config).unwrap());
        db.migrate().unwrap();

        let manager = DeviceManager::new(db).unwrap();
        
        // Get or create fingerprint
        let device1 = manager.get_or_create_fingerprint().unwrap();
        assert!(!device1.id.is_empty());
        
        // Should return same device on second call
        let device2 = manager.get_or_create_fingerprint().unwrap();
        assert_eq!(device1.id, device2.id);
        
        // List devices
        let devices = manager.list_devices().unwrap();
        assert_eq!(devices.len(), 1);
    }
}
