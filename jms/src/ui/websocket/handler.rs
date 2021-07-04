use log::info;
use serde_json::{Value, json};

use crate::arena::{ArenaSignal, ArenaState, SharedArena, matches::Match};

use super::WebsocketMessageHandler;

pub struct ArenaWebsocketHandler {
  pub arena: SharedArena
}

impl WebsocketMessageHandler for ArenaWebsocketHandler {
  fn handle(&mut self, msg: super::JsonMessage) -> super::Result<Option<super::JsonMessage>> {
    let response_msg = msg.response();

    Ok(match msg.noun.as_str() {
      "state" => match (msg.verb.as_str(), msg.data) {
        ("signal", Some(data)) => {
          let sig: ArenaSignal = serde_json::from_value(data)?;
          self.arena.lock().unwrap().signal(sig);
          None
        },
        ("current", None) => {
          let current = self.arena.lock().unwrap().current_state();
          Some(response_msg.data(serde_json::to_value(current)?))
        },
        _ => Some(response_msg.invalid_verb_or_data())
      },
      "alliances" => match (msg.verb.as_str(), msg.data) {
        _ => Some(response_msg.invalid_verb_or_data())
      },
      "match" => match (msg.verb.as_str(), msg.data) {
        ("loadTest", None) => {
          self.arena.lock().unwrap().load_match(Match::new())?;
          None
        },
        _ => Some(response_msg.invalid_verb_or_data())
      }
      "status" => match (msg.verb.as_str(), msg.data) {
        ("get", None) => self.status()?.map(|x| response_msg.data(x)),
        _ => Some(response_msg.invalid_verb_or_data())
      }
      _ => Some(response_msg.unknown_noun())
    })
  }
}

impl ArenaWebsocketHandler {
  fn status(&self) -> super::Result<Option<Value>> {
    let arena = self.arena.lock().unwrap();
    let ref the_match = arena.current_match;  // match is a reserved word in rust :)

    Ok(Some(json!({ 
      "state": arena.current_state(),
      "alliances": arena.stations,
      "match": the_match.as_ref().map(|m| {
        json!({
          "state": m.current_state(),
          "remaining_time": m.remaining_time()
        })
      })
    })))
  }
}