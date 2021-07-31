use serde::{Deserialize, Serialize};
use std::{convert::TryInto, fmt::Display};
use crate::models::Alliance;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize, Hash)]
pub struct AllianceStationId {
  pub alliance: Alliance,
  pub station: u32,
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

impl Into<u8> for AllianceStationId {
  fn into(self) -> u8 {
    // Note that for teams per alliance > 3, this will result in multiple alliances per
    // station ID. Thus, these are not unique.
    let stn: u8 = ((self.station - 1) % 3).try_into().unwrap();
    match self.alliance {
      Alliance::Red => stn,
      Alliance::Blue => stn + 3,
    }
  }
}
