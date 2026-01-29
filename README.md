# Airway Device Base Service

A lightweight, embedded aviation database service written in Rust, designed for offline-first aviation applications. It provides a robust local storage solution for navigation data, device identity management, and flight planning components.

### Key Features

- **Embedded SQLite Database**: Zero-configuration, cross-platform database with ACID guarantees
- **Device Fingerprint Management**: Unique device identification and hardware information tracking
- **Offline-First Architecture**: Full functionality without network connectivity
- **Spatial Query Engine**: Efficient geographic queries optimized for aviation use cases
- **Flight Planning Support**: Route calculation, fuel estimation, and flight plan validation
- **Data Synchronization**: Framework for incremental data sync (extensible)

## Architecture

The library is organized into several core modules:

- **db**: Database connection management, schema definitions, and migrations
- **device**: Device fingerprint generation and identity management
- **models**: Aviation data models (airports, waypoints, airways, navaids, airspaces)
- **spatial**: Geographic query engine with R-Tree indexing
- **flight**: Flight planning, route calculation, and validation
- **sync**: Data synchronization framework (extensible)

## Installation

### From Source

Clone the repository and build:

```bash
git clone https://github.com/star-reader/Airway_Device_Base_Service.git
cd Airway_Device_Base_Service
cargo build --release
```

### Using as a Rust Library

Add this to your `Cargo.toml`:

```toml
[dependencies]
airway-device-base-service = { git = "https://github.com/star-reader/Airway_Device_Base_Service.git" }
```

### C/C++ Integration

Build the C library interface:

```bash
cargo build --release --features ffi
```

This generates a shared library that can be used from C/C++:

- Linux: `target/release/libairway_device_base_service.so`
- macOS: `target/release/libairway_device_base_service.dylib`
- Windows: `target/release/airway_device_base_service.dll`

Include the header file in your C/C++ project:

```cpp
#include "airway_device_base_service.h"
```

See the C++ Integration section below for detailed usage instructions.

## Quick Start

### Basic Initialization

```rust
use airway_device_base_service::{AeroBase, Config};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create configuration
    let config = Config {
        db_path: PathBuf::from("aerobase.db"),
        enable_wal: true,
        pool_size: 4,
    };

    // Initialize AeroBase
    let aerobase = AeroBase::new(config).await?;

    // Get device fingerprint
    let device = aerobase.device().get_or_create_fingerprint()?;
    println!("Device ID: {}", device.id);

    Ok(())
}
```

### Working with Aviation Data

#### Insert Airport Data

```rust
let conn = aerobase.db().get_conn()?;

conn.execute(
    "INSERT INTO airports (id, icao, iata, name, latitude, longitude, elevation, country, created_at)
     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
    rusqlite::params![
        "ZBAA",
        "ZBAA",
        "PEK",
        "Beijing Capital International Airport",
        40.0801,
        116.5846,
        116,
        "China",
        chrono::Utc::now().timestamp(),
    ],
)?;
```

#### Spatial Queries

```rust
use airway_device_base_service::models::Coordinate;

// Define search center
let beijing = Coordinate::new(39.9042, 116.4074);

// Find airports within 100 nautical miles
let airports = aerobase.spatial().find_airports_within(beijing, 100.0)?;

for airport in airports {
    let distance = airport.distance_from(beijing);
    println!("{} - {} nm away", airport.name, distance);
}

// Find nearest airport
if let Some(nearest) = aerobase.spatial().find_nearest_airport(beijing)? {
    println!("Nearest: {}", nearest.name);
}
```

#### Flight Planning

```rust
use airway_device_base_service::flight::FlightPlanBuilder;

// Create flight plan
let plan = FlightPlanBuilder::new()
    .departure("ZBAA")
    .destination("ZSSS") 
    .alternate("ZGSZ")
    .cruise_altitude(35000)
    .cruise_speed(450)
    .build()?;

// Validate plan
let is_valid = aerobase.flight().validate_plan(&plan)?;

// Calculate route
let route = aerobase.flight().calculate_route(&plan)?;
println!("Distance: {:.1} nm", route.total_distance);
println!("Time: {} minutes", route.estimated_time);

// Calculate fuel requirements
let fuel_flow = 50.0; // gallons per hour
let required_fuel = aerobase.flight().calculate_fuel(&route, fuel_flow)?;
println!("Fuel required: {:.1} gallons", required_fuel);
```

## Data Models

### Airport

Represents an airport with ICAO/IATA codes, geographic coordinates, and metadata.

```rust
pub struct Airport {
    pub id: String,
    pub icao: String,
    pub iata: Option<String>,
    pub name: String,
    pub coordinate: Coordinate,
    pub elevation: Option<i32>,
    pub country: Option<String>,
    pub region: Option<String>,
    pub created_at: i64,
}
```

### Waypoint

Navigation points including fixes, VORs, NDBs, and GPS waypoints.

```rust
pub struct Waypoint {
    pub id: String,
    pub name: String,
    pub coordinate: Coordinate,
    pub region: Option<String>,
    pub waypoint_type: WaypointType,
    pub created_at: i64,
}
```

### Airway

Defined routes between waypoints with altitude restrictions.

```rust
pub struct Airway {
    pub id: String,
    pub name: String,
    pub airway_type: AirwayType,
    pub min_altitude: Option<i32>,
    pub max_altitude: Option<i32>,
    pub created_at: i64,
}
```

### Navaid

Radio navigation aids (VOR, NDB, DME, etc.).

