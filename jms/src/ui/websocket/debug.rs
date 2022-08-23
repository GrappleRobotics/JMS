use jms_macros::define_websocket_msg;

use crate::{db::{self, TableType}, models, scoring::scores::{LiveScore, MatchScore}};

use super::{ws::{WebsocketHandler, Websocket}, WebsocketMessage2JMS};

define_websocket_msg!($DebugMessage {
  recv $Match {
    FillRandomScores(Option<String>),
    DeleteAll
  },
  ReplyTest(String)
});

pub struct WSDebugHandler;

#[async_trait::async_trait]
impl WebsocketHandler for WSDebugHandler {
  async fn handle(&self, msg: &WebsocketMessage2JMS, ws: &mut Websocket) -> anyhow::Result<()> {
    if let WebsocketMessage2JMS::Debug(msg) = msg {
      ws.require_fta().await?;

      match msg.clone() {
        DebugMessage2JMS::Match(msg) => match msg {
          DebugMessageMatch2JMS::FillRandomScores(selected_match) => {
            for mut m in models::Match::all(&db::database())? {
              if m.ready && !m.played && selected_match.as_ref().map(|id| m.id().as_ref() == Some(id)).unwrap_or(true) {
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
