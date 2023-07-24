use chrono::{Local, TimeZone, Duration, NaiveTime, Date};
use jms_base::kv;

use crate::db::{Table, DBDuration, generate_id};

#[derive(Debug, strum::EnumString, strum::Display, Hash, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub enum ScheduleBlockType {
  General,
  Qualification,
  Playoff
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct ScheduleBlock {
  pub id: String,
  pub block_type: ScheduleBlockType,
  pub name: String,
  pub start_time: chrono::DateTime<chrono::Local>,
  pub end_time: chrono::DateTime<chrono::Local>,
  pub cycle_time: DBDuration,
}

impl Table for ScheduleBlock {
  const PREFIX: &'static str = "db:schedule:block";

  fn id(&self) -> String {
    self.id.clone()
  }
}

impl ScheduleBlock {
  pub fn num_matches(&self) -> usize {
    let duration = self.end_time - self.start_time;
    (duration.num_seconds() / self.cycle_time.0.num_seconds()) as usize
  }

  pub fn by_type(block_type: ScheduleBlockType, db: &kv::KVConnection) -> anyhow::Result<Vec<ScheduleBlock>> {
    let v = Self::sorted(db)?;
    Ok(v.into_iter().filter(|x| x.block_type == block_type).collect())
  }

  pub fn sorted(db: &kv::KVConnection) -> anyhow::Result<Vec<ScheduleBlock>> {
    let mut v = Self::all(db)?;
    v.sort_by(|a, b| a.start_time.cmp(&b.start_time));
    Ok(v)
  }

  #[allow(deprecated)]
  pub fn append_default(db: &kv::KVConnection) -> anyhow::Result<()> {
    // TODO: Validate, can't do it if the schedule is locked in
    let mut start = Local::today().and_hms(9, 00, 00);

    let mut all = Self::all(db)?;
    all.sort_by(|a, b| a.start_time.cmp(&b.start_time));

    if let Some(last) = all.last() {
      let end = Local.from_utc_datetime(&last.end_time.naive_utc());
      let new_start = end + Duration::hours(1);
      let new_end = new_start + Duration::hours(3);

      if new_end.time() >= NaiveTime::from_hms_opt(17, 00, 00).unwrap() {
        // Automatically move to tomorrow
        start = (end + Duration::days(1)).date().and_hms(9, 00, 00);
      } else {
        start = new_start;
      }
    }

    let sb = ScheduleBlock {
      id: generate_id(),
      name: "Unnamed Block".to_owned(),
      block_type: ScheduleBlockType::General,
      start_time: start.into(),
      end_time: (start + Duration::hours(3)).into(),
      cycle_time: DBDuration(Duration::minutes(13))
    };

    sb.insert(db)?;

    Ok(())
  }

  #[allow(deprecated)]
  pub fn generate_default_2day(start_date: Date<Local>, db: &kv::KVConnection) -> anyhow::Result<()> {
    // use crate::schema::schedule_blocks::dsl::*;
    let day1 = start_date;
    let day2 = day1 + Duration::days(1);

    // Clear any existing
    // Self::table(store)?.clear()?;
    Self::clear(db)?;
    // diesel::delete(schedule_blocks).execute(conn)?;

    let cycle_time = DBDuration(Duration::minutes(13));
    
    // Day 1
    ScheduleBlock {
      id: generate_id(), name: "Opening Ceremony".to_owned(),
      block_type: ScheduleBlockType::General,
      start_time: day1.and_hms_opt(08, 30, 00).unwrap().into(),
      end_time: day1.and_hms_opt(09, 00, 00).unwrap().into(),
      cycle_time: cycle_time.clone()
    }.insert(db)?;
    ScheduleBlock {
      id: generate_id(), name: "Field Tests & Practice".to_owned(),
      block_type: ScheduleBlockType::General,
      start_time: day1.and_hms_opt(09, 00, 00).unwrap().into(),
      end_time: day1.and_hms_opt(12, 00, 00).unwrap().into(),
      cycle_time: cycle_time.clone()
    }.insert(db)?;
    ScheduleBlock {
      id: generate_id(), name: "Qualifications".to_owned(),
      block_type: ScheduleBlockType::Qualification,
      start_time: day1.and_hms_opt(13, 00, 00).unwrap().into(),
      end_time: day1.and_hms_opt(17, 00, 00).unwrap().into(),
      cycle_time: cycle_time.clone()
    }.insert(db)?;
    ScheduleBlock {
      id: generate_id(), name: "Awards & Closing Ceremony".to_owned(),
      block_type: ScheduleBlockType::General,
      start_time: day1.and_hms_opt(17, 30, 00).unwrap().into(),
      end_time: day1.and_hms_opt(18, 00, 00).unwrap().into(),
      cycle_time: cycle_time.clone()
    }.insert(db)?;

    // Day 2
    ScheduleBlock {
      id: generate_id(), name: "Opening Ceremony".to_owned(),
      block_type: ScheduleBlockType::General,
      start_time: day2.and_hms_opt(08, 30, 00).unwrap().into(),
      end_time: day2.and_hms_opt(09, 00, 00).unwrap().into(),
      cycle_time: cycle_time.clone()
    }.insert(db)?;
    ScheduleBlock {
      id: generate_id(), name: "Qualifications (cont'd)".to_owned(),
      block_type: ScheduleBlockType::Qualification,
      start_time: day2.and_hms_opt(09, 00, 00).unwrap().into(),
      end_time: day2.and_hms_opt(12, 00, 00).unwrap().into(),
      cycle_time: cycle_time.clone()
    }.insert(db)?;
    ScheduleBlock {
      id: generate_id(), name: "Alliance Selections".to_owned(),
      block_type: ScheduleBlockType::General,
      start_time: day2.and_hms_opt(12, 00, 00).unwrap().into(),
      end_time: day2.and_hms_opt(12, 30, 00).unwrap().into(),
      cycle_time: cycle_time.clone()
    }.insert(db)?;
    ScheduleBlock {
      id: generate_id(), name: "Playoffs".to_owned(),
      block_type: ScheduleBlockType::Playoff,
      start_time: day2.and_hms_opt(13, 30, 00).unwrap().into(),
      end_time: day2.and_hms_opt(17, 00, 00).unwrap().into(),
      cycle_time: cycle_time.clone()
    }.insert(db)?;
    ScheduleBlock {
      id: generate_id(), name: "Awards & Closing Ceremony".to_owned(),
      block_type: ScheduleBlockType::General,
      start_time: day2.and_hms_opt(17, 30, 00).unwrap().into(),
      end_time: day2.and_hms_opt(18, 00, 00).unwrap().into(),
      cycle_time: cycle_time.clone()
    }.insert(db)?;

    Ok(())
  }
}
