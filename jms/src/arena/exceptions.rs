use super::{matches::MatchPlayState, state::ArenaState};

#[derive(thiserror::Error, Debug)]
#[error("Arena State Transition Error: {from:?} to {to:?} ({why})")]
pub struct ArenaIllegalStateChange {
  pub from: ArenaState,
  pub to: ArenaState,
  pub why: String,
}

#[derive(thiserror::Error, Debug)]
#[error("Match State Transition Error: {from:?} to {to:?} ({why})")]
pub struct MatchIllegalStateChange {
  pub from: MatchPlayState,
  pub to: MatchPlayState,
  pub why: String,
}

#[derive(thiserror::Error, Debug)]
#[error("{0}")]
pub struct CannotLoadMatchError(pub String);

#[derive(thiserror::Error, Debug)]
#[error("Wrong State: {state:?} ({why})")]
pub struct MatchWrongState {
  pub state: MatchPlayState,
  pub why: String,
}
