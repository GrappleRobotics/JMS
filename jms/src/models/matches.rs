use crate::{db, models::SQLJson, schema::match_generation_records, schema::matches, scoring::scores::MatchScore, sql_mapped_enum};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use serde::{Serialize, Serializer, ser::SerializeStruct};

use super::{SQLDatetime, SQLJsonVector};

// #[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Hash)]
// pub enum Alliance {
//   Blue,
//   Red,
// }

sql_mapped_enum!(Alliance, Blue, Red);
sql_mapped_enum!(MatchType, Test, Qualification, Playoff);
sql_mapped_enum!(MatchSubtype, Quarterfinal, Semifinal, Final);

#[derive(Identifiable, Insertable, Queryable, Associations, AsChangeset, Debug, Clone)]
#[belongs_to(MatchGenerationRecord, foreign_key="match_type")]
#[table_name = "matches"]
pub struct Match {
  pub id: i32,
  pub start_time: Option<SQLDatetime>,
  pub match_type: MatchType,
  pub set_number: i32,
  pub match_number: i32,
  // Usually, these would be in a many-to-many join table, but we want to be able to make test matches
  // without committing to the database. It's not neat, but it's the most convenient option for our goals.
  pub blue_teams: SQLJsonVector<Option<i32>>,
  pub red_teams: SQLJsonVector<Option<i32>>,
  pub played: bool,
  pub score: Option<SQLJson<MatchScore>>,
  pub winner: Option<Alliance>,   // Will be None if tie, but means nothing if the match isn't played yet
  // Playoffs only
  pub match_subtype: Option<MatchSubtype>,
  pub red_alliance: Option<i32>,
  pub blue_alliance: Option<i32>
}

impl Match {
  pub fn new_test() -> Self {
    Match {
      id: -1,
      start_time: Some(SQLDatetime(chrono::Local::now().naive_utc())),
      match_type: MatchType::Test,
      set_number: 1,
      match_number: 1,
      blue_teams: SQLJson(vec![None, None, None]),
      red_teams: SQLJson(vec![None, None, None]),
      played: false,
      score: None,
      winner: None,
      match_subtype: None,
      red_alliance: None,
      blue_alliance: None
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
      }
    }
  }

  pub fn with_type(mtype: MatchType) -> Vec<Match> {
    use crate::schema::matches::dsl::*;
    matches.filter(match_type.eq(mtype)).load::<Match>(&db::connection()).unwrap()
  }
}

impl Serialize for Match {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
      S: Serializer,
{
    let mut state = serializer.serialize_struct("Match", 14)?;
    state.serialize_field("id", &self.id)?;
    state.serialize_field("type", &self.match_type)?;
    state.serialize_field("subtype", &self.match_subtype)?;
    state.serialize_field("time", &self.start_time)?;
    state.serialize_field("name", &self.name())?;
    state.serialize_field("set_number", &self.set_number)?;
    state.serialize_field("match_number", &self.match_number)?;
    state.serialize_field("blue", &self.blue_teams)?;
    state.serialize_field("blue_alliance", &self.blue_alliance)?;
    state.serialize_field("red", &self.red_teams)?;
    state.serialize_field("red_alliance", &self.red_alliance)?;
    state.serialize_field("played", &self.played)?;
    state.serialize_field("score", &self.score)?;
    state.serialize_field("winner", &self.winner)?;
    state.end()
  }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum PlayoffMode {
  Bracket,
  RoundRobin
}

#[derive(Identifiable, Insertable, Queryable, Debug, Clone, serde::Serialize)]
#[primary_key(match_type)]
pub struct MatchGenerationRecord {
  pub match_type: MatchType,
  pub data: Option<SQLJson<MatchGenerationRecordData>>
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum MatchGenerationRecordData {
  Qualification {
    team_balance: f64,
    station_balance: f64,
    cooccurrence: SQLJsonVector<Vec<usize>>,
    station_dist: SQLJsonVector<Vec<usize>>,
  },
  Playoff {
    mode: PlayoffMode
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
    self.start_time.cmp(&other.start_time)
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

impl Eq for Match { }

impl PartialEq for Match {
  fn eq(&self, other: &Self) -> bool {
    self.id == other.id
  }
}