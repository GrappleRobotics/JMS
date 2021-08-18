use crate::db;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
