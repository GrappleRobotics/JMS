use std::{convert::Infallible, str::FromStr, num::ParseIntError, path::Display};

use chrono::Local;

use jms_base::kv;
use serde::Serialize;
use strum::ParseError;

use crate::db::Table;

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
  pub start_time: Option<chrono::DateTime<Local>>,
  pub match_type: MatchType,
  pub match_subtype: Option<MatchSubtype>,

  pub set_number: usize,
  pub match_number: usize,
  
  pub blue_teams: Vec<Option<usize>>,
  pub blue_alliance: Option<usize>,
  pub red_teams: Vec<Option<usize>>,
  pub red_alliance: Option<usize>,

  pub winner: Option<Alliance>, // Will be None if tie, but means nothing if the match isn't played yet
  pub played: bool,
  pub ready: bool,
}

// To send to frontend, as the impls of serde::Serialize are for DB storage and not
// transport to frontend (which requires name() and id()) to be called.
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct SerializedMatch {
  #[serde(flatten)]
  pub match_meta: Match,
  pub id: String,
  pub name: String,
  pub short_name: String,
}

impl Match {
  pub fn new_test() -> Self {
    Match {
      start_time: Some(chrono::Local::now().into()),
      match_type: MatchType::Test,
      match_subtype: None,
      set_number: 1,
      match_number: 1,
      blue_teams: vec![None, None, None],
      blue_alliance: None,
      red_teams: vec![None, None, None],
      red_alliance: None,
      winner: None,
      played: false,
      ready: true,
    }
  }

  pub fn name(&self) -> String {
    match self.match_type {
      MatchType::Test => "Test Match".to_owned(),
      MatchType::Qualification => format!("Qualification {}", self.match_number),
      MatchType::Playoff => match self.match_subtype.unwrap() {
        MatchSubtype::Quarterfinal => format!("Quarterfinal {}-{}", self.set_number, self.match_number),
        MatchSubtype::Semifinal => format!("Semifinal {}-{}", self.set_number, self.match_number),
        MatchSubtype::Final => format!("Final {}-{}", self.set_number, self.match_number),
      },
    }
  }

  pub fn short_name(&self) -> String {
    match self.match_type {
      MatchType::Test => "Test".to_owned(),
      MatchType::Qualification => format!("Q{}", self.match_number),
      MatchType::Playoff => match self.match_subtype.unwrap() {
        MatchSubtype::Quarterfinal => format!("QF{}-{}", self.set_number, self.match_number),
        MatchSubtype::Semifinal => format!("SF{}-{}", self.set_number, self.match_number),
        MatchSubtype::Final => format!("F{}-{}", self.set_number, self.match_number),
      },
    }
  }

  pub fn has_team(&self, team: usize) -> bool {
    self.red_teams.iter().chain(self.blue_teams.iter()).find(|&ot| (*ot) == Some(team)).is_some()
  }

  // pub fn by_type(mtype: MatchType, store: &db::Store) -> db::Result<Vec<Match>> {
  //   let mut v = Self::table(store)?.iter_values().filter(|a| {
  //     a.as_ref().map(|sb| sb.match_type == mtype ).unwrap_or(false)
  //   }).collect::<db::Result<Vec<Match>>>()?;
  //   v.sort_by(|a, b| a.start_time.cmp(&b.start_time));
  //   Ok(v)
  // }

  // pub fn by_set_match(mtype: MatchType, st: Option<MatchSubtype>, set: usize, match_num: usize, store: &db::Store) -> db::Result<Option<Match>> {
  //   Ok(Self::table(store)?.iter_values().find_map(|a| {
  //     a.ok().filter(|sb| sb.match_type == mtype && sb.match_subtype == st && sb.set_number == set && sb.match_number == match_num)
  //   }))
  // }

  pub fn reset(&mut self) {
    self.played = false;
    self.winner = None;
  }

  pub fn subtype_idx(&self) -> usize {
    match self.match_subtype {
      None => 0,
      Some(MatchSubtype::Quarterfinal) => 1,
      Some(MatchSubtype::Semifinal) => 2,
      Some(MatchSubtype::Final) => 3,
    }
  }

  // pub async fn commit<'a>(&'a mut self, score: &MatchScore, db: &db::Store) -> db::Result<&'a Self> {
  //   let red = score.red.derive(&score.blue);
  //   let blue = score.blue.derive(&score.red);

  //   let mut winner = None;
  //   if blue.win_status == WinStatus::WIN {
  //     winner = Some(Alliance::Blue);
  //   } else if red.win_status == WinStatus::WIN {
  //     winner = Some(Alliance::Red);
  //   }

  //   self.played = true;
  //   self.winner = winner;
  //   self.score = Some(score.clone());
  //   self.score_time = Some(chrono::Local::now().into());

  //   if self.match_type != MatchType::Test {
  //     self.insert(db)?;

  //     if self.match_type == MatchType::Playoff {
  //       // TODO: This should be event based
  //       let worker = MatchGenerationWorker::new(PlayoffMatchGenerator::new());
  //       let record = worker.record();
  //       if let Some(record) = record {
  //         if let Some(MatchGenerationRecordData::Playoff { mode }) = record.data {
  //           worker.generate(mode).await;
  //         }
  //       }
  //     }
  //   }

  //   Ok(self)
  // }
}

#[async_trait::async_trait]
impl Table for Match {
  const PREFIX: &'static str = "db:match";
  type Id = String;
  type Err = Infallible;

  fn id(&self) -> String {
    match self.match_type {
      MatchType::Test => format!("test"),
      MatchType::Qualification => format!("qm{}", self.match_number),
      MatchType::Playoff => match self.match_subtype.unwrap() {
        MatchSubtype::Quarterfinal => format!("qf{}m{}", self.set_number, self.match_number),
        MatchSubtype::Semifinal => format!("sf{}m{}", self.set_number, self.match_number),
        MatchSubtype::Final => format!("f{}m{}", self.set_number, self.match_number),
      },
    }
  }
}

impl From<Match> for SerializedMatch {
  fn from(m: Match) -> Self {
    Self {
      id: m.id(),
      name: m.name(),
      short_name: m.short_name(),
      match_meta: m
    }
  }
}

pub fn serialize_match<S>(m: &Match, s: S) -> Result<S::Ok, S::Error>
where
  S: serde::Serializer
{
  SerializedMatch::from(m.clone()).serialize(s)
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub enum PlayoffMode {
  Bracket,
  DoubleBracket,
  RoundRobin,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct MatchGenerationRecord {
  pub match_type: MatchType,
  pub data: Option<MatchGenerationRecordData>,
}

#[async_trait::async_trait]
impl Table for MatchGenerationRecord {
  const PREFIX: &'static str = "db:match_gen";
  type Id = MatchType;
  type Err = ParseError;

  fn id(&self) -> MatchType {
    self.match_type
  }
}

impl MatchGenerationRecord {
  pub fn get_by(match_type: MatchType, db: &kv::KVConnection) -> anyhow::Result<Option<MatchGenerationRecordData>> {
    let first = Self::get(&match_type, db).ok();

    match first {
      Some(mgr) => Ok(mgr.data),
      None => {
        let mgr = MatchGenerationRecord { match_type, data: None };
        mgr.insert(db)?;
        Ok(mgr.data)
      },
    }
  }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, schemars::JsonSchema)]
pub enum MatchGenerationRecordData {
  Qualification {
    team_balance: f64,
    station_balance: f64,
    cooccurrence: Vec<Vec<usize>>,
    station_dist: Vec<Vec<usize>>,
  },
  Playoff {
    mode: PlayoffMode,
  },
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
