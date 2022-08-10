use jms_macros::define_websocket_msg;

use crate::{db::{self, TableType}, models, scoring::scores::{LiveScore, MatchScore}};

define_websocket_msg!($DebugMessage {
  recv $Match {
    FillRandomScores,
    DeleteAll
  }
});

pub async fn ws_recv_debug(data: &DebugMessage2JMS) -> super::Result<Vec<super::WebsocketMessage2UI>> {
  match data.clone() {
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
  }
  
  return Ok(vec![]);
}
