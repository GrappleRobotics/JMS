use std::{
  mem,
  sync::{
    mpsc::{channel, Receiver, TryRecvError},
    Arc, Mutex,
  },
  thread,
};

use log::{error, info};

use super::exceptions::{ArenaError, ArenaResult, StateTransitionError};
use crate::{
  context, log_expect,
  models::Team,
  network::{NetworkProvider, NetworkResult},
};

use super::matches::Match;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display)]
pub enum ArenaState {
  Idle,  // Idle state
  Estop, // Arena is emergency stopped and can only be unlocked by FTA

  // Match Pipeline //
  PreMatch(/* force */ bool), // Configure network and devices.
  PreMatchComplete,           // Pre-Match configuration is complete.
  MatchArmed,                 // Arm the match - ensure field crew is off. Can revert to PreMatch.
  MatchPlay,                  // Currently running a match - handed off to Match runner
  MatchComplete, // Match just finished, waiting to commit. Refs can still change scores
  MatchCommit,   // Commit the match score - lock ref tablets, publish to TBA and Audience Display
}

enum StateData {
  Idle,
  Estop,

  PreMatch(Receiver<NetworkResult<()>>),
  PreMatchComplete,
  MatchArmed,
  MatchPlay,
  MatchComplete,
  MatchCommit,
}

struct LoadedState {
  state: ArenaState,
  data: StateData,
}

// Functions responsible for initialising a new state to be loaded onto the arena.
type ArenaStateInitialiser = dyn FnOnce(&mut Arena, ArenaState) -> ArenaResult<LoadedState>;

struct PendingState {
  state: ArenaState,
  init: Box<ArenaStateInitialiser>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display)]
pub enum ArenaSignal {
  Estop,
  PreMatch(bool),
  MatchArm,
  MatchPlay,
  MatchCommit,
}

/**
 * Who's permitted on the field?
 */
pub enum ArenaEntryCondition {
  Locked,    // No one / FTA's discretion (no lights)
  ResetOnly, // Field reset crew (purple lights)
  Teams,     // Teams can collect robots (green lights)
  Any,       // Anyone (Idle only - awards etc)
}

#[derive(Debug, Clone, Copy)]
pub enum Alliance {
  Blue,
  Red,
}
#[derive(Debug, Clone)]
pub struct AllianceStation {
  alliance: Alliance,
  number: u32,
  team: Option<Team>,
  bypass: bool,
  estop: bool,
  astop: bool,
}

pub struct Arena {
  network: Arc<Mutex<Box<dyn NetworkProvider + Send>>>,
  state: LoadedState,
  pending_state_change: Option<PendingState>,
  pending_signal: Arc<Mutex<Option<ArenaSignal>>>,
  current_match: Option<Match>,
  stations: Vec<AllianceStation>,
}

impl Arena {
  pub fn new(num_stations_per_alliance: u32, network: Box<dyn NetworkProvider + Send>) -> Arena {
    let mut a = Arena {
      network: Arc::new(Mutex::new(network)),
      state: LoadedState {
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
        a.stations.push(AllianceStation {
          alliance,
          number: i,
          team: None,
          bypass: false,
          estop: false,
          astop: false,
        });
      }
    }

    a
  }

  pub fn load_match(&mut self, m: Match) {
    self.current_match = Some(m);
  }

  pub fn signal(&mut self, signal: ArenaSignal) {
    *log_expect!(self.pending_signal.lock()) = Some(signal);
  }

  fn current_signal(&self) -> Option<ArenaSignal> {
    *log_expect!(self.pending_signal.lock())
  }

  fn clear_signal(&self) {
    *log_expect!(self.pending_signal.lock()) = None;
  }

