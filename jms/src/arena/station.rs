use crate::{models::Alliance, ds::DSMode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{convert::TryInto, fmt::Display};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize, Hash, JsonSchema)]
pub struct AllianceStationId {
  pub alliance: Alliance,
  pub station: u32,
}

impl AllianceStationId {
  // To station idx, where 0-2 are Blue 1-3, and 3-5 are Red 4-6
  pub fn to_station_idx(&self) -> u8 {
    let stn: u8 = ((self.station - 1) % 3).try_into().unwrap();

    // 0, 1, 2 = Blue 1, 2, 3
    // 3, 4, 5 = Red  1, 2, 3
    match self.alliance {
      Alliance::Blue => stn,
      Alliance::Red => stn + 3,
    }
  }

  pub fn to_ds_number(&self) -> u8 {
    let stn: u8 = ((self.station - 1) % 3).try_into().unwrap();

    // Driver Station uses a different format, where Red is seen as 0, 1, 2
    match self.alliance {
      Alliance::Blue => stn + 3,
      Alliance::Red => stn,
    }
  }

  pub fn to_id(&self) -> String {
    format!("{}{}", self.alliance, self.station)
  }
}

impl AllianceStationId {
  pub fn blue1() -> AllianceStationId {
    AllianceStationId {
      alliance: Alliance::Blue,
      station: 1,
    }
  }
}

impl Display for AllianceStationId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{} {}", self.alliance.to_string(), self.station)
  }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum AllianceStationOccupancy {
  Vacant,
  Occupied,
  WrongStation,
  WrongMatch,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub struct AllianceStationDSReport {
  pub robot_ping: bool,
  pub rio_ping: bool,
  pub radio_ping: bool,
  pub battery: f64,

  pub estop: bool,
  pub mode: Option<DSMode>,

  pub pkts_sent: u16,
  pub pkts_lost: u16,
  pub rtt: u8,
}

impl Default for AllianceStationDSReport {
  fn default() -> Self {
    Self {
      robot_ping: false,
      rio_ping: false,
      radio_ping: false,
      battery: 0.0f64,
      estop: false,
      mode: None,
      pkts_sent: 0,
      pkts_lost: 0,
      rtt: 0,
    }
  }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub struct AllianceStation {
  pub station: AllianceStationId,
  pub team: Option<u16>,
  pub bypass: bool,
  pub estop: bool,
  pub astop: bool,
  pub ds_eth: bool,
  pub ds_report: Option<AllianceStationDSReport>,
  pub occupancy: AllianceStationOccupancy,
}

#[derive(Debug, Clone, Copy, Serialize, JsonSchema)]
pub struct SerialisedAllianceStation {
  #[serde(flatten)]
  pub s: AllianceStation,
  pub can_arm: bool
}

impl AllianceStation {
  pub fn new(id: AllianceStationId) -> AllianceStation {
    return AllianceStation {
      station: id,
      team: None,
      bypass: false,
      estop: false,
      astop: false,
      ds_eth: false,
      ds_report: None,
      occupancy: AllianceStationOccupancy::Vacant,
    };
  }

  pub fn reset(&mut self) {
    self.team = None;
    self.bypass = false;
    self.estop = false;
    self.astop = false;
    self.ds_report = None;
    self.occupancy = AllianceStationOccupancy::Vacant;
  }

  pub fn can_arm_match(&self) -> bool {
    self.bypass || self.estop || (self.occupancy == AllianceStationOccupancy::Occupied)
  }

  pub fn connection_ok(&self) -> bool {
    let mut ok = true;
    match &self.ds_report {
      Some(ds) => {
        if !ds.robot_ping || !ds.rio_ping || !ds.radio_ping {
          ok = false;
        }
      },
      None => ok = false
    }
    ok
  }
}

impl From<AllianceStation> for SerialisedAllianceStation {
  fn from(stn: AllianceStation) -> Self {
    SerialisedAllianceStation { 
      can_arm: stn.can_arm_match(),
      s: stn
    }
  }
}
