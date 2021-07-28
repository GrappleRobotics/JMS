use diesel::{QueryDsl, RunQueryDsl, ExpressionMethods};
use serde::Deserialize;
use serde_json::Value;

use crate::{arena::{AllianceStationOccupancy, ArenaSignal, ArenaState, SharedArena, matches::LoadedMatch, station::{Alliance, AllianceStationId}}, db, models, scoring::scores::ScoreUpdateData};

use super::{JsonMessage, WebsocketError, WebsocketMessageHandler};

pub struct ArenaWebsocketHandler {
  pub arena: SharedArena,
}

impl ArenaWebsocketHandler {
  pub fn new(arena: SharedArena) -> Self {
    ArenaWebsocketHandler {
      arena
    }
  }
}

#[async_trait::async_trait]
impl WebsocketMessageHandler for ArenaWebsocketHandler {
  async fn update(&mut self) -> super::Result<Vec<JsonMessage>> {
    let arena = self.arena.lock().await;
    let msg = JsonMessage::update("arena", "");
    let mut response = vec![];

    {
      // Match
      response.push(msg.noun("match").to_data(&arena.current_match)?);
    }
    {
      // State
      response.push(msg.noun("state").to_data(&arena.state)?);
    }
    {
      // Stations
      response.push(msg.noun("stations").to_data(&arena.stations)?);
    }
    // Ok(vec![ msg.to_data(&*arena)? ])
    Ok(response)
  }

  async fn handle(&mut self, msg: super::JsonMessage) -> super::Result<Vec<JsonMessage>> {
    let response_msg = msg.response();

    let response = vec![];

    match msg.noun.as_str() {
      "state" => match (msg.verb.as_str(), msg.data) {
        ("signal", Some(data)) => {
          let sig: ArenaSignal = serde_json::from_value(data)?;
          self.arena.lock().await.signal(sig).await;
        }
        _ => Err(response_msg.invalid_verb_or_data())?,
      },
      "alliances" => match (msg.verb.as_str(), msg.data) {
        ("update", Some(data)) => {
          self.alliance_update(serde_json::from_value(data)?).await?;
        }
        _ => Err(response_msg.invalid_verb_or_data())?,
      },
      "match" => match (msg.verb.as_str(), msg.data) {
        ("loadTest", None) => {
          self.arena.lock().await.load_match(LoadedMatch::new(models::Match::new_test()))?;
        },
        ("load", Some(serde_json::Value::Number(match_id))) => {
          use crate::schema::matches::dsl::*;
          if let Some(n) = match_id.as_i64() {
            let match_meta = matches.filter(id.eq(n as i32)).first::<models::Match>(&db::connection())?;
            self.arena.lock().await.load_match(LoadedMatch::new(match_meta))?;
          } else {
            Err(response_msg.invalid_verb_or_data())?
          }
        },
        ("scoreUpdate", Some(data)) => {
          let update: ScoreUpdateData = serde_json::from_value(data)?;
          match self.arena.lock().await.current_match.as_mut() {
            Some(m) => {
              match update.alliance {
                Alliance::Blue => m.score.blue.update(update.update),
                Alliance::Red => m.score.red.update(update.update),
              }
            },
            None => Err(WebsocketError::Other("No Match!".to_owned()))?
          }
          // self.arena.lock().await.current_match
        },
        _ => Err(response_msg.invalid_verb_or_data())?,
      },
      _ => Err(response_msg.unknown_noun())?,
    };

    return Ok(response);
  }
}

#[derive(Deserialize)]
struct AllianceStationUpdate {
  station: AllianceStationId,
  update: Value,
}

impl ArenaWebsocketHandler {
  async fn alliance_update(&self, data: AllianceStationUpdate) -> super::Result<()> {
    let mut arena = self.arena.lock().await;

    let current_state = arena.current_state();
    let idle = matches!(current_state, ArenaState::Idle);
    let prestart = matches!(current_state, ArenaState::Prestart { .. });

    if let Value::Object(ref map) = data.update {
      let stn = arena.station_mut(data.station).ok_or(WebsocketError::Other(format!(
        "No alliance station: {:?}",
        data.station
      )))?;
      for (k, v) in map {
        match (k.as_str(), v) {
          ("bypass", Value::Bool(v)) if (idle || prestart) => stn.bypass = *v,
          ("team", Value::Null) if idle => {
            stn.team = None;
            // Reset DS reports
            stn.occupancy = AllianceStationOccupancy::Vacant;
            stn.ds_report = None;
          }
          ("team", Value::Number(x)) if idle => {
            stn.occupancy = AllianceStationOccupancy::Vacant;
            stn.ds_report = None;
            stn.team = Some(x.as_u64().unwrap_or(0) as u16);
          },
          _ => {
            return Err(WebsocketError::Other(format!(
              "Unknown data key or format (or state): key={} value={:?}",
              k, v
            )))
          }
        }
      }
      Ok(())
    } else {
      Err(WebsocketError::Other("Update must be an object!".to_owned()))
    }
  }
}
