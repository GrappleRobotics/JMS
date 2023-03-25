use std::{sync::{Arc, atomic::{AtomicBool, Ordering}}, time::Duration};

use tokio::{sync::{mpsc, RwLock}, task::JoinHandle};

use crate::{network::{NetworkResult, NetworkProvider}, scoring::scores::MatchScore, db, models::{self, Alliance, StationStatusRecord}};

use super::{state::{ArenaState, ArenaSignal}, matches::{LoadedMatch, MatchPlayState}, audience::AudienceDisplay, station::{AllianceStation, AllianceStationId}};

#[derive(Clone)]
pub struct Arena
where
  Self: Send + Sync,
{
  a: Arc<ArenaImpl>,
  signal_channel: mpsc::Sender<ArenaSignal>
}

impl Arena {
  pub fn new(arena: Arc<ArenaImpl>) -> Self {
    Self { signal_channel: arena.signal_channel.0.clone(), a: arena }
  }

  pub async fn state(&self) -> ArenaState {
    self.a.state.read().await.clone()
  }

  pub async fn audience(&self) -> AudienceDisplay {
    self.a.audience.read().await.clone()
  }

  pub async fn current_match(&self) -> Option<LoadedMatch> {
    self.a.current_match.read().await.clone()
  }

  pub async fn score(&self) -> MatchScore {
    self.a.score.read().await.clone()
  }

  pub async fn stations(&self) -> Vec<AllianceStation> {
    let mut stns = vec![];
    for stn in &self.a.stations {
      stns.push(stn.read().await.clone())
    }
    stns
  }

  pub async fn station_for_id(&self, id: AllianceStationId) -> Option<&ArenaLock<AllianceStation>> {
    for stn in &self.a.stations {
      if stn.read().await.station == id {
        return Some(stn)
      }
    }
    None
  }

  pub async fn station_for_team(&self, team: Option<u16>) -> Option<&ArenaLock<AllianceStation>> {
    match team {
      None => None,
      Some(team) => {
        for stn in &self.a.stations {
          if stn.read().await.team == Some(team) {
            return Some(stn)
          }
        }

        None
      }
    }
  }

  pub async fn signal(&self, signal: ArenaSignal) -> anyhow::Result<()> {
    self.signal_channel.send(signal).await?;
    Ok(())
  }

  pub fn arena_impl(&self) -> Arc<ArenaImpl> {
    self.a.clone()
  }
}

// type MaybeSharedNetwork = Option<Arc<Mutex<Box<dyn NetworkProvider + Send + Sync>>>>;
type ArenaLock<T> = RwLock<T>;

pub struct ArenaNetwork {
  provider: Arc<Box<dyn NetworkProvider + Send + Sync>>,
  handle: Option<JoinHandle<NetworkResult<()>>>
}

pub struct ArenaImpl {
  pub state: ArenaLock<ArenaState>,
  state_has_changed: AtomicBool,
  signal_channel: (mpsc::Sender<ArenaSignal>, ArenaLock<mpsc::Receiver<ArenaSignal>>),
  shutdown: AtomicBool,
  network: ArenaLock<Option<ArenaNetwork>>,
  
  pub audience: ArenaLock<AudienceDisplay>,
  pub score: ArenaLock<MatchScore>,
  pub current_match: ArenaLock<Option<LoadedMatch>>,

  pub stations: [ ArenaLock<AllianceStation>; 6 ],
  pub station_records: [ ArenaLock<StationStatusRecord>; 6 ]
}

impl ArenaImpl {
  pub fn new(network: Option<Box<dyn NetworkProvider + Send + Sync>>) -> Self {
    let sigchan = mpsc::channel(32);
    Self {
      state: ArenaLock::new(ArenaState::Init),
      state_has_changed: AtomicBool::new(false),
      audience: ArenaLock::new(AudienceDisplay::Field),
      signal_channel: (sigchan.0, ArenaLock::new(sigchan.1)),
      shutdown: AtomicBool::new(false),
      network: ArenaLock::new(network.map(|n| ArenaNetwork {
        provider: Arc::new(n),
        handle: None
      })),
      score: ArenaLock::new(MatchScore::new(3, 3)),
      current_match: ArenaLock::new(None),
      stations: [
        ArenaLock::new(AllianceStation::new(AllianceStationId { alliance: Alliance::Blue, station: 1 })),
        ArenaLock::new(AllianceStation::new(AllianceStationId { alliance: Alliance::Blue, station: 2 })),
        ArenaLock::new(AllianceStation::new(AllianceStationId { alliance: Alliance::Blue, station: 3 })),
        ArenaLock::new(AllianceStation::new(AllianceStationId { alliance: Alliance::Red, station: 1 })),
        ArenaLock::new(AllianceStation::new(AllianceStationId { alliance: Alliance::Red, station: 2 })),
        ArenaLock::new(AllianceStation::new(AllianceStationId { alliance: Alliance::Red, station: 3 })),
      ],
      station_records: [
        ArenaLock::new(vec![]),
        ArenaLock::new(vec![]),
        ArenaLock::new(vec![]),
        ArenaLock::new(vec![]),
        ArenaLock::new(vec![]),
        ArenaLock::new(vec![]),
      ]
    }
  }

