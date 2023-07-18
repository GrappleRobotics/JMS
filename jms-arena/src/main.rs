pub mod matches;

use std::time::Duration;

use jms_arena_lib::{ArenaSignal, ArenaSignalMessage, ArenaState, MatchPlayState};
use jms_base::{logging, redis::{redis_connect, JsonAsyncCommands, AsyncCommands}, mq::MessageQueue};
use log::{info, error};
use matches::LoadedMatch;

struct Arena {
  redis: jms_base::redis::aio::Connection,
  mq: MessageQueue,

  last_state: Option<ArenaState>,
  state: ArenaState,

  current_match: Option<LoadedMatch>
}

impl Arena {
  pub async fn new(redis: jms_base::redis::aio::Connection, mq: MessageQueue) -> Self {
    Self {
      redis, mq,
      state: ArenaState::Init,
      last_state: None,

      current_match: None,
    }
  }

  pub async fn set_state(&mut self, new_state: ArenaState) -> anyhow::Result<()> {
    info!("Arena State Change {:?} -> {:?}...", self.state, new_state);
    self.last_state = Some(self.state);
    self.state = new_state;

    self.redis.json_set("arena:state", "$", &self.state).await?;
    self.mq.publish("arena.state.new", new_state).await?;

    Ok(())
  }

  pub async fn request_network(&mut self) -> anyhow::Result<()> {
    info!("Requesting Network Start");
    self.redis.hset("arena:network", "ready", false).await?;
    self.mq.publish("arena.network.start", ()).await?;    // TODO: Move this to RPC?
    Ok(())
  }

  pub async fn commit_scores(&mut self) -> anyhow::Result<()> {
    info!("Committing Scores");
    self.mq.publish("arena.scores.publish", ()).await?;
    Ok(())
  }

  pub async fn spin_once(&mut self, signal: Option<ArenaSignalMessage>) -> anyhow::Result<()> {
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

        if signal.map(|s| s.signal) == Some(ArenaSignal::EstopReset) {
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

        if self.redis.hget("arena:network", "ready").await? {
          info!("Network Ready");
          self.set_state(ArenaState::Idle { net_ready: true }).await?;
        }
      },
      ArenaState::Idle { net_ready: true } => {
        if signal.map(|s| s.signal) == Some(ArenaSignal::Prestart) {
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

        if self.redis.hget("arena:network", "ready").await? {
          info!("Network Ready");
          self.set_state(ArenaState::Prestart { net_ready: true }).await?;
        }
      },
      ArenaState::Prestart { net_ready: true } => {
        match signal {
          Some(sig) => match sig.signal {
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
        if signal.map(|s| s.signal) == Some(ArenaSignal::MatchPlay) {
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

        if self.redis.hget("arena:network", "ready").await? {
          info!("Network Ready");
          self.set_state(ArenaState::Prestart { net_ready: true }).await?;
        }
      },
      ArenaState::MatchComplete { net_ready: true } => {
        if signal.map(|s| s.signal) == Some(ArenaSignal::MatchCommit) {
          self.commit_scores().await?;
          self.set_state(ArenaState::Reset).await?;
        }
      },
    }

    match self.current_match.as_ref() {
      Some(m) => m.write_state(&mut self.redis).await?,
      None => self.redis.del("arena:match").await?,
    }

    Ok(())
  }

  pub async fn run(&mut self) -> anyhow::Result<()> {
    let mut interval = tokio::time::interval(Duration::from_millis(1000 / 50));
    let mut signal_consumer = self.mq.rpc_subscribe("arena.signal", "arena-signal", "arena", false).await?;

    loop {
      tokio::select! {
        msg = signal_consumer.next() => match msg {
          Some(Ok(msg)) => {
            let reply = self.spin_once(Some(msg.data)).await.map_err(|e| { error!("{}", e); format!("{}", e) });
            self.mq.rpc_reply(&msg.properties, reply).await?;
            interval.reset();
          },
          Some(Err(e)) => {
            error!("Error in receiving from Rabbit: {}", e);
          },
          None => ()
        },
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
  info!("Connecting to Redis");
  let (_, redis_connection) = redis_connect().await?;
  info!("Connecting to Message Queue");
  let mq = MessageQueue::new("arena-reply").await?;
  info!("Connected!");

  
  let mut arena = Arena::new(redis_connection, mq).await;
  arena.run().await?;

  Ok(())
}
