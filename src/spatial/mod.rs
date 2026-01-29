pub mod geometry;
pub mod index;
pub mod query;

use crate::db::Database;
use crate::error::Result;
use crate::models::{airport::Airport, waypoint::Waypoint, Coordinate};
use std::sync::Arc;

/// 空间查询引擎
pub struct SpatialEngine {
    db: Arc<Database>,
}

impl SpatialEngine {
    /// 创建新的空间查询引擎
    pub fn new(db: Arc<Database>) -> Result<Self> {
        Ok(Self { db })
    }

    /// 查找半径范围内的航路点
    pub fn find_waypoints_within(
        &self,
        center: Coordinate,
        radius_nm: f64,
    ) -> Result<Vec<Waypoint>> {
        query::find_waypoints_within(&self.db, center, radius_nm)
    }

    /// 查找半径范围内的机场
    pub fn find_airports_within(
        &self,
        center: Coordinate,
        radius_nm: f64,
    ) -> Result<Vec<Airport>> {
        query::find_airports_within(&self.db, center, radius_nm)
    }

    /// 查找最近的航路点
    pub fn find_nearest_waypoint(&self, coord: Coordinate) -> Result<Option<Waypoint>> {
        query::find_nearest_waypoint(&self.db, coord)
    }

    /// 查找最近的机场
    pub fn find_nearest_airport(&self, coord: Coordinate) -> Result<Option<Airport>> {
        query::find_nearest_airport(&self.db, coord)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;
    use tempfile::NamedTempFile;

    #[test]
    fn test_spatial_engine_creation() {
        let temp_file = NamedTempFile::new().unwrap();
        let config = Config {
            db_path: temp_file.path().to_path_buf(),
            enable_wal: false,
            pool_size: 1,
        };

        let db = Arc::new(Database::new(&config).unwrap());
        db.migrate().unwrap();

        let engine = SpatialEngine::new(db);
        assert!(engine.is_ok());
    }
}
