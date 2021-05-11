use std::error;

use crate::network::NetworkError;

use super::ArenaState;

pub type ArenaResult<T> = std::result::Result<T, ArenaError>;

#[derive(Debug)]
pub struct StateTransitionError {
  pub from: ArenaState,
  pub to: ArenaState,
  pub why: String,
}

impl std::fmt::Display for StateTransitionError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "State Transition Error: {:?} to {:?} ({})",
      self.from, self.to, self.why
    )
  }
}

#[derive(Debug)]
pub enum ArenaError {
  IllegalStateChange(StateTransitionError),
  UnimplementedStateError(ArenaState),
  NetworkError(NetworkError),
}

impl std::fmt::Display for ArenaError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match *self {
      ArenaError::IllegalStateChange(ref e) => write!(f, "{}", e),
      ArenaError::UnimplementedStateError(ref s) => write!(f, "Unimplemented State: {}", s),
      ArenaError::NetworkError(ref e) => write!(f, "Network Error: {}", e),
    }
  }
}

impl error::Error for ArenaError {
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    match *self {
      ArenaError::NetworkError(ref e) => Some(e),
      _ => None,
    }
  }
}

impl From<NetworkError> for ArenaError {
  fn from(err: NetworkError) -> ArenaError {
    ArenaError::NetworkError(err)
  }
}
