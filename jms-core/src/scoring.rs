use std::time::Duration;

use jms_base::{mq, kv};
use jms_core_lib::{models::{self, TeamRanking}, db::{Table, Singleton}, scoring::scores::MatchScore};
use log::error;

use crate::schedule::playoffs::PlayoffMatchGenerator;

pub struct ScoringService {
  pub kv: kv::KVConnection,
  pub mq: mq::MessageQueueChannel
}

impl ScoringService {
  pub async fn run(self) -> anyhow::Result<()> {
    let mut publish_sub: mq::MessageQueueSubscriber<String> = self.mq.subscribe("arena.scores.publish", "core-scoring-publish", "ScoringService", false).await?;
    let mut ranking_update_interval = tokio::time::interval(Duration::from_millis(10*60*1000)); // 10 mins, just in case it doesn't get triggered elsewhere
    loop {
      tokio::select! {
        _ = ranking_update_interval.tick() => {
          TeamRanking::update(&self.kv)?;
          PlayoffMatchGenerator::update(&self.kv)?;
        },
        msg = publish_sub.next() => match msg {
          Some(Ok(td)) => {
            let mut c = match models::CommittedMatchScores::get(&td.data, &self.kv) {
              Ok(c) => c,
              Err(_) => models::CommittedMatchScores { match_id: td.data, scores: vec![], last_update: chrono::Local::now() }
            };
            c.push_and_insert(MatchScore::get(&self.kv)?, &self.kv)?;
            MatchScore::delete(&self.kv)?;    // Reset the scores once they're committed

            // Update the playoffs bracket
            PlayoffMatchGenerator::update(&self.kv)?;
          },
          Some(Err(e)) => error!("Error: {}", e),
          None => ()
        }
      }
    }
  }
}