use chrono::NaiveDateTime;

use crate::schema::schedule_blocks;

use super::SQLDuration;

#[derive(Insertable, Queryable, Debug, Clone, AsChangeset, serde::Serialize, serde::Deserialize)]
#[changeset_options(treat_none_as_null="true")]
pub struct ScheduleBlock {
  pub id: i32,
  pub name: String,
  #[serde(deserialize_with = "super::deserialize_naivedatetime", serialize_with = "super::serialize_naivedatetime")]
  pub start_time: NaiveDateTime,
  #[serde(deserialize_with = "super::deserialize_naivedatetime", serialize_with = "super::serialize_naivedatetime")]
  pub end_time: NaiveDateTime,
  pub cycle_time: SQLDuration,
  pub quals: bool
}