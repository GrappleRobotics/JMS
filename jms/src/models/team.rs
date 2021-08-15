use diesel::{QueryDsl, RunQueryDsl};

use crate::{db, schema::teams};

#[derive(Insertable, Queryable, Debug, Clone, AsChangeset, serde::Serialize, serde::Deserialize)]
#[changeset_options(treat_none_as_null = "true")]
pub struct Team {
  pub id: i32,
  pub name: Option<String>,
  pub affiliation: Option<String>,
  pub location: Option<String>,
  pub notes: Option<String>,
  pub wpakey: Option<String>,
}

impl Team {
  pub fn wpakey(team_id: usize, conn: &db::ConnectionT) -> Option<String> {
    use crate::schema::teams::dsl::*;

    match teams.find(team_id as i32).first::<Team>(conn) {
      Ok(t) => t.wpakey,
      Err(_) => None,
    }
  }
}
