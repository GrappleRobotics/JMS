use crate::schema::teams;

#[derive(Insertable, Queryable, Debug, Clone)]
pub struct Team {
  pub id: i32,
  pub name: String,
  pub affiliation: Option<String>,
  pub location: Option<String>,
  pub notes: Option<String>,
}
