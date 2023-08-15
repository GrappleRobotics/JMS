use std::{error::Error, convert::Infallible};

use jms_base::{kv, mq::{MessageQueueSubscriber, MessageQueueChannel}};
use jms_core_lib::{models::{AllianceStationId, AllianceParseError, Alliance, JmsComponent}, db::{DBDuration, Table}};

pub const ARENA_STATE_KEY: &'static str = "arena:state";
pub const ARENA_MATCH_KEY: &'static str = "arena:match";

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(tag = "state")]
pub enum ArenaState {
  Init,
  Reset { ready: bool },
  Idle,
  Estop,
  Prestart { ready: bool },
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
  pub match_time: Option<DBDuration>,
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
  async fn unload_match() -> Result<(), String>;
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct HookReply {
  pub id: String,
  pub failure: Option<String>
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArenaHookDB {
  pub id: String,
  pub component_id: String,
  pub state: ArenaState,
  pub timeout: std::time::Duration
}

impl Table for ArenaHookDB {
  const PREFIX: &'static str = "db:arena_hook";
  type Err = Infallible;
  type Id = String;

  fn id(&self) -> Self::Id {
    self.id.clone()
  }
}

pub struct ArenaStateHook {
  pub id: String,
  pub component_id: String,
  pub state: ArenaState,
  pub timeout: std::time::Duration,
  sub: MessageQueueSubscriber<ArenaState>
}

impl ArenaStateHook {
  pub async fn new(id: &str, component: &JmsComponent, state: ArenaState, timeout: std::time::Duration, kv: &kv::KVConnection, mq: &MessageQueueChannel) -> anyhow::Result<Self> {
    let me = Self { 
      id: id.to_owned(),
      component_id: component.id.clone(),
      state,
      timeout,
      sub: mq.subscribe("arena.state.new", &format!("hook-{}", id), &format!("hook-{}", id), false).await?
    };
    me.insert(&kv)?;
    Ok(me)
  }

  pub fn insert(&self, kv: &kv::KVConnection) -> anyhow::Result<()> {
    ArenaHookDB { id: self.id.clone(), component_id: self.component_id.clone(), state: self.state.clone(), timeout: self.timeout.clone() }.insert(kv)
  }

  pub async fn next(&mut self) -> anyhow::Result<ArenaState> {
    loop {
      let next = self.sub.next().await;
      if let Some(next) = next {
        let data = next?.data;
        if data == self.state {
          return Ok(data)
        }
      }
    }
  }

  pub async fn success(&self, mq: &MessageQueueChannel) -> anyhow::Result<()> {
    mq.publish("arena.state.hook", HookReply {
      id: self.id.clone(),
      failure: None
    }).await?;
    Ok(())
  }

  pub async fn failure<E: ToString>(&self, err: E, mq: &MessageQueueChannel) -> anyhow::Result<()> {
    mq.publish("arena.state.hook", HookReply {
      id: self.id.clone(),
      failure: Some(err.to_string())
    }).await?;
    Ok(())
  }
}