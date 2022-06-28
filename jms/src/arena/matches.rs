use std::time::{Duration, Instant};

use anyhow::{bail, Result};

use log::{info, warn};
use schemars::JsonSchema;

use crate::{arena::exceptions::MatchWrongState, db, models, scoring::scores::MatchScore};

use serde::{Serialize, Serializer};

use super::exceptions::MatchIllegalStateChange;

#[derive(Clone, Debug, Copy, PartialEq, Eq, Display, Serialize, JsonSchema)]
pub enum MatchPlayState {
  Waiting,
  Warmup,
  Auto,
  Pause,
  Teleop,
  Cooldown,
  Complete,
  Fault, // E-stop, cancelled, etc. Fault is unrecoverable without reloading the match.
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct MatchConfig {
  warmup_cooldown_time: Duration,
  auto_time: Duration,
  pause_time: Duration,
  teleop_time: Duration,
  endgame_time: Duration,
}

// TODO: Abstract out the loaded parts of the match

#[derive(Clone, Debug, Serialize, JsonSchema)]
// #[serde(into = "SerializedLoadedMatch")]
// #[schemars(schema_with = "SerializedLoadedMatch::")]
pub struct LoadedMatch {
  #[serde(serialize_with = "serialize_match")]
  #[schemars(with = "models::SerializedMatch")]
  pub match_meta: models::Match,
  state: MatchPlayState,
  remaining_time: Duration,
  pub score: MatchScore,

  #[serde(skip)]
  state_first: bool,
  #[serde(skip)]
  state_start_time: Instant,

  config: MatchConfig,
  endgame: bool,
}

fn serialize_match<S>(m: &models::Match, s: S) -> Result<S::Ok, S::Error>
where
  S: Serializer
{
  models::SerializedMatch::from(m.clone()).serialize(s)
}

// mod serialised_loaded_match {
//   // use crate::models::SerializedMatch;

//   // pub fn schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
//   //   <SerializedMatch>::json_schema(gen).into()
//   // }
// }

// #[derive(Clone, Debug, Serialize, JsonSchema)]
// pub struct SerializedLoadedMatch {
//   #[serde(rename="match")]
//   match_meta: models::Match,
//   id: Option<String>,
//   name: String,
//   state: MatchPlayState,
//   remaining_time: Duration,
//   score: MatchScore,
//   config: MatchConfig,
//   endgame: bool
// }

// impl From<LoadedMatch> for SerializedLoadedMatch {
//   fn from(lm: LoadedMatch) -> Self {
//     let LoadedMatch { match_meta, state, remaining_time, score, state_first, state_start_time, config, endgame  } = lm;
//     Self {
//       match_meta, id: match_meta.id(), name: match_meta.name(),
//       state, remaining_time, score, config, endgame
//     }
//   }
// }

// impl Serialize for LoadedMatch {
//   fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//   where
//     S: serde::Serializer
//   {
//     let mut state = serializer.serialize_struct("LoadedMatch", 6)?;
//     state.serialize_field("match", &models::SerializedMatch(self.match_meta.clone()))?;
//     state.serialize_field("state", &self.state)?;
//     state.serialize_field("remaining_time", &self.remaining_time)?;
//     state.serialize_field("score", &self.score)?;
//     state.serialize_field("config", &self.config)?;
//     state.serialize_field("endgame", &self.endgame)?;
//     state.end()
//   }
// }

impl LoadedMatch {
  pub fn new(m: models::Match) -> LoadedMatch {
    LoadedMatch {
      state: MatchPlayState::Waiting,
      score: MatchScore::new(m.red_teams.len(), m.blue_teams.len()),
      match_meta: m,
      state_first: true,
      state_start_time: Instant::now(),
      remaining_time: Duration::from_secs(0),
      config: MatchConfig {
        warmup_cooldown_time: Duration::from_secs(3),
        auto_time: Duration::from_secs(15),
        pause_time: Duration::from_secs(1),
        teleop_time: Duration::from_secs(2 * 60 + 15),
        endgame_time: Duration::from_secs(30),
      },
      endgame: false,
    }
  }

  pub fn current_state(&self) -> MatchPlayState {
    self.state
  }

  pub fn metadata(&self) -> &models::Match {
    &self.match_meta
  }

  pub fn start(&mut self) -> Result<()> {
    if self.state == MatchPlayState::Waiting {
      self.do_change_state(MatchPlayState::Warmup);
      Ok(())
    } else {
      bail!(MatchIllegalStateChange {
        from: self.state,
        to: MatchPlayState::Waiting,
        why: "Match not ready!".to_owned(),
      })
    }
  }

  pub async fn commit_score(&mut self) -> Result<models::Match> {
    if self.state == MatchPlayState::Complete {
      Ok(self.match_meta.commit(&self.score, &db::database()).await?.clone())
    } else {
      bail!(MatchWrongState {
        state: self.state,
        why: "Can't commit score before Match is complete!".to_owned()
      })
    }
  }

  pub fn fault(&mut self) {
    self.do_change_state(MatchPlayState::Fault);
  }

  // TODO: Implement a self-timing guard for update functions, generating an error if we miss our timing.
  // TODO: Or, async with timeout?
  pub fn update(&mut self) {
    let first = self.state_first;
    self.state_first = false;
    let elapsed = self.elapsed();

    let mut endgame = false;

    match self.state {
      MatchPlayState::Waiting => (),
      MatchPlayState::Warmup => {
        self.remaining_time = self.config.warmup_cooldown_time.saturating_sub(elapsed);
        if self.remaining_time == Duration::ZERO {
          self.do_change_state(MatchPlayState::Auto);
        }
      }
      MatchPlayState::Auto => {
        self.remaining_time = self.config.auto_time.saturating_sub(elapsed);
        if self.remaining_time == Duration::ZERO {
          self.do_change_state(MatchPlayState::Pause);
        }
      }
      MatchPlayState::Pause => {
        self.remaining_time = self.config.pause_time.saturating_sub(elapsed);
        if self.remaining_time == Duration::ZERO {
          self.do_change_state(MatchPlayState::Teleop);
        }
      }
      MatchPlayState::Teleop => {
        self.remaining_time = self.config.teleop_time.saturating_sub(elapsed);
        if self.remaining_time == Duration::ZERO {
          self.do_change_state(MatchPlayState::Cooldown);
        }
        endgame = self.remaining_time <= self.config.endgame_time;
      }
      MatchPlayState::Cooldown => {
        self.remaining_time = self.config.warmup_cooldown_time.saturating_sub(elapsed);
        if self.remaining_time == Duration::ZERO {
          self.do_change_state(MatchPlayState::Complete);
        }
        endgame = true;
      }
      MatchPlayState::Complete => {
        endgame = true;
      }
      MatchPlayState::Fault => {
        if first {
          warn!("Match fault");
        }
      }
    }

    self.endgame = endgame;
  }

  pub fn remaining_time(&self) -> Duration {
    self.remaining_time
  }

  fn do_change_state(&mut self, state: MatchPlayState) {
    if state != self.state {
      info!("Transitioning {:?} -> {:?}...", self.state, state);
      self.state = state;
      self.state_start_time = Instant::now();
      self.state_first = true;
    }
  }

  fn elapsed(&self) -> Duration {
    return Instant::now() - self.state_start_time;
  }
}
