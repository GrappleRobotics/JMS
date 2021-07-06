use std::convert::TryFrom;

mod udp_codec;
mod tcp_codec;
pub use udp_codec::*;
pub use tcp_codec::*;

pub mod connector;

#[derive(Debug, Clone)]
pub enum DSMode {
  Teleop = 0,
  Test = 1,
  Auto = 2
}

impl Default for DSMode {
  fn default() -> Self { DSMode::Teleop }
}

impl TryFrom<u8> for DSMode {
  type Error = ();

  fn try_from(value: u8) -> Result<Self, Self::Error> {
    match value {
      x if x == DSMode::Teleop as u8 => Ok(DSMode::Teleop),
      x if x == DSMode::Test as u8 => Ok(DSMode::Test),
      x if x == DSMode::Auto as u8 => Ok(DSMode::Auto),
      _ => Err(())
    }
  }
}

#[derive(Debug)]
pub enum TournamentLevel {
  Test = 0,
  Practice = 1,
  Qualification = 2,
  Playoff = 3
}