use std::time::{Duration, Instant};

use log::{info, warn};

use super::exceptions::{MatchError, MatchResult};
use crate::{context};

#[derive(Clone, Debug, Copy, PartialEq, Eq, Display)]
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

#[derive(Clone, Debug)]
pub struct MatchConfig {
  warmup_cooldown_time: Duration,
  auto_time: Duration,
  pause_time: Duration,
  teleop_time: Duration,
}

#[derive(Clone, Debug)]
pub struct Match {
  state: MatchPlayState,
  state_first: bool,
  state_start_time: Instant,
  config: MatchConfig,
}

impl Match {
  pub fn new() -> Match {
    Match {
      state: MatchPlayState::Waiting,
      state_first: true,
      state_start_time: Instant::now(),
      config: MatchConfig {
        warmup_cooldown_time: Duration::from_secs(3),
        auto_time: Duration::from_secs(4),
        pause_time: Duration::from_secs(1),
        teleop_time: Duration::from_secs(4),
      },
    }
  }

  pub fn current_state(&self) -> MatchPlayState {
    self.state
  }

  pub fn start(&mut self) -> MatchResult<()> {
    if self.state == MatchPlayState::Waiting {
      self.do_change_state(MatchPlayState::Warmup);
      Ok(())
    } else {
      Err(MatchError::IllegalStateChange {
        from: self.state,
        to: MatchPlayState::Waiting,
        why: "Match not ready!".to_owned(),
      })
    }
  }

  pub fn fault(&mut self) {
    self.do_change_state(MatchPlayState::Fault);
  }

  // TODO: Implement a self-timing guard for update functions, generating an error if we miss our timing.
  pub fn update(&mut self) {
    let first = self.state_first;
    self.state_first = false;
    let elapsed = self.elapsed();

    context!(&format!("Match::Update ({:?})", self.state), {
      match self.state {
        MatchPlayState::Waiting => (),
        MatchPlayState::Warmup => {
          if elapsed >= self.config.warmup_cooldown_time {
            self.do_change_state(MatchPlayState::Auto);
          }
        }
        MatchPlayState::Auto => {
          if elapsed >= self.config.auto_time {
            self.do_change_state(MatchPlayState::Pause);
          }
        }
        MatchPlayState::Pause => {
          if elapsed >= self.config.pause_time {
            self.do_change_state(MatchPlayState::Teleop);
          }
        }
        MatchPlayState::Teleop => {
          if elapsed >= self.config.teleop_time {
            self.do_change_state(MatchPlayState::Cooldown);
          }
        }
        MatchPlayState::Cooldown => {
          if elapsed >= self.config.warmup_cooldown_time {
            self.do_change_state(MatchPlayState::Complete);
          }
        }
        MatchPlayState::Complete => {}
        MatchPlayState::Fault => {
          if first {
            warn!("Match fault");
          }
        }
      }
    });
  }

  fn do_change_state(&mut self, state: MatchPlayState) {
    info!("Transitioning {:?} -> {:?}...", self.state, state);
    self.state = state;
    self.state_start_time = Instant::now();
    self.state_first = true;
  }

  fn elapsed(&self) -> Duration {
    return Instant::now() - self.state_start_time;
  }
}
