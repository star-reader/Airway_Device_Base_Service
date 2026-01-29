use airway_device_base_service::{models::Coordinate, AeroBase, Config};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("=== AeroBase Spatial Query Example ===\n");

    let config = Config {
        db_path: PathBuf::from("example_aerobase.db"),
        enable_wal: true,
        pool_size: 4,
    };

    println!("Initializing AeroBase...");
    let aerobase = AeroBase::new(config).await?;
    println!("✓ AeroBase initialized\n");

    // Insert sample data if not exists
    println!("Ensuring sample data exists...");
    let conn = aerobase.db().get_conn()?;

    conn.execute(
        "INSERT OR IGNORE INTO airports (id, icao, iata, name, latitude, longitude, elevation, country, created_at)
         VALUES ('ZBAA', 'ZBAA', 'PEK', 'Beijing Capital', 40.0801, 116.5846, 116, 'China', ?1)",
        [chrono::Utc::now().timestamp()],
    )?;

    conn.execute(
        "INSERT OR IGNORE INTO waypoints (id, name, latitude, longitude, type, created_at)
         VALUES ('WP001', 'BEKOL', 39.9042, 116.4074, 'FIX', ?1)",
        [chrono::Utc::now().timestamp()],
    )?;

    conn.execute(
        "INSERT OR IGNORE INTO waypoints (id, name, latitude, longitude, type, created_at)
         VALUES ('WP002', 'PIKAS', 40.5, 116.2, 'FIX', ?1)",
        [chrono::Utc::now().timestamp()],
    )?;

    println!("✓ Sample data ready\n");

    // Define search center (Beijing)
    let beijing = Coordinate::new(39.9042, 116.4074);
    println!("Search center: Beijing ({:.4}, {:.4})", beijing.latitude, beijing.longitude);

    // Search for airports within 100 nautical miles
    println!("\nSearching for airports within 100 nm...");
    let airports = aerobase.spatial().find_airports_within(beijing, 100.0)?;
    println!("✓ Found {} airport(s):", airports.len());
    for airport in &airports {
        let distance = airport.distance_from(beijing);
        println!("  {} - {} ({:.1} nm away)", airport.icao, airport.name, distance);
    }

    // Search for waypoints within 50 nautical miles
    println!("\nSearching for waypoints within 50 nm...");
    let waypoints = aerobase.spatial().find_waypoints_within(beijing, 50.0)?;
    println!("✓ Found {} waypoint(s):", waypoints.len());
    for wp in &waypoints {
        let distance = wp.distance_from(beijing);
        let bearing = beijing.bearing_to(&wp.coordinate);
        println!(
            "  {} - {} ({:.1} nm at {:.0}°)",
            wp.id, wp.name, distance, bearing
        );
    }

    // Find nearest airport
    println!("\nFinding nearest airport...");
    if let Some(nearest) = aerobase.spatial().find_nearest_airport(beijing)? {
        let distance = nearest.distance_from(beijing);
        println!("✓ Nearest airport: {} - {} ({:.1} nm)", 
            nearest.icao, nearest.name, distance);
    } else {
        println!("  No airport found nearby");
    }

    // Find nearest waypoint
    println!("\nFinding nearest waypoint...");
    if let Some(nearest) = aerobase.spatial().find_nearest_waypoint(beijing)? {
        let distance = nearest.distance_from(beijing);
        println!("✓ Nearest waypoint: {} ({:.1} nm)", nearest.name, distance);
    } else {
        println!("  No waypoint found nearby");
    }

    println!("\n=== Example completed successfully ===");

    Ok(())
}
