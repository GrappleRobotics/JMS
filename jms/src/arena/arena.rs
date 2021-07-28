use std::{
  mem,
  sync::{
    mpsc::{channel, Receiver, TryRecvError},
    Arc,
  },
};

use enum_as_inner::EnumAsInner;
use log::{error, info};
use tokio::sync::Mutex;

use super::{
  exceptions::{ArenaError, ArenaResult, StateTransitionError},
  matches::MatchPlayState,
  station::{AllianceStationId},
};

use crate::{arena::station::Alliance, ds::DSMode, log_expect, models::{self, MatchType}, network::{NetworkProvider, NetworkResult}};

use serde::{Deserialize, Serialize};

use super::matches::LoadedMatch;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display, EnumAsInner, Serialize)]
#[serde(tag = "state")]
pub enum ArenaState {
  Idle,       // Idle state
  Estop,      // Arena is emergency stopped and can only be unlocked by FTA
  EstopReset, // E-stop resetting...

  // Match Pipeline //
  Prestart { ready: bool, force: bool },
  MatchArmed,    // Arm the match - ensure field crew is off. Can revert to Prestart.
  MatchPlay,     // Currently running a match - handed off to Match runner
  MatchComplete, // Match just finished, waiting to commit. Refs can still change scores
  MatchCommit,   // Commit the match score - lock ref tablets, publish to TBA and Audience Display
}

#[derive(EnumAsInner)]
enum StateData {
  Idle,
  Estop,
  EstopReset,

  Prestart(Option<Receiver<NetworkResult<()>>>), // recv: network ready receiver
  MatchArmed,
  MatchPlay,
  MatchComplete,
  MatchCommit,
}

#[derive(Serialize)]
#[serde(transparent)]
pub struct BoundState {
  #[serde(skip)]
  first: bool, // First run?
  state: ArenaState,
  #[serde(skip)]
  data: StateData,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display, Deserialize)]
#[serde(tag = "signal")]
pub enum ArenaSignal {
  Estop,
  EstopReset,
  Prestart { force: bool },
  MatchArm,
  MatchPlay,
  MatchCommit,
}

// /**
//  * Who's permitted on the field?
//  */
// pub enum ArenaEntryCondition {
//   Locked,    // No one / FTA's discretion (no lights)
//   ResetOnly, // Field reset crew (purple lights)
//   Teams,     // Teams can collect robots (green lights)
//   Any,       // Anyone (Idle only - awards etc)
// }

#[derive(Debug, Clone, Copy, Serialize)]
pub struct AllianceStationDSReport {
  pub robot_ping: bool,
  pub rio_ping: bool,
  pub radio_ping: bool,
  pub battery: f64,

  pub estop: bool,
  pub mode: Option<DSMode>,

  pub pkts_sent: u16,
  pub pkts_lost: u16,
  pub rtt: u8,
}