  pub async fn load_match(&self, m: LoadedMatch) -> anyhow::Result<()> {
    let state = self.state.read().await;
    match &*state {
      ArenaState::Idle { .. } => {
        self.reset_stations().await;

        // Load teams for match
        for stn in &self.stations {
          let mut stn = stn.write().await;

          let match_teams = match stn.station.alliance {
            Alliance::Blue => &m.match_meta.blue_teams,
            Alliance::Red => &m.match_meta.red_teams
          };

          let i = (stn.station.station - 1) as usize;
          stn.team = match_teams.get(i).and_then(|t| t.map(|t| t as u16));
        }

        *self.current_match.write().await = Some(m);
        Ok(())
      },
      s => anyhow::bail!("Can't load match in state {}", s)
    }
  }

  pub async fn unload_match(&self) -> anyhow::Result<()> {
    let state = self.state.read().await;
    match &*state {
      ArenaState::Idle { .. } => {
        *self.current_match.write().await = None;
        self.reset_stations().await;
        Ok(())
      },
      s => anyhow::bail!("Can't unload match in state {}", s)
    }
  }

  async fn reset_stations(&self) {
    for stn in &self.stations {
      stn.write().await.reset();
    }
    for record in &self.station_records {
      record.write().await.clear();
    }
  }

  async fn spin_once(&self, signal: Option<ArenaSignal>) -> anyhow::Result<()> {
    // Estop takes priority
    if signal == Some(ArenaSignal::Estop) {
      self.set_state(ArenaState::Estop).await
    }

    let state = self.state.read().await.clone();
    let first = self.state_is_new();
    self.state_has_changed.store(false, Ordering::SeqCst);

    let mut current_match = self.current_match.write().await;

    match state {
      ArenaState::Init => {
        self.set_state(ArenaState::Idle { net_ready: false } ).await
      },
      ArenaState::Estop => {
        if let Some(m) = &mut *current_match {
          m.fault();
        }

        if signal == Some(ArenaSignal::EstopReset) {
          self.set_state(ArenaState::Idle { net_ready: false }).await
        }
      },

      ArenaState::Idle { net_ready: false } => {
        // Idle Not Ready
        if first {
          self.start_network_config().await;

          // Need to drop the current match since unload_match() takes the lock
          // It's messy, but it's the tradeoff we have for having the match in an RwLock
          drop(current_match);
          self.unload_match().await?;
          current_match = self.current_match.write().await;
        }

        if let Some(result) = self.poll_network().await {
          match result {
            Err(e) => anyhow::bail!("Network Configuration Error: {:?}", e), // TODO: retry
            Ok(_) => { self.set_state(ArenaState::Idle { net_ready: true }).await }
          }
        }
      },
      ArenaState::Idle { net_ready: true } => {
        // Idle Ready
        if signal == Some(ArenaSignal::Prestart) {
          match &*current_match {
            Some(m) if m.state == MatchPlayState::Waiting => {
              self.set_state(ArenaState::Prestart { net_ready: false }).await
            },
            Some(m) => anyhow::bail!("Cannot Prestart when Match is in state: {:?}", m.state),
            None => anyhow::bail!("Cannot prestart without a match loaded!")
          }
        }
      },

      ArenaState::Prestart { net_ready: false } => {
        // Prestart Not Ready
        if first {
          // Reset estops
          for stn in &self.stations {
            let mut stn = stn.write().await;
            stn.astop = false;
            stn.estop = false;
          }
          // Reset scores
          *self.score.write().await = MatchScore::new(3, 3);
          self.start_network_config().await;
        }

        if let Some(result) = self.poll_network().await {
          match result {
            Err(e) => anyhow::bail!("Network Configuration Error: {:?}", e), // TODO: retry
            Ok(_) => { self.set_state(ArenaState::Prestart { net_ready: true }).await }
          }
        }
      },
      ArenaState::Prestart { net_ready: true } => {
        // TODO: Resources

        match signal {
          Some(ArenaSignal::MatchArm { force }) => {
            // TODO: If resources_ok, else error
            self.set_state(ArenaState::MatchArmed).await
          },
          Some(ArenaSignal::Prestart)     => self.set_state(ArenaState::Prestart { net_ready: false }).await,
          Some(ArenaSignal::PrestartUndo) => self.set_state(ArenaState::Idle { net_ready: false }).await,
          _ => ()
        }
      },

      ArenaState::MatchArmed => {
        // TODO: Resources
        match signal {
          Some(ArenaSignal::MatchPlay) => {
            self.set_state(ArenaState::MatchPlay).await;
          },
          _ => ()
        }
      },
      ArenaState::MatchPlay => {
        // TODO: Station resources
        let current_match = current_match.as_mut().unwrap();
        if first {
          *self.audience.write().await = AudienceDisplay::MatchPlay;
          current_match.start()?;
        }

        current_match.update();

        match current_match.state {
          MatchPlayState::Complete => self.set_state(ArenaState::MatchComplete { net_ready: false }).await,
          _ => ()
        }
      },
      ArenaState::MatchComplete { net_ready: false }  => {
        // TODO: Commit station records

        if let Some(result) = self.poll_network().await {
          match result {
            Err(e) => anyhow::bail!("Network Configuration Error: {:?}", e), // TODO: retry
            Ok(_) => { self.set_state(ArenaState::MatchComplete { net_ready: true }).await }
          }
        }
      },
      ArenaState::MatchComplete { net_ready: true }  => {
        if let Some(ArenaSignal::MatchCommit) = signal {
          let current_match = current_match.as_mut().unwrap();
          let m = {
            let score = self.score.read().await;
            current_match.match_meta.commit(&score, db::database()).await?
          };
          
          *self.audience.write().await = AudienceDisplay::MatchResults(models::SerializedMatch::from(m.clone()));
          // Reset scores
          *self.score.write().await = MatchScore::new(3, 3);
          self.set_state(ArenaState::Idle { net_ready: false }).await;
        }
      }
    }

    // Update match if applicable 
    if let Some(m) = &mut *current_match {
      m.update();
    }

    // TODO: Update station commands / states

    Ok(())
  }

