use crate::db::Database;
use crate::error::{AeroBaseError, Result};
use crate::flight::{calculator, FlightPlan, FlightRoute, RouteWaypoint};
use crate::models::Coordinate;

/// Calculate route for a flight plan
pub fn calculate_route(db: &Database, plan: &FlightPlan) -> Result<FlightRoute> {
    let conn = db.get_conn()?;

    // Get departure and destination coordinates
    let dep_coord = get_airport_coordinate(&conn, &plan.departure)?;
    let dest_coord = get_airport_coordinate(&conn, &plan.destination)?;

    let mut waypoints = Vec::new();
    let mut cumulative_distance = 0.0;
    let mut prev_coord = dep_coord;

    // Add departure
    waypoints.push(RouteWaypoint {
        id: plan.departure.clone(),
        name: plan.departure.clone(),
        coordinate: dep_coord,
        distance_from_previous: 0.0,
        cumulative_distance: 0.0,
        estimated_time: 0,
    });

    // Add route waypoints
    for waypoint_id in &plan.route {
        let waypoint = get_waypoint(&conn, waypoint_id)?;
        let distance = prev_coord.distance_to(&waypoint.coordinate);
        cumulative_distance += distance;

        let time = calculator::calculate_segment_time(cumulative_distance, plan.cruise_speed);

        waypoints.push(RouteWaypoint {
            id: waypoint.id,
            name: waypoint.name,
            coordinate: waypoint.coordinate,
            distance_from_previous: distance,
            cumulative_distance,
            estimated_time: time,
        });

        prev_coord = waypoint.coordinate;
    }

    // Add destination
    let final_distance = prev_coord.distance_to(&dest_coord);
    cumulative_distance += final_distance;
    let total_time = calculator::calculate_segment_time(cumulative_distance, plan.cruise_speed);

    waypoints.push(RouteWaypoint {
        id: plan.destination.clone(),
        name: plan.destination.clone(),
        coordinate: dest_coord,
        distance_from_previous: final_distance,
        cumulative_distance,
        estimated_time: total_time,
    });

    Ok(FlightRoute {
        plan: plan.clone(),
        total_distance: cumulative_distance,
        estimated_time: total_time,
        waypoints,
    })
}

/// Get airport coordinate by ICAO code
fn get_airport_coordinate(
    conn: &rusqlite::Connection,
    icao: &str,
) -> Result<Coordinate> {
    conn.query_row(
        "SELECT latitude, longitude FROM airports WHERE icao = ?1",
        [icao],
        |row| Ok(Coordinate::new(row.get(0)?, row.get(1)?)),
    )
    .map_err(|_| {
        AeroBaseError::NotFound(format!("Airport {} not found", icao))
    })
}

/// Get waypoint data
fn get_waypoint(
    conn: &rusqlite::Connection,
    waypoint_id: &str,
) -> Result<WaypointData> {
    conn.query_row(
        "SELECT id, name, latitude, longitude FROM waypoints WHERE id = ?1",
        [waypoint_id],
        |row| {
            Ok(WaypointData {
                id: row.get(0)?,
                name: row.get(1)?,
                coordinate: Coordinate::new(row.get(2)?, row.get(3)?),
            })
        },
    )
    .map_err(|_| {
        AeroBaseError::NotFound(format!("Waypoint {} not found", waypoint_id))
    })
}

struct WaypointData {
    id: String,
    name: String,
    coordinate: Coordinate,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;
    use std::sync::Arc;
    use tempfile::NamedTempFile;

    #[test]
    fn test_calculate_route_simple() {
        let temp_file = NamedTempFile::new().unwrap();
        let config = Config {
            db_path: temp_file.path().to_path_buf(),
            enable_wal: false,
            pool_size: 1,
        };

        let db = Arc::new(Database::new(&config).unwrap());
        db.migrate().unwrap();

        // Insert test airports
        let conn = db.get_conn().unwrap();
        conn.execute(
            "INSERT INTO airports (id, icao, name, latitude, longitude, created_at)
             VALUES ('AP1', 'ZBAA', 'Beijing', 40.0801, 116.5846, 0)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO airports (id, icao, name, latitude, longitude, created_at)
             VALUES ('AP2', 'ZSSS', 'Shanghai', 31.1434, 121.8052, 0)",
            [],
        )
        .unwrap();

        let plan = FlightPlan {
            departure: "ZBAA".to_string(),
            destination: "ZSSS".to_string(),
            alternate: None,
            cruise_altitude: 35000,
            cruise_speed: 450,
            route: vec![],
        };

        let route = calculate_route(&db, &plan).unwrap();
        assert!(route.total_distance > 0.0);
        assert_eq!(route.waypoints.len(), 2); // departure + destination
    }
}