impl Default for AllianceStationDSReport {
  fn default() -> Self {
    Self {
      robot_ping: false,
      rio_ping: false,
      radio_ping: false,
      battery: 0.0f64,
      estop: false,
      mode: None,
      pkts_sent: 0,
      pkts_lost: 0,
      rtt: 0
    }
  }
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub enum AllianceStationOccupancy {
  Vacant,
  Occupied,
  WrongStation,
  WrongMatch,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct AllianceStation {
  pub station: AllianceStationId,
  pub team: Option<u16>,
  pub bypass: bool,
  pub estop: bool,
  pub astop: bool, // TODO: Handle this
  pub ds_report: Option<AllianceStationDSReport>,
  pub occupancy: AllianceStationOccupancy,
}

impl AllianceStation {
  pub fn new(id: AllianceStationId) -> AllianceStation {
    return AllianceStation {
      station: id,
      team: None,
      bypass: false,
      estop: false,
      astop: false,
      ds_report: None,
      occupancy: AllianceStationOccupancy::Vacant,
    };
  }

  pub fn reset(&mut self) {
    self.team = None;
    self.bypass = false;
    self.estop = false;
    self.astop = false;
    self.ds_report = None;
    self.occupancy = AllianceStationOccupancy::Vacant;
  }

  pub fn can_arm_match(&self) -> bool {
    self.bypass || self.estop || (self.occupancy == AllianceStationOccupancy::Occupied)
  }
}

#[derive(Serialize)]
pub struct Arena {
  // network: Arc<Mutex<Option<Box<dyn NetworkProvider + Send>>>>,
  #[serde(skip)]
  network: Option<Arc<Mutex<Box<dyn NetworkProvider + Send + Sync>>>>,
  pub state: BoundState,
  #[serde(skip)]
  pending_state_change: Option<ArenaState>,
  #[serde(skip)]
  pending_signal: Arc<Mutex<Option<ArenaSignal>>>,
  #[serde(rename = "match")]
  pub current_match: Option<LoadedMatch>,
  pub stations: Vec<AllianceStation>,
}

pub type SharedArena = Arc<Mutex<Arena>>;

impl Arena {
  pub fn new(num_stations_per_alliance: u32, network: Option<Box<dyn NetworkProvider + Send + Sync>>) -> Arena {
    let mut a = Arena {
      network: network.map(|x| Arc::new(Mutex::new(x))),
      state: BoundState {
        first: true,
        state: ArenaState::Idle,
        data: StateData::Idle,
      },
      pending_state_change: None,
      pending_signal: Arc::new(Mutex::new(None)),
      current_match: None,
      stations: vec![],
    };

    for alliance in vec![Alliance::Blue, Alliance::Red] {
      for i in 1..(num_stations_per_alliance + 1) {
        a.stations
          .push(AllianceStation::new(AllianceStationId { alliance, station: i }));
      }
    }

    a
  }

  pub fn load_match(&mut self, m: LoadedMatch) -> ArenaResult<()> {
    match self.state.state {
      ArenaState::Idle => {
        self.load_match_teams(m.metadata())?;
        self.current_match = Some(m);
        Ok(())
      }
      ref s => Err(ArenaError::CannotLoadMatchError(format!(
        "Can't load match in state {}",
        s
      ))),
    }
  }

  fn load_match_teams(&mut self, m: &models::Match) -> ArenaResult<()> {
    for stn in self.stations.iter_mut() {
      let v = match stn.station.alliance {
        Alliance::Blue => &m.blue_teams,
        Alliance::Red => &m.red_teams,
      };
      
      stn.reset();

      let i = (stn.station.station - 1) as usize;
      if let Some(&t) = v.0.get(i) {
        stn.team = if t <= 0 { None } else { Some(t as u16) };
      } else {
        // Test matches are an exception - they start off blank
        if m.match_type != MatchType::Test {
          error!("{} does not have the correct amount of alliance members! Defaulting to None...", m.name());
        }
        stn.team = None;
      }
    }

    Ok(())
  }

  pub fn station_for_team(&self, team: Option<u16>) -> Option<AllianceStation> {
    match team {
      None => None,
      Some(team) => {
        self.stations.iter().find(|&&stn| stn.team == Some(team)).map(|&a| a) // Copy the AllianceStation to avoid reference lifetime issues
      }
    }
  }

  pub fn station_for_team_mut(&mut self, team: Option<u16>) -> Option<&mut AllianceStation> {
    match team {
      None => None,
      Some(team) => self.stations.iter_mut().find(|stn| stn.team == Some(team)),
    }
  }

  pub fn station_mut(&mut self, station: AllianceStationId) -> Option<&mut AllianceStation> {
    self.stations.iter_mut().find(|stn| stn.station == station)
  }

  async fn update_field_estop(&mut self) -> ArenaResult<()> {
    if self.state.state != ArenaState::Estop {
      if let Some(ArenaSignal::Estop) = self.current_signal().await {
        self.prepare_state_change(ArenaState::Estop)?;
      }
    }
    Ok(())
  }

  async fn update_states(&mut self) -> ArenaResult<()> {
    let first = self.state.first;
    let signal = self.current_signal().await;
    match (self.state.state, &mut self.state.data) {
      (ArenaState::Idle, _) => {
        if first {
          self.current_match = None;
          for stn in self.stations.iter_mut() {
            stn.reset();
          }
        } else if let Some(ArenaSignal::Prestart { force }) = signal {
          self.prepare_state_change(ArenaState::Prestart { ready: false, force })?;
        }
      }
      (ArenaState::Estop, _) => {
        // TODO: Implement transition out of estop
        if let Some(ref mut m) = self.current_match {
          // Fault the match - it can't be run and must be reloaded.
          m.fault();
        }

        if let Some(ArenaSignal::EstopReset) = signal {
          self.prepare_state_change(ArenaState::EstopReset)?;
        }
      }
      (ArenaState::EstopReset, _) => {
        // TODO:
        self.current_match = None;
        self.prepare_state_change(ArenaState::Idle)?;
      }
      (ArenaState::Prestart { ready: false, force }, StateData::Prestart(maybe_recv)) => {
        if first {
          info!("Prestart begin...")
        }

        if let Some(recv) = maybe_recv {
          // Check if network is ready
          let recv_result = recv.try_recv();
          match recv_result {
            Err(TryRecvError::Empty) => (), // Not ready yet
            Err(e) => panic!("Network runner fault: {}", e),
            Ok(result) => {
              result?;
              self.prepare_state_change(ArenaState::Prestart { ready: true, force })?;
            }
          };
        }
      }
      (ArenaState::Prestart { ready: true, force: _ }, _) => {
        if first {
          info!("Prestart Ready!")
        }
        if let Some(ArenaSignal::MatchArm) = signal {
          self.prepare_state_change(ArenaState::MatchArmed)?;
        }
      }
      (ArenaState::MatchArmed, _) => {
        if first {
          info!("Match Armed!")
        }
        if let Some(ArenaSignal::MatchPlay) = signal {
          self.prepare_state_change(ArenaState::MatchPlay)?;
        }
      }
      (ArenaState::MatchPlay, _) => {
        let m = self.current_match.as_mut().unwrap();
        if first {
          info!("Match play!");
          m.start()?;
        }
        if m.current_state() == MatchPlayState::Complete {
          self.prepare_state_change(ArenaState::MatchComplete)?;
        }
      }
      (ArenaState::MatchComplete, _) => {
        if first {
          info!("Match complete!")
        }
        if let Some(ArenaSignal::MatchCommit) = signal {
          self.prepare_state_change(ArenaState::MatchCommit)?;
        }
      }
      (ArenaState::MatchCommit, _) => {
        if first {
          self.current_match.as_mut().unwrap().commit_score().await?;
          self.prepare_state_change(ArenaState::Idle)?;
        }
      },
      (state, _) => Err(ArenaError::UnimplementedStateError(state))?,
    };
    Ok(())
  }

  pub fn can_change_state_to(&self, desired: ArenaState) -> ArenaResult<()> {
    let current = self.state.state;
    let illegal = move |why: &str| {
      ArenaError::IllegalStateChange(StateTransitionError {
        from: current,
        to: desired,
        why: why.to_owned(),
      })
    };

    if current == desired {
      return Err(illegal("Can't change state to the current state!"));
    }

    match (&self.state.state, desired, &self.state.data) {
      // E-Stops
      (_, ArenaState::Estop, _) => Ok(()),
      (ArenaState::Estop, ArenaState::EstopReset, _) => Ok(()),
      (ArenaState::EstopReset, ArenaState::Idle, _) => Ok(()),

      // Primary Flows
      (ArenaState::Idle, ArenaState::Prestart { ready: false, force: _ }, _) => {
        // Prestart must not be ready (false)
        let m = self
          .current_match
          .as_ref()
          .ok_or(illegal("Cannot PreStart without a Match"))?;
        if m.current_state() != MatchPlayState::Waiting {
          Err(illegal(&format!(
            "Match is not in waiting state! {:?}",
            m.current_state()
          )))
        } else {
          Ok(())
        }
      }
      (ArenaState::Prestart { ready: false, force: _ }, ArenaState::Prestart { ready: true, force: _ }, _) => Ok(()),
      (ArenaState::Prestart { ready: true, force: _ }, ArenaState::MatchArmed, _) => {
        // Prestart must be ready (true)
        if self.stations.iter().all(|x| x.can_arm_match()) {
          Ok(())
        } else {
          Err(illegal(
            "Cannot Arm Match: Not all teams are ready. Bypass any no-show teams.",
          ))
        }
      }
      (ArenaState::MatchArmed, ArenaState::MatchPlay, _) => Ok(()),
      (ArenaState::MatchPlay, ArenaState::MatchComplete, _) => {
        let m = log_expect!(self.current_match.as_ref().ok_or("No match!"));
        if m.current_state() != MatchPlayState::Complete {
          Err(illegal("Match is not complete."))
        } else {
          Ok(())
        }
      }
      (ArenaState::MatchComplete, ArenaState::MatchCommit, _) => Ok(()),
      (ArenaState::MatchCommit, ArenaState::Idle, _) => Ok(()),

      _ => Err(illegal("Undefined Transition")),
    }
  }

  fn do_state_init(&mut self, state: ArenaState) -> ArenaResult<BoundState> {
    self.can_change_state_to(state)?;

    let current = self.state.state;

    let basic = move |data: StateData| -> ArenaResult<BoundState> {
      Ok(BoundState {
        first: true,
        state,
        data,
      })
    };

    match (current, state, &self.state.data) {
      (_, ArenaState::Estop, _) => basic(StateData::Estop),
      (_, ArenaState::EstopReset, _) => basic(StateData::EstopReset),
      (_, ArenaState::Idle, _) => basic(StateData::Idle),
      (_, ArenaState::Prestart { ready: false, force: _ }, _) => self.state_init_prestart(state),
      (_, ArenaState::Prestart { ready: true, force: _ }, _) => basic(StateData::Prestart(None)),
      (_, ArenaState::MatchArmed, _) => basic(StateData::MatchArmed),
      (_, ArenaState::MatchPlay, _) => basic(StateData::MatchPlay),
      (_, ArenaState::MatchComplete, _) => basic(StateData::MatchComplete),
      (_, ArenaState::MatchCommit, _) => basic(StateData::MatchCommit),
    }
  }

  fn state_init_prestart(&mut self, state: ArenaState) -> ArenaResult<BoundState> {
    let (_, force) = state.into_prestart().unwrap();
    let the_rx = self.network.clone().map(|nw| {
      let (tx, rx) = channel();

      let stations = self.stations.clone();

      tokio::task::spawn(async move {
        info!("Configuring Alliances...");
        let mtx = nw.lock().await;
        let result = mtx.configure(&stations[..], force).await;
        tx.send(result).unwrap();
        info!("Alliances configured!");
      });

      rx
    });

    Ok(BoundState {
      first: true,
      state: ArenaState::Prestart {
        ready: the_rx.is_none(),
        force,
      }, // Ready if there's no network
      data: StateData::Prestart(the_rx),
    })
  }

  pub async fn update(&mut self) {
    // Field Emergency Stop
    let estop_result = self.update_field_estop().await;
    match estop_result {
      Err(ArenaError::IllegalStateChange(ref isc)) => {
        error!("Cannot transition to E-STOP from {} ({})", isc.from, isc.why);
      }
      Err(x) => error!("Other error for estop: {}", x),
      Ok(()) => (),
    }

    // If E-stop state change detected, do the state change ASAP
    if self.pending_state_change.is_some() {
      self.clear_signal().await;
      match self.perform_state_change() {
        Ok(()) => (),
        Err(e) => error!("Error during state change: {}", e),
      };
    }

    // General state updates
    let state_result = self.update_states().await;
    match state_result {
      Err(e) => {
        error!("Error during state update: {}", e)
      }
      Ok(()) => (),
    }

    self.state.first = false;

    // Match update
    if let Some(ref mut m) = self.current_match {
      m.update();
    }

    // Perform state update
    self.clear_signal().await;
    match self.perform_state_change() {
      Ok(()) => (),
      Err(e) => error!("Error during state change: {}", e),
    };
  }

  // Signals
  pub async fn signal(&mut self, signal: ArenaSignal) {
    *self.pending_signal.lock().await = Some(signal);
  }

  async fn current_signal(&self) -> Option<ArenaSignal> {
    *self.pending_signal.lock().await
  }

  async fn clear_signal(&self) {
    *self.pending_signal.lock().await = None;
  }

  // State Generals
  pub fn current_state(&self) -> ArenaState {
    return self.state.state;
  }

  fn prepare_state_change(&mut self, desired: ArenaState) -> ArenaResult<()> {
    info!("Queuing state transition: {:?} -> {:?}", self.state.state, desired);

    match self.can_change_state_to(desired) {
      Err(e) => {
        error!("Could not perform state transition: {}", e);
        Err(e)
      }
      Ok(_) => {
        self.pending_state_change = Some(desired);
        Ok(())
      }
    }
  }

  fn perform_state_change(&mut self) -> ArenaResult<()> {
    let pending = mem::replace(&mut self.pending_state_change, None);
    match pending {
      None => Ok(()),
      Some(pend) => {
        self.state = self.do_state_init(pend)?;
        info!("State transition performed!");
        Ok(())
      }
    }
  }
}