  // TODO: Split into phases - state transition failures should not prevent I/O or DS comms.
  //        DS comms and I/O should include a 'heartbeat' and signify a fault.
  pub fn update(&mut self) {
    context!("Arena Update", {
      // Field Emergency Stop
      context!("E-stop", {
        let estop_result = self.update_field_estop();
        match estop_result {
          Err(ArenaError::IllegalStateChange(ref isc)) => {
            error!(
              "Cannot transition to E-STOP from {} ({})",
              isc.from, isc.why
            );
          }
          Err(x) => error!("Other error for estop: {}", x),
          Ok(()) => (),
        }
      });

      // If E-stop state change detected, do the state change ASAP
      if self.pending_state_change.is_some() {
        context!("Post E-stop State Change", {
          self.clear_signal();
          match self.perform_state_change() {
            Ok(()) => (),
            Err(e) => error!("Error during state change: {}", e),
          };
        });
      }

      // Need to start scaffolding the Network, DS Comms, and Field I/O to get the rest of
      // this fleshed out.

      // let result = match self.state {
      //   ArenaState::Idle => {
      //     self.maybe_process_signal(ArenaSignal::PreStart, ArenaState::PreMatch(None))?;
      //     Ok(())
      //   }
      //   ArenaState::Estop => Err(ArenaError::UnimplementedStateError(self.state.variant())),
      //   ArenaState::PreMatch(_) => Err(ArenaError::UnimplementedStateError(self.state.variant())),
      //   ArenaState::PreMatchComplete => Err(ArenaError::UnimplementedStateError(self.state.variant())),
      //   ArenaState::MatchArmed => Err(ArenaError::UnimplementedStateError(self.state.variant())),
      //   ArenaState::MatchPlay => Err(ArenaError::UnimplementedStateError(self.state.variant())),
      //   ArenaState::MatchComplete => Err(ArenaError::UnimplementedStateError(self.state.variant())),
      //   ArenaState::CommitMatch => Err(ArenaError::UnimplementedStateError(self.state.variant())),
      // };

      // self.do_change_state()?;

      context!("State Change", {
        self.clear_signal();
        match self.perform_state_change() {
          Ok(()) => (),
          Err(e) => error!("Error during state change: {}", e),
        };
      });
    })
  }

  fn update_field_estop(&mut self) -> ArenaResult<()> {
    if self.state.state != ArenaState::Estop {
      if let Some(ArenaSignal::Estop) = self.current_signal() {
        self.queue_state_change(ArenaState::Estop)?;
      }
    }
    Ok(())
  }

  fn update_states(&mut self) -> ArenaResult<()> {
    Ok(())
  }

  pub fn current_state(&self) -> &ArenaState {
    return &self.state.state;
  }