  async fn set_state(&self, state: ArenaState) {
    let mut current_state = self.state.write().await;

    let last_state = current_state.clone();
    *current_state = state;

    info!("State Transition: {:?} -> {:?}", last_state, current_state);
    self.state_has_changed.store(true, Ordering::SeqCst);
  }

  fn state_is_new(&self) -> bool {
    self.state_has_changed.load(Ordering::SeqCst)
  }

  // TODO: Store network config futures in a vec, pop it once it's complete
  async fn start_network_config(&self) {
    let mut stations = vec![];
    for stn in &self.stations {
      stations.push(stn.read().await.clone());
    }

    let mut nw = self.network.write().await;
    match &mut *nw {
      Some(nw) => {
        let provider = nw.provider.clone();
        nw.handle = Some(tokio::task::spawn(async move {
          info!("Configuring Network....");
          // TODO: Fill stations
          
          provider.configure(&stations[..]).await
        }));
      },
      None => (),
    }
  }

  async fn poll_network(&self) -> Option<NetworkResult<()>> {
    let mut nw = self.network.write().await;
    match &mut *nw {
      None => Some(Ok(())),
      Some(nw) => match &mut nw.handle {
        None => None,
        Some(jh) => {
          if jh.is_finished() {
            Some(jh.await.unwrap())
          } else {
            None
          }
        }
      }
    }
  }

  #[allow(dead_code)]
  #[tokio::main(flavor = "current_thread")]
  pub async fn run(&self) -> anyhow::Result<()> {
    let mut spin_interval = tokio::time::interval(Duration::from_millis(100));
    let mut signal_chan = self.signal_channel.1.write().await;

    spin_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    while !self.shutdown.load(Ordering::Relaxed) {
      tokio::select! {
        signal = signal_chan.recv() => match signal {
          None => anyhow::bail!("No signals can possibly be received"),
          Some(signal) => {
            self.spin_once(Some(signal)).await?;
            spin_interval.reset();
          }
        },
        _ = spin_interval.tick() => {
          self.spin_once(None).await?;
        }
      }
    }

    Ok(())
  }

}