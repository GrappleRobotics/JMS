use std::{
  mem,
  sync::{
    mpsc::{channel, Receiver, TryRecvError},
    Arc, Mutex,
  },
  thread,
};

use enum_as_inner::EnumAsInner;
use log::{error, info};

use super::{
  exceptions::{ArenaError, ArenaResult, StateTransitionError},
  matches::MatchPlayState,
};
use crate::{
  context, log_expect,
  models::Team,
  network::{NetworkProvider, NetworkResult},
};

use super::matches::Match;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display, EnumAsInner)]
pub enum ArenaState {
  Idle,  // Idle state
  Estop, // Arena is emergency stopped and can only be unlocked by FTA

  // Match Pipeline //
  Prestart(/* ready */ bool, /* forced */ bool), // Configure network and devices. Expose ready so we can see it outside.
  MatchArmed,    // Arm the match - ensure field crew is off. Can revert to Prestart.
  MatchPlay,     // Currently running a match - handed off to Match runner
  MatchComplete, // Match just finished, waiting to commit. Refs can still change scores
  MatchCommit,   // Commit the match score - lock ref tablets, publish to TBA and Audience Display
}

#[derive(EnumAsInner)]
enum StateData {
  Idle,
  Estop,

  Prestart(Option<Receiver<NetworkResult<()>>>), // recv: network ready receiver
  MatchArmed,
  MatchPlay,
  MatchComplete,
  MatchCommit,
}

struct BoundState {
  first: bool, // First run?
  state: ArenaState,
  data: StateData,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display)]
pub enum ArenaSignal {
  Estop,
  Prestart(bool),
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
  network: Arc<Mutex<Option<Box<dyn NetworkProvider + Send>>>>,
  state: BoundState,
  pending_state_change: Option<ArenaState>,
  pending_signal: Arc<Mutex<Option<ArenaSignal>>>,
  current_match: Option<Match>,
  stations: Vec<AllianceStation>,
}

