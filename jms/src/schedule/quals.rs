use chrono::NaiveDateTime;
use lazy_static::lazy_static;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, connection::SimpleConnection};
use log::info;
use serde::ser::SerializeStruct;
use tokio::sync::Mutex;

use crate::{db, models::{self, Match, SQLDatetime, SQLJsonVector, ScheduleBlock}};

use super::{Annealer, ScheduleGenerator, TeamSchedule};

lazy_static! {
  static ref QUAL_SCHED: Mutex<QualificationSchedule> = Mutex::new(QualificationSchedule::new());
}

pub struct QualificationSchedule {
  generating: bool
}

impl QualificationSchedule {
  fn new() -> Self {
    QualificationSchedule {
      generating: false
    }
  }

  pub fn instance() -> &'static Mutex<QualificationSchedule> {
    &QUAL_SCHED
  }

  pub fn locked(&self) -> bool {
    use crate::schema::matches::dsl::*;
    matches.filter(played.eq(true)).count().get_result::<i64>(&db::connection()).unwrap() > 0
  }

  pub fn exists(&self) -> bool {
    use crate::schema::matches::dsl::*;
    matches.count().get_result::<i64>(&db::connection()).unwrap() > 0
  }

  pub fn running(&self) -> bool {
    self.generating
  }

  pub fn matches(&self) -> Vec<Match> {
    use crate::schema::matches::dsl::*;
    matches.filter(match_type.eq(models::MatchType::Qualification)).load::<Match>(&db::connection()).unwrap()
  }

  pub fn clear(&self) {
    use crate::schema::matches::dsl::*;
    diesel::delete(matches).execute(&db::connection()).unwrap();
  }

  pub async fn generate(&self) {
    if !self.locked() { // TODO: Error if locked
      tokio::spawn(async move {
        QualificationSchedule::instance().lock().await.generating = true;
        let sched = generate_quals().await.unwrap();
        commit_quals(&sched).await.unwrap();
        QualificationSchedule::instance().lock().await.generating = false;
      });
    }
  }
}

impl serde::Serialize for QualificationSchedule {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer
  {
    let mut state = serializer.serialize_struct("QualificationSchedule", 4)?;
    state.serialize_field("running", &self.running())?;
    state.serialize_field("exists", &self.exists())?;
    state.serialize_field("matches", &self.matches())?;
    state.serialize_field("locked", &self.locked())?;
    state.end()
  }
}

async fn generate_quals() -> Result<TeamSchedule, Box<dyn std::error::Error>> {
  let teams = {
    use crate::schema::teams::dsl::*;
    teams.select(id).get_results::<i32>(&db::connection())?
  };

  let num_matches = ScheduleBlock::qual_blocks(&db::connection())?.iter().map(|x| x.num_matches()).sum();

  let generator = ScheduleGenerator::new(teams.len(), num_matches, 6);

  let anneal_team_balance = Annealer::new(1.0, 0.0, 100_000);
  let anneal_station_balance = Annealer::new(1.0, 0.0, 40_000);

  let (sched, _tb, _sb) = generator.generate(anneal_team_balance, anneal_station_balance);
  let team_sched = sched.contextualise(&teams.iter().map(|&x| x as u16).collect::<Vec<u16>>());

  Ok(team_sched)
}

async fn commit_quals(schedule: &TeamSchedule) -> Result<(), Box<dyn std::error::Error>> {
  use crate::schema::matches::dsl::*;
  let blocks = ScheduleBlock::qual_blocks(&db::connection())?;

  let mut match_vec = vec![];
  let mut match_i = 0usize;
  for block in blocks {
    for i in 0..block.num_matches() {
      let col = schedule.0.column(match_i);
      let teams = col.as_slice();
      let blue = teams[0..3].to_vec();
      let red = teams[3..6].to_vec();
      
      let start = block.start_time.0 + (block.cycle_time.0 * (i as i32));
      match_vec.push((
        start_time.eq(SQLDatetime(start)),
        match_type.eq(models::MatchType::Qualification),
        set_number.eq(0),
        match_number.eq((match_i + 1) as i32),
        blue_teams.eq(SQLJsonVector(blue)),
        red_teams.eq(SQLJsonVector(red)),
        played.eq(false)
      ));
      match_i += 1;
    }
  }

  diesel::delete(matches).execute(&db::connection())?;
  diesel::insert_into(matches).values(&match_vec).execute(&*db::connection())?;

  Ok(())
}