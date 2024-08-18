use std::{convert::Infallible, str::FromStr, num::ParseIntError};

use chrono::Local;

use jms_base::kv;

use crate::{db::{Table, DBDuration, Singleton}, scoring::scores::MatchScore};

use super::TeamRanking;

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
  Test, Qualification, Playoff, Final
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct Match {
  pub id: String,
  pub name: String,
  pub start_time: chrono::DateTime<Local>,
  pub match_type: MatchType,

  pub round: usize,
  pub set_number: usize,
  pub match_number: usize,
  
  pub blue_teams: Vec<Option<usize>>,
  pub blue_alliance: Option<usize>,
  pub red_teams: Vec<Option<usize>>,
  pub red_alliance: Option<usize>,
  pub dqs: Vec<usize>,

  pub played: bool,
  pub ready: bool,
}

impl Match {
  // We work a little differently to TBA and the official FMS. We mark Qualification matches with set number since they may be replayed, 
  // in which case the match number increases. You can think of it as if the teams on each alliance are the same, the set number is the same, otherwise
  // it is different. The match number increments for each 'replay'.
  // We also have a generic "Playoff" type, that covers Bracket, Double Bracket, and Round-Robin through the use of a 'round' number as opposed
  // to the qf, sf marking that you may be used to. We separate finals into their own type since they may be different to the playoff type (finals are bracket, 
  // where playoffs may be a bracket, double bracket, or round robin). 
  // TODO: Implement Qualification replay elsewhere, only include the latest replay (match) number in rankings.

  pub fn gen_id(ty: MatchType, round: usize, set: usize, match_n: usize) -> String {
    match ty {
      MatchType::Test => format!("test{}m{}", set, match_n),
      MatchType::Qualification => format!("qm{}m{}", set, match_n),
      MatchType::Playoff => format!("el{}s{}m{}", round, set, match_n),
      MatchType::Final => format!("f{}", match_n)
    }
  }

  pub fn gen_name(ty: MatchType, round: usize, set: usize, match_n: usize) -> String {
    match ty {
      MatchType::Test => format!("Test Match {}-{}", set, match_n),
      MatchType::Qualification if match_n == 1 => format!("Qualification {}", set),
      MatchType::Qualification => format!("Qualification {} (replay {})", set, match_n - 1),
      MatchType::Playoff => format!("Elimination Round {} - {}-{}", round, set, match_n),
      MatchType::Final => format!("Final {}", match_n)
    }
  }

  pub fn has_team(&self, team: usize) -> bool {
    self.red_teams.iter().chain(self.blue_teams.iter()).find(|&ot| (*ot) == Some(team)).is_some()
  }

  pub fn reset(&mut self) {
    self.played = false;
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

impl Ord for Match {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self
      .start_time
      .cmp(&other.start_time)
      .then(self.round.cmp(&other.round))
      .then(self.match_number.cmp(&other.match_number))
      .then(self.set_number.cmp(&other.set_number))
  }
}

impl PartialOrd for Match {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(&other))
  }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct PlayoffMode {
  pub mode: PlayoffModeType,
  pub n_alliances: usize,
  pub awards: Vec<String>,
  pub time_per_award: DBDuration,
  pub minimum_round_break: DBDuration
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub enum PlayoffModeType {
  Bracket,
  DoubleBracket
}

impl Default for PlayoffMode {
  fn default() -> Self {
    Self {
      mode: PlayoffModeType::DoubleBracket,
      n_alliances: 8,
      awards: vec![],
      time_per_award: DBDuration(chrono::Duration::minutes(5)),
      minimum_round_break: DBDuration(chrono::Duration::minutes(8))
    }
  }
}

impl Singleton for PlayoffMode {
  const KEY: &'static str = "db:playoff_mode";
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct CommittedMatchScores {
  pub match_id: String,
  pub scores: Vec<MatchScore>,
  pub last_update: chrono::DateTime<chrono::Local>
}

impl Table for CommittedMatchScores {
  const PREFIX: &'static str = "db:scores";
  type Err = Infallible;
  type Id = String;

  fn id(&self) -> Self::Id {
    self.match_id.clone()
  }
}

impl CommittedMatchScores {
  pub fn push_and_insert(&mut self, mut score: MatchScore, kv: &kv::KVConnection) -> anyhow::Result<()> {
    // Update match played status
    let mut m = Match::get(&self.match_id, kv)?;
    m.played = true;

    // Propagate any DQ's
    if m.match_type == MatchType::Playoff || m.match_type == MatchType::Final {
      for dq in &m.dqs {
        let is_red = m.red_teams.contains(&Some(*dq));
        let is_blue = m.blue_teams.contains(&Some(*dq));

        if is_red {
          score.red.is_dq = true;
        }

        if is_blue {
          score.blue.is_dq = true;
        }
      }
    }

    // Push the score into the committed record
    self.scores.push(score);
    self.last_update = Local::now();
    self.insert(kv)?;

    // Update the match played status
    m.insert(kv)?;

    // Update team rankings
    TeamRanking::update(kv)?;

    Ok(())
  }
}