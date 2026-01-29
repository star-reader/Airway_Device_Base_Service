use crate::models::Coordinate;
use geo::{Contains, Coord, LineString, Point, Polygon};

/// Calculate bounding box for a circle
pub fn bounding_box(center: Coordinate, radius_nm: f64) -> (Coordinate, Coordinate) {
    // Convert nautical miles to degrees (approximate)
    // 1 nautical mile ≈ 1.852 km ≈ 0.0167 degrees at equator
    let radius_deg = radius_nm * 0.0167;
    
    let min_lat = center.latitude - radius_deg;
    let max_lat = center.latitude + radius_deg;
    let min_lon = center.longitude - radius_deg;
    let max_lon = center.longitude + radius_deg;
    
    (
        Coordinate::new(min_lat, min_lon),
        Coordinate::new(max_lat, max_lon),
    )
}

/// Check if a point is inside a polygon
pub fn point_in_polygon(point: Coordinate, vertices: &[Coordinate]) -> bool {
    if vertices.len() < 3 {
        return false;
    }
    
    let coords: Vec<Coord<f64>> = vertices
        .iter()
        .map(|c| Coord {
            x: c.longitude,
            y: c.latitude,
        })
        .collect();
    
    let line_string = LineString::new(coords);
    let polygon = Polygon::new(line_string, vec![]);
    let pt = Point::new(point.longitude, point.latitude);
    
    polygon.contains(&pt)
}

/// Calculate great circle distance in nautical miles
pub fn great_circle_distance(from: Coordinate, to: Coordinate) -> f64 {
    from.distance_to(&to)
}

/// Calculate initial bearing in degrees
pub fn initial_bearing(from: Coordinate, to: Coordinate) -> f64 {
    from.bearing_to(&to)
}

/// Calculate destination point given distance and bearing
pub fn destination_point(
    start: Coordinate,
    distance_nm: f64,
    bearing_deg: f64,
) -> Coordinate {
    use geo::HaversineDestination;
    
    let point = start.to_point();
    let distance_m = distance_nm * 1852.0; // Convert to meters
    
    let dest = point.haversine_destination(bearing_deg, distance_m);
    
    Coordinate::new(dest.y(), dest.x())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounding_box() {
        let center = Coordinate::new(39.9042, 116.4074);
        let (min, max) = bounding_box(center, 50.0);
        
        assert!(min.latitude < center.latitude);
        assert!(max.latitude > center.latitude);
        assert!(min.longitude < center.longitude);
        assert!(max.longitude > center.longitude);
    }

    #[test]
    fn test_point_in_polygon() {
        let polygon = vec![
            Coordinate::new(0.0, 0.0),
            Coordinate::new(0.0, 1.0),
            Coordinate::new(1.0, 1.0),
            Coordinate::new(1.0, 0.0),
            Coordinate::new(0.0, 0.0),
        ];
        
        assert!(point_in_polygon(Coordinate::new(0.5, 0.5), &polygon));
        assert!(!point_in_polygon(Coordinate::new(2.0, 2.0), &polygon));
    }

    #[test]
    fn test_destination_point() {
        let start = Coordinate::new(0.0, 0.0);
        let dest = destination_point(start, 60.0, 0.0); // 60 nm north
        
        // Should be approximately at 1 degree north
        assert!(dest.latitude > 0.9 && dest.latitude < 1.1);
        assert!(dest.longitude.abs() < 0.1);
    }
}
