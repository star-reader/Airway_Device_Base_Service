pub mod fingerprint;
pub mod identity;
pub mod secure;

use crate::db::Database;
use crate::error::Result;
use rusqlite::OptionalExtension;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub fingerprint: String,
    pub hardware_info: Option<String>,
    pub created_at: i64,
    pub last_seen: i64,
}

pub struct DeviceManager {
    db: Arc<Database>,
}

impl DeviceManager {
    pub fn new(db: Arc<Database>) -> Result<Self> {
        Ok(Self { db })
    }

    pub fn get_or_create_fingerprint(&self) -> Result<Device> {
        let fingerprint = fingerprint::generate_fingerprint()?;
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
            device.last_seen = Utc::now().timestamp();
            conn.execute(
                "UPDATE devices SET last_seen = ?1 WHERE id = ?2",
                rusqlite::params![device.last_seen, &device.id],
            )?;
            
            log::info!("Found existing device: {}", device.id);
            Ok(device)
        } else {
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
        
        let device1 = manager.get_or_create_fingerprint().unwrap();
        assert!(!device1.id.is_empty());

        let device2 = manager.get_or_create_fingerprint().unwrap();
        assert_eq!(device1.id, device2.id);
        
        let devices = manager.list_devices().unwrap();
        assert_eq!(devices.len(), 1);
    }
}
