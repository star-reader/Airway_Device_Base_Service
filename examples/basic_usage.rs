use airway_device_base_service::{AeroBase, Config};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::init();

    println!("=== AeroBase Basic Usage Example ===\n");

    // Create configuration
    let config = Config {
        db_path: PathBuf::from("example_aerobase.db"),
        enable_wal: true,
        pool_size: 4,
    };

    println!("Initializing AeroBase...");
    let aerobase = AeroBase::new(config).await?;
    println!("✓ AeroBase initialized\n");

    // Get or create device fingerprint
    println!("Getting device fingerprint...");
    let device = aerobase.device().get_or_create_fingerprint()?;
    println!("✓ Device ID: {}", device.id);
    println!("  Fingerprint: {}", device.fingerprint);
    if let Some(hw_info) = device.hardware_info {
        println!("  Hardware Info: {}", hw_info);
    }
    println!();

    // List all devices
    println!("Listing all devices...");
    let devices = aerobase.device().list_devices()?;
    println!("✓ Found {} device(s)\n", devices.len());

    // Insert sample data
    println!("Inserting sample airports...");
    let conn = aerobase.db().get_conn()?;
    
    conn.execute(
        "INSERT OR REPLACE INTO airports (id, icao, iata, name, latitude, longitude, elevation, country, created_at)
         VALUES ('ZBAA', 'ZBAA', 'PEK', 'Beijing Capital International Airport', 40.0801, 116.5846, 116, 'China', ?1)",
        [chrono::Utc::now().timestamp()],
    )?;

    conn.execute(
        "INSERT OR REPLACE INTO airports (id, icao, iata, name, latitude, longitude, elevation, country, created_at)
         VALUES ('ZSSS', 'ZSSS', 'PVG', 'Shanghai Pudong International Airport', 31.1434, 121.8052, 13, 'China', ?1)",
        [chrono::Utc::now().timestamp()],
    )?;

    conn.execute(
        "INSERT OR REPLACE INTO airports (id, icao, iata, name, latitude, longitude, elevation, country, created_at)
         VALUES ('ZGSZ', 'ZGSZ', 'SZX', 'Shenzhen Bao''an International Airport', 22.6393, 113.8108, 13, 'China', ?1)",
        [chrono::Utc::now().timestamp()],
    )?;

    println!("✓ Inserted 3 airports\n");

    // Query database
    println!("Querying airports...");
    let mut stmt = conn.prepare("SELECT icao, name, latitude, longitude FROM airports")?;
    let airports = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, f64>(2)?,
            row.get::<_, f64>(3)?,
        ))
    })?;

    for airport in airports {
        let (icao, name, lat, lon) = airport?;
        println!("  {} - {} ({:.4}, {:.4})", icao, name, lat, lon);
    }
    println!();

    println!("=== Example completed successfully ===");

    Ok(())
}
