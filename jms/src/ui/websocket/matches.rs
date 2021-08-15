use anyhow::bail;

use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};

use crate::{
  db, models,
  schedule::{playoffs::PlayoffMatchGenerator, quals::QualsMatchGenerator, worker::MatchGenerationWorker},
  ui::websocket::JsonMessage,
};

use super::WebsocketMessageHandler;
pub struct MatchWebsocketHandler {
  quals: MatchGenerationWorker<QualsMatchGenerator>,
  playoffs: MatchGenerationWorker<PlayoffMatchGenerator>,
}

impl MatchWebsocketHandler {
  pub fn new() -> Self {
    MatchWebsocketHandler {
      quals: MatchGenerationWorker::new(QualsMatchGenerator::new()),
      playoffs: MatchGenerationWorker::new(PlayoffMatchGenerator::new()),
    }
  }
}

#[async_trait::async_trait]
impl WebsocketMessageHandler for MatchWebsocketHandler {
  async fn update(&mut self) -> super::Result<Vec<JsonMessage>> {
    let msg = JsonMessage::update("matches", "");
    let mut response = vec![];
    {
      // Quals
      response.push(msg.noun("quals").to_data(&self.quals)?);
    }
    {
      // Playoffs
      response.push(msg.noun("playoffs").to_data(&self.playoffs)?);
    }
    {
      // Next Match
      use crate::schema::matches::dsl::*;
      let next_match = matches
        .filter(played.eq(false))
        .first::<models::Match>(&db::connection())
        .optional()?;
      response.push(msg.noun("next").to_data(&next_match)?);
    }
    {
      // Last Match
      use crate::schema::matches::dsl::*;
      let last_match = matches
        .filter(played.eq(true))
        .order_by(score_time.desc())
        .first::<models::Match>(&db::connection())
        .optional()?;
      response.push(msg.noun("last").to_data(&last_match)?);
    }
    Ok(response)
  }

  async fn handle(&mut self, msg: JsonMessage) -> super::Result<Vec<JsonMessage>> {
    // let response_msg = msg.response();

    match msg.noun.as_str() {
      "quals" => match (msg.verb.as_str(), msg.data) {
        ("clear", None) => {
          if self.quals.has_played() {
            bail!("Cannot delete after matches have started")
          } else {
            self.quals.delete();
          }
        }
        ("generate", Some(data)) => {
          let params = serde_json::from_value(data)?;
          self.quals.generate(params).await;
        }
        _ => bail!("Invalid verb or data"),
      },
      "playoffs" => match (msg.verb.as_str(), msg.data) {
        ("clear", None) => {
          if self.playoffs.has_played() {
            bail!("Cannot delete after matches have started")
          } else {
            self.playoffs.delete();
          }
        }
        ("generate", Some(data)) => {
          let params = serde_json::from_value(data)?;
          self.playoffs.generate(params).await;
        }
        _ => bail!("Invalid verb or data"),
      },
      _ => bail!("Unknown noun"),
    }
    Ok(vec![])
  }
}
