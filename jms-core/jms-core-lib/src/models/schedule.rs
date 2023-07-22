use jms_base::kv;

use crate::db::{Table, DBDuration};

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

  pub async fn by_type(block_type: ScheduleBlockType, db: &kv::KVConnection) -> anyhow::Result<Vec<ScheduleBlock>> {
    let v = Self::sorted(db).await?;
    Ok(v.into_iter().filter(|x| x.block_type == block_type).collect())
  }

  pub async fn sorted(db: &kv::KVConnection) -> anyhow::Result<Vec<ScheduleBlock>> {
    let mut v = Self::all(db).await?;
    v.sort_by(|a, b| a.start_time.cmp(&b.start_time));
    Ok(v)
  }

  // pub fn append_default(store: &db::Store) -> db::Result<()> {
  //   // TODO: Validate, can't do it if the schedule is locked in
  //   let mut start = Local::today().and_hms(9, 00, 00);

  //   let mut all = Self::all(store)?;
  //   all.sort_by(|a, b| a.start_time.cmp(&b.start_time));

  //   if let Some(last) = all.last() {
  //     let end = Local.from_utc_datetime(&last.end_time.0);
  //     let new_start = end + Duration::hours(1);
  //     let new_end = new_start + Duration::hours(3);

  //     if new_end.time() >= NaiveTime::from_hms_opt(17, 00, 00).unwrap() {
  //       // Automatically move to tomorrow
  //       start = (end + Duration::days(1)).date().and_hms(9, 00, 00);
  //     } else {
  //       start = new_start;
  //     }
  //   }

  //   let mut sb = ScheduleBlock {
  //     id: None,
  //     name: "Unnamed Block".to_owned(),
  //     block_type: ScheduleBlockType::General,
  //     start_time: start.into(),
  //     end_time: (start + Duration::hours(3)).into(),
  //     cycle_time: DBDuration(Duration::minutes(13))
  //   };

  //   sb.insert(store)?;

  //   Ok(())
  // }

  // pub fn generate_default_2day(start_date: Date<Local>, store: &db::Store) -> db::Result<()> {
  //   // use crate::schema::schedule_blocks::dsl::*;
  //   let day1 = start_date;
  //   let day2 = day1 + Duration::days(1);

  //   // Clear any existing
  //   Self::table(store)?.clear()?;
  //   // diesel::delete(schedule_blocks).execute(conn)?;

  //   let cycle_time = DBDuration(Duration::minutes(13));
    
  //   // Day 1
  //   ScheduleBlock {
  //     id: None, name: "Opening Ceremony".to_owned(),
  //     block_type: ScheduleBlockType::General,
  //     start_time: day1.and_hms_opt(08, 30, 00).unwrap().into(),
  //     end_time: day1.and_hms_opt(09, 00, 00).unwrap().into(),
  //     cycle_time: cycle_time.clone()
  //   }.insert(store)?;
  //   ScheduleBlock {
  //     id: None, name: "Field Tests & Practice".to_owned(),
  //     block_type: ScheduleBlockType::General,
  //     start_time: day1.and_hms_opt(09, 00, 00).unwrap().into(),
  //     end_time: day1.and_hms_opt(12, 00, 00).unwrap().into(),
  //     cycle_time: cycle_time.clone()
  //   }.insert(store)?;
  //   ScheduleBlock {
  //     id: None, name: "Qualifications".to_owned(),
  //     block_type: ScheduleBlockType::Qualification,
  //     start_time: day1.and_hms_opt(13, 00, 00).unwrap().into(),
  //     end_time: day1.and_hms_opt(17, 00, 00).unwrap().into(),
  //     cycle_time: cycle_time.clone()
  //   }.insert(store)?;
  //   ScheduleBlock {
  //     id: None, name: "Awards & Closing Ceremony".to_owned(),
  //     block_type: ScheduleBlockType::General,
  //     start_time: day1.and_hms_opt(17, 30, 00).unwrap().into(),
  //     end_time: day1.and_hms_opt(18, 00, 00).unwrap().into(),
  //     cycle_time: cycle_time.clone()
  //   }.insert(store)?;

  //   // Day 2
  //   ScheduleBlock {
  //     id: None, name: "Opening Ceremony".to_owned(),
  //     block_type: ScheduleBlockType::General,
  //     start_time: day2.and_hms_opt(08, 30, 00).unwrap().into(),
  //     end_time: day2.and_hms_opt(09, 00, 00).unwrap().into(),
  //     cycle_time: cycle_time.clone()
  //   }.insert(store)?;
  //   ScheduleBlock {
  //     id: None, name: "Qualifications (cont'd)".to_owned(),
  //     block_type: ScheduleBlockType::Qualification,
  //     start_time: day2.and_hms_opt(09, 00, 00).unwrap().into(),
  //     end_time: day2.and_hms_opt(12, 00, 00).unwrap().into(),
  //     cycle_time: cycle_time.clone()
  //   }.insert(store)?;
  //   ScheduleBlock {
  //     id: None, name: "Alliance Selections".to_owned(),
  //     block_type: ScheduleBlockType::General,
  //     start_time: day2.and_hms_opt(12, 00, 00).unwrap().into(),
  //     end_time: day2.and_hms_opt(12, 30, 00).unwrap().into(),
  //     cycle_time: cycle_time.clone()
  //   }.insert(store)?;
  //   ScheduleBlock {
  //     id: None, name: "Playoffs".to_owned(),
  //     block_type: ScheduleBlockType::Playoff,
  //     start_time: day2.and_hms_opt(13, 30, 00).unwrap().into(),
  //     end_time: day2.and_hms_opt(17, 00, 00).unwrap().into(),
  //     cycle_time: cycle_time.clone()
  //   }.insert(store)?;
  //   ScheduleBlock {
  //     id: None, name: "Awards & Closing Ceremony".to_owned(),
  //     block_type: ScheduleBlockType::General,
  //     start_time: day2.and_hms_opt(17, 30, 00).unwrap().into(),
  //     end_time: day2.and_hms_opt(18, 00, 00).unwrap().into(),
  //     cycle_time: cycle_time.clone()
  //   }.insert(store)?;

  //   Ok(())
  // }
}
