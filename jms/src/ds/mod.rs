use std::convert::TryFrom;

mod tcp_codec;
mod udp_codec;
pub use tcp_codec::*;
pub use udp_codec::*;

use crate::models;

pub mod connector;

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub enum DSMode {
  Teleop = 0,
  Test = 1,
  Auto = 2,
}

impl Default for DSMode {
  fn default() -> Self {
    DSMode::Teleop
  }
}

impl TryFrom<u8> for DSMode {
  type Error = ();

  fn try_from(value: u8) -> Result<Self, Self::Error> {
    match value {
      x if x == DSMode::Teleop as u8 => Ok(DSMode::Teleop),
      x if x == DSMode::Test as u8 => Ok(DSMode::Test),
      x if x == DSMode::Auto as u8 => Ok(DSMode::Auto),
      _ => Err(()),
    }
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

impl From<models::MatchType> for TournamentLevel {
  fn from(mt: models::MatchType) -> Self {
    match mt {
      models::MatchType::Test => TournamentLevel::Test,
      models::MatchType::Qualification => TournamentLevel::Qualification,
      models::MatchType::Playoff => TournamentLevel::Playoff,
    }
  }
}
