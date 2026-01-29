use super::Coordinate;
use serde::{Deserialize, Serialize};

/// 航路点类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WaypointType {
    Airport,
    VOR,
    NDB,
    Fix,
    GPS,
    Other,
}

impl WaypointType {
    pub fn as_str(&self) -> &str {
        match self {
            WaypointType::Airport => "AIRPORT",
            WaypointType::VOR => "VOR",
            WaypointType::NDB => "NDB",
            WaypointType::Fix => "FIX",
            WaypointType::GPS => "GPS",
            WaypointType::Other => "OTHER",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "AIRPORT" => WaypointType::Airport,
            "VOR" => WaypointType::VOR,
            "NDB" => WaypointType::NDB,
            "FIX" => WaypointType::Fix,
            "GPS" => WaypointType::GPS,
            _ => WaypointType::Other,
        }
    }
}

/// 航路点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Waypoint {
    pub id: String,
    pub name: String,
    pub coordinate: Coordinate,
    pub region: Option<String>,
    pub waypoint_type: WaypointType,
    pub created_at: i64,
}

impl Waypoint {
    /// 创建新的航路点
    pub fn new(
        id: String,
        name: String,
        coordinate: Coordinate,
        waypoint_type: WaypointType,
    ) -> Self {
        Self {
            id,
            name,
            coordinate,
            region: None,
            waypoint_type,
            created_at: chrono::Utc::now().timestamp(),
        }
    }

    /// 计算从另一个坐标的距离
    pub fn distance_from(&self, coord: Coordinate) -> f64 {
        self.coordinate.distance_to(&coord)
    }

    /// 计算从另一个坐标的方位角
    pub fn bearing_from(&self, coord: Coordinate) -> f64 {
        coord.bearing_to(&self.coordinate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_waypoint_creation() {
        let wp = Waypoint::new(
            "WP001".to_string(),
            "TEST".to_string(),
            Coordinate::new(39.9042, 116.4074),
            WaypointType::Fix,
        );
        
        assert_eq!(wp.id, "WP001");
        assert_eq!(wp.name, "TEST");
        assert_eq!(wp.waypoint_type, WaypointType::Fix);
    }

    #[test]
    fn test_waypoint_type_conversion() {
        assert_eq!(WaypointType::from_str("VOR"), WaypointType::VOR);
        assert_eq!(WaypointType::from_str("vor"), WaypointType::VOR);
        assert_eq!(WaypointType::GPS.as_str(), "GPS");
    }
}
