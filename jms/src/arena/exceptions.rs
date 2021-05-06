use super::ArenaState;

pub type ArenaResult<T> = std::result::Result<T, ArenaError>;

#[derive(Debug, Clone)]
pub enum ArenaError {
  Misc(String),
  IllegalStateChange { from: ArenaState, to: ArenaState, condition: String },
  AlreadyInState(ArenaState),
  UnimplementedStateError(ArenaState)
}

impl std::fmt::Display for ArenaError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ArenaError::Misc(s) => write!(f, "ArenaError::Misc: {}", s),
      ArenaError::IllegalStateChange {from, to, condition} => {
        write!(f, "ArenaError: Illegal State Change from {:?} to {:?} (failed condition: {})", from, to, condition)
      },
      ArenaError::AlreadyInState(s) => write!(f, "Already in state {:?}", s),
      ArenaError::UnimplementedStateError(s) => write!(f, "Unimplemented state: {:?}", s),
    }
  }
}