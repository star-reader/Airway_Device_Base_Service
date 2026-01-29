use super::Coordinate;
use serde::{Deserialize, Serialize};

/// Airspace class
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AirspaceClass {
    ClassA,
    ClassB,
    ClassC,
    ClassD,
    ClassE,
    ClassG,
    Other,
}

impl AirspaceClass {
    pub fn as_str(&self) -> &str {
        match self {
            AirspaceClass::ClassA => "A",
            AirspaceClass::ClassB => "B",
            AirspaceClass::ClassC => "C",
            AirspaceClass::ClassD => "D",
            AirspaceClass::ClassE => "E",
            AirspaceClass::ClassG => "G",
            AirspaceClass::Other => "OTHER",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "A" | "CLASS A" => AirspaceClass::ClassA,
            "B" | "CLASS B" => AirspaceClass::ClassB,
            "C" | "CLASS C" => AirspaceClass::ClassC,
            "D" | "CLASS D" => AirspaceClass::ClassD,
            "E" | "CLASS E" => AirspaceClass::ClassE,
            "G" | "CLASS G" => AirspaceClass::ClassG,
            _ => AirspaceClass::Other,
        }
    }
}

/// 空域类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AirspaceType {
    ControlZone,    // 管制区
    TerminalArea,   // 终端区
    FlightInfoRegion, // 飞行情报区
    Restricted,     // 限制区
    Danger,         // 危险区
    Prohibited,     // 禁飞区
    Other,
}

impl AirspaceType {
    pub fn as_str(&self) -> &str {
        match self {
            AirspaceType::ControlZone => "CTR",
            AirspaceType::TerminalArea => "TMA",
            AirspaceType::FlightInfoRegion => "FIR",
            AirspaceType::Restricted => "RESTRICTED",
            AirspaceType::Danger => "DANGER",
            AirspaceType::Prohibited => "PROHIBITED",
            AirspaceType::Other => "OTHER",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "CTR" | "CONTROL ZONE" => AirspaceType::ControlZone,
            "TMA" | "TERMINAL AREA" => AirspaceType::TerminalArea,
            "FIR" | "FLIGHT INFO REGION" => AirspaceType::FlightInfoRegion,
            "RESTRICTED" | "R" => AirspaceType::Restricted,
            "DANGER" | "D" => AirspaceType::Danger,
            "PROHIBITED" | "P" => AirspaceType::Prohibited,
            _ => AirspaceType::Other,
        }
    }
}

/// 空域
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Airspace {
    pub id: String,
    pub name: String,
    pub airspace_type: AirspaceType,
    pub class: Option<AirspaceClass>,
    pub lower_limit: Option<i32>, // 英尺
    pub upper_limit: Option<i32>, // 英尺
    pub created_at: i64,
}

impl Airspace {
    /// 创建新的空域
    pub fn new(id: String, name: String, airspace_type: AirspaceType) -> Self {
        Self {
            id,
            name,
            airspace_type,
            class: None,
            lower_limit: None,
            upper_limit: None,
            created_at: chrono::Utc::now().timestamp(),
        }
    }

    /// 检查高度是否在空域范围内
    pub fn is_altitude_in_airspace(&self, altitude: i32) -> bool {
        match (self.lower_limit, self.upper_limit) {
            (Some(lower), Some(upper)) => altitude >= lower && altitude <= upper,
            (Some(lower), None) => altitude >= lower,
            (None, Some(upper)) => altitude <= upper,
            (None, None) => true,
        }
    }
}

/// 空域边界点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirspaceBoundary {
    pub id: String,
    pub airspace_id: String,
    pub coordinate: Coordinate,
    pub sequence: i32,
}

impl AirspaceBoundary {
    /// 创建新的边界点
    pub fn new(
        id: String,
        airspace_id: String,
        coordinate: Coordinate,
        sequence: i32,
    ) -> Self {
        Self {
            id,
            airspace_id,
            coordinate,
            sequence,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_airspace_creation() {
        let airspace = Airspace::new(
            "AS001".to_string(),
            "Beijing CTR".to_string(),
            AirspaceType::ControlZone,
        );
        
        assert_eq!(airspace.name, "Beijing CTR");
        assert_eq!(airspace.airspace_type, AirspaceType::ControlZone);
    }

    #[test]
    fn test_altitude_in_airspace() {
        let mut airspace = Airspace::new(
            "AS001".to_string(),
            "Test".to_string(),
            AirspaceType::TerminalArea,
        );
        
        airspace.lower_limit = Some(5000);
        airspace.upper_limit = Some(15000);
        
        assert!(airspace.is_altitude_in_airspace(10000));
        assert!(!airspace.is_altitude_in_airspace(3000));
        assert!(!airspace.is_altitude_in_airspace(20000));
    }

    #[test]
    fn test_airspace_class_conversion() {
        assert_eq!(AirspaceClass::from_str("B"), AirspaceClass::ClassB);
        assert_eq!(AirspaceClass::from_str("class c"), AirspaceClass::ClassC);
        assert_eq!(AirspaceClass::ClassD.as_str(), "D");
    }
}
