use anyhow::{anyhow, bail};

use jms_macros::define_websocket_msg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{arena::{matches::LoadedMatch, station::AllianceStationId, ArenaSignal, ArenaState, AudienceDisplay, SharedArena, AllianceStation, ArenaAccessRestriction}, db::{self, TableType}, models, scoring::scores::ScoreUpdateData};

use super::{JsonMessage, WebsocketMessageHandler};

// #[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
// pub enum ArenaMessageState {
//   #[serde(skip_deserializing)]
//   Update(ArenaState),
//   #[serde(skip_serializing)]
//   Signal(ArenaSignal)
// }

// // TODO: Split into direction? Serialize / Deserialize - To/From. Maybe a macro is the better
// // way to go
// #[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
// pub enum ArenaMessageAlliances {
//   #[serde(skip_deserializing)]
//   Update(Vec<AllianceStation>),
  
// }

// #[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
// pub enum ArenaMessage {
//   State(ArenaMessageState),
//   Alliances(),
//   Match(),
//   AudienceDisplay(),
//   Access()
// }

define_websocket_msg!($ArenaMessage {
  $State {
    send Current(ArenaState),
    recv Signal(ArenaSignal)
  },
  $Alliance {
    send CurrentStations(Vec<AllianceStation>),
    recv UpdateAlliance(AllianceStationUpdate)
  },
  $Match {
    send Current(Option<LoadedMatch>),
    recv LoadTest,
    recv Unload,
    recv Load(String),
    recv ScoreUpdate(ScoreUpdateData)
  },
  $AudienceDisplay {
    send Current(AudienceDisplay),
    recv $Set {
      Field,
      MatchPreview,
      MatchPlay,
      MatchResults(Option<String>),
      AllianceSelection,
      Award(usize),
      CustomMessage(String)
    }
  },
  $Access {
    send Current(ArenaAccessRestriction),
    recv Set(ArenaAccessRestriction)
  }
});

pub struct ArenaWebsocketHandler {
  pub arena: SharedArena,
}

impl ArenaWebsocketHandler {
  pub fn new(arena: SharedArena) -> Self {
    ArenaWebsocketHandler { arena }
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
    {
      // Audience Display
      response.push(msg.noun("audience_display").to_data(&arena.audience_display)?);
    }
    {
      // Access
      response.push(msg.noun("access").to_data(&arena.access)?);
    }
    // Ok(vec![ msg.to_data(&*arena)? ])
    Ok(response)
  }

  async fn handle(&mut self, msg: super::JsonMessage) -> super::Result<Vec<JsonMessage>> {
    // let response_msg = msg.response();

    let response = vec![];

    match msg.noun.as_str() {
      "state" => match (msg.verb.as_str(), msg.data) {
        ("signal", Some(data)) => {
          let sig: ArenaSignal = serde_json::from_value(data)?;
          self.arena.lock().await.signal(sig).await;
        }
        _ => bail!("Invalid verb or data"),
      },
      "alliances" => match (msg.verb.as_str(), msg.data) {
        ("update", Some(data)) => {
          self.alliance_update(serde_json::from_value(data)?).await?;
        }
        _ => bail!("Invalid verb or data"),
      },
      "match" => match (msg.verb.as_str(), msg.data) {
        ("loadTest", None) => {
          self
            .arena
            .lock()
            .await
            .load_match(LoadedMatch::new(models::Match::new_test()))?;
        }
        ("load", Some(serde_json::Value::String(id))) => {
          let m = models::Match::get_or_err(id, &db::database())?;
          self.arena.lock().await.load_match(LoadedMatch::new(m))?;
        }
        ("unload", None) => {
          self.arena.lock().await.unload_match()?;
        }
        ("scoreUpdate", Some(data)) => {
          let update: ScoreUpdateData = serde_json::from_value(data)?;
          match self.arena.lock().await.current_match.as_mut() {
            Some(m) => match update.alliance {
              models::Alliance::Blue => m.score.blue.update(update.update),
              models::Alliance::Red => m.score.red.update(update.update),
            },
            None => bail!("Can't update score: no match is running!"),
          }
          // self.arena.lock().await.current_match
        }
        _ => bail!("Invalid verb or data"),
      },
      "audience_display" => match (msg.verb.as_str(), msg.data) {
        ("set", Some(data)) => {
          let ad = self.audience_display_update(serde_json::from_value(data)?).await?;
          self.arena.lock().await.audience_display = ad;
        }
        _ => bail!("Invalid verb or data"),
      },
      "access" => match (msg.verb.as_str(), msg.data) {
        ("set", Some(data)) => {
          let v = serde_json::from_value(data)?;
          self.arena.lock().await.access = v;
        }
        _ => bail!("Invalid verb or data"),
      },
      _ => bail!("Unknown noun"),
    };

    return Ok(response);
  }
}

#[derive(Deserialize)]
struct AudienceDisplayUpdate {
  scene: String,
  params: Option<Value>,
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
    let idle = matches!(current_state, ArenaState::Idle { .. });
    let prestart = matches!(current_state, ArenaState::Prestart { .. });

    if let Value::Object(ref map) = data.update {
      let stn = arena
        .station_mut(data.station)
        .ok_or(anyhow!("No alliance station: {:?}", data.station))?;
      for (k, v) in map {
        match (k.as_str(), v) {
          ("bypass", Value::Bool(v)) if (idle || prestart) => stn.bypass = *v,
          ("team", Value::Null) if idle => {
            // Reset DS reports
            stn.reset();
          }
          ("team", Value::Number(x)) if idle => {
            stn.reset();
            stn.team = Some(x.as_u64().unwrap_or(0) as u16);
          }
          ("estop", Value::Bool(v)) => stn.estop = stn.estop || *v,
          ("astop", Value::Bool(v)) => stn.astop = stn.astop || *v,
          _ => {
            bail!("Unknown data key or format (or state): key={} value={:?}", k, v)
          }
        }
      }
      Ok(())
    } else {
      bail!("update must be an object!")
    }
  }

  async fn audience_display_update(&self, data: AudienceDisplayUpdate) -> super::Result<AudienceDisplay> {
    Ok(match (data.scene.as_str(), data.params) {
      ("Field", None) => AudienceDisplay::Field,
      ("MatchPreview", None) => AudienceDisplay::MatchPreview,
      ("MatchPlay", None) => AudienceDisplay::MatchPlay,
      ("MatchResults", None) => {
        let last_match = models::Match::sorted(&db::database())?
          .iter().filter(|&t| t.played).last().cloned();
        if let Some(last_match) = last_match {
          AudienceDisplay::MatchResults(models::SerializedMatch::from(last_match))
        } else {
          bail!("Can't display results when no matches have been played!");
        }
      }
      ("MatchResults", Some(Value::String(match_id))) => {
        let m = models::Match::get_or_err(match_id, &db::database())?;
        AudienceDisplay::MatchResults(models::SerializedMatch::from(m))
      }
      ("AllianceSelection", None) => AudienceDisplay::AllianceSelection,
      ("Award", Some(Value::Number(award_id))) => {
        if let Some(n) = award_id.as_u64() {
          let award = models::Award::get_or_err(n, &db::database())?;
          AudienceDisplay::Award(award)
        } else {
          bail!("{} is not a u64", award_id);
        }
      }
      ("CustomMessage", Some(Value::String(msg))) => AudienceDisplay::CustomMessage(msg),
      (_, _) => bail!("Invalid Audience Display scene"),
    })
  }
}
