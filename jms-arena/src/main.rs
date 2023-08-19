pub mod matches;

use std::{time::{Duration, Instant}, collections::HashMap};

use jms_arena_lib::{ArenaSignal, ArenaState, MatchPlayState, ArenaRPC, AllianceStation, ARENA_STATE_KEY, ArenaHookDB, HookReply};
use jms_base::{kv::KVConnection, mq::{MessageQueueChannel, MessageQueue, MessageQueueSubscriber}, logging::JMSLogger};
use jms_core_lib::{models::{AllianceStationId, self, JmsComponent, Match, Alliance}, db::{Table, Singleton}, scoring::scores::MatchScore};
use log::{info, error};
use matches::LoadedMatch;

struct Arena {
  kv: KVConnection,
  mq: MessageQueueChannel,

  last_state: Option<ArenaState>,
  state: ArenaState,
  last_state_change: Instant,

  current_match: Option<LoadedMatch>,

  stations: HashMap<AllianceStationId, AllianceStation>,
  component: JmsComponent,

  hook_cache: Vec<ArenaHookDB>,
  hook_replies: HashMap<String, HookReply>
}

impl Arena {
  pub async fn new(kv: KVConnection, mq: MessageQueueChannel) -> Self {
    Self {
      kv, mq,
      state: ArenaState::Init,
      last_state: None,

      current_match: None,
      last_state_change: Instant::now(),

      stations: HashMap::new(),
      component: JmsComponent::new("jms.arena", "JMS-Arena", "A", 500),

      hook_cache: vec![],
      hook_replies: HashMap::new()
    }
  }

  pub async fn set_state(&mut self, new_state: ArenaState) -> anyhow::Result<()> {
    if new_state != self.state {
      info!("Arena State Change {:?} -> {:?}...", self.state, new_state);
      self.last_state = Some(self.state);
      self.state = new_state;

      self.hook_cache = ArenaHookDB::all(&self.kv)?;
      self.hook_replies = HashMap::new();
      self.last_state_change = Instant::now();

      self.kv.json_set(ARENA_STATE_KEY, "$", &self.state)?;
      self.mq.publish("arena.state.new", new_state).await?;

      self.kv.bgsave()?;
    }

    Ok(())
  }

  pub async fn commit_scores(&mut self, match_id: String) -> anyhow::Result<()> {
    info!("Committing Scores");
    // Save stations into the match so we get an accurate record of who competed
    if let Ok(mut m) = Match::get(&match_id, &self.kv) {
      // We have to load AllianceStation from the DB since the ones stored in the arena aren't hydrated with team number
      let stns = AllianceStation::sorted(&self.kv)?;

      m.red_teams = stns.iter().filter(|x| x.id.alliance == Alliance::Red).map(|x| x.team).collect();
      m.blue_teams = stns.iter().filter(|x| x.id.alliance == Alliance::Blue).map(|x| x.team).collect();

      m.insert(&self.kv)?;
    }
    // Notify scoring, etc
    self.mq.publish("arena.scores.publish", match_id).await?;
    Ok(())
  }

  pub async fn reset_stations(&mut self) -> anyhow::Result<()> {
    info!("Resetting Alliance Stations");
    self.stations.clear();
    for stn in AllianceStationId::all() {
      let stn_inst = AllianceStation::default(stn);
      stn_inst.insert(&self.kv)?;
      self.stations.insert(stn, stn_inst);
    }
    Ok(())
  }

  pub async fn ready(&mut self) -> anyhow::Result<bool> {
    for hook in &self.hook_cache {
      if hook.state == self.state.into() {
        if hook.timeout < (Instant::now() - self.last_state_change) {
          anyhow::bail!("Hook Timed Out: {}", hook.id);
        }

        if let Some(hr) = self.hook_replies.get(&hook.id) {
          if let Some(fail) = &hr.failure {
            anyhow::bail!("Hook Failed: {} - {}", hook.id, fail)
          }
        } else {
          return Ok(false);
        }
      }
    }
    Ok(true)
  }

