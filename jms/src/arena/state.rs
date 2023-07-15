use schemars::JsonSchema;
use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display, Serialize, JsonSchema)]
#[serde(tag = "state")]
pub enum ArenaState {
  Init,
  Reset,
  Idle { net_ready: bool },
  Estop,
  Prestart { net_ready: bool },
  MatchArmed,
  MatchPlay,
  MatchComplete { net_ready: bool }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display, Deserialize, JsonSchema)]
pub enum ArenaSignal {
  Estop,
  EstopReset,
  Prestart,
  PrestartUndo,
  MatchArm {
    force: bool
  },
  MatchPlay,
  MatchCommit,
}