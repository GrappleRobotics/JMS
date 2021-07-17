use crate::{schema::matches, sql_mapped_enum};
use serde::{Serialize, Serializer, ser::SerializeStruct};

use super::{SQLDatetime, SQLJsonVector};

sql_mapped_enum!(MatchType, Test, Qualification, Quarterfinal, Semifinal, Final);

#[derive(Insertable, Queryable, Debug, Clone)]
#[table_name = "matches"]
pub struct Match {
  pub id: i32,
  pub start_time: SQLDatetime,
  pub match_type: MatchType,
  pub set_number: i32,
  pub match_number: i32,
  // Usually, these would be in a many-to-many join table, but we want to be able to make test matches
  // without committing to the database. It's not neat, but it's the most convenient option for our goals.
  pub blue_teams: SQLJsonVector<i32>,  // 0 if unoccupied
  pub red_teams: SQLJsonVector<i32>,
}

impl Match {
  pub fn new_test() -> Self {
    Match {
      id: -1,
      start_time: SQLDatetime(chrono::Local::now().naive_utc()),
      match_type: MatchType::Test,
      set_number: 1,
      match_number: 1,
      blue_teams: SQLJsonVector(vec![]),
      red_teams: SQLJsonVector(vec![]),
    }
  }

  pub fn name(&self) -> String {
    match self.match_type {
      MatchType::Test => "Test Match".to_owned(),
      MatchType::Qualification => format!("Qualification {}", self.match_number),
      MatchType::Quarterfinal => format!("Quarterfinal {}-{}", self.set_number, self.match_number),
      MatchType::Semifinal => format!("Semifinal {}-{}", self.set_number, self.match_number),
      MatchType::Final => format!("Final {}-{}", self.set_number, self.match_number),
    }
  }
}

impl Serialize for Match {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
      S: Serializer,
{
    let mut state = serializer.serialize_struct("Match", 7)?;
    state.serialize_field("type", &self.match_type)?;
    state.serialize_field("time", &self.start_time)?;
    state.serialize_field("name", &self.name())?;
    state.serialize_field("set_number", &self.set_number)?;
    state.serialize_field("match_number", &self.match_number)?;
    state.serialize_field("blue", &self.blue_teams)?;
    state.serialize_field("red", &self.red_teams)?;
    state.end()
  }
}