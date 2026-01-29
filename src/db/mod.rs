pub mod connection;
pub mod migrations;
pub mod schema;

use crate::error::{AeroBaseError, Result};
use crate::Config;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;
use std::path::Path;

pub type DbPool = Pool<SqliteConnectionManager>;

/// 数据库管理器
pub struct Database {
    pool: DbPool,
}

impl Database {
    /// 创建新的数据库实例
    pub fn new(config: &Config) -> Result<Self> {
        let manager = SqliteConnectionManager::file(&config.db_path)
            .with_init(|conn| {
                if config.enable_wal {
                    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
                }
                conn.execute_batch(
                    "PRAGMA foreign_keys=ON;
                     PRAGMA synchronous=NORMAL;
                     PRAGMA cache_size=-64000;
                     PRAGMA temp_store=MEMORY;",
                )?;
                Ok(())
            });

        let pool = Pool::builder()
            .max_size(config.pool_size)
            .build(manager)?;

        Ok(Self { pool })
    }

    /// 从连接池获取数据库连接
    pub fn get_conn(&self) -> Result<r2d2::PooledConnection<SqliteConnectionManager>> {
        self.pool.get().map_err(|e| AeroBaseError::Pool(e.to_string()))
    }

    /// 运行数据库迁移
    pub fn migrate(&self) -> Result<()> {
        let conn = self.get_conn()?;
        migrations::run_migrations(&conn)?;
        Ok(())
    }

    /// 检查数据库是否存在且有效
    pub fn exists(path: &Path) -> bool {
        if !path.exists() {
            return false;
        }

        // 尝试打开并验证是否为有效的 SQLite 数据库
        Connection::open(path).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_database_creation() {
        let temp_file = NamedTempFile::new().unwrap();
        let config = Config {
            db_path: temp_file.path().to_path_buf(),
            enable_wal: true,
            pool_size: 2,
        };

        let db = Database::new(&config).unwrap();
        assert!(db.get_conn().is_ok());
    }

    #[test]
    fn test_database_exists() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        let config = Config {
            db_path: path.to_path_buf(),
            enable_wal: false,
            pool_size: 1,
        };

        let _db = Database::new(&config).unwrap();
        assert!(Database::exists(path));
    }
}
