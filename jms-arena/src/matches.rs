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
  pub remaining_max: Duration,
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
      remaining_max: Duration::zero(),
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

  pub async fn update(&mut self) -> anyhow::Result<bool> {
    let first = self.last_state != Some(self.state);
    self.last_state = Some(self.state);

    let elapsed = Duration::from_std(Instant::now() - self.state_start_time).unwrap();
    let mut remaining = Duration::zero();
    let mut remaining_max = Duration::zero();
    let mut endgame = false;

    match self.state {
      MatchPlayState::Waiting => (),
      MatchPlayState::Warmup => {
        remaining_max = Duration::seconds(3);
        remaining = remaining_max - elapsed;
        if remaining <= Duration::zero() {
          self.do_change_state(MatchPlayState::Auto);
        }
      }
      MatchPlayState::Auto => {
        if first {
          self.match_start_time = Some(Instant::now());
        }
        remaining_max = Duration::seconds(15);
        remaining = remaining_max - elapsed;
        if remaining <= Duration::zero() {
          self.do_change_state(MatchPlayState::Pause);
        }
      }
      MatchPlayState::Pause => {
        remaining_max = Duration::seconds(3);
        remaining = remaining_max - elapsed;
        if remaining <= Duration::zero() {
          self.do_change_state(MatchPlayState::Teleop);
        }
      }
      MatchPlayState::Teleop => {
        remaining_max = Duration::seconds(2*60 + 15);
        remaining = remaining_max - elapsed;
        if remaining <= Duration::zero() {
          self.do_change_state(MatchPlayState::Cooldown);
        }
        endgame = remaining <= Duration::seconds(20);
      }
      MatchPlayState::Cooldown => {
        remaining_max = Duration::seconds(3);
        remaining = remaining_max - elapsed;
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

    self.remaining_max = remaining_max;
    self.remaining = remaining;
    self.endgame = endgame;

    Ok(first)
  }

  pub fn write_state(&self, kv: &mut KVConnection) -> anyhow::Result<()> {
    let serialised = SerialisedLoadedMatch {
      match_id: self.match_id.clone(),
      remaining: DBDuration(self.remaining),
      remaining_max: DBDuration(self.remaining_max),
      match_time: self.match_start_time.map(|mt| DBDuration(Duration::from_std(Instant::now() - mt).unwrap())),
      endgame: self.endgame,
      state: self.state
    };
    kv.json_set(ARENA_MATCH_KEY, "$", &serialised)?;
    Ok(())
  }
}