  fn get_state_change(&self, desired: ArenaState) -> ArenaResult<PendingState> {
    let current = self.state.state;

    let illegal = move |why: &str| {
      ArenaError::IllegalStateChange(StateTransitionError {
        from: current,
        to: desired,
        why: why.to_owned(),
      })
    };

    if current == desired {
      return Err(illegal("Can't change to the same state!"));
    }

    let basic = |data: StateData| -> PendingState {
      let init = Box::new(move |_: &mut Arena, state: ArenaState| Ok(LoadedState { state, data }));
      PendingState {
        state: desired,
        init,
      }
    };

    let wrap = |init: Box<ArenaStateInitialiser>| -> PendingState {
      PendingState {
        state: desired,
        init,
      }
    };

    match (&self.state.state, desired) {
      // E-Stops
      (_, ArenaState::Estop) => Ok(basic(StateData::Estop)),
      (ArenaState::Estop, ArenaState::Idle) => Ok(basic(StateData::Idle)),

      // Primary Flows
      (ArenaState::Idle, ArenaState::PreMatch(_)) => {
        let m = self
          .current_match
          .ok_or(illegal("Cannot PreStart without a Match"))?;
        if !m.ready() {
          Err(illegal("Match is not ready"))
        } else {
          Ok(wrap(Box::new(Arena::state_init_prestart)))
        }
      }
      (ArenaState::PreMatch(_), ArenaState::PreMatchComplete) => {
        if let StateData::PreMatch(recv) = &self.state.data {
          let recv_result = recv.try_recv();
          match recv_result {
            Err(TryRecvError::Empty) => Err(illegal("Network not ready")),
            Err(TryRecvError::Disconnected) => panic!("Network runner fault!"), // TODO: Better fatal handling here
            Ok(net_result) => {
              net_result?; // TODO: In update, if this error is hit the arena should fall back to Idle.
                           // In any case, the receiver should be cleared.
              Ok(basic(StateData::PreMatchComplete))
            }
          }
        } else {
          panic!("PreMatch data is not the correct type!")
        }
      }
      (ArenaState::PreMatchComplete, ArenaState::MatchArmed) => {
        // TODO: Driver stations ready and match ready
        let m = log_expect!(self.current_match.ok_or("No match!"));
        if !m.ready() {
          Err(illegal("Match is not ready."))
        } else {
          Ok(basic(StateData::MatchArmed))
        }
      }
      (ArenaState::MatchArmed, ArenaState::MatchPlay) => Ok(basic(StateData::MatchPlay)),
      (ArenaState::MatchPlay, ArenaState::MatchComplete) => {
        let m = log_expect!(self.current_match.ok_or("No match!"));
        if !m.complete() {
          Err(illegal("Match is not complete."))
        } else {
          Ok(basic(StateData::MatchComplete))
        }
      }
      (ArenaState::MatchComplete, ArenaState::MatchCommit) => Ok(basic(StateData::MatchCommit)),
      (ArenaState::MatchCommit, ArenaState::Idle) => Ok(basic(StateData::Idle)),

      _ => Err(illegal("Undefined Transition")),
    }
  }

  fn state_init_prestart(&mut self, state: ArenaState) -> ArenaResult<LoadedState> {
    // Need to clone these since there's no guarantee the thread will finish before
    // the arena is destructed.
    // Note of course that the arena won't be able to exit prestart until this is complete.
    if let ArenaState::PreMatch(force) = state {
      let netw_arc = self.network.clone();
      let stations = self.stations.clone();

      let (tx, rx) = channel();

      thread::spawn(move || {
        context!("Arena Prestart Network", {
          let mut net = netw_arc.lock().unwrap();
          let result = net.configure_alliances(&mut stations.iter(), force);
          tx.send(result).unwrap(); // TODO: Better fatal handling
        })
      });

      Ok(LoadedState {
        state,
        data: StateData::PreMatch(rx),
      })
    } else {
      panic!("PreMatch state is not the correct type!")
    }
  }

  pub fn can_change_state_to(&self, desired: ArenaState) -> bool {
    self.get_state_change(desired).is_ok()
  }

  fn queue_state_change(&mut self, desired: ArenaState) -> ArenaResult<()> {
    context!("Queue State Change", {
      info!(
        "Queuing state transition: {} -> {}",
        self.state.state, desired
      );

      match self.get_state_change(desired) {
        Err(e) => {
          error!("Could not perform state transition: {}", e);
          Err(e)
        }
        Ok(pending) => {
          info!("State transition queued!");
          self.pending_state_change = Some(pending);
          Ok(())
        }
      }
    })
  }

  // Don't error to the console if a state transition is not possible. Good for automatic transitions
  // like from PreMatch to PreMatchComplete, without evaluating can_change_state_to.
  fn maybe_queue_state_change(&mut self, desired: ArenaState) -> ArenaResult<()> {
    match self.get_state_change(desired) {
      Err(e) => Err(e),
      Ok(pending) => {
        info!(
          "State transition queued: {} -> {}",
          self.state.state, desired
        );
        self.pending_state_change = Some(pending);
        Ok(())
      }
    }
  }

  fn perform_state_change(&mut self) -> ArenaResult<()> {
    let pending = mem::replace(&mut self.pending_state_change, None);
    match pending {
      None => Ok(()),
      Some(pend) => {
        self.state = (pend.init)(self, pend.state)?;
        Ok(())
      }
    }
  }
}
