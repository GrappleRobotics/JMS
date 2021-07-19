use crate::schema::teams;

#[derive(Insertable, Queryable, Debug, Clone, AsChangeset, serde::Serialize, serde::Deserialize)]
#[changeset_options(treat_none_as_null="true")]
pub struct Team {
  pub id: i32,
  pub name: Option<String>,
  pub affiliation: Option<String>,
  pub location: Option<String>,
  pub notes: Option<String>,
}
