use rusqlite::Connection;

/// Database connection utilities
pub struct ConnectionManager;

impl ConnectionManager {
    /// Execute a query and return the number of affected rows
    pub fn execute(conn: &Connection, sql: &str) -> rusqlite::Result<usize> {
        conn.execute(sql, [])
    }

    /// Execute a batch of SQL statements
    pub fn execute_batch(conn: &Connection, sql: &str) -> rusqlite::Result<()> {
        conn.execute_batch(sql)
    }

    /// Begin a transaction
    pub fn begin_transaction(conn: &Connection) -> rusqlite::Result<()> {
        conn.execute("BEGIN TRANSACTION", []).map(|_| ())
    }

    /// Commit a transaction
    pub fn commit(conn: &Connection) -> rusqlite::Result<()> {
        conn.execute("COMMIT", []).map(|_| ())
    }

    /// Rollback a transaction
    pub fn rollback(conn: &Connection) -> rusqlite::Result<()> {
        conn.execute("ROLLBACK", []).map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_transaction() {
        let conn = Connection::open_in_memory().unwrap();
        
        conn.execute("CREATE TABLE test (id INTEGER PRIMARY KEY)", []).unwrap();
        
        ConnectionManager::begin_transaction(&conn).unwrap();
        conn.execute("INSERT INTO test (id) VALUES (1)", []).unwrap();
        ConnectionManager::commit(&conn).unwrap();

        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM test", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }
}
