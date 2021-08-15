use crate::schema::awards;

use super::SQLJsonVector;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AwardRecipient {
  pub team: Option<usize>,
  pub awardee: Option<String>,
}

#[derive(
  Identifiable, Insertable, Queryable, Associations, AsChangeset, Debug, Clone, serde::Serialize, serde::Deserialize,
)]
pub struct Award {
  pub id: i32,
  pub name: String,
  pub recipients: SQLJsonVector<AwardRecipient>,
}
