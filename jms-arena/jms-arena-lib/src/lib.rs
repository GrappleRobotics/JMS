use jms_core_lib::{models::AllianceStationId, db::DBDuration};

pub const ARENA_STATE_KEY: &'static str = "arena:state";
pub const ARENA_MATCH_KEY: &'static str = "arena:match";

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

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct SerialisedLoadedMatch {
  pub match_id: String,
  pub remaining: DBDuration,
  pub endgame: bool,
  pub state: MatchPlayState
}

/* ALLIANCE STATIONS */
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct AllianceStation {
  pub id: AllianceStationId,
  pub team: Option<u16>,
  pub bypass: bool,
  pub estop: bool,
  pub astop: bool
}

impl AllianceStation {
  pub fn default(id: AllianceStationId) -> Self {
    Self {
      id,
      team: None,
      bypass: false,
      estop: false,
      astop: false
    }
  }
}

/* RPC */

#[jms_macros::service]
pub trait ArenaRPC {
  async fn signal(signal: ArenaSignal, source: String) -> Result<(), String>;

  async fn load_match(id: String) -> Result<(), String>;
  async fn load_test_match() -> Result<(), String>;
  async fn unload_match() -> Result<(), String>;
}