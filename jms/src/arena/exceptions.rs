use std::error;

use crate::network::NetworkError;

use super::ArenaState;

pub type ArenaResult<T> = std::result::Result<T, ArenaError>;

#[derive(Debug)]
pub enum ArenaError {
  Misc(String),
  IllegalStateChange {
    from: ArenaState,
    to: ArenaState,
    condition: String,
  },
  AlreadyInState(ArenaState),
  UnimplementedStateError(ArenaState),
  NetworkError(NetworkError)
}

impl std::fmt::Display for ArenaError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match *self {
      ArenaError::Misc(ref s) => write!(f, "ArenaError::Misc: {}", s),
      ArenaError::IllegalStateChange {
        ref from,
        ref to,
        ref condition,
      } => {
        write!(
          f,
          "ArenaError: Illegal State Change from {:?} to {:?} (failed condition: {})",
          from, to, condition
        )
      }
      ArenaError::AlreadyInState(ref s) => write!(f, "Already in state {:?}", s),
      ArenaError::UnimplementedStateError(ref s) => write!(f, "Unimplemented state: {:?}", s),
      ArenaError::NetworkError(ref e) => write!(f, "Network Error: {}", e),
    }
  }
}

impl error::Error for ArenaError {
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    match *self {
      ArenaError::NetworkError(ref e) => Some(e),
      _ => None
    }
  }
}

impl From<NetworkError> for ArenaError {
  fn from(err: NetworkError) -> ArenaError {
    ArenaError::NetworkError(err)
  }
}