/// Database schema definitions
pub const SCHEMA_VERSION: i32 = 1;

/// Get all table creation SQL statements
pub fn get_schema_sql() -> Vec<&'static str> {
    vec![
        // Schema version table
        r#"
        CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY,
            applied_at INTEGER NOT NULL
        )
        "#,
        
        // Devices table
        r#"
        CREATE TABLE IF NOT EXISTS devices (
            id TEXT PRIMARY KEY,
            fingerprint TEXT UNIQUE NOT NULL,
            hardware_info TEXT,
            created_at INTEGER NOT NULL,
            last_seen INTEGER NOT NULL
        )
        "#,
        
        // Secure devices table (encrypted)
        r#"
        CREATE TABLE IF NOT EXISTS secure_devices (
            id TEXT PRIMARY KEY,
            encrypted_fingerprint TEXT NOT NULL,
            encrypted_hardware_info TEXT NOT NULL,
            encrypted_aes_key TEXT NOT NULL,
            public_key_pem TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            last_seen INTEGER NOT NULL
        )
        "#,
        
        // Waypoints table
        r#"
        CREATE TABLE IF NOT EXISTS waypoints (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            latitude REAL NOT NULL,
            longitude REAL NOT NULL,
            region TEXT,
            type TEXT,
            created_at INTEGER NOT NULL
        )
        "#,
        
        // Waypoints spatial index
        r#"
        CREATE INDEX IF NOT EXISTS idx_waypoints_location 
        ON waypoints(latitude, longitude)
        "#,
        
        // Airports table
        r#"
        CREATE TABLE IF NOT EXISTS airports (
            id TEXT PRIMARY KEY,
            icao TEXT UNIQUE NOT NULL,
            iata TEXT,
            name TEXT NOT NULL,
            latitude REAL NOT NULL,
            longitude REAL NOT NULL,
            elevation INTEGER,
            country TEXT,
            region TEXT,
            created_at INTEGER NOT NULL
        )
        "#,
        
        // Airports spatial index
        r#"
        CREATE INDEX IF NOT EXISTS idx_airports_location 
        ON airports(latitude, longitude)
        "#,
        
        // Airways table
        r#"
        CREATE TABLE IF NOT EXISTS airways (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            type TEXT NOT NULL,
            min_altitude INTEGER,
            max_altitude INTEGER,
            created_at INTEGER NOT NULL
        )
        "#,
        
        // Airway segments table
        r#"
        CREATE TABLE IF NOT EXISTS airway_segments (
            id TEXT PRIMARY KEY,
            airway_id TEXT NOT NULL,
            from_waypoint_id TEXT NOT NULL,
            to_waypoint_id TEXT NOT NULL,
            sequence INTEGER NOT NULL,
            distance REAL,
            created_at INTEGER NOT NULL,
            FOREIGN KEY (airway_id) REFERENCES airways(id) ON DELETE CASCADE,
            FOREIGN KEY (from_waypoint_id) REFERENCES waypoints(id),
            FOREIGN KEY (to_waypoint_id) REFERENCES waypoints(id)
        )
        "#,
        
        // Airway segments indexes
        r#"
        CREATE INDEX IF NOT EXISTS idx_airway_segments_airway 
        ON airway_segments(airway_id)
        "#,
        
        // Navaids table (VOR, NDB, etc.)
        r#"
        CREATE TABLE IF NOT EXISTS navaids (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            type TEXT NOT NULL,
            latitude REAL NOT NULL,
            longitude REAL NOT NULL,
            frequency REAL,
            range_nm INTEGER,
            elevation INTEGER,
            region TEXT,
            created_at INTEGER NOT NULL
        )
        "#,
        
        // Navaids spatial index
        r#"
        CREATE INDEX IF NOT EXISTS idx_navaids_location 
        ON navaids(latitude, longitude)
        "#,
        
        // Airspaces table
        r#"
        CREATE TABLE IF NOT EXISTS airspaces (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            type TEXT NOT NULL,
            class TEXT,
            lower_limit INTEGER,
            upper_limit INTEGER,
            created_at INTEGER NOT NULL
        )
        "#,
        
        // Airspace boundaries (polygon vertices)
        r#"
        CREATE TABLE IF NOT EXISTS airspace_boundaries (
            id TEXT PRIMARY KEY,
            airspace_id TEXT NOT NULL,
            latitude REAL NOT NULL,
            longitude REAL NOT NULL,
            sequence INTEGER NOT NULL,
            FOREIGN KEY (airspace_id) REFERENCES airspaces(id) ON DELETE CASCADE
        )
        "#,
        
        // Sync metadata table
        r#"
        CREATE TABLE IF NOT EXISTS sync_metadata (
            table_name TEXT PRIMARY KEY,
            last_sync INTEGER NOT NULL,
            record_count INTEGER DEFAULT 0
        )
        "#,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_sql_count() {
        let statements = get_schema_sql();
        assert!(!statements.is_empty());
        assert_eq!(statements.len(), 18); // Update if schema changes
    }
}
