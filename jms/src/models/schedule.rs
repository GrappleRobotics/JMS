use chrono::{Duration, Local, NaiveTime, TimeZone};
use diesel::{QueryDsl, QueryResult, RunQueryDsl, ExpressionMethods};

use crate::{db, schema::schedule_blocks};

use super::{SQLDuration, SQLDatetime};

#[derive(Insertable, Queryable, Debug, Clone, AsChangeset, serde::Serialize, serde::Deserialize)]
#[changeset_options(treat_none_as_null="true")]
pub struct ScheduleBlock {
  pub id: i32,
  pub name: String,
  // #[serde(deserialize_with = "super::deserialize_naivedatetime", serialize_with = "super::serialize_naivedatetime")]
  pub start_time: SQLDatetime,
  // #[serde(deserialize_with = "super::deserialize_naivedatetime", serialize_with = "super::serialize_naivedatetime")]
  pub end_time: SQLDatetime,
  pub cycle_time: SQLDuration,
  pub quals: bool
}

impl ScheduleBlock {
  pub fn num_matches(&self) -> usize {
    let duration = self.end_time.0 - self.start_time.0;
    (duration.num_seconds() / self.cycle_time.0.num_seconds()) as usize
  }

  pub fn qual_blocks(conn: &db::ConnectionT) -> QueryResult<Vec<ScheduleBlock>> {
    use crate::schema::schedule_blocks::dsl::*;
    schedule_blocks.filter(quals.eq(true))
                   .order_by(start_time.asc())
                   .load::<ScheduleBlock>(conn)
  }

  pub fn append_default(conn: &db::ConnectionT) -> QueryResult<()> {
    // TODO: Validate, can't do it if the schedule is locked in
    use crate::schema::schedule_blocks::dsl::*;
    let mut start = Local::today().and_hms(9, 00, 00);

    match schedule_blocks.order(id.desc()).first::<ScheduleBlock>(&db::connection()) {
      Ok(sb) => {
        let end = Local.from_utc_datetime(&sb.end_time.0);
        let new_start = end + Duration::hours(1);
        let new_end = new_start + Duration::hours(3);

        if new_end.time() >= NaiveTime::from_hms(17, 00, 00) {
          // Automatically move to tomorrow
          start = (end + Duration::days(1)).date().and_hms(9, 00, 00);
        } else {
          start = new_start;
        }
      },
      Err(diesel::NotFound) => (),
      Err(e) => return Err(e),
    }

    diesel::insert_into(schedule_blocks)
      .values((
        start_time.eq(SQLDatetime(start.naive_utc())),
        end_time.eq(SQLDatetime((start + Duration::hours(3)).naive_utc())),
        cycle_time.eq(SQLDuration(Duration::minutes(13)))
      ))
      .execute(conn)?;

    Ok(())
  }

  pub fn generate_default_2day(conn: &db::ConnectionT) -> QueryResult<()> {
    use crate::schema::schedule_blocks::dsl::*;
    let day1 = Local::today() + Duration::days(1);
    let day2 = day1 + Duration::days(1);
    
    // Clear any existing
    diesel::delete(schedule_blocks).execute(conn)?;

    // Generate the new blocks
    diesel::insert_into(schedule_blocks)
      .values(&vec![
        // Day 1
        (
          name.eq("Opening Ceremony"),
          start_time.eq(SQLDatetime::from(day1.and_hms(08, 30, 00))),
          end_time.eq(SQLDatetime::from(day1.and_hms(09, 00, 00))),
          quals.eq(false)
        ),
        (
          name.eq("Field Tests & Practice"), 
          start_time.eq(SQLDatetime::from(day1.and_hms(09, 00, 00))),
          end_time.eq(SQLDatetime::from(day1.and_hms(12, 00, 00))),
          quals.eq(false)
        ),
        (
          name.eq("Qualifications"),
          start_time.eq(SQLDatetime::from(day1.and_hms(13, 00, 00))),
          end_time.eq(SQLDatetime::from(day1.and_hms(17, 00, 00))),
          quals.eq(true)
        ),
        (
          name.eq("Awards & Closing Ceremony"),
          start_time.eq(SQLDatetime::from(day1.and_hms(17, 30, 00))),
          end_time.eq(SQLDatetime::from(day1.and_hms(18, 00, 00))),
          quals.eq(false)
        ),
        // Day 2
        (
          name.eq("Opening Ceremony"),
          start_time.eq(SQLDatetime::from(day2.and_hms(08, 30, 00))),
          end_time.eq(SQLDatetime::from(day2.and_hms(09, 00, 00))),
          quals.eq(false)
        ),
        (
          name.eq("Qualifications (cont.)"), 
          start_time.eq(SQLDatetime::from(day2.and_hms(09, 00, 00))),
          end_time.eq(SQLDatetime::from(day2.and_hms(12, 00, 00))),
          quals.eq(true)
        ),
        (
          name.eq("Alliance Selection"),
          start_time.eq(SQLDatetime::from(day2.and_hms(12, 00, 00))),
          end_time.eq(SQLDatetime::from(day2.and_hms(12, 30, 00))),
          quals.eq(false)
        ),
        (
          name.eq("Playoffs"),
          start_time.eq(SQLDatetime::from(day2.and_hms(13, 30, 00))),
          end_time.eq(SQLDatetime::from(day2.and_hms(17, 00, 00))),
          quals.eq(false)
        ),
        (
          name.eq("Awards & Closing Ceremony"),
          start_time.eq(SQLDatetime::from(day2.and_hms(17, 30, 00))),
          end_time.eq(SQLDatetime::from(day2.and_hms(18, 00, 00))),
          quals.eq(false)
        )
      ])
      .execute(conn)?;
    Ok(())
  }
}