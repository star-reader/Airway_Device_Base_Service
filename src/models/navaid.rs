use super::Coordinate;
use serde::{Deserialize, Serialize};

/// 导航设施类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NavaidType {
    VOR,    // 甚高频全向信标台
    VORDME, // VOR加测距仪
    DME,    // 测距仪
    NDB,    // 无方向信标
    TACAN,  // 战术空中导航
    Other,
}

impl NavaidType {
    pub fn as_str(&self) -> &str {
        match self {
            NavaidType::VOR => "VOR",
            NavaidType::VORDME => "VORDME",
            NavaidType::DME => "DME",
            NavaidType::NDB => "NDB",
            NavaidType::TACAN => "TACAN",
            NavaidType::Other => "OTHER",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "VOR" => NavaidType::VOR,
            "VORDME" | "VOR-DME" | "VOR/DME" => NavaidType::VORDME,
            "DME" => NavaidType::DME,
            "NDB" => NavaidType::NDB,
            "TACAN" => NavaidType::TACAN,
            _ => NavaidType::Other,
        }
    }
}

/// 导航设施
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Navaid {
    pub id: String,
    pub name: String,
    pub navaid_type: NavaidType,
    pub coordinate: Coordinate,
    pub frequency: Option<f64>, // MHz 或 KHz，取决于类型
    pub range_nm: Option<i32>,  // 海里
    pub elevation: Option<i32>, // 英尺
    pub region: Option<String>,
    pub created_at: i64,
}

impl Navaid {
    /// 创建新的导航设施
    pub fn new(
        id: String,
        name: String,
        navaid_type: NavaidType,
        coordinate: Coordinate,
    ) -> Self {
        Self {
            id,
            name,
            navaid_type,
            coordinate,
            frequency: None,
            range_nm: None,
            elevation: None,
            region: None,
            created_at: chrono::Utc::now().timestamp(),
        }
    }

    /// 计算从某个坐标的距离
    pub fn distance_from(&self, coord: Coordinate) -> f64 {
        self.coordinate.distance_to(&coord)
    }

    /// 检查坐标是否在导航设施范围内
    pub fn is_in_range(&self, coord: Coordinate) -> bool {
        if let Some(range) = self.range_nm {
            let distance = self.distance_from(coord);
            distance <= range as f64
        } else {
            false // 未指定范围
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navaid_creation() {
        let navaid = Navaid::new(
            "NAV001".to_string(),
            "BJS".to_string(),
            NavaidType::VOR,
            Coordinate::new(39.9042, 116.4074),
        );
        
        assert_eq!(navaid.name, "BJS");
        assert_eq!(navaid.navaid_type, NavaidType::VOR);
    }

    #[test]
    fn test_navaid_range() {
        let mut navaid = Navaid::new(
            "NAV001".to_string(),
            "TEST".to_string(),
            NavaidType::VOR,
            Coordinate::new(0.0, 0.0),
        );
        
        navaid.range_nm = Some(100);
        
        // Within range
        assert!(navaid.is_in_range(Coordinate::new(0.5, 0.5)));
        
        // Out of range
        assert!(!navaid.is_in_range(Coordinate::new(10.0, 10.0)));
    }

    #[test]
    fn test_navaid_type_conversion() {
        assert_eq!(NavaidType::from_str("VOR-DME"), NavaidType::VORDME);
        assert_eq!(NavaidType::from_str("vor"), NavaidType::VOR);
        assert_eq!(NavaidType::NDB.as_str(), "NDB");
    }
}
