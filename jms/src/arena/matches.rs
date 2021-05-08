#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum MatchPlayState {
  Waiting,
  Warmup,
  Auto,
  Pause,
  Teleop,
  Cooldown,
  Complete,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub struct Match {
  pub state: MatchPlayState,
}

impl Match {
  pub fn current_state(&self) -> MatchPlayState {
    self.state
  }

  pub fn ready(&self) -> bool {
    self.state == MatchPlayState::Waiting
  }

  pub fn complete(&self) -> bool {
    self.state == MatchPlayState::Complete
  }
}
