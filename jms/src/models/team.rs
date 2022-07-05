use crate::db;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct Team {
  pub id: usize,
  pub name: Option<String>,
  pub affiliation: Option<String>,
  pub location: Option<String>,
  pub notes: Option<String>,
  pub wpakey: Option<String>,
  pub schedule: bool,
}

impl db::TableType for Team {
  const TABLE: &'static str = "teams";
  type Id = db::Integer;

  fn id(&self) -> Option<Self::Id> {
    Some(self.id.into())
  }

  fn set_id(&mut self, id: Self::Id) {
    self.id = id.into()
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

// impl Team {
//   pub fn wpakey(team_id: usize, conn: &db::ConnectionT) -> Option<String> {
//     use crate::schema::teams::dsl::*;

//     match teams.find(team_id as i32).first::<Team>(conn) {
//       Ok(t) => t.wpakey,
//       Err(_) => None,
//     }
//   }
// }
