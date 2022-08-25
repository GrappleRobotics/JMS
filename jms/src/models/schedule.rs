use chrono::{Date, Duration, Local, NaiveTime, TimeZone};

use crate::db::{self, DBDateTime, DBDuration, TableType};

#[derive(Debug, strum_macros::EnumString, Display, Hash, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub enum ScheduleBlockType {
  General,
  Qualification,
  Playoff
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct ScheduleBlock {
  pub id: Option<u64>,
  pub block_type: ScheduleBlockType,
  pub name: String,
  pub start_time: DBDateTime,
  pub end_time: DBDateTime,
  pub cycle_time: DBDuration,
}

impl db::TableType for ScheduleBlock {
  const TABLE: &'static str = "schedule_blocks";
  type Id = db::Integer;

  fn id(&self) -> Option<Self::Id> {
    self.id.map(|id| id.into())
  }

  fn generate_id(&mut self, store: &db::Store) -> db::Result<()> {
    self.id = Some(store.generate_id()?);
    Ok(())
  }
}

impl ScheduleBlock {
  pub fn num_matches(&self) -> usize {
    let duration = self.end_time.0 - self.start_time.0;
    (duration.num_seconds() / self.cycle_time.0.num_seconds()) as usize
  }

  pub fn by_type(block_type: ScheduleBlockType, store: &db::Store) -> db::Result<Vec<ScheduleBlock>> {
    let mut v: Vec<ScheduleBlock> = Self::table(store)?.iter_values().filter(|a| {
      a.as_ref().map(|sb| sb.block_type == block_type ).unwrap_or(false)
    }).collect::<db::Result<Vec<ScheduleBlock>>>()?;
    v.sort_by(|a, b| a.start_time.cmp(&b.start_time));
    Ok(v)
  }

  pub fn sorted(store: &db::Store) -> db::Result<Vec<ScheduleBlock>> {
    let mut v = Self::all(store)?;
    v.sort_by(|a, b| a.start_time.cmp(&b.start_time));
    Ok(v)
  }

  pub fn qual_blocks(store: &db::Store) -> db::Result<Vec<ScheduleBlock>> {
    Self::by_type(ScheduleBlockType::Qualification, store)
  }

  // pub fn playoff_blocks(store: &db::Store) -> db::Result<Vec<ScheduleBlock>> {
  //   Self::by_type(ScheduleBlockType::Playoff, store)
  // }

  pub fn append_default(store: &db::Store) -> db::Result<()> {
    // TODO: Validate, can't do it if the schedule is locked in
    let mut start = Local::today().and_hms(9, 00, 00);

    let mut all = Self::all(store)?;
    all.sort_by(|a, b| a.start_time.cmp(&b.start_time));

    if let Some(last) = all.last() {
      let end = Local.from_utc_datetime(&last.end_time.0);
      let new_start = end + Duration::hours(1);
      let new_end = new_start + Duration::hours(3);

      if new_end.time() >= NaiveTime::from_hms(17, 00, 00) {
        // Automatically move to tomorrow
        start = (end + Duration::days(1)).date().and_hms(9, 00, 00);
      } else {
        start = new_start;
      }
    }

    let mut sb = ScheduleBlock {
      id: None,
      name: "Unnamed Block".to_owned(),
      block_type: ScheduleBlockType::General,
      start_time: start.into(),
      end_time: (start + Duration::hours(3)).into(),
      cycle_time: DBDuration(Duration::minutes(13))
    };

    sb.insert(store)?;

    Ok(())
  }

  pub fn generate_default_2day(start_date: Date<Local>, store: &db::Store) -> db::Result<()> {
    // use crate::schema::schedule_blocks::dsl::*;
    let day1 = start_date;
    let day2 = day1 + Duration::days(1);

    // Clear any existing
    Self::table(store)?.clear()?;
    // diesel::delete(schedule_blocks).execute(conn)?;

    let cycle_time = DBDuration(Duration::minutes(13));
    
    // Day 1
    ScheduleBlock {
      id: None, name: "Opening Ceremony".to_owned(),
      block_type: ScheduleBlockType::General,
      start_time: day1.and_hms(08, 30, 00).into(),
      end_time: day1.and_hms(09, 00, 00).into(),
      cycle_time: cycle_time.clone()
    }.insert(store)?;
    ScheduleBlock {
      id: None, name: "Field Tests & Practice".to_owned(),
      block_type: ScheduleBlockType::General,
      start_time: day1.and_hms(09, 00, 00).into(),
      end_time: day1.and_hms(12, 00, 00).into(),
      cycle_time: cycle_time.clone()
    }.insert(store)?;
    ScheduleBlock {
      id: None, name: "Qualifications".to_owned(),
      block_type: ScheduleBlockType::Qualification,
      start_time: day1.and_hms(13, 00, 00).into(),
      end_time: day1.and_hms(17, 00, 00).into(),
      cycle_time: cycle_time.clone()
    }.insert(store)?;
    ScheduleBlock {
      id: None, name: "Awards & Closing Ceremony".to_owned(),
      block_type: ScheduleBlockType::General,
      start_time: day1.and_hms(17, 30, 00).into(),
      end_time: day1.and_hms(18, 00, 00).into(),
      cycle_time: cycle_time.clone()
    }.insert(store)?;

    // Day 2
    ScheduleBlock {
      id: None, name: "Opening Ceremony".to_owned(),
      block_type: ScheduleBlockType::General,
      start_time: day2.and_hms(08, 30, 00).into(),
      end_time: day2.and_hms(09, 00, 00).into(),
      cycle_time: cycle_time.clone()
    }.insert(store)?;
    ScheduleBlock {
      id: None, name: "Qualifications (cont'd)".to_owned(),
      block_type: ScheduleBlockType::Qualification,
      start_time: day2.and_hms(09, 00, 00).into(),
      end_time: day2.and_hms(12, 00, 00).into(),
      cycle_time: cycle_time.clone()
    }.insert(store)?;
    ScheduleBlock {
      id: None, name: "Alliance Selections".to_owned(),
      block_type: ScheduleBlockType::General,
      start_time: day2.and_hms(12, 00, 00).into(),
      end_time: day2.and_hms(12, 30, 00).into(),
      cycle_time: cycle_time.clone()
    }.insert(store)?;
    ScheduleBlock {
      id: None, name: "Playoffs".to_owned(),
      block_type: ScheduleBlockType::Playoff,
      start_time: day2.and_hms(13, 30, 00).into(),
      end_time: day2.and_hms(17, 00, 00).into(),
      cycle_time: cycle_time.clone()
    }.insert(store)?;
    ScheduleBlock {
      id: None, name: "Awards & Closing Ceremony".to_owned(),
      block_type: ScheduleBlockType::General,
      start_time: day2.and_hms(17, 30, 00).into(),
      end_time: day2.and_hms(18, 00, 00).into(),
      cycle_time: cycle_time.clone()
    }.insert(store)?;

    Ok(())
  }
}