impl Arena {
  pub fn new(
    num_stations_per_alliance: u32,
    network: Option<Box<dyn NetworkProvider + Send>>,
  ) -> Arena {
    let mut a = Arena {
      network: Arc::new(Mutex::new(network)),
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

  fn update_field_estop(&mut self) -> ArenaResult<()> {
    if self.state.state != ArenaState::Estop {
      if let Some(ArenaSignal::Estop) = self.current_signal() {
        self.prepare_state_change(ArenaState::Estop)?;
      }
    }
    Ok(())
  }

  fn update_states(&mut self) -> ArenaResult<()> {
    let first = self.state.first;
    match (self.state.state, &mut self.state.data) {
      (ArenaState::Idle, _) => {
        if let Some(ArenaSignal::Prestart(force)) = self.current_signal() {
          self.prepare_state_change(ArenaState::Prestart(false, force))?;
        }
      }
      (ArenaState::Estop, _) => {
        if let Some(ref mut m) = self.current_match {
          // Fault the match - it can't be run and must be reloaded.
          m.fault();
        }
      }
      (ArenaState::Prestart(false, force), StateData::Prestart(maybe_recv)) => {
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
              self.prepare_state_change(ArenaState::Prestart(true, force))?;
            }
          };
        }
      }
      (ArenaState::Prestart(true, _), _) => {
        if first {
          info!("Prestart Ready!")
        }
        if let Some(ArenaSignal::MatchArm) = self.current_signal() {
          self.prepare_state_change(ArenaState::MatchArmed)?;
        }
      }
      (ArenaState::MatchArmed, _) => {
        if first {
          info!("Match Armed!")
        }
        if let Some(ArenaSignal::MatchPlay) = self.current_signal() {
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
        if let Some(ArenaSignal::MatchCommit) = self.current_signal() {
          self.prepare_state_change(ArenaState::MatchCommit)?;
        }
      }
      (ArenaState::MatchCommit, _) => (),
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
      (ArenaState::Estop, ArenaState::Idle, _) => Ok(()),

      // Primary Flows
      (ArenaState::Idle, ArenaState::Prestart(false, _), _) => {
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
      (ArenaState::Prestart(false, _), ArenaState::Prestart(true, _), _) => Ok(()),
      (ArenaState::Prestart(true, _), ArenaState::MatchArmed, _) => {
        // Prestart must be ready (true)
        Ok(())
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
      (_, ArenaState::Idle, _) => basic(StateData::Idle),
      (_, ArenaState::Prestart(false, _), _) => self.state_init_prestart(state),
      (_, ArenaState::Prestart(true, _), _) => basic(StateData::Prestart(None)),
      (_, ArenaState::MatchArmed, _) => basic(StateData::MatchArmed),
      (_, ArenaState::MatchPlay, _) => basic(StateData::MatchPlay),
      (_, ArenaState::MatchComplete, _) => basic(StateData::MatchComplete),
      (_, ArenaState::MatchCommit, _) => basic(StateData::MatchCommit),
    }
  }

  fn state_init_prestart(&mut self, state: ArenaState) -> ArenaResult<BoundState> {
    let (_, force) = state.into_prestart().unwrap();
    let the_rx = match *self.network.lock().unwrap() {
      // No network provided means prestart is ready.
      None => None,
      Some(_) => {
        // Need to clone these since there's no guarantee the thread will finish before
        // the arena is destructed.
        // Note of course that the arena won't be able to exit prestart until this is complete.
        let netw_arc = self.network.clone();
        let stations = self.stations.clone();

        let (tx, rx) = channel();

        thread::spawn(move || {
          context!("Arena Prestart Network", {
            info!("Configuring alliances...");
            let mut mtx_net = netw_arc.lock().unwrap();
            let ref mut net = mtx_net.as_mut().unwrap();
            let result = net.configure_alliances(&mut stations.iter(), force); // Unwrap is safe if net optional is immutable due to match call.
            tx.send(result).unwrap(); // TODO: Better fatal handling
            info!("Alliances configured!");
          })
        });

        Some(rx)
      }
    };

    Ok(BoundState {
      first: true,
      state: ArenaState::Prestart(the_rx.is_none(), force), // Ready if there's no network
      data: StateData::Prestart(the_rx),
    })
  }

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

      // General state updates
      context!(&format!("State Update ({})", self.state.state), {
        let state_result = self.update_states();
        match state_result {
          Err(e) => {
            error!("Error during state update: {}", e)
          }
          Ok(()) => (),
        }
      });

      self.state.first = false;

      // Match update
      if let Some(ref mut m) = self.current_match {
        m.update();
      }

      // Perform state update
      context!("State Change", {
        self.clear_signal();
        match self.perform_state_change() {
          Ok(()) => (),
          Err(e) => error!("Error during state change: {}", e),
        };
      });
    })
  }

  // Signals
  pub fn signal(&mut self, signal: ArenaSignal) {
    *log_expect!(self.pending_signal.lock()) = Some(signal);
  }

  fn current_signal(&self) -> Option<ArenaSignal> {
    *log_expect!(self.pending_signal.lock())
  }

  fn clear_signal(&self) {
    *log_expect!(self.pending_signal.lock()) = None;
  }

  // State Generals
  pub fn current_state(&self) -> ArenaState {
    return self.state.state;
  }

  fn prepare_state_change(&mut self, desired: ArenaState) -> ArenaResult<()> {
    context!("Queue State Change", {
      info!(
        "Queuing state transition: {:?} -> {:?}",
        self.state.state, desired
      );

      match self.can_change_state_to(desired) {
        Err(e) => {
          error!("Could not perform state transition: {}", e);
          Err(e)
        }
        Ok(pending) => {
          self.pending_state_change = Some(desired);
          Ok(())
        }
      }
    })
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
