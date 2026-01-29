pub mod calculator;
pub mod planner;
pub mod validator;

use crate::db::Database;
use crate::error::Result;
use crate::models::Coordinate;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 飞行计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlightPlan {
    pub departure: String,      // ICAO 代码
    pub destination: String,    // ICAO 代码
    pub alternate: Option<String>, // ICAO 代码
    pub cruise_altitude: i32,   // 英尺
    pub cruise_speed: i32,      // 节
    pub route: Vec<String>,     // 航路点 ID
}

/// 带有计算数据的飞行航线
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlightRoute {
    pub plan: FlightPlan,
    pub total_distance: f64,    // 海里
    pub estimated_time: i32,    // 分钟
    pub waypoints: Vec<RouteWaypoint>,
}

/// 航线中的航路点及其计算数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteWaypoint {
    pub id: String,
    pub name: String,
    pub coordinate: Coordinate,
    pub distance_from_previous: f64, // 海里
    pub cumulative_distance: f64,    // 海里
    pub estimated_time: i32,         // 从出发的分钟数
}

/// 飞行计划器
pub struct FlightPlanner {
    db: Arc<Database>,
}

impl FlightPlanner {
    /// 创建新的飞行计划器
    pub fn new(db: Arc<Database>) -> Result<Self> {
        Ok(Self { db })
    }

    /// 计算飞行计划的航线
    pub fn calculate_route(&self, plan: &FlightPlan) -> Result<FlightRoute> {
        planner::calculate_route(&self.db, plan)
    }

    /// 验证飞行计划
    pub fn validate_plan(&self, plan: &FlightPlan) -> Result<bool> {
        validator::validate_plan(&self.db, plan)
    }

    /// 计算燃油需求
    pub fn calculate_fuel(&self, route: &FlightRoute, fuel_flow: f64) -> Result<f64> {
        calculator::calculate_fuel(route, fuel_flow)
    }
}

/// Flight plan builder
pub struct FlightPlanBuilder {
    departure: Option<String>,
    destination: Option<String>,
    alternate: Option<String>,
    cruise_altitude: Option<i32>,
    cruise_speed: Option<i32>,
    route: Vec<String>,
}

impl FlightPlanBuilder {
    pub fn new() -> Self {
        Self {
            departure: None,
            destination: None,
            alternate: None,
            cruise_altitude: None,
            cruise_speed: None,
            route: Vec::new(),
        }
    }

    pub fn departure(mut self, icao: &str) -> Self {
        self.departure = Some(icao.to_string());
        self
    }

    pub fn destination(mut self, icao: &str) -> Self {
        self.destination = Some(icao.to_string());
        self
    }

    pub fn alternate(mut self, icao: &str) -> Self {
        self.alternate = Some(icao.to_string());
        self
    }

    pub fn cruise_altitude(mut self, altitude: i32) -> Self {
        self.cruise_altitude = Some(altitude);
        self
    }

    pub fn cruise_speed(mut self, speed: i32) -> Self {
        self.cruise_speed = Some(speed);
        self
    }

    pub fn add_waypoint(mut self, waypoint_id: &str) -> Self {
        self.route.push(waypoint_id.to_string());
        self
    }

    pub fn build(self) -> Result<FlightPlan> {
        let departure = self.departure.ok_or_else(|| {
            crate::error::AeroBaseError::InvalidInput("Departure required".to_string())
        })?;

        let destination = self.destination.ok_or_else(|| {
            crate::error::AeroBaseError::InvalidInput("Destination required".to_string())
        })?;

        let cruise_altitude = self.cruise_altitude.ok_or_else(|| {
            crate::error::AeroBaseError::InvalidInput("Cruise altitude required".to_string())
        })?;

        let cruise_speed = self.cruise_speed.ok_or_else(|| {
            crate::error::AeroBaseError::InvalidInput("Cruise speed required".to_string())
        })?;

        Ok(FlightPlan {
            departure,
            destination,
            alternate: self.alternate,
            cruise_altitude,
            cruise_speed,
            route: self.route,
        })
    }
}

impl Default for FlightPlanBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flight_plan_builder() {
        let plan = FlightPlanBuilder::new()
            .departure("ZBAA")
            .destination("ZSSS")
            .cruise_altitude(35000)
            .cruise_speed(450)
            .build();

        assert!(plan.is_ok());
        let plan = plan.unwrap();
        assert_eq!(plan.departure, "ZBAA");
        assert_eq!(plan.destination, "ZSSS");
    }

    #[test]
    fn test_flight_plan_builder_missing_fields() {
        let plan = FlightPlanBuilder::new()
            .departure("ZBAA")
            .build();

        assert!(plan.is_err());
    }
}
