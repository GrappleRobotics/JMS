use std::time::Instant;

use chrono::Duration;
use jms_arena_lib::MatchPlayState;
use jms_base::redis::{aio::Connection, AsyncCommands};
use log::{warn, info};

pub struct LoadedMatch {
  pub state: MatchPlayState,
  last_state: Option<MatchPlayState>,
  
  match_start_time: Option<Instant>,
  state_start_time: Instant,

  pub remaining: Duration,
  pub endgame: bool,
}

impl LoadedMatch {
  pub fn new() -> Self {
    Self {
      state: MatchPlayState::Waiting,
      last_state: None,
      match_start_time: None,
      state_start_time: Instant::now(),

      remaining: Duration::zero(),
      endgame: false
    }
  }

  pub fn start(&mut self) -> anyhow::Result<()> {
    if self.state == MatchPlayState::Waiting {
      self.do_change_state(MatchPlayState::Warmup);
      Ok(())
    } else {
      anyhow::bail!("Can't start a match if the match has already been run.")
    }
  }

  pub fn fault(&mut self) {
    self.do_change_state(MatchPlayState::Fault);
  }

  fn do_change_state(&mut self, state: MatchPlayState) {
    if state != self.state {
      info!("Transitioning {:?} -> {:?}...", self.state, state);
      self.last_state = Some(self.state);
      self.state = state;
      self.state_start_time = Instant::now();
    }
  }

  pub async fn update(&mut self) -> anyhow::Result<()> {
    let first = self.last_state != Some(self.state);
    self.last_state = Some(self.state);

    let elapsed = Duration::from_std(Instant::now() - self.state_start_time).unwrap();
    let mut remaining = Duration::zero();
    let mut endgame = false;

    match self.state {
      MatchPlayState::Waiting => (),
      MatchPlayState::Warmup => {
        remaining = Duration::seconds(3) - elapsed;
        if remaining <= Duration::zero() {
          self.do_change_state(MatchPlayState::Auto);
        }
      }
      MatchPlayState::Auto => {
        self.match_start_time = Some(Instant::now());
        remaining = Duration::seconds(15) - elapsed;
        if remaining <= Duration::zero() {
          self.do_change_state(MatchPlayState::Pause);
        }
      }
      MatchPlayState::Pause => {
        remaining = Duration::seconds(1) - elapsed;
        if remaining <= Duration::zero() {
          self.do_change_state(MatchPlayState::Teleop);
        }
      }
      MatchPlayState::Teleop => {
        remaining = Duration::seconds(2*60 + 15) - elapsed;
        if remaining <= Duration::zero() {
          self.do_change_state(MatchPlayState::Cooldown);
        }
        endgame = remaining <= Duration::seconds(30);
      }
      MatchPlayState::Cooldown => {
        remaining = Duration::seconds(3) - elapsed;
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

    self.remaining = remaining;
    self.endgame = endgame;

    Ok(())
  }

  pub async fn write_state(&self, redis: &mut Connection) -> anyhow::Result<()> {
    redis.hset("arena:match", "remaining_ms", self.remaining.num_milliseconds()).await?;
    redis.hset("arena:match", "endgame", self.endgame).await?;
    redis.hset("arena:match", "state", serde_json::to_string(&self.state)?).await?;
    Ok(())
  }
}