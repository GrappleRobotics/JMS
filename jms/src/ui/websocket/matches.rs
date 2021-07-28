use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

use crate::{db, models, schedule::{quals::QualsMatchGenerator, worker::MatchGenerationWorker}, ui::websocket::JsonMessage};

use super::WebsocketMessageHandler;
pub struct MatchWebsocketHandler {
  quals: MatchGenerationWorker<QualsMatchGenerator>
}

impl MatchWebsocketHandler {
  pub fn new() -> Self {
    MatchWebsocketHandler {
      quals: MatchGenerationWorker::new(QualsMatchGenerator::new())
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
      // Next Match
      use crate::schema::matches::dsl::*;
      let next_match = matches.filter(played.eq(false)).first::<models::Match>(&db::connection())?;
      response.push(msg.noun("next").to_data(&next_match)?);
    }
    Ok(response)
  }

  async fn handle(&mut self, msg: JsonMessage) -> super::Result<Vec<JsonMessage>> {
    let response_msg = msg.response();

    match msg.noun.as_str() {
      "quals" => match (msg.verb.as_str(), msg.data) {
        ("clear", None) => {
          self.quals.delete();
        },
        ("generate", Some(data)) => {
          let params = serde_json::from_value(data)?;
          self.quals.generate(params).await;
        },
        _ => Err(response_msg.invalid_verb_or_data())?
      },
      _ => Err(response_msg.invalid_verb_or_data())?
    }
    Ok(vec![])
  }
}