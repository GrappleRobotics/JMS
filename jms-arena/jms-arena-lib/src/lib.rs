use jms_base::kv;
use jms_core_lib::{models::{AllianceStationId, AllianceParseError, Alliance}, db::{DBDuration, Table}};

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
#[derive(jms_macros::DbPartialUpdate, jms_macros::Updateable)]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
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

impl AllianceStation {
  pub fn sorted(db: &kv::KVConnection) -> anyhow::Result<Vec<AllianceStation>> {
    let mut v = Self::all(db)?;
    v.sort();
    Ok(v)
  }
}

impl PartialOrd for AllianceStation {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    match (self.id, other.id) {
      (a, b) if a.alliance == b.alliance => a.station.partial_cmp(&b.station),
      (a, _) => if a.alliance == Alliance::Blue { Some(std::cmp::Ordering::Less) } else { Some(std::cmp::Ordering::Greater) }
    }
  }
}

impl Ord for AllianceStation {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
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