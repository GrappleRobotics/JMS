use std::num::ParseIntError;

use jms_base::kv;

use crate::db::{self, Table};

#[derive(jms_macros::Updateable)]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct Team {
  pub number: usize,
  pub display_number: String,
  pub name: Option<String>,
  pub affiliation: Option<String>,
  pub location: Option<String>,
  pub notes: Option<String>,
  pub wpakey: String,
  pub schedule: bool,
}

#[async_trait::async_trait]
impl db::Table for Team {
  const PREFIX: &'static str = "db:team";
  type Id = usize;
  type Err = ParseIntError;

  fn id(&self) -> Self::Id {
    self.number
  }
}

impl Team {
  pub fn new(number: usize, display_number: String, name: Option<String>, affiliation: Option<String>, location: Option<String>) -> Self {
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
      
    let wpakey = thread_rng()
      .sample_iter(&Alphanumeric)
      .take(30)
      .map(char::from)
      .collect();

    Self {
      number, display_number,
      name, affiliation, location,
      notes: None, wpakey, schedule: true
    }
  }

  pub fn sorted(kv: &kv::KVConnection) -> anyhow::Result<Vec<Team>> {
    let mut teams = Self::all(kv)?;
    teams.sort_by(|a, b| a.number.cmp(&b.number));
    Ok(teams)
  }

  pub fn display_number(team_number: usize, kv: &kv::KVConnection) -> String {
    match Self::get(&team_number, kv) {
      Ok(t) => t.display_number,
      Err(_) => format!("{}", team_number)
    }
  }

  pub fn display_number_for(team: &String, kv: &kv::KVConnection) -> String {
    match team.parse() {
      Ok(team_number) => Self::display_number(team_number, kv),
      Err(_) => team.clone()
    }
  }
}
