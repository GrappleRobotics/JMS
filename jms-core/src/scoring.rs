use jms_base::{mq, kv};
use jms_core_lib::{models, db::{Table, Singleton}, scoring::scores::MatchScore};
use log::error;

pub struct ScoringService {
  pub kv: kv::KVConnection,
  pub mq: mq::MessageQueueChannel
}

impl ScoringService {
  // TODO: Need to send something back to the arena to say that score publish was OK - maybe we generify it?
  // i.e. for each state, each component reports whether it is ready or not (if required)
  pub async fn run(self) -> anyhow::Result<()> {
    let mut publish_sub: mq::MessageQueueSubscriber<String> = self.mq.subscribe("arena.scores.publish", "core-scoring-publish", "ScoringService", false).await?;
    loop {
      tokio::select! {
        msg = publish_sub.next() => match msg {
          Some(Ok(td)) => {
            let mut c = match models::CommittedMatchScores::get(&td.data, &self.kv) {
              Ok(c) => c,
              Err(_) => models::CommittedMatchScores { match_id: td.data, scores: vec![] }
            };
            c.scores.push(MatchScore::get(&self.kv)?);
            c.insert(&self.kv)?;
            MatchScore::delete(&self.kv)?;    // Reset the scores once they're committed
          },
          Some(Err(e)) => error!("Error: {}", e),
          None => ()
        }
      }
    }
  }
}