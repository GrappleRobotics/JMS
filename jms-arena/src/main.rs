pub mod matches;

use std::time::Duration;

use jms_arena_lib::{ArenaSignal, ArenaState, MatchPlayState, ArenaRPC};
use jms_base::{logging, kv::KVConnection, mq::{MessageQueueChannel, MessageQueue}};
use log::info;
use matches::LoadedMatch;

struct Arena {
  kv: KVConnection,
  mq: MessageQueueChannel,

  last_state: Option<ArenaState>,
  state: ArenaState,

  current_match: Option<LoadedMatch>
}

impl Arena {
  pub async fn new(kv: KVConnection, mq: MessageQueueChannel) -> Self {
    Self {
      kv, mq,
      state: ArenaState::Init,
      last_state: None,

      current_match: None,
    }
  }

  pub async fn set_state(&mut self, new_state: ArenaState) -> anyhow::Result<()> {
    info!("Arena State Change {:?} -> {:?}...", self.state, new_state);
    self.last_state = Some(self.state);
    self.state = new_state;

    self.kv.json_set("arena:state", "$", &self.state).await?;
    self.mq.publish("arena.state.new", new_state).await?;

    Ok(())
  }

  pub async fn request_network(&mut self) -> anyhow::Result<()> {
    info!("Requesting Network Start");
    self.kv.hset("arena:network", "ready", false).await?;
    self.mq.publish("arena.network.start", ()).await?;    // TODO: Move this to RPC?
    Ok(())
  }

  pub async fn commit_scores(&mut self) -> anyhow::Result<()> {
    info!("Committing Scores");
    self.mq.publish("arena.scores.publish", ()).await?;
    Ok(())
  }

  pub async fn spin_once(&mut self, signal: Option<ArenaSignal>) -> anyhow::Result<()> {
    let first = self.last_state != Some(self.state);
    self.last_state = Some(self.state);

    // Run through match logic
    match self.state.clone() {
      ArenaState::Init => {
        self.set_state(ArenaState::Reset).await?;
      },
      ArenaState::Estop => {
        if let Some(m) = self.current_match.as_mut() {
          m.fault();
        }

        if signal == Some(ArenaSignal::EstopReset) {
          self.set_state(ArenaState::Reset).await?;
        }
      },
      ArenaState::Reset => {
        self.set_state(ArenaState::Idle { net_ready: false }).await?;
      },
      ArenaState::Idle { net_ready: false } => {
        if first {
          self.request_network().await?;
        }

        if self.kv.hget("arena:network", "ready").await? {
          info!("Network Ready");
          self.set_state(ArenaState::Idle { net_ready: true }).await?;
        }
      },
      ArenaState::Idle { net_ready: true } => {
        if signal == Some(ArenaSignal::Prestart) {
          match &self.current_match {
            Some(m) if m.state == MatchPlayState::Waiting => {
              self.set_state(ArenaState::Prestart { net_ready: false }).await?;
            },
            Some(m) => anyhow::bail!("Cannot Prestart when Match is in state: {:?}", m.state),
            None => anyhow::bail!("Cannot prestart without a match loaded!")
          }
        }
      },
      ArenaState::Prestart { net_ready: false } => {
        if first {
          self.request_network().await?;
        }

        if self.kv.hget("arena:network", "ready").await? {
          info!("Network Ready");
          self.set_state(ArenaState::Prestart { net_ready: true }).await?;
        }
      },
      ArenaState::Prestart { net_ready: true } => {
        match signal {
          Some(sig) => match sig {
            ArenaSignal::MatchArm { force } => {
              // TODO: If consensus says ready (how to do that? maybe scan over a subnamespace?)
              self.set_state(ArenaState::MatchArmed).await?;
            },
            ArenaSignal::Prestart => self.set_state(ArenaState::Prestart { net_ready: false }).await?,
            ArenaSignal::PrestartUndo => self.set_state(ArenaState::Idle { net_ready: false }).await?,
            _ => ()
          },
          _ => ()
        }
      },
      ArenaState::MatchArmed => {
        if signal == Some(ArenaSignal::MatchPlay) {
          self.set_state(ArenaState::MatchPlay).await?;
        }
      },
      ArenaState::MatchPlay => {
        let current_match = self.current_match.as_mut().ok_or(anyhow::anyhow!("No match present!"))?;
        if first {
          current_match.start()?;
        }

        current_match.update().await?;

        match current_match.state {
          MatchPlayState::Complete => { self.set_state(ArenaState::MatchComplete { net_ready: false }).await?; },
          _ => ()
        }
      },
      ArenaState::MatchComplete { net_ready: false } => {
        if first {
          self.request_network().await?;
        }

        if self.kv.hget("arena:network", "ready").await? {
          info!("Network Ready");
          self.set_state(ArenaState::Prestart { net_ready: true }).await?;
        }
      },
      ArenaState::MatchComplete { net_ready: true } => {
        if signal == Some(ArenaSignal::MatchCommit) {
          self.commit_scores().await?;
          self.set_state(ArenaState::Reset).await?;
        }
      },
    }

    match self.current_match.as_ref() {
      Some(m) => m.write_state(&mut self.kv).await?,
      None => self.kv.del("arena:match").await?,
    }

    Ok(())
  }
}

#[async_trait::async_trait]
impl ArenaRPC for Arena {
  fn mq(&self) -> &MessageQueueChannel {
    &self.mq
  }

  async fn signal(&mut self, signal: ArenaSignal, _source: String) -> Result<(), String> {
    self.spin_once(Some(signal)).await.map_err(|e| format!("{}", e))
  }

  async fn load_test_match(&mut self) -> Result<(), String> {
    info!("Loading Test Match...");
    match self.state {
      ArenaState::Idle { .. } => {
        self.current_match = Some(LoadedMatch::new());
        Ok(())
      },
      _ => Err(format!("Can't load match in state: {:?}", self.state))
    }
  }

  async fn unload_match(&mut self) -> Result<(), String> {
    info!("Unloading Match...");
    match self.state {
      ArenaState::Idle { .. } => {
        self.current_match = None;
        Ok(())
      },
      _ => Err(format!("Can't unload match in state: {:?}", self.state))
    }
  }
}

impl Arena {
  async fn run(&mut self) -> anyhow::Result<()> {
    let mut interval = tokio::time::interval(Duration::from_millis(1000 / 50));
    let mut rpc = self.rpc_handle().await?;

    loop {
      tokio::select! {
        msg = rpc.next() => self.rpc_process(msg).await?,
        _ = interval.tick() => {
          self.spin_once(None).await?;
        }
      }
    }
  }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  logging::configure(false);
  let kv = KVConnection::new().await?;
  let mq = MessageQueue::new("arena-reply").await?;
  info!("Connected!");

  let mut arena = Arena::new(kv, mq.channel().await?).await;
  arena.run().await?;

  Ok(())
}