```rust
pub struct Navaid {
    pub id: String,
    pub name: String,
    pub navaid_type: NavaidType,
    pub coordinate: Coordinate,
    pub frequency: Option<f64>,
    pub range_nm: Option<i32>,
    pub elevation: Option<i32>,
    pub region: Option<String>,
    pub created_at: i64,
}
```

## Database Schema

The database uses SQLite with the following core tables:

- **devices**: Device fingerprint and hardware information
- **airports**: Airport data with spatial indexes
- **waypoints**: Navigation waypoints with spatial indexes
- **airways**: Airway definitions
- **airway_segments**: Airway segment connections between waypoints
- **navaids**: Radio navigation aids with spatial indexes
- **airspaces**: Airspace definitions
- **airspace_boundaries**: Airspace boundary polygons
- **sync_metadata**: Synchronization tracking

All spatial data is indexed for efficient geographic queries.

## Spatial Query Performance

The spatial engine uses bounding box filtering followed by precise distance calculations:

1. Initial filtering using latitude/longitude indexes
2. Precise Haversine distance calculation
3. R-Tree spatial indexing for in-memory operations

Typical query performance:

- Spatial search within 50nm radius: < 10ms
- Nearest point query: < 5ms
- Route calculation (500nm): < 50ms

## Error Handling

The library uses a custom error type with comprehensive error variants:

```rust
pub enum AeroBaseError {
    Database(rusqlite::Error),
    Io(std::io::Error),
    Serialization(serde_json::Error),
    DeviceFingerprint(String),
    SpatialQuery(String),
    FlightPlanning(String),
    Sync(String),
    InvalidInput(String),
    NotFound(String),
    Pool(String),
    Unknown(String),
}
```

All public functions return `Result<T, AeroBaseError>` for proper error handling.

## Examples

The repository includes several complete examples:

- **basic_usage.rs**: Database initialization and device fingerprinting
- **spatial_query.rs**: Geographic queries and nearest-point searches
- **flight_plan.rs**: Complete flight planning workflow

Run examples with:

```bash
cargo run --example basic_usage
cargo run --example spatial_query
cargo run --example flight_plan
```

## Testing

Run the test suite:

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific module tests
cargo test --lib device
cargo test --lib spatial
```

## Development

### Building from Source

```bash
git clone https://github.com/star-reader/Airway_Device_Base_Service.git
cd Airway_Device_Base_Service
cargo build --release
```

### Running Tests

```bash
cargo test --all-features
```

### Generating Documentation

```bash
cargo doc --no-deps --open
```

## Configuration

The `Config` struct accepts the following parameters:

- **db_path**: Path to SQLite database file (default: "aerobase.db")
- **enable_wal**: Enable Write-Ahead Logging for better concurrency (default: true)
- **pool_size**: Connection pool size (default: 4)

## Performance Considerations

1. **WAL Mode**: Enabled by default for better concurrent read/write performance
2. **Connection Pooling**: Reuses database connections to minimize overhead
3. **Spatial Indexes**: All geographic data is indexed for fast queries
4. **Foreign Keys**: Enabled for data integrity

## C++ Integration

The library provides a C API that can be used from C++ applications. This allows seamless integration with existing C++ aviation software.

### Building the C Library

Enable the FFI feature when building:

```bash
cargo build --release --features ffi
```

### C/C++ Header File

Include the generated header file `airway_device_base_service.h`:

Linking with CMake

Example `CMakeLists.txt`:

```cmake
cmake_minimum_required(VERSION 3.10)
project(MyAvionicsApp)

set(CMAKE_CXX_STANDARD 17)

# Find the Airway Device Base Service library
find_library(AEROBASE_LIB 
    NAMES airway_device_base_service
    PATHS ${CMAKE_SOURCE_DIR}/lib
)

add_executable(my_app main.cpp)
target_include_directories(my_app PRIVATE ${CMAKE_SOURCE_DIR}/include)
target_link_libraries(my_app ${AEROBASE_LIB})
```

### API Functions

Key C API functions available:

- `aerobase_new()`: Initialize the database
- `aerobase_get_device_fingerprint()`: Get device identity
- `aerobase_find_airports_within()`: Spatial search for airports
- `aerobase_find_waypoints_within()`: Spatial search for waypoints
- `aerobase_calculate_route()`: Calculate flight route
- `aerobase_validate_flight_plan()`: Validate flight plan
- `aerobase_free()`: Clean up resources

### Thread Safety

The C API is thread-safe. Each `AeroBase` instance can be safely shared across threads with proper synchronization.

## Platform Support

- Linux (x86_64, aarch64)
- macOS (Intel, Apple Silicon)
- Windows (x86_64)

## Safety and Security

- No unsafe code in the library
- SQL injection protection via parameterized queries
- Device fingerprint hashing with SHA-256
- Secure storage of sensitive data

## Contributing

Contributions are welcome! Please follow these guidelines:

1. Fork the repository
2. Create a feature branch
3. Write tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

Built with:

- rusqlite - SQLite bindings for Rust
- geo - Geospatial algorithms
- rstar - R-Tree spatial indexing
- tokio - Async runtime
- serde - Serialization framework

## Support

For issues and questions:

- GitHub Issues: https://github.com/star-reader/Airway_Device_Base_Service/issues
- Documentation: Run `cargo doc --open` for API documentation

## Roadmap

- [ ] Support for ARINC 424 navigation data import
- [ ] Enhanced route optimization algorithms
- [ ] Weather data integration
- [ ] NOTAM (Notice to Airmen) management
- [ ] Performance profiles for different aircraft types
- [ ] Network synchronization implementation
- [ ] WebAssembly support for browser-based applications
