#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
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

// #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
// pub struct ArenaSignalMessage {
//   pub signal: ArenaSignal,
//   pub source: String,
// }

// pub type ArenaSignalReply = Result<(), String>;

/* MATCHES */

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub enum MatchPlayState {
  Waiting,
  Warmup,
  Auto,
  Pause,
  Teleop,
  Cooldown,
  Complete,
  Fault
}

// #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
// pub enum LoadMatchCall {
//   TestMatch,
//   Unload
// }

// pub type LoadMatchReply = Result<(), String>;

/* RPC */
#[jms_macros::service]
pub trait ArenaRPC {
  async fn signal(signal: ArenaSignal, source: String) -> Result<(), String>;

  async fn load_test_match() -> Result<(), String>;
  async fn unload_match() -> Result<(), String>;
}