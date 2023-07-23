use std::time::Instant;

use chrono::Duration;
use jms_arena_lib::{MatchPlayState, SerialisedLoadedMatch, ARENA_MATCH_KEY};
use jms_base::kv::KVConnection;
use jms_core_lib::db::DBDuration;
use log::{warn, info};

pub struct LoadedMatch {
  pub match_id: String,
  pub state: MatchPlayState,
  last_state: Option<MatchPlayState>,
  
  match_start_time: Option<Instant>,
  state_start_time: Instant,

  pub remaining: Duration,
  pub endgame: bool,
}

impl LoadedMatch {
  pub fn new(match_id: String) -> Self {
    Self {
      match_id,
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

  pub async fn write_state(&self, kv: &mut KVConnection) -> anyhow::Result<()> {
    let serialised = SerialisedLoadedMatch {
      match_id: self.match_id.clone(),
      remaining: DBDuration(self.remaining),
      endgame: self.endgame,
      state: self.state
    };
    kv.json_set(ARENA_MATCH_KEY, "$", &serialised).await?;
    Ok(())
  }
}