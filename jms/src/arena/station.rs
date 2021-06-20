use std::convert::TryInto;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq)]
pub enum Alliance {
  Blue,
  Red,
}

#[derive(Debug, Copy, Clone)]
pub struct AllianceStationId {
  pub alliance: Alliance,
  pub station: u32
}

impl Into<u8> for AllianceStationId {
  fn into(self) -> u8 {
    // Note that for teams per alliance > 3, this will result in multiple alliances per
    // station ID. Thus, these are not unique.
    let stn: u8 = ((self.station - 1) % 3).try_into().unwrap();
    match self.alliance {
      Alliance::Red => { stn },
      Alliance::Blue => { stn + 3 }
    }
  }
}