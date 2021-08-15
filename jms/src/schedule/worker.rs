use std::{
  error::Error,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
};

use diesel::{BelongingToDsl, ExpressionMethods, QueryDsl, RunQueryDsl};
use log::error;
use serde::{ser::SerializeStruct, Serialize, Serializer};

use crate::{
  db,
  models::{self, MatchGenerationRecord},
};

pub struct MatchGenerationWorker<T>
where
  T: MatchGenerator + Send + Sync + 'static,
  <T as MatchGenerator>::ParamType: Send,
{
  running: Arc<AtomicBool>,
  generator: T,
}

impl<T> MatchGenerationWorker<T>
where
  T: MatchGenerator + Send + Sync + Clone,
  <T as MatchGenerator>::ParamType: Send,
{
  pub fn new(gen: T) -> Self {
    Self {
      running: Arc::new(AtomicBool::new(false)),
      generator: gen,
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
    match_generation_records
      .find(self.match_type())
      .first::<MatchGenerationRecord>(&db::connection())
      .ok()
  }

  pub fn matches(&self) -> Vec<models::Match> {
    models::Match::with_type(self.match_type())
    // self.record().map(|record| {
    //   models::Match::belonging_to(&record).load::<models::Match>(&db::connection()).unwrap()
    // }).unwrap_or(vec![])
  }

  pub fn has_played(&self) -> bool {
    self
      .record()
      .map(|record| {
        use crate::schema::matches::dsl::*;
        models::Match::belonging_to(&record)
          .filter(played.eq(true))
          .count()
          .get_result::<i64>(&db::connection())
          .unwrap()
          > 0
      })
      .unwrap_or(false)
  }

  pub fn delete(&self) {
    {
      use crate::schema::match_generation_records::dsl::*;
      diesel::delete(match_generation_records.find(self.match_type()))
        .execute(&db::connection())
        .unwrap();
    }
    {
      use crate::schema::matches::dsl::*;
      diesel::delete(matches.filter(match_type.eq(self.match_type())))
        .execute(&db::connection())
        .unwrap();
    }
  }

  pub async fn generate(&self, params: T::ParamType) {
    let running = self.running.clone();
    let gen = self.generator.clone();
    let record = self.record();
    tokio::spawn(async move {
      // *running.get_mut() = true;
      running.swap(true, Ordering::Relaxed);
      match gen.generate(params, record).await {
        Ok(_) => (),
        Err(e) => error!("Match Generation Error: {}", e),
      }
      // *running.get_mut() = false;
      running.swap(false, Ordering::Relaxed);
    });
  }
}

impl<T> Serialize for MatchGenerationWorker<T>
where
  T: MatchGenerator + Send + Sync + Clone + 'static,
  <T as MatchGenerator>::ParamType: Send,
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
  type ParamType;

  fn match_type(&self) -> models::MatchType;
  async fn generate(
    &self,
    params: Self::ParamType,
    record: Option<MatchGenerationRecord>,
  ) -> Result<(), Box<dyn Error>>;
}
