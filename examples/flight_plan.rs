use airway_device_base_service::{flight::FlightPlanBuilder, AeroBase, Config};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("=== AeroBase Flight Planning Example ===\n");

    let config = Config {
        db_path: PathBuf::from("example_aerobase.db"),
        enable_wal: true,
        pool_size: 4,
    };

    println!("Initializing AeroBase...");
    let aerobase = AeroBase::new(config).await?;
    println!("✓ AeroBase initialized\n");

    // Ensure airports exist
    println!("Setting up airports...");
    let conn = aerobase.db().get_conn()?;

    conn.execute(
        "INSERT OR REPLACE INTO airports (id, icao, iata, name, latitude, longitude, elevation, country, created_at)
         VALUES ('ZBAA', 'ZBAA', 'PEK', 'Beijing Capital', 40.0801, 116.5846, 116, 'China', ?1)",
        [chrono::Utc::now().timestamp()],
    )?;

    conn.execute(
        "INSERT OR REPLACE INTO airports (id, icao, iata, name, latitude, longitude, elevation, country, created_at)
         VALUES ('ZSSS', 'ZSSS', 'PVG', 'Shanghai Pudong', 31.1434, 121.8052, 13, 'China', ?1)",
        [chrono::Utc::now().timestamp()],
    )?;

    conn.execute(
        "INSERT OR REPLACE INTO airports (id, icao, iata, name, latitude, longitude, elevation, country, created_at)
         VALUES ('ZGSZ', 'ZGSZ', 'SZX', 'Shenzhen Baoan', 22.6393, 113.8108, 13, 'China', ?1)",
        [chrono::Utc::now().timestamp()],
    )?;

    println!("✓ Airports ready\n");

    // Create flight plan
    println!("Creating flight plan...");
    let plan = FlightPlanBuilder::new()
        .departure("ZBAA")
        .destination("ZSSS")
        .alternate("ZGSZ")
        .cruise_altitude(35000)
        .cruise_speed(450)
        .build()?;

    println!("✓ Flight Plan created:");
    println!("  Departure: {}", plan.departure);
    println!("  Destination: {}", plan.destination);
    println!("  Alternate: {:?}", plan.alternate);
    println!("  Cruise Altitude: {} ft", plan.cruise_altitude);
    println!("  Cruise Speed: {} kts", plan.cruise_speed);
    println!();

    // Validate flight plan
    println!("Validating flight plan...");
    let is_valid = aerobase.flight().validate_plan(&plan)?;
    println!("✓ Flight plan is valid: {}", is_valid);
    println!();

    // Calculate route
    println!("Calculating route...");
    let route = aerobase.flight().calculate_route(&plan)?;
    println!("✓ Route calculated:");
    println!("  Total Distance: {:.1} nm", route.total_distance);
    println!("  Estimated Time: {} min ({:.1} hours)", 
        route.estimated_time, 
        route.estimated_time as f64 / 60.0
    );
    println!("\n  Route waypoints:");
    for (i, wp) in route.waypoints.iter().enumerate() {
        println!("    {}. {} - {} ({:.1} nm, +{} min)",
            i + 1,
            wp.id,
            wp.name,
            wp.cumulative_distance,
            wp.estimated_time
        );
    }
    println!();

    // Calculate fuel requirements
    println!("Calculating fuel requirements...");
    let fuel_flow = 50.0; // gallons per hour
    let required_fuel = aerobase.flight().calculate_fuel(&route, fuel_flow)?;
    println!("✓ Fuel required: {:.1} gallons (at {} gph)", required_fuel, fuel_flow);
    println!("  - Trip fuel: {:.1} gal", (route.estimated_time as f64 / 60.0) * fuel_flow);
    println!("  - Reserve fuel: {:.1} gal", (45.0 / 60.0) * fuel_flow);
    println!("  - Taxi fuel: {:.1} gal", ((route.estimated_time as f64 / 60.0) * fuel_flow) * 0.05);
    println!();

    println!("=== Example completed successfully ===");

    Ok(())
}
