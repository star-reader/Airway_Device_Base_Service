pub mod db;
pub mod device;
pub mod error;
pub mod flight;
pub mod models;
pub mod spatial;
pub mod sync;

#[cfg(feature = "ffi")]
pub mod ffi;

use error::Result;
use std::path::PathBuf;
use std::sync::Arc;

/// AeroBase 配置
#[derive(Debug, Clone)]
pub struct Config {
    /// 数据库文件路径
    pub db_path: PathBuf,
    /// 启用 WAL 模式以提高并发性能
    pub enable_wal: bool,
    /// 连接池大小
    pub pool_size: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            db_path: PathBuf::from("aerobase.db"),
            enable_wal: true,
            pool_size: 4,
        }
    }
}

/// AeroBase 服务主入口
pub struct AeroBase {
    db: Arc<db::Database>,
    device_manager: Arc<device::DeviceManager>,
    spatial_engine: Arc<spatial::SpatialEngine>,
    flight_planner: Arc<flight::FlightPlanner>,
}

impl AeroBase {
    /// 创建新的 AeroBase 实例
    pub async fn new(config: Config) -> Result<Self> {
        log::info!("正在初始化 AeroBase，配置: {:?}", config);

        // 初始化数据库
        let db = Arc::new(db::Database::new(&config)?);
        
        // 运行数据库迁移
        db.migrate()?;

        // 初始化各个组件
        let device_manager = Arc::new(device::DeviceManager::new(Arc::clone(&db))?);
        let spatial_engine = Arc::new(spatial::SpatialEngine::new(Arc::clone(&db))?);
        let flight_planner = Arc::new(flight::FlightPlanner::new(Arc::clone(&db))?);

        log::info!("AeroBase 初始化成功");

        Ok(Self {
            db,
            device_manager,
            spatial_engine,
            flight_planner,
        })
    }

    /// 获取设备管理器
    pub fn device(&self) -> &device::DeviceManager {
        &self.device_manager
    }

    /// 获取空间查询引擎
    pub fn spatial(&self) -> &spatial::SpatialEngine {
        &self.spatial_engine
    }

    /// 获取飞行计划器
    pub fn flight(&self) -> &flight::FlightPlanner {
        &self.flight_planner
    }

    /// 获取数据库句柄
    pub fn db(&self) -> &db::Database {
        &self.db
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_aerobase_init() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        
        let config = Config {
            db_path,
            enable_wal: true,
            pool_size: 2,
        };

        let aerobase = AeroBase::new(config).await;
        assert!(aerobase.is_ok());
    }
}
