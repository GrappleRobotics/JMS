use std::{
  error::Error,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
};

use log::error;
use schemars::JsonSchema;
use serde::Serialize;

use crate::{db::{self, TableType}, models::{self, MatchGenerationRecord}};

#[derive(Debug, Clone)]
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
    MatchGenerationRecord::get(self.match_type(), &db::database()).ok()
  }

  pub fn matches(&self) -> Vec<models::Match> {
    let mut matches = models::Match::by_type(self.match_type(), &db::database()).unwrap();
    matches.sort();
    matches
  }

  pub fn has_played(&self) -> bool {
    self.matches().iter().any(|t| t.played)
  }

  pub fn delete(&self) {
    #[allow(unused_must_use)]
    if let Some(record) = self.record() {
      record.remove(&db::database());
    }

    #[allow(unused_must_use)]
    for m in self.matches() {
      m.remove(&db::database());
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

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct SerialisedMatchGeneration {
  running: bool,
  matches: Vec<models::SerializedMatch>,
  record: Option<MatchGenerationRecord>
}

impl<T> From<&MatchGenerationWorker<T>> for SerialisedMatchGeneration
where
  T: MatchGenerator + Send + Sync + Clone,
  <T as MatchGenerator>::ParamType: Send
{
  fn from(worker: &MatchGenerationWorker<T>) -> Self {
    SerialisedMatchGeneration { 
      running: worker.running(), 
      matches: worker.matches().iter().map(|m| models::SerializedMatch::from(m.clone())).collect::<Vec<models::SerializedMatch>>(),
      record: worker.record()
    }
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

pub struct MatchGenerators {
  pub quals: MatchGenerationWorker<super::quals::QualsMatchGenerator>,
  pub playoffs: MatchGenerationWorker<super::playoffs::PlayoffMatchGenerator>,
}

pub type SharedMatchGenerators = std::sync::Arc<tokio::sync::Mutex<MatchGenerators>>;