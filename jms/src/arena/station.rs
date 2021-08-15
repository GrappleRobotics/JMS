use crate::models::Alliance;
use serde::{Deserialize, Serialize};
use std::{convert::TryInto, fmt::Display};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize, Hash)]
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
