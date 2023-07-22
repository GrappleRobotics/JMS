use crate::db;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct Team {
  pub id: String,
  pub number: usize,
  pub name: Option<String>,
  pub affiliation: Option<String>,
  pub location: Option<String>,
  pub notes: Option<String>,
  pub wpakey: Option<String>,
  pub schedule: bool,
}

#[async_trait::async_trait]
impl db::Table for Team {
  const PREFIX: &'static str = "db:team";

  fn id(&self) -> String {
    self.id.clone()
  }
}

impl Team {
  pub fn maybe_gen_wpa(&self) -> Team {
    if self.wpakey.is_some() {
      return self.clone()
    } else {
      use rand::distributions::Alphanumeric;
      use rand::{thread_rng, Rng};
      
      let mut new_team = self.clone();
      new_team.wpakey = Some(
        thread_rng()
          .sample_iter(&Alphanumeric)
          .take(30)
          .map(char::from)
          .collect()
      );
      return new_team
    }
  }
}
