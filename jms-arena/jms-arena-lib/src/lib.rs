use jms_core_lib::{models::{AllianceStationId, AllianceParseError}, db::{DBDuration, Table}};

pub const ARENA_STATE_KEY: &'static str = "arena:state";
pub const ARENA_MATCH_KEY: &'static str = "arena:match";

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(tag = "state")]
pub enum ArenaState {
  Init,
  Reset,
  Idle,
  Estop,
  Prestart,
  MatchArmed,
  MatchPlay,
  MatchComplete
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
#[derive(jms_macros::DbPartialUpdate, Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct AllianceStation {
  pub id: AllianceStationId,
  pub team: Option<usize>,
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

impl Table for AllianceStation {
  const PREFIX: &'static str = "arena:station";
  type Id = AllianceStationId;
  type Err = AllianceParseError;

  fn id(&self) -> Self::Id {
    self.id.clone()
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