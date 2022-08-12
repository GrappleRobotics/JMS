use jms_macros::define_websocket_msg;

use crate::{db::{self, TableType}, models, scoring::scores::{LiveScore, MatchScore}};

use super::{ws::{WebsocketHandler, Websocket}, WebsocketMessage2JMS};

define_websocket_msg!($DebugMessage {
  recv $Match {
    FillRandomScores,
    DeleteAll
  },
  ReplyTest(String)
});

pub struct WSDebugHandler;

#[async_trait::async_trait]
impl WebsocketHandler for WSDebugHandler {
  async fn handle(&self, msg: &WebsocketMessage2JMS, ws: &mut Websocket) -> anyhow::Result<()> {
    if let WebsocketMessage2JMS::Debug(msg) = msg {
      if !ws.is_fta().await {
        anyhow::bail!("You need to be an FTA to do that!");
      }

      match msg.clone() {
        DebugMessage2JMS::Match(msg) => match msg {
          DebugMessageMatch2JMS::FillRandomScores => {
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
          DebugMessageMatch2JMS::DeleteAll => models::Match::table(&db::database())?.clear()?,
        },
        DebugMessage2JMS::ReplyTest(word) => { ws.reply::<DebugMessage2UI>(DebugMessage2UI::ReplyTest(word)).await }
      }
    }

    Ok(())
  }
}
