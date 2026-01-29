use serde::{Deserialize, Serialize};

/// 航路类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AirwayType {
    High,    // 高空航路
    Low,     // 低空航路
    RNAV,    // 区域导航
    Other,
}

impl AirwayType {
    pub fn as_str(&self) -> &str {
        match self {
            AirwayType::High => "HIGH",
            AirwayType::Low => "LOW",
            AirwayType::RNAV => "RNAV",
            AirwayType::Other => "OTHER",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "HIGH" => AirwayType::High,
            "LOW" => AirwayType::Low,
            "RNAV" => AirwayType::RNAV,
            _ => AirwayType::Other,
        }
    }
}

/// 航路
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Airway {
    pub id: String,
    pub name: String,
    pub airway_type: AirwayType,
    pub min_altitude: Option<i32>, // 英尺
    pub max_altitude: Option<i32>, // 英尺
    pub created_at: i64,
}

impl Airway {
    /// 创建新的航路
    pub fn new(id: String, name: String, airway_type: AirwayType) -> Self {
        Self {
            id,
            name,
            airway_type,
            min_altitude: None,
            max_altitude: None,
            created_at: chrono::Utc::now().timestamp(),
        }
    }

    /// 检查高度是否在航路限制范围内
    pub fn is_altitude_valid(&self, altitude: i32) -> bool {
        match (self.min_altitude, self.max_altitude) {
            (Some(min), Some(max)) => altitude >= min && altitude <= max,
            (Some(min), None) => altitude >= min,
            (None, Some(max)) => altitude <= max,
            (None, None) => true,
        }
    }
}

/// 连接两个航路点的航路段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirwaySegment {
    pub id: String,
    pub airway_id: String,
    pub from_waypoint_id: String,
    pub to_waypoint_id: String,
    pub sequence: i32,
    pub distance: Option<f64>, // 海里
    pub created_at: i64,
}

impl AirwaySegment {
    /// 创建新的航路段
    pub fn new(
        id: String,
        airway_id: String,
        from_waypoint_id: String,
        to_waypoint_id: String,
        sequence: i32,
    ) -> Self {
        Self {
            id,
            airway_id,
            from_waypoint_id,
            to_waypoint_id,
            sequence,
            distance: None,
            created_at: chrono::Utc::now().timestamp(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_airway_creation() {
        let airway = Airway::new(
            "AW001".to_string(),
            "A1".to_string(),
            AirwayType::High,
        );
        
        assert_eq!(airway.name, "A1");
        assert_eq!(airway.airway_type, AirwayType::High);
    }

    #[test]
    fn test_altitude_validation() {
        let mut airway = Airway::new(
            "AW001".to_string(),
            "A1".to_string(),
            AirwayType::High,
        );
        
        airway.min_altitude = Some(10000);
        airway.max_altitude = Some(30000);
        
        assert!(airway.is_altitude_valid(20000));
        assert!(!airway.is_altitude_valid(5000));
        assert!(!airway.is_altitude_valid(35000));
    }
}
