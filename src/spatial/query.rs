use crate::db::Database;
use crate::error::Result;
use crate::models::{
    airport::Airport, waypoint::{Waypoint, WaypointType}, Coordinate,
};
use crate::spatial::geometry;

/// Find waypoints within a radius
pub fn find_waypoints_within(
    db: &Database,
    center: Coordinate,
    radius_nm: f64,
) -> Result<Vec<Waypoint>> {
    let conn = db.get_conn()?;
    
    // Get bounding box for initial filtering
    let (min, max) = geometry::bounding_box(center, radius_nm);
    
    let mut stmt = conn.prepare(
        "SELECT id, name, latitude, longitude, region, type, created_at
         FROM waypoints
         WHERE latitude BETWEEN ?1 AND ?2
           AND longitude BETWEEN ?3 AND ?4",
    )?;
    
    let waypoints: Vec<Waypoint> = stmt
        .query_map(
            rusqlite::params![min.latitude, max.latitude, min.longitude, max.longitude],
            |row| {
                Ok(Waypoint {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    coordinate: Coordinate::new(row.get(2)?, row.get(3)?),
                    region: row.get(4)?,
                    waypoint_type: WaypointType::from_str(&row.get::<_, String>(5)?),
                    created_at: row.get(6)?,
                })
            },
        )?
        .filter_map(|wp| wp.ok())
        .filter(|wp| wp.distance_from(center) <= radius_nm)
        .collect();
    
    Ok(waypoints)
}

/// Find airports within a radius
pub fn find_airports_within(
    db: &Database,
    center: Coordinate,
    radius_nm: f64,
) -> Result<Vec<Airport>> {
    let conn = db.get_conn()?;
    
    let (min, max) = geometry::bounding_box(center, radius_nm);
    
    let mut stmt = conn.prepare(
        "SELECT id, icao, iata, name, latitude, longitude, elevation, country, region, created_at
         FROM airports
         WHERE latitude BETWEEN ?1 AND ?2
           AND longitude BETWEEN ?3 AND ?4",
    )?;
    
    let airports: Vec<Airport> = stmt
        .query_map(
            rusqlite::params![min.latitude, max.latitude, min.longitude, max.longitude],
            |row| {
                Ok(Airport {
                    id: row.get(0)?,
                    icao: row.get(1)?,
                    iata: row.get(2)?,
                    name: row.get(3)?,
                    coordinate: Coordinate::new(row.get(4)?, row.get(5)?),
                    elevation: row.get(6)?,
                    country: row.get(7)?,
                    region: row.get(8)?,
                    created_at: row.get(9)?,
                })
            },
        )?
        .filter_map(|ap| ap.ok())
        .filter(|ap| ap.distance_from(center) <= radius_nm)
        .collect();
    
    Ok(airports)
}

/// Find nearest waypoint
pub fn find_nearest_waypoint(db: &Database, coord: Coordinate) -> Result<Option<Waypoint>> {
    // Search within 500 nm and find the closest
    let waypoints = find_waypoints_within(db, coord, 500.0)?;
    
    let nearest = waypoints
        .into_iter()
        .min_by(|a, b| {
            let dist_a = a.distance_from(coord);
            let dist_b = b.distance_from(coord);
            dist_a.partial_cmp(&dist_b).unwrap()
        });
    
    Ok(nearest)
}

/// Find nearest airport
pub fn find_nearest_airport(db: &Database, coord: Coordinate) -> Result<Option<Airport>> {
    let airports = find_airports_within(db, coord, 500.0)?;
    
    let nearest = airports
        .into_iter()
        .min_by(|a, b| {
            let dist_a = a.distance_from(coord);
            let dist_b = b.distance_from(coord);
            dist_a.partial_cmp(&dist_b).unwrap()
        });
    
    Ok(nearest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;
    use std::sync::Arc;
    use tempfile::NamedTempFile;

    #[test]
    fn test_find_waypoints_within() {
        let temp_file = NamedTempFile::new().unwrap();
        let config = Config {
            db_path: temp_file.path().to_path_buf(),
            enable_wal: false,
            pool_size: 1,
        };

        let db = Arc::new(Database::new(&config).unwrap());
        db.migrate().unwrap();

        // Insert test waypoint
        let conn = db.get_conn().unwrap();
        conn.execute(
            "INSERT INTO waypoints (id, name, latitude, longitude, type, created_at)
             VALUES ('WP1', 'TEST', 39.9042, 116.4074, 'FIX', 0)",
            [],
        )
        .unwrap();

        let center = Coordinate::new(39.9042, 116.4074);
        let result = find_waypoints_within(&db, center, 50.0).unwrap();
        assert_eq!(result.len(), 1);
    }
}
