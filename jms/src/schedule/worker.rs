use std::{error::Error, sync::{Arc, atomic::{AtomicBool, Ordering}}};

use diesel::{QueryDsl, RunQueryDsl, ExpressionMethods, BelongingToDsl};
use log::error;
use serde::{Serialize, Serializer, ser::SerializeStruct};

use crate::{db, models::{self, MatchGenerationRecord, SQLDatetime, SQLJsonVector, ScheduleBlock}};

use super::{Annealer, GenerationResult, ScheduleGenerator, TeamSchedule};

pub struct MatchGenerationWorker<T>
  where T: MatchGenerator + Send + Sync + 'static
{
  running: Arc<AtomicBool>,
  generator: T
}

impl<T> MatchGenerationWorker<T>
  where T: MatchGenerator + Send + Sync + Clone
{
  pub fn new(gen: T) -> Self {
    Self {
      running: Arc::new(AtomicBool::new(false)),
      generator: gen
    }
  }

  pub fn running(&self) -> bool {
    self.running.load(Ordering::Relaxed)
  }

  pub fn match_type(&self) -> models::MatchType {
    self.generator.match_type()
  }

  pub fn record(&self) -> Option<MatchGenerationRecord> {
    use crate::schema::match_generation_records::dsl::*;
    // match_generation_records.filter(match_type.eq(self.match_type()))
    //   .first::<MatchGenerationRecord>(&db::connection())
    //   .ok()
    match_generation_records.find(self.match_type()).first::<MatchGenerationRecord>(&db::connection()).ok()
  }

  pub fn matches(&self) -> Vec<models::Match> {
    self.record().map(|record| {
      models::Match::belonging_to(&record).load::<models::Match>(&db::connection()).unwrap()
    }).unwrap_or(vec![])
  }

  pub fn locked(&self) -> bool {
    self.record().map(|record| {
      use crate::schema::matches::dsl::*;
      models::Match::belonging_to(&record).filter(played.eq(false)).count().get_result::<i64>(&db::connection()).unwrap() > 0
    }).unwrap_or(false)
  }

  pub fn delete(&self) {
    {
      use crate::schema::match_generation_records::dsl::*;
      diesel::delete(match_generation_records.find(self.match_type())).execute(&db::connection()).unwrap();
    }
    {
      use crate::schema::matches::dsl::*;
      diesel::delete(matches.filter(match_type.eq(self.match_type()))).execute(&db::connection()).unwrap();
    }
  }

  pub async fn generate(&self) {
    if !self.locked() {
      let running = self.running.clone();
      let gen = self.generator.clone();
      tokio::spawn(async move {
        // *running.get_mut() = true;
        running.swap(true, Ordering::Relaxed);
        match gen.generate().await {
          Ok(_) => (),
          Err(e) => error!("Match Generation Error: {}", e),
        }
        // *running.get_mut() = false;
        running.swap(false, Ordering::Relaxed);
      });
    }
  }
}

impl<T> Serialize for MatchGenerationWorker<T>
  where T: MatchGenerator + Send + Sync + Clone + 'static
{
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
      S: Serializer,
{
    let mut state = serializer.serialize_struct("MatchGenerationWorker", 10)?;
    state.serialize_field("running", &self.running())?;
    state.serialize_field("matches", &self.matches())?;
    state.serialize_field("record", &self.record())?;
    state.end()
  }
}

#[async_trait::async_trait]
pub trait MatchGenerator {
  fn match_type(&self) -> models::MatchType;
  async fn generate(&self) -> Result<(), Box<dyn Error>>;
}

#[derive(Clone)]
pub struct QualsMatchGenerator {
  team_balance_anneal: Annealer,
  station_balance_anneal: Annealer
}

impl QualsMatchGenerator {
  pub fn new() -> Self {
    Self {
      // team_balance_anneal: Annealer::new(1.0, 0.0, 200_000),
      // station_balance_anneal: Annealer::new(1.0, 0.0, 100_000)
      team_balance_anneal: Annealer::new(1.0, 0.0, 2_000),
      station_balance_anneal: Annealer::new(1.0, 0.0, 1_000)
    }
  }

  async fn commit_generation_record(&self, result: &GenerationResult) -> Result<(), Box<dyn Error>> {
    use crate::schema::match_generation_records::dsl::*;

    diesel::replace_into(match_generation_records)
      .values(MatchGenerationRecord {
        match_type: models::MatchType::Qualification,
        team_balance: Some(result.team_balance_score),
        station_balance: Some(result.station_balance_score),
        cooccurrence: Some(SQLJsonVector(result.cooccurrence.column_iter().map(|col| col.iter().cloned().collect::<Vec<usize>>() ).collect())),
        station_dist: Some(SQLJsonVector(result.station_dist.column_iter().map(|col| col.iter().cloned().collect::<Vec<usize>>() ).collect()))
      })
      .execute(&db::connection())?;

    Ok(())
  }

  async fn commit_matches(&self, schedule: &TeamSchedule) -> Result<(), Box<dyn Error>> {
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

    diesel::delete(matches.filter(match_type.eq(models::MatchType::Qualification))).execute(&db::connection())?;
    diesel::insert_into(matches).values(&match_vec).execute(&*db::connection())?;

    Ok(())
  }
}

#[async_trait::async_trait]
impl MatchGenerator for QualsMatchGenerator {
  fn match_type(&self) -> models::MatchType {
    models::MatchType::Qualification
  }

  async fn generate(&self) -> Result<(), Box<dyn Error>> {
    let teams = {
      use crate::schema::teams::dsl::*;
      teams.select(id).get_results::<i32>(&db::connection())?
    };

    // Generate
    let num_matches = ScheduleBlock::qual_blocks(&db::connection())?.iter().map(|x| x.num_matches()).sum();

    let generator = ScheduleGenerator::new(teams.len(), num_matches, 6);

    let generation_result = generator.generate(self.team_balance_anneal, self.station_balance_anneal);
    let team_sched = generation_result.schedule.contextualise(&teams.iter().map(|&x| x as u16).collect::<Vec<u16>>());

    // Commit
    self.commit_generation_record(&generation_result).await?;
    self.commit_matches(&team_sched).await?;

    Ok(())
  }
}