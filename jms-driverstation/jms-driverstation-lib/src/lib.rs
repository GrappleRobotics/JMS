use std::num::ParseIntError;

use jms_core_lib::{models::AllianceStationId, db::Table};

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub enum RobotState {
  Auto,
  Test,
  Teleop
}

impl Default for RobotState {
  fn default() -> Self {
    Self::Teleop
  }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct DriverStationReport {
  pub team: u16,
  pub robot_ping: bool,
  pub rio_ping: bool,
  pub radio_ping: bool,
  pub battery_voltage: f64,

  pub estop: bool,
  pub mode: RobotState,

  pub pkts_sent: u16,
  pub pkts_lost: u16,
  pub rtt: u8,

  pub actual_station: Option<AllianceStationId>
}

impl DriverStationReport {
  pub fn diagnosis(&self) -> Option<&'static str> {
    if self.estop { return Some("ESTOP") }
    if !self.radio_ping  { return Some("RADIO") }
    if !self.rio_ping { return Some("RIO") }
    if !self.robot_ping { return Some("CODE") }

    if self.rtt > 100 { return Some("L8NC") }
    if self.battery_voltage < 9.0 { return Some("LBATT") }

    return None
  }
}

impl Table for DriverStationReport {
  const PREFIX: &'static str = "ds";
  type Id = u16;
  type Err = ParseIntError;

  fn id(&self) -> Self::Id {
    self.team
  }
}

#[derive(Debug)]
pub enum TournamentLevel {
  Test = 0,
  #[allow(dead_code)]
  Practice = 1,
  Qualification = 2,
  Playoff = 3,
}