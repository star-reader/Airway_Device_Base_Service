pub mod airport;
pub mod airspace;
pub mod airway;
pub mod navaid;
pub mod waypoint;

use geo::Point;
use serde::{Deserialize, Serialize};

/// 地理坐标
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Coordinate {
    pub latitude: f64,
    pub longitude: f64,
}

impl Coordinate {
    /// 创建新的坐标
    pub fn new(latitude: f64, longitude: f64) -> Self {
        Self {
            latitude,
            longitude,
        }
    }

    /// 转换为 geo::Point
    pub fn to_point(&self) -> Point<f64> {
        Point::new(self.longitude, self.latitude)
    }

    /// 计算到另一个坐标的距离（单位：海里）
    pub fn distance_to(&self, other: &Coordinate) -> f64 {
        use geo::HaversineDistance;
        let p1 = self.to_point();
        let p2 = other.to_point();
        let meters = p1.haversine_distance(&p2);
        meters / 1852.0 // 将米转换为海里
    }

    /// 计算到另一个坐标的方位角（单位：度）
    pub fn bearing_to(&self, other: &Coordinate) -> f64 {
        use geo::HaversineBearing;
        let p1 = self.to_point();
        let p2 = other.to_point();
        p1.haversine_bearing(p2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinate_distance() {
        // Beijing to Shanghai (approximate)
        let beijing = Coordinate::new(39.9042, 116.4074);
        let shanghai = Coordinate::new(31.2304, 121.4737);
        
        let distance = beijing.distance_to(&shanghai);
        
        // Should be around 600 nautical miles
        assert!(distance > 500.0 && distance < 700.0);
    }

    #[test]
    fn test_coordinate_bearing() {
        let start = Coordinate::new(0.0, 0.0);
        let north = Coordinate::new(1.0, 0.0);
        
        let bearing = start.bearing_to(&north);
        
        // Should be roughly north (0 degrees)
        assert!(bearing.abs() < 1.0);
    }
}
