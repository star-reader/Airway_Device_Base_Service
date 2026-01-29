use super::Coordinate;
use serde::{Deserialize, Serialize};

/// 机场
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Airport {
    pub id: String,
    pub icao: String,
    pub iata: Option<String>,
    pub name: String,
    pub coordinate: Coordinate,
    pub elevation: Option<i32>, // 英尺
    pub country: Option<String>,
    pub region: Option<String>,
    pub created_at: i64,
}

impl Airport {
    /// 创建新的机场
    pub fn new(
        id: String,
        icao: String,
        name: String,
        coordinate: Coordinate,
    ) -> Self {
        Self {
            id,
            icao,
            iata: None,
            name,
            coordinate,
            elevation: None,
            country: None,
            region: None,
            created_at: chrono::Utc::now().timestamp(),
        }
    }

    /// 计算从某个坐标的距离
    pub fn distance_from(&self, coord: Coordinate) -> f64 {
        self.coordinate.distance_to(&coord)
    }

    /// 检查机场是否有IATA代码
    pub fn has_iata(&self) -> bool {
        self.iata.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_airport_creation() {
        let airport = Airport::new(
            "AP001".to_string(),
            "ZBAA".to_string(),
            "Beijing Capital International Airport".to_string(),
            Coordinate::new(40.0801, 116.5846),
        );
        
        assert_eq!(airport.icao, "ZBAA");
        assert!(!airport.has_iata());
    }

    #[test]
    fn test_airport_with_iata() {
        let mut airport = Airport::new(
            "AP001".to_string(),
            "ZBAA".to_string(),
            "Beijing Capital".to_string(),
            Coordinate::new(40.0801, 116.5846),
        );
        
        airport.iata = Some("PEK".to_string());
        assert!(airport.has_iata());
    }
}
