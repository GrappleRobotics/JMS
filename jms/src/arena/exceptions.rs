use std::error;

use crate::network::NetworkError;

use super::{matches::MatchPlayState, ArenaState};

pub type ArenaResult<T> = std::result::Result<T, ArenaError>;
pub type MatchResult<T> = std::result::Result<T, MatchError>;

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

// Arena

#[derive(Debug)]
pub enum ArenaError {
  IllegalStateChange(StateTransitionError),
  UnimplementedStateError(ArenaState),
  NetworkError(NetworkError),
  MatchError(MatchError),
  CannotLoadMatchError(String)
}

impl std::fmt::Display for ArenaError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match *self {
      ArenaError::IllegalStateChange(ref e) => write!(f, "{}", e),
      ArenaError::UnimplementedStateError(ref s) => write!(f, "Unimplemented State: {}", s),
      ArenaError::NetworkError(ref e) => write!(f, "Network Error: {}", e),
      ArenaError::MatchError(ref e) => write!(f, "Match Error: {}", e),
      ArenaError::CannotLoadMatchError(ref s) => write!(f, "Could not load match: {}", s),
    }
  }
}

impl error::Error for ArenaError {
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    match *self {
      ArenaError::NetworkError(ref e) => Some(e),
      ArenaError::MatchError(ref e) => Some(e),
      _ => None,
    }
  }
}

impl From<NetworkError> for ArenaError {
  fn from(err: NetworkError) -> ArenaError {
    ArenaError::NetworkError(err)
  }
}

impl From<MatchError> for ArenaError {
  fn from(err: MatchError) -> ArenaError {
    ArenaError::MatchError(err)
  }
}

// Match
#[derive(Debug)]
pub enum MatchError {
  IllegalStateChange {
    from: MatchPlayState,
    to: MatchPlayState,
    why: String,
  },
}

impl std::fmt::Display for MatchError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match *self {
      MatchError::IllegalStateChange {
        ref from,
        ref to,
        ref why,
      } => {
        write!(f, "Illegal State Change: {:?} -> {:?} ({})", from, to, why)
      }
    }
  }
}

impl error::Error for MatchError {
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    None
  }
}
