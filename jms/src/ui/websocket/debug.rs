use anyhow::bail;
use jms_macros::define_websocket_msg;
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};

use crate::{db::{self, TableType}, models, scoring::scores::{LiveScore, MatchScore}};

use super::WebsocketMessageHandler;

define_websocket_msg!($DebugMessage {
  recv $Match {
    FillRandomScores,
    DeleteAll
  }
});

// #[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
// pub enum MatchDebugMessage {
//   FillRandom,
//   DeleteAll
// }

// #[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
// pub enum DebugMessage {
//   Matches(MatchDebugMessage)
// }

pub struct DebugWebsocketHandler {}

impl DebugWebsocketHandler {
  pub fn new() -> Self {
    DebugWebsocketHandler {}
  }
}

#[async_trait::async_trait]
impl WebsocketMessageHandler for DebugWebsocketHandler {
  async fn update(&mut self) -> anyhow::Result<Vec<super::JsonMessage>> {
    Ok(vec![])
  }

  async fn handle(&mut self, msg: super::JsonMessage) -> anyhow::Result<Vec<super::JsonMessage>> {
    match msg.noun.as_str() {
      "matches" => match (msg.verb.as_str(), msg.data) {
        ("fill_rand", None) => {
          // Fill the matches with random bogus scores (for testing)
          for mut m in models::Match::all(&db::database())? {
            if !m.played {
              let score = MatchScore {
                red: LiveScore::randomise(),
                blue: LiveScore::randomise()
              };

              m.commit(&score, &db::database()).await?;
            }
          }
        },
        ("delete_all", None) => {
          // Deletes all matches (DANGEROUS - ONLY FOR TESTING)
          models::Match::table(&db::database())?.clear()?;
        },
        _ => bail!("Invalid verb or data")
      },
      _ => bail!("Invalid verb or data")
    }
    Ok(vec![])
  }
}