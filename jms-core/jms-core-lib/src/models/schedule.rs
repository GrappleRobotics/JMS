use std::convert::Infallible;

use jms_base::kv;
use uuid::Uuid;

use crate::db::{Table, DBDuration};

use super::PlayoffMode;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(tag = "type")]
pub enum ScheduleBlockType {
  General,
  Ceremonies,
  Lunch,
  FieldTests,
  SetupTeardown,
  Qualification {
    cycle_time: DBDuration
  },
  Playoff
}

#[derive(jms_macros::Updateable)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct ScheduleBlock {
  pub id: String,
  pub block_type: ScheduleBlockType,
  pub name: String,
  pub start_time: chrono::DateTime<chrono::Local>,
  pub end_time: chrono::DateTime<chrono::Local>,
}

impl Table for ScheduleBlock {
  const PREFIX: &'static str = "db:schedule:block";
  type Id = String;
  type Err = Infallible;

  fn id(&self) -> String {
    self.id.clone()
  }
}

impl ScheduleBlock {
  pub fn new(block_type: ScheduleBlockType, name: String, start: chrono::DateTime<chrono::Local>, end: chrono::DateTime<chrono::Local>) -> Self {
    Self {
      id: Uuid::new_v4().to_string(),
      block_type,
      name,
      start_time: start,
      end_time: end
    }
  }

  pub fn num_qual_matches(&self) -> usize {
    let duration = self.end_time - self.start_time;
    match &self.block_type {
      ScheduleBlockType::Qualification { cycle_time } => {
        (duration.num_seconds() / cycle_time.0.num_seconds()) as usize
      },
      _ => 0,
    }
  }

  pub fn sorted(db: &kv::KVConnection) -> anyhow::Result<Vec<ScheduleBlock>> {
    let mut v = Self::all(db)?;
    v.sort_by(|a, b| a.start_time.cmp(&b.start_time));
    Ok(v)
  }
}
