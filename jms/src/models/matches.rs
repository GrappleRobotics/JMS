use crate::{db::{self, DBDateTime, TableType, DBDuration}, schedule::{playoffs::PlayoffMatchGenerator, worker::MatchGenerationWorker}, scoring::scores::{MatchScore, WinStatus, MatchScoreSnapshot}};

use super::TeamRanking;

#[derive(Debug, strum_macros::EnumString, strum_macros::ToString, EnumIter, Hash, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub enum Alliance {
  Blue, Red
}

#[derive(Debug, strum_macros::EnumString, strum_macros::ToString, Hash, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub enum MatchType {
  Test, Qualification, Playoff
}

#[derive(Debug, strum_macros::EnumString, strum_macros::ToString, Hash, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub enum MatchSubtype {
  Quarterfinal, Semifinal, Final
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct Match {
  pub start_time: Option<DBDateTime>,
  pub match_type: MatchType,
  pub match_subtype: Option<MatchSubtype>,

  pub set_number: usize,
  pub match_number: usize,
  
  pub blue_teams: Vec<Option<usize>>,
  pub blue_alliance: Option<usize>,
  pub red_teams: Vec<Option<usize>>,
  pub red_alliance: Option<usize>,
  pub score: Option<MatchScore>,
  pub score_time: Option<DBDateTime>,

  pub winner: Option<Alliance>, // Will be None if tie, but means nothing if the match isn't played yet
  pub played: bool,
}

// To send to frontend, as the impls of serde::Serialize are for DB storage and not
// transport to frontend (which requires name() and id()) to be called.
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct SerializedMatch {
  #[serde(flatten)]
  pub match_meta: Match,
  pub id: Option<String>,
  pub name: String,
  pub full_score: Option<MatchScoreSnapshot>
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
      score: None,
      score_time: None,
      winner: None,
      played: false,
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

  pub fn by_type(mtype: MatchType, store: &db::Store) -> db::Result<Vec<Match>> {
    let mut v = Self::table(store)?.iter().filter(|a| {
      a.as_ref().map(|sb| sb.match_type == mtype ).unwrap_or(false)
    }).collect::<db::Result<Vec<Match>>>()?;
    v.sort_by(|a, b| a.start_time.cmp(&b.start_time));
    Ok(v)
  }

  pub fn sorted(store: &db::Store) -> db::Result<Vec<Match>> {
    let mut v = Self::all(store)?;
    v.sort_by(|a, b| a.start_time.cmp(&b.start_time));
    Ok(v)
  }

  pub async fn commit<'a>(&'a mut self, score: &MatchScore, db: &db::Store) -> db::Result<&'a Self> {
    let red = score.red.derive(&score.blue);
    let blue = score.blue.derive(&score.red);

    let mut winner = None;
    if blue.win_status == WinStatus::WIN {
      winner = Some(Alliance::Blue);
    } else if red.win_status == WinStatus::WIN {
      winner = Some(Alliance::Red);
    }

    self.played = true;
    self.winner = winner;
    self.score = Some(score.clone());
    self.score_time = Some(chrono::Local::now().into());

    if self.match_type != MatchType::Test {
      self.insert(db)?;

      if self.match_type == MatchType::Qualification {
        // Update rankings
        for team in &self.blue_teams {
          if let Some(team) = team {
            TeamRanking::get(*team, &db)?.update(&blue, &db)?;
          }
        }

        for team in &self.red_teams {
          if let Some(team) = team {
            TeamRanking::get(*team, &db)?.update(&red, &db)?;
          }
        }
      } else if self.match_type == MatchType::Playoff {
        // TODO: This should be event based
        let worker = MatchGenerationWorker::new(PlayoffMatchGenerator::new());
        let record = worker.record();
        if let Some(record) = record {
          if let Some(MatchGenerationRecordData::Playoff { mode }) = record.data {
            worker.generate(mode).await;
          }
        }
      }
    }

    Ok(self)
  }
}

impl db::TableType for Match {
  const TABLE: &'static str = "matches";
  type Id = String;

  fn id(&self) -> Option<Self::Id> {
    Some(match self.match_type {
      MatchType::Test => format!("test"),
      MatchType::Qualification => format!("qm{}", self.match_number),
      MatchType::Playoff => match self.match_subtype.unwrap() {
        MatchSubtype::Quarterfinal => format!("qf{}m{}", self.set_number, self.match_number),
        MatchSubtype::Semifinal => format!("sf{}m{}", self.set_number, self.match_number),
        MatchSubtype::Final => format!("f{}m{}", self.set_number, self.match_number),
      },
    })
  }

  fn set_id(&mut self, _id: Self::Id) { }
}

impl From<Match> for SerializedMatch {
  fn from(m: Match) -> Self {
    Self {
      id: m.id(),
      name: m.name(),
      full_score: m.score.clone().map(Into::into),
      match_meta: m
    }
  }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub enum PlayoffMode {
  Bracket,
  RoundRobin,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct MatchGenerationRecord {
  pub match_type: MatchType,
  pub data: Option<MatchGenerationRecordData>,
}

impl db::TableType for MatchGenerationRecord {
  const TABLE: &'static str = "match_generation_records";
  type Id = String;

  fn id(&self) -> Option<Self::Id> {
    Some(self.match_type.to_string())
  }

  fn set_id(&mut self, _id: Self::Id) { }
}

impl MatchGenerationRecord {
  pub fn get(match_type: MatchType, store: &db::Store) -> db::Result<Self> {
    let first = Self::table(store)?.get(match_type.to_string())?;

    match first {
      Some(mgr) => Ok(mgr),
      None => {
        let mut mgr = MatchGenerationRecord { match_type, data: None };
        mgr.insert(store)?;
        Ok(mgr)
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
    gen_time: DBDuration
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