  pub async fn spin_once(&mut self, signal: Option<ArenaSignal>) -> anyhow::Result<()> {
    let first = self.last_state != Some(self.state);
    self.last_state = Some(self.state);

    if signal == Some(ArenaSignal::Estop) && self.state != ArenaState::Estop {
      self.set_state(ArenaState::Estop).await?;
    }

    // Process physical E-Stop button for each station
    {
      let mut physical_astop = false;
      if let Some(current_match) = &self.current_match {
        if current_match.state == MatchPlayState::Auto {
          physical_astop = true;
        }
      }

      for stn_id in self.stations.keys() {
        if AllianceStation::get_physical_estop(stn_id.clone(), &self.kv)? {
          if physical_astop {
            AllianceStation::set_astop_by_id(stn_id.clone(), true, &self.kv)?;
          } else {
            AllianceStation::set_estop_by_id(stn_id.clone(), true, &self.kv)?;
          }
        }
      }
    }

    // Run through match logic
    match self.state.clone() {
      ArenaState::Init => {
        self.set_state(ArenaState::Reset { ready: false }).await?;
      },
      ArenaState::Estop => {
        if let Some(m) = self.current_match.as_mut() {
          m.fault();
        }

        if signal == Some(ArenaSignal::EstopReset) {
          self.set_state(ArenaState::Reset { ready: false }).await?;
        }
      },
      ArenaState::Reset { ready: false } => {
        if first {
          self.reset_stations().await?;
          self.current_match = None;
        }
        match self.ready().await {
          Ok(true) => { self.set_state(ArenaState::Reset { ready: true }).await? },
          Ok(false) => (),
          Err(e) => { error!("{}", e); self.set_state(ArenaState::Estop).await? }
        }
      },
      ArenaState::Reset { ready: true } => {
        self.set_state(ArenaState::Idle).await?;
      }
      ArenaState::Idle => {
        if first {
          for stn in self.stations.values_mut() {
            stn.set_estop(false, &self.kv)?;
            stn.set_astop(false, &self.kv)?;
            stn.set_physical_estop(false, &self.kv)?;
            stn.set_bypass(false, &self.kv)?;
          }
        }

        if signal == Some(ArenaSignal::Prestart) {
          match &self.current_match {
            Some(m) if m.state == MatchPlayState::Waiting => {
              self.set_state(ArenaState::Prestart { ready: false }).await?;
            },
            Some(m) => anyhow::bail!("Cannot Prestart when Match is in state: {:?}", m.state),
            None => anyhow::bail!("Cannot prestart without a match loaded!")
          }
        }
      },
      ArenaState::Prestart { ready: false } => {
        match self.ready().await {
          Ok(true) => { self.set_state(ArenaState::Prestart { ready: true }).await? },
          Ok(false) => (),
          Err(e) => { error!("{}", e); self.set_state(ArenaState::Estop).await? }
        }
      },
      ArenaState::Prestart { ready: true } => {
        match signal {
          Some(sig) => match sig {
            ArenaSignal::MatchArm { force: _ } => {
              // TODO: Force
              self.set_state(ArenaState::MatchArmed).await?;
            },
            ArenaSignal::PrestartUndo => self.set_state(ArenaState::Idle).await?,
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
          MatchPlayState::Complete => { self.set_state(ArenaState::MatchComplete).await?; },
          _ => ()
        }
      },
      ArenaState::MatchComplete => {
        if signal == Some(ArenaSignal::MatchCommit) {
          self.commit_scores(self.current_match.as_ref().ok_or(anyhow::anyhow!("No Match Present!"))?.match_id.clone()).await?;
          self.set_state(ArenaState::Reset { ready: false }).await?;
          self.current_match = None;
        }
      },
    }

    match self.current_match.as_ref() {
      Some(m) => m.write_state(&mut self.kv)?,
      None => self.kv.del("arena:match")?,
    }

    Ok(())
  }
}

#[async_trait::async_trait]
impl ArenaRPC for Arena {
  fn mq(&self) -> &MessageQueueChannel {
    &self.mq
  }

  async fn signal(&mut self, signal: ArenaSignal, source: String) -> Result<(), String> {
    info!("Signal: {:?} from {}", signal, source);
    self.spin_once(Some(signal)).await.map_err(|e| format!("{}", e))
  }

  async fn load_match(&mut self, id: String) -> Result<(), String> {
    let m = models::Match::get(&id, &self.kv).map_err(|e| e.to_string())?;
    match self.state {
      ArenaState::Idle { .. } => {
        // Load match
        self.current_match = Some(LoadedMatch::new(m.id()));

        // Set teams
        for (i, team) in m.blue_teams.into_iter().enumerate() {
          let id = AllianceStationId::new(models::Alliance::Blue, i + 1);
          self.stations.get_mut(&id).ok_or("No Station Available".to_string())?.set_team(team, &self.kv).map_err(|e| e.to_string())?;
        }

        for (i, team) in m.red_teams.into_iter().enumerate() {
          let id = AllianceStationId::new(models::Alliance::Red, i + 1);
          self.stations.get_mut(&id).ok_or("No Station Available".to_string())?.set_team(team, &self.kv).map_err(|e| e.to_string())?;
        }
        MatchScore::delete(&self.kv).map_err(|e| e.to_string())?;
        
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

        self.reset_stations().await.map_err(|e| e.to_string())?;
        MatchScore::delete(&self.kv).map_err(|e| e.to_string())?;

        Ok(())
      },
      _ => Err(format!("Can't unload match in state: {:?}", self.state))
    }
  }
}

impl Arena {
  async fn run(&mut self) -> anyhow::Result<()> {
    let mut interval = tokio::time::interval(Duration::from_millis(1000 / 20));
    let mut rpc = self.rpc_handle().await?;
    let mut hook_replies: MessageQueueSubscriber<HookReply> = self.mq.subscribe("arena.state.hook", "arena-hooks", "arena", false).await?;

    self.component.insert(&self.kv)?;

    loop {
      tokio::select! {
        msg = rpc.next() => self.rpc_process(msg).await?,
        reply = hook_replies.next() => match reply {
          Some(Ok(value)) => { self.hook_replies.insert(value.data.id.clone(), value.data); },
          Some(Err(e)) => error!("Hook Error: {}", e),
          None => ()
        },
        _ = interval.tick() => {
          self.spin_once(None).await?;
          self.component.tick(&self.kv)?;
        }
      }
    }
  }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let _ = JMSLogger::init().await?;
  let kv = KVConnection::new()?;
  let mq = MessageQueue::new("arena-reply").await?;
  info!("Connected!");

  let mut arena = Arena::new(kv, mq.channel().await?).await;
  arena.run().await?;

  Ok(())
}
