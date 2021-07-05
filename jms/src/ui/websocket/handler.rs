use log::info;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::arena::{AllianceStationOccupancy, ArenaSignal, ArenaState, SharedArena, matches::Match, station::AllianceStationId};

use super::{WebsocketError, WebsocketMessageHandler};

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
        ("update", Some(data)) => {
          self.alliance_update(serde_json::from_value(data)?)?;
          None
        },
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

#[derive(Deserialize)]
struct AllianceStationUpdate {
  station: AllianceStationId,
  update: Value
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

  fn alliance_update(&self, data: AllianceStationUpdate) -> super::Result<()> {
    let mut arena = self.arena.lock().unwrap();

    let current_state = arena.current_state();
    let idle = matches!(current_state, ArenaState::Idle);
    let prestart = matches!(current_state, ArenaState::Prestart {..});

    if let Value::Object(ref map) = data.update {
      let stn = arena.station_mut(data.station).ok_or(WebsocketError::Other(format!("No alliance station: {:?}", data.station)))?;
      for (k, v) in map {
        match (k.as_str(), v) {
          ("bypass", Value::Bool(v)) if (idle || prestart) => stn.bypass = *v,
          ("team", Value::Null) if idle => {
            stn.team = None;
            // Reset DS reports
            stn.occupancy = AllianceStationOccupancy::Vacant;
            stn.ds_report = None;
          },
          ("team", Value::Number(x)) if idle => stn.team = Some(x.as_u64().unwrap_or(0) as u16),
          _ => return Err(WebsocketError::Other(format!("Unknown data key or format (or state): key={} value={:?}", k, v)))
        }
      }
      Ok(())
    } else {
      Err(WebsocketError::Other("Update must be an object!".to_owned()))
    }
  }
}