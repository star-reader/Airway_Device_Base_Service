use crate::error::{AeroBaseError, Result};
use crate::flight::{FlightRoute, RouteWaypoint};

/// Calculate fuel requirements in gallons
pub fn calculate_fuel(route: &FlightRoute, fuel_flow_gph: f64) -> Result<f64> {
    if fuel_flow_gph <= 0.0 {
        return Err(AeroBaseError::InvalidInput(
            "Fuel flow must be positive".to_string(),
        ));
    }

    let flight_time_hours = route.estimated_time as f64 / 60.0;
    let trip_fuel = flight_time_hours * fuel_flow_gph;

    // Add reserves (45 minutes standard)
    let reserve_fuel = (45.0 / 60.0) * fuel_flow_gph;

    // Add taxi fuel (typically 5% of trip)
    let taxi_fuel = trip_fuel * 0.05;

    let total_fuel = trip_fuel + reserve_fuel + taxi_fuel;

    Ok(total_fuel)
}

/// Calculate time between waypoints
pub fn calculate_segment_time(distance_nm: f64, speed_knots: i32) -> i32 {
    if speed_knots <= 0 {
        return 0;
    }

    let hours = distance_nm / speed_knots as f64;
    (hours * 60.0).round() as i32
}

/// Calculate estimated arrival time
pub fn calculate_eta(departure_time: i64, flight_time_minutes: i32) -> i64 {
    departure_time + (flight_time_minutes as i64 * 60)
}

/// Calculate wind correction angle
pub fn calculate_wind_correction(
    wind_direction: f64,
    wind_speed: f64,
    true_course: f64,
    true_airspeed: f64,
) -> f64 {
    let wind_angle = wind_direction - true_course;
    let wind_angle_rad = wind_angle.to_radians();

    let wca = ((wind_speed * wind_angle_rad.sin()) / true_airspeed).asin();
    wca.to_degrees()
}

/// Calculate ground speed
pub fn calculate_ground_speed(
    wind_direction: f64,
    wind_speed: f64,
    true_course: f64,
    true_airspeed: f64,
) -> f64 {
    let wind_angle = wind_direction - true_course;
    let wind_angle_rad = wind_angle.to_radians();

    let head_wind = wind_speed * wind_angle_rad.cos();
    true_airspeed - head_wind
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flight::{FlightPlan, FlightRoute};

    #[test]
    fn test_calculate_fuel() {
        let plan = FlightPlan {
            departure: "ZBAA".to_string(),
            destination: "ZSSS".to_string(),
            alternate: None,
            cruise_altitude: 35000,
            cruise_speed: 450,
            route: vec![],
        };

        let route = FlightRoute {
            plan,
            total_distance: 540.0,
            estimated_time: 72, // 1.2 hours
            waypoints: vec![],
        };

        let fuel = calculate_fuel(&route, 50.0).unwrap();

        // Should be around 60 gallons trip + 37.5 reserves + 3 taxi = ~100 gallons
        assert!(fuel > 90.0 && fuel < 110.0);
    }

    #[test]
    fn test_calculate_segment_time() {
        let time = calculate_segment_time(100.0, 200);
        assert_eq!(time, 30); // 100nm at 200kts = 30 minutes
    }

    #[test]
    fn test_calculate_eta() {
        let departure = 1000;
        let flight_time = 120; // 2 hours
        let eta = calculate_eta(departure, flight_time);
        assert_eq!(eta, 1000 + 7200); // 7200 seconds = 2 hours
    }

    #[test]
    fn test_calculate_ground_speed() {
        // Headwind scenario
        let gs = calculate_ground_speed(180.0, 20.0, 0.0, 200.0);
        assert!(gs < 200.0); // Should be slower than TAS

        // Tailwind scenario
        let gs = calculate_ground_speed(0.0, 20.0, 0.0, 200.0);
        assert!(gs > 200.0); // Should be faster than TAS
    }
}
