use anyhow::bail;
use jms_macros::define_websocket_msg;

use crate::{
  db, models,
  schedule::{playoffs::PlayoffMatchGenerator, quals::QualsMatchGenerator, worker::{SerialisedMatchGeneration, MatchGenerator, SharedMatchGenerators}},
};

use super::WebsocketMessage2UI;

define_websocket_msg!($MatchMessage {
  $Quals {
    send Generation(SerialisedMatchGeneration),
    recv Clear,
    recv Generate(<QualsMatchGenerator as MatchGenerator>::ParamType)
  },
  $Playoffs {
    send Generation(SerialisedMatchGeneration),
    recv Clear,
    recv Generate(<PlayoffMatchGenerator as MatchGenerator>::ParamType)
  },
  send All(Vec<models::SerializedMatch>),
  send Next(Option<models::SerializedMatch>),
  send Last(Option<models::SerializedMatch>)
});


pub async fn ws_periodic_match(params_arc: SharedMatchGenerators) -> super::Result<Vec<MatchMessage2UI>> {
  let mut data: Vec<MatchMessage2UI> = vec![];
  let sorted = models::Match::sorted(&db::database())?;

  {
    let params = params_arc.lock().await;
    data.push(MatchMessageQuals2UI::Generation((&params.quals).into()).into());
    data.push(MatchMessagePlayoffs2UI::Generation((&params.playoffs).into()).into());
  } {
    let matches = sorted.iter().map(|m| models::SerializedMatch::from(m.clone()) ).collect();
    data.push(MatchMessage2UI::All(matches));
  } {
    let next_match = sorted.iter().find(|&m| !m.played).map(|m| models::SerializedMatch::from(m.clone()));
    data.push(MatchMessage2UI::Next(next_match.clone()))
  } {
    let last_match = sorted.iter().rev().find(|&m| m.played).map(|m| models::SerializedMatch::from(m.clone()));
    data.push(MatchMessage2UI::Last(last_match.clone()))
  }

  return Ok(data);
}

pub async fn ws_recv_match(data: &MatchMessage2JMS, params_arc: SharedMatchGenerators) -> super::Result<Vec<WebsocketMessage2UI>> {
  let params = params_arc.lock().await;

  match data.clone() {
    MatchMessage2JMS::Quals(msg) => match msg {
      MatchMessageQuals2JMS::Clear if !params.quals.has_played() => params.quals.delete(),
      MatchMessageQuals2JMS::Clear => bail!("Cannot clear match generator after matches have started!"),
      MatchMessageQuals2JMS::Generate(p) => params.quals.generate(p).await,
    },
    MatchMessage2JMS::Playoffs(msg) => match msg {
      MatchMessagePlayoffs2JMS::Clear if !params.playoffs.has_played() => params.playoffs.delete(),
      MatchMessagePlayoffs2JMS::Clear => bail!("Cannot clear match generator after matches have started!"),
      MatchMessagePlayoffs2JMS::Generate(p) => params.playoffs.generate(p).await,
    },
  }
  return Ok(vec![]);
} 


//   async fn handle(&mut self, msg: JsonMessage) -> super::Result<Vec<JsonMessage>> {
//     // let response_msg = msg.response();

//     match msg.noun.as_str() {
//       "quals" => match (msg.verb.as_str(), msg.data) {
//         ("clear", None) => {
//           if self.quals.has_played() {
//             bail!("Cannot delete after matches have started")
//           } else {
//             self.quals.delete();
//           }
//         }
//         ("generate", Some(data)) => {
//           let params = serde_json::from_value(data)?;
//           self.quals.generate(params).await;
//         }
//         _ => bail!("Invalid verb or data"),
//       },
//       "playoffs" => match (msg.verb.as_str(), msg.data) {
//         ("clear", None) => {
//           if self.playoffs.has_played() {
//             bail!("Cannot delete after matches have started")
//           } else {
//             self.playoffs.delete();
//           }
//         }
//         ("generate", Some(data)) => {
//           let params = serde_json::from_value(data)?;
//           self.playoffs.generate(params).await;
//         }
//         _ => bail!("Invalid verb or data"),
//       },
//       _ => bail!("Unknown noun"),
//     }
//     Ok(vec![])
//   }
// }
