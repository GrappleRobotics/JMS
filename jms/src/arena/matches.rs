use std::time::Instant;

use anyhow::{bail, Result};

use chrono::Duration;
use log::{info, warn};
use schemars::JsonSchema;

use crate::{arena::exceptions::MatchWrongState, db, models, scoring::scores::{MatchScore, MatchScoreSnapshot}};

use serde::{Serialize, Deserialize};

use super::exceptions::MatchIllegalStateChange;

#[derive(Clone, Debug, Copy, PartialEq, Eq, Display, Serialize, Deserialize, JsonSchema)]
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

// TODO: Abstract out the loaded parts of the match

#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct LoadedMatch {
  #[serde(serialize_with = "models::serialize_match")]
  #[schemars(with = "models::SerializedMatch")]
  pub match_meta: models::Match,
  pub state: MatchPlayState,
  pub remaining_time: std::time::Duration,
  pub match_time: Option<std::time::Duration>,
  pub endgame: bool,

  #[serde(skip)]
  match_start_time: Option<Instant>,
  #[serde(skip)]
  state_first: bool,
  #[serde(skip)]
  state_start_time: Instant,
}

impl LoadedMatch {
  pub fn new(m: models::Match) -> LoadedMatch {
    LoadedMatch {
      state: MatchPlayState::Waiting,
      match_meta: m,
      state_first: true,
      state_start_time: Instant::now(),
      match_start_time: None,
      remaining_time: std::time::Duration::ZERO,
      match_time: None,
      endgame: false,
    }
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

  // pub async fn commit_score(&mut self) -> Result<models::Match> {
  //   if self.state == MatchPlayState::Complete {
  //     Ok(self.match_meta.commit(&self.score, &db::database()).await?.clone())
  //   } else {
  //     bail!(MatchWrongState {
  //       state: self.state,
  //       why: "Can't commit score before Match is complete!".to_owned()
  //     })
  //   }
  // }

  pub fn fault(&mut self) {
    self.do_change_state(MatchPlayState::Fault);
  }

  // TODO: Implement a self-timing guard for update functions, generating an error if we miss our timing.
  // TODO: Or, async with timeout?
  pub fn update(&mut self) {
    let first = self.state_first;
    self.state_first = false;
    let mut endgame = false;

    let elapsed = Duration::from_std(Instant::now() - self.state_start_time).unwrap();
    if let Some(start) = self.match_start_time {
      self.match_time = Some(Instant::now() - start);
    }

    let mut remaining = Duration::zero();

    match self.state {
      MatchPlayState::Waiting => (),
      MatchPlayState::Warmup => {
        remaining = self.match_meta.config.warmup_cooldown_time.0 - elapsed;
        if remaining <= Duration::zero() {
          self.do_change_state(MatchPlayState::Auto);
        }
      }
      MatchPlayState::Auto => {
        self.match_start_time = Some(Instant::now());
        remaining = self.match_meta.config.auto_time.0 - elapsed;
        if remaining <= Duration::zero() {
          self.do_change_state(MatchPlayState::Pause);
        }
      }
      MatchPlayState::Pause => {
        remaining = self.match_meta.config.pause_time.0 - elapsed;
        if remaining <= Duration::zero() {
          self.do_change_state(MatchPlayState::Teleop);
        }
      }
      MatchPlayState::Teleop => {
        remaining = self.match_meta.config.teleop_time.0 - elapsed;
        if remaining <= Duration::zero() {
          self.do_change_state(MatchPlayState::Cooldown);
        }
        endgame = remaining <= self.match_meta.config.endgame_time.0;
      }
      MatchPlayState::Cooldown => {
        remaining = self.match_meta.config.warmup_cooldown_time.0 - elapsed;
        if remaining <= Duration::zero() {
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

    self.remaining_time = remaining.to_std().unwrap_or(std::time::Duration::ZERO);

    self.endgame = endgame;
  }

  fn do_change_state(&mut self, state: MatchPlayState) {
    if state != self.state {
      info!("Transitioning {:?} -> {:?}...", self.state, state);
      self.state = state;
      self.state_start_time = Instant::now();
      self.state_first = true;
    }
  }
}
