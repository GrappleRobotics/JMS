use std::{convert::Infallible, str::FromStr, num::ParseIntError};

use chrono::Local;

use jms_base::kv;

use crate::{db::{Table, DBDuration, Singleton}, scoring::scores::MatchScore};

#[derive(Debug, strum::EnumString, strum::Display, strum::EnumIter, Hash, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all="snake_case")]
pub enum Alliance {
  Blue, Red
}

#[derive(Debug, Clone)]
pub enum AllianceParseError {
  InvalidAlliance(String),
  InvalidStation(ParseIntError)
}

impl std::fmt::Display for AllianceParseError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      AllianceParseError::InvalidAlliance(s) => write!(f, "Invalid Alliance: {}", s),
      AllianceParseError::InvalidStation(e) => write!(f, "Could not parse station int: {}", e),
    }
  }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Hash, schemars::JsonSchema)]
pub struct AllianceStationId {
  pub alliance: Alliance,
  pub station: usize,
}

impl AllianceStationId {
  pub fn new(alliance: Alliance, station: usize) -> Self {
    Self { alliance, station }
  }

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
    format!("{}{}", self.alliance.to_string().to_lowercase(), self.station)
  }

  pub fn all() -> Vec<AllianceStationId> {
    vec![
      Self::new(Alliance::Blue, 1), Self::new(Alliance::Blue, 2), Self::new(Alliance::Blue, 3),
      Self::new(Alliance::Red, 1), Self::new(Alliance::Red, 2), Self::new(Alliance::Red, 3),
    ]
  }
}

impl ToString for AllianceStationId {
  fn to_string(&self) -> String {
    self.to_id()
  }
}

impl FromStr for AllianceStationId {
  type Err = AllianceParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s.len() < 4 {
      return Err(AllianceParseError::InvalidAlliance(s.to_owned()))
    }

    if &s[0..=2] == "red" && s.len() > 3 {
      let n: usize = s[3..].parse().map_err(|e| AllianceParseError::InvalidStation(e))?;
      Ok(AllianceStationId { alliance: Alliance::Red, station: n })
    } else if &s[0..=3] == "blue" && s.len() > 4 {
      let n: usize = s[4..].parse().map_err(|e| AllianceParseError::InvalidStation(e))?;
      Ok(AllianceStationId { alliance: Alliance::Blue, station: n })
    } else {
      Err(AllianceParseError::InvalidAlliance(s.to_owned()))
    }
  }
}

#[derive(Debug, strum::EnumString, strum::Display, Hash, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub enum MatchType {
  Test, Qualification, Playoff
}

#[derive(Debug, strum::EnumString, strum::Display, Hash, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub enum MatchSubtype {
  Quarterfinal, Semifinal, Final
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct Match {
  pub id: String,
  pub name: String,
  pub start_time: chrono::DateTime<Local>,
  pub match_type: MatchType,
  pub match_subtype: Option<MatchSubtype>,

  pub set_number: usize,
  pub match_number: usize,
  
  pub blue_teams: Vec<Option<usize>>,
  pub blue_alliance: Option<usize>,
  pub red_teams: Vec<Option<usize>>,
  pub red_alliance: Option<usize>,

  pub played: bool,
}

impl Match {
  pub fn new_test() -> Self {
    Match {
      id: "test".to_owned(),
      name: "Test Match".to_owned(),
      start_time: chrono::Local::now().into(),
      match_type: MatchType::Test,
      match_subtype: None,
      set_number: 1,
      match_number: 1,
      blue_teams: vec![None, None, None],
      blue_alliance: None,
      red_teams: vec![None, None, None],
      red_alliance: None,
      played: false,
    }
  }

  pub fn gen_id(ty: MatchType, st: Option<MatchSubtype>, set: usize, match_n: usize) -> String {
    match ty {
      MatchType::Test => format!("test"),
      MatchType::Qualification => format!("qm{}", match_n),
      MatchType::Playoff => match st.unwrap() {
        MatchSubtype::Quarterfinal => format!("qf{}m{}", set, match_n),
        MatchSubtype::Semifinal => format!("sf{}m{}", set, match_n),
        MatchSubtype::Final => format!("f{}m{}", set, match_n),
      },
    }
  }

  pub fn gen_name(ty: MatchType, st: Option<MatchSubtype>, set: usize, match_n: usize) -> String {
    match ty {
      MatchType::Test => format!("Test Match"),
      MatchType::Qualification => format!("Qualification {}", match_n),
      MatchType::Playoff => match st.unwrap() {
        MatchSubtype::Quarterfinal => format!("Quarterfinal {}-{}", set, match_n),
        MatchSubtype::Semifinal => format!("Semifinal {}-{}", set, match_n),
        MatchSubtype::Final => format!("Final {}-{}", set, match_n),
      },
    }
  }

  pub fn has_team(&self, team: usize) -> bool {
    self.red_teams.iter().chain(self.blue_teams.iter()).find(|&ot| (*ot) == Some(team)).is_some()
  }

  pub fn reset(&mut self) {
    self.played = false;
  }

  pub fn subtype_idx(&self) -> usize {
    match self.match_subtype {
      None => 0,
      Some(MatchSubtype::Quarterfinal) => 1,
      Some(MatchSubtype::Semifinal) => 2,
      Some(MatchSubtype::Final) => 3,
    }
  }

  pub fn sorted(db: &kv::KVConnection) -> anyhow::Result<Vec<Self>> {
    let mut v = Self::all(db)?;
    v.sort_by(|a, b| a.start_time.cmp(&b.start_time));
    Ok(v)
  }
}

#[async_trait::async_trait]
impl Table for Match {
  const PREFIX: &'static str = "db:match";
  type Id = String;
  type Err = Infallible;

  fn id(&self) -> String {
    self.id.clone()
  }
}

pub fn n_sets(level: MatchSubtype) -> usize {
  match level {
    MatchSubtype::Quarterfinal => 4,
    MatchSubtype::Semifinal => 2,
    MatchSubtype::Final => 1,
  }
}

impl Ord for MatchSubtype {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    let us_n = n_sets(*self);
    let them_n = n_sets(*other);

    them_n.cmp(&us_n)
  }
}

impl PartialOrd for MatchSubtype {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for Match {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self
      .start_time
      .cmp(&other.start_time)
      .then(self.match_subtype.cmp(&other.match_subtype))
      .then(self.match_number.cmp(&other.match_number))
      .then(self.set_number.cmp(&other.set_number))
  }
}

impl PartialOrd for Match {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(&other))
  }
}

impl Eq for Match {}

impl PartialEq for Match {
  fn eq(&self, other: &Self) -> bool {
    self.match_type == other.match_type && self.match_subtype == other.match_subtype && self.match_number == other.match_number && self.set_number == other.match_number
  }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(tag = "mode")]
pub enum PlayoffMode {
  Bracket { n_alliances: usize },
  DoubleBracket { n_alliances: usize, awards: Vec<String>, time_per_award: DBDuration },
  RoundRobin { n_alliances: usize },
}

impl Default for PlayoffMode {
  fn default() -> Self {
    Self::DoubleBracket { n_alliances: 8, awards: vec![], time_per_award: DBDuration(chrono::Duration::minutes(5)) }
  }
}

impl Singleton for PlayoffMode {
  const KEY: &'static str = "db:playoff_mode";
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct CommittedMatchScores {
  pub match_id: String,
  pub scores: Vec<MatchScore>
}

impl Table for CommittedMatchScores {
  const PREFIX: &'static str = "db:scores";
  type Err = Infallible;
  type Id = String;

  fn id(&self) -> Self::Id {
    self.match_id.clone()
  }
}