use rusqlite::Connection;
use crate::db::schema::{get_schema_sql, SCHEMA_VERSION};
use crate::error::Result;

/// 运行所有数据库迁移
pub fn run_migrations(conn: &Connection) -> Result<()> {
    let current_version = get_current_version(conn)?;
    
    log::info!("当前数据库版本: {}", current_version);
    
    if current_version < SCHEMA_VERSION {
        log::info!("正在运行迁移，从版本 {} 到 {}", current_version, SCHEMA_VERSION);
        apply_migrations(conn)?;
    } else {
        log::info!("数据库模式已是最新");
    }
    
    Ok(())
}

/// 获取当前数据库版本
fn get_current_version(conn: &Connection) -> Result<i32> {
    // 检查 schema_version 表是否存在
    let table_exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='schema_version')",
        [],
        |row| row.get(0),
    )?;
    
    if !table_exists {
        return Ok(0);
    }
    
    // Get the highest version number
    let version: Result<i32> = conn.query_row(
        "SELECT MAX(version) FROM schema_version",
        [],
        |row| row.get(0),
    ).map_err(Into::into);
    
    match version {
        Ok(v) => Ok(v),
        Err(_) => Ok(0),
    }
}

/// 应用所有迁移
fn apply_migrations(conn: &Connection) -> Result<()> {
    let tx = conn.unchecked_transaction()?;
    
    // 执行所有数据库模式 SQL 语句
    for sql in get_schema_sql() {
        tx.execute_batch(sql)?;
    }
    
    // 记录迁移
    let now = chrono::Utc::now().timestamp();
    tx.execute(
        "INSERT OR REPLACE INTO schema_version (version, applied_at) VALUES (?1, ?2)",
        rusqlite::params![SCHEMA_VERSION, now],
    )?;
    
    tx.commit()?;
    
    log::info!("迁移应用成功");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_migrations() {
        let conn = Connection::open_in_memory().unwrap();
        
        let version_before = get_current_version(&conn).unwrap();
        assert_eq!(version_before, 0);
        
        run_migrations(&conn).unwrap();
        
        let version_after = get_current_version(&conn).unwrap();
        assert_eq!(version_after, SCHEMA_VERSION);
    }

    #[test]
    fn test_migrations_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        
        // Run migrations twice
        run_migrations(&conn).unwrap();
        run_migrations(&conn).unwrap();
        
        // Should still be at the same version
        let version = get_current_version(&conn).unwrap();
        assert_eq!(version, SCHEMA_VERSION);
    }
}
