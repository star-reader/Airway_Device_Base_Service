#ifndef AIRWAY_DEVICE_BASE_SERVICE_H
#define AIRWAY_DEVICE_BASE_SERVICE_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

typedef struct AeroBase AeroBase;

/**
 * Configuration structure for AeroBase initialization
 */
typedef struct {
    const char* db_path;      /* Path to SQLite database file */
    bool enable_wal;          /* Enable Write-Ahead Logging */
    uint32_t pool_size;       /* Database connection pool size */
} AeroBaseConfig;

/**
 * Device information structure
 */
typedef struct {
    char* id;                 /* Device unique identifier */
    char* fingerprint;        /* Device fingerprint hash */
    char* hardware_info;      /* Hardware information JSON (nullable) */
    int64_t created_at;       /* Creation timestamp */
    int64_t last_seen;        /* Last seen timestamp */
} Device;

/**
 * Geographic coordinate structure
 */
typedef struct {
    double latitude;          /* Latitude in degrees */
    double longitude;         /* Longitude in degrees */
} Coordinate;

/**
 * Airport structure
 */
typedef struct {
    char* id;                 /* Airport identifier */
    char* icao;               /* ICAO code */
    char* iata;               /* IATA code (nullable) */
    char* name;               /* Airport name */
    double latitude;          /* Latitude in degrees */
    double longitude;         /* Longitude in degrees */
    int32_t elevation;        /* Elevation in feet */
    char* country;            /* Country name (nullable) */
} Airport;

/**
 * Waypoint structure
 */
typedef struct {
    char* id;                 /* Waypoint identifier */
    char* name;               /* Waypoint name */
    double latitude;          /* Latitude in degrees */
    double longitude;         /* Longitude in degrees */
    char* type;               /* Waypoint type */
    char* region;             /* Region (nullable) */
} Waypoint;

/**
 * Flight plan structure
 */
typedef struct {
    char* departure;          /* Departure airport ICAO */
    char* destination;        /* Destination airport ICAO */
    char* alternate;          /* Alternate airport ICAO (nullable) */
    int32_t cruise_altitude;  /* Cruise altitude in feet */
    int32_t cruise_speed;     /* Cruise speed in knots */
    char** route;             /* Array of waypoint IDs */
    size_t route_count;       /* Number of waypoints in route */
} FlightPlan;

/**
 * Flight route with calculated data
 */
typedef struct {
    FlightPlan plan;          /* Original flight plan */
    double total_distance;    /* Total distance in nautical miles */
    int32_t estimated_time;   /* Estimated time in minutes */
} FlightRoute;

/**
 * Initialize a new AeroBase instance
 * 
 * @param config Configuration parameters
 * @return Pointer to AeroBase instance, or NULL on failure
 */
AeroBase* aerobase_new(const AeroBaseConfig* config);

/**
 * Free an AeroBase instance and release all resources
 * 
 * @param aerobase AeroBase instance to free
 */
void aerobase_free(AeroBase* aerobase);

/**
 * Get or create device fingerprint
 * 
 * @param aerobase AeroBase instance
 * @param device Output parameter for device information
 * @return 0 on success, -1 on failure
 */
int aerobase_get_device_fingerprint(const AeroBase* aerobase, Device* device);

/**
 * Free device structure memory
 * 
 * @param device Device structure to free
 */
void aerobase_free_device(Device* device);

/**
 * Find airports within a radius
 * 
 * @param aerobase AeroBase instance
 * @param center Center coordinate
 * @param radius_nm Radius in nautical miles
 * @param airports Output parameter for airports array
 * @param count Output parameter for number of airports
 * @return 0 on success, -1 on failure
 */
int aerobase_find_airports_within(
    const AeroBase* aerobase,
    Coordinate center,
    double radius_nm,
    Airport** airports,
    size_t* count
);

/**
 * Free airports array memory
 * 
 * @param airports Array of airports to free
 * @param count Number of airports in array
 */
void aerobase_free_airports(Airport* airports, size_t count);

/**
 * Find waypoints within a radius
 * 
 * @param aerobase AeroBase instance
 * @param center Center coordinate
 * @param radius_nm Radius in nautical miles
 * @param waypoints Output parameter for waypoints array
 * @param count Output parameter for number of waypoints
 * @return 0 on success, -1 on failure
 */
int aerobase_find_waypoints_within(
    const AeroBase* aerobase,
    Coordinate center,
    double radius_nm,
    Waypoint** waypoints,
    size_t* count
);

/**
 * Free waypoints array memory
 * 
 * @param waypoints Array of waypoints to free
 * @param count Number of waypoints in array
 */
void aerobase_free_waypoints(Waypoint* waypoints, size_t count);

/**
 * Validate a flight plan
 * 
 * @param aerobase AeroBase instance
 * @param plan Flight plan to validate
 * @return 0 if valid, -1 if invalid
 */
int aerobase_validate_flight_plan(const AeroBase* aerobase, const FlightPlan* plan);

/**
 * Calculate route for a flight plan
 * 
 * @param aerobase AeroBase instance
 * @param plan Flight plan
 * @param route Output parameter for calculated route
 * @return 0 on success, -1 on failure
 */
int aerobase_calculate_route(
    const AeroBase* aerobase,
    const FlightPlan* plan,
    FlightRoute* route
);

/**
 * Free flight route memory
 * 
 * @param route Flight route to free
 */
void aerobase_free_flight_route(FlightRoute* route);

/**
 * Get last error message
 * 
 * @return Pointer to error message string, or NULL if no error
 */
const char* aerobase_last_error(void);

#ifdef __cplusplus
}
#endif

#endif /* AIRWAY_DEVICE_BASE_SERVICE_H */
