use crate::db::Database;
use crate::error::{AeroBaseError, Result};
use crate::flight::FlightPlan;

/// Validate a flight plan
pub fn validate_plan(db: &Database, plan: &FlightPlan) -> Result<bool> {
    let conn = db.get_conn()?;

    // Validate departure airport exists
    let dep_exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM airports WHERE icao = ?1)",
            [&plan.departure],
            |row| row.get(0),
        )?;

    if !dep_exists {
        return Err(AeroBaseError::InvalidInput(format!(
            "Departure airport {} not found",
            plan.departure
        )));
    }

    // Validate destination airport exists
    let dest_exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM airports WHERE icao = ?1)",
            [&plan.destination],
            |row| row.get(0),
        )?;

    if !dest_exists {
        return Err(AeroBaseError::InvalidInput(format!(
            "Destination airport {} not found",
            plan.destination
        )));
    }

    // Validate departure != destination
    if plan.departure == plan.destination {
        return Err(AeroBaseError::InvalidInput(
            "Departure and destination cannot be the same".to_string(),
        ));
    }

    // Validate alternate airport if specified
    if let Some(ref alternate) = plan.alternate {
        let alt_exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM airports WHERE icao = ?1)",
                [alternate],
                |row| row.get(0),
            )?;

        if !alt_exists {
            return Err(AeroBaseError::InvalidInput(format!(
                "Alternate airport {} not found",
                alternate
            )));
        }
    }

    // Validate cruise altitude (must be between 1000 and 60000 feet)
    if plan.cruise_altitude < 1000 || plan.cruise_altitude > 60000 {
        return Err(AeroBaseError::InvalidInput(
            "Cruise altitude must be between 1000 and 60000 feet".to_string(),
        ));
    }

    // Validate cruise speed (must be between 50 and 1000 knots)
    if plan.cruise_speed < 50 || plan.cruise_speed > 1000 {
        return Err(AeroBaseError::InvalidInput(
            "Cruise speed must be between 50 and 1000 knots".to_string(),
        ));
    }

    // Validate all waypoints exist
    for waypoint_id in &plan.route {
        let wp_exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM waypoints WHERE id = ?1)",
                [waypoint_id],
                |row| row.get(0),
            )?;

        if !wp_exists {
            return Err(AeroBaseError::InvalidInput(format!(
                "Waypoint {} not found",
                waypoint_id
            )));
        }
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;
    use std::sync::Arc;
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

        let conn = db.get_conn().unwrap();
        conn.execute(
            "INSERT INTO airports (id, icao, name, latitude, longitude, created_at)
             VALUES ('AP1', 'ZBAA', 'Beijing', 40.0, 116.0, 0)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO airports (id, icao, name, latitude, longitude, created_at)
             VALUES ('AP2', 'ZSSS', 'Shanghai', 31.0, 121.0, 0)",
            [],
        )
        .unwrap();

        db
    }

    #[test]
    fn test_valid_plan() {
        let db = setup_test_db();

        let plan = FlightPlan {
            departure: "ZBAA".to_string(),
            destination: "ZSSS".to_string(),
            alternate: None,
            cruise_altitude: 35000,
            cruise_speed: 450,
            route: vec![],
        };

        let result = validate_plan(&db, &plan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_departure() {
        let db = setup_test_db();

        let plan = FlightPlan {
            departure: "XXXX".to_string(),
            destination: "ZSSS".to_string(),
            alternate: None,
            cruise_altitude: 35000,
            cruise_speed: 450,
            route: vec![],
        };

        let result = validate_plan(&db, &plan);
        assert!(result.is_err());
    }

    #[test]
    fn test_same_departure_destination() {
        let db = setup_test_db();

        let plan = FlightPlan {
            departure: "ZBAA".to_string(),
            destination: "ZBAA".to_string(),
            alternate: None,
            cruise_altitude: 35000,
            cruise_speed: 450,
            route: vec![],
        };

        let result = validate_plan(&db, &plan);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_altitude() {
        let db = setup_test_db();

        let plan = FlightPlan {
            departure: "ZBAA".to_string(),
            destination: "ZSSS".to_string(),
            alternate: None,
            cruise_altitude: 100, // Too low
            cruise_speed: 450,
            route: vec![],
        };

        let result = validate_plan(&db, &plan);
        assert!(result.is_err());
    }
}
