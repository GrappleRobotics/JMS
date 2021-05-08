use std::{mem, sync::{Arc, Mutex, mpsc::{Receiver, TryRecvError, channel}}, thread::{self, current}};

use futures::Future;
use log::{error, info};

use super::exceptions::{ArenaError, ArenaResult, StateTransitionError};
use crate::{context, log_expect, models::Team, network::{NetworkProvider, NetworkResult}};

use super::matches::Match;

#[derive(EnumDiscriminants)]
#[strum_discriminants(name(ArenaStateVariant), derive(Display))]
pub enum ArenaState {
  Idle,  // Idle state
  Estop, // Arena is emergency stopped and can only be unlocked by FTA

  // Match Pipeline //
  PreMatch(Receiver<NetworkResult<()>>),         // Configure network and devices.
  PreMatchComplete, // Pre-Match configuration is complete.
  MatchArmed,       // Arm the match - ensure field crew is off. Can revert to PreMatch.
  MatchPlay,        // Currently running a match - handed off to Match runner
  MatchComplete,    // Match just finished, waiting to commit. Refs can still change scores
  MatchCommit,      // Commit the match score - lock ref tablets, publish to TBA and Audience Display
}

impl ArenaState {
  pub fn variant(&self) -> ArenaStateVariant {
    return self.into();
  }
}

pub type ArenaStateInitialiser = dyn FnOnce(&mut Arena) -> ArenaResult<()>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display)]
pub enum ArenaSignal {
  Estop,
  PreMatch,
  MatchArm,
  MatchPlay,
  MatchCommit
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
  state: ArenaState,
  pending_transition: Option<Box<ArenaStateInitialiser>>,
  pending_signal: Arc<Mutex<Option<ArenaSignal>>>,
  current_match: Option<Match>,
  stations: Vec<AllianceStation>,
}

impl Arena {
  pub fn new(num_stations_per_alliance: u32, network: Box<dyn NetworkProvider + Send>) -> Arena {
    let mut a = Arena {
      network: Arc::new(Mutex::new(network)),
      state: ArenaState::Idle,
      pending_transition: None,
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

  // TODO: Split into phases - state transition failures should not prevent I/O or DS comms.
  //        DS comms and I/O should include a 'heartbeat' and signify a fault.
  pub fn update(&mut self) {
    context!("Arena::Update", {
      // Field Emergency Stop
      // context!("Estops", {
      //   let estop_result = self.update_field_estop();
      //   match estop_result {
      //     Err(ArenaError::IllegalStateChange { from, to: _, condition }) => {
      //       error!("Cannot transition to E-STOP from {} ({})", from, condition);
      //     },
      //     Err(x) => error!("Other error for estop: {}", x),
      //     Ok(()) => (),
      //   }
      // });

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

      // result
    })
  }

  // fn update_field_estop(&mut self) -> ArenaResult<()> {
  //   if self.state.variant() != ArenaStateVariant::Estop {
  //     if self.has_pending_signal(ArenaSignal::Estop) {
  //       self.change_state(ArenaState::Estop)?;
  //     }
  //   }
  //   Ok(())
  // }

  // fn update_signals(&mut self) -> ArenaResult<()> {
  //   if self.pending_signal.is_some() {
  //     let Some(sig) = self.pending_signal;
  //     match (self.state, sig) {
  //       (state, signal) => 
  //     }
  //   }
  //   Ok(())
  // }

  fn update_states(&mut self) -> ArenaResult<()> {
    Ok(())
  }

  
  pub fn current_state(&self) -> &ArenaState {
    return &self.state;
  }

  fn get_state_change(&self, desired: ArenaStateVariant) -> ArenaResult<Box<ArenaStateInitialiser>> {
    let current_variant = self.state.variant();

    // Generic failure
    let fail = move |why: &str| {
      ArenaError::IllegalStateChange(StateTransitionError { from: current_variant, to: desired, condition: why.to_owned() })
    };

    if current_variant == desired {
      return Err(fail("Can't transition to the same state!"));
    }

    // Basic transition - perform the state change directly
    let basic = |new_state: ArenaState| -> Box<ArenaStateInitialiser> {
      Box::new(move |arena: &mut Arena| {
        arena.state = new_state;
        Ok(())
      })
    };

    match (&self.state, desired) {
      // E-stop
      (_, ArenaStateVariant::Estop) => Ok(basic(ArenaState::Estop)),
      (ArenaState::Estop, ArenaStateVariant::Idle) => Ok(basic(ArenaState::Idle)),

      // Primary flows
      (ArenaState::Idle, ArenaStateVariant::PreMatch) => {
        let m = self.current_match.ok_or(fail("Cannot PreStart without a match"))?;
        if !m.ready() {
          Err(fail("Match is not ready."))
        } else {
          Ok(Box::new(Arena::state_init_prestart))
        }
      },
      (ArenaState::PreMatch(ref recv), ArenaStateVariant::PreMatchComplete) => {
        let recv_result = recv.try_recv();
        match recv_result {
          Err(TryRecvError::Empty) => Err(fail("Network not ready")),
          Err(TryRecvError::Disconnected) => panic!("Network runner fault!"), // TODO: Better fatal handling here
          Ok(net_result) => {
            net_result?;  // TODO: In update, if this error is hit the arena should fall back to Idle.
                          // In any case, the receiver should be cleared.
            Ok(basic(ArenaState::PreMatchComplete))
          }
        }
      },
      (ArenaState::PreMatchComplete, ArenaStateVariant::MatchArmed) => {
        // TODO: Driver stations ready and match ready
        let m = log_expect!(self.current_match.ok_or("No match!"));
        if !m.ready() {
          Err(fail("Match is not ready."))
        } else {
          Ok(basic(ArenaState::MatchArmed))
        }
      },
      (ArenaState::MatchArmed, ArenaStateVariant::MatchPlay) => Ok(basic(ArenaState::MatchPlay)),
      (ArenaState::MatchPlay, ArenaStateVariant::MatchComplete) => {
        let m = log_expect!(self.current_match.ok_or("No match!"));
        if !m.complete() {
          Err(fail("Match is not complete."))
        } else {
          Ok(basic(ArenaState::MatchComplete))
        }
      },
      (ArenaState::MatchComplete, ArenaStateVariant::MatchCommit) => Ok(basic(ArenaState::MatchCommit)),
      (ArenaState::MatchCommit, ArenaStateVariant::Idle) => Ok(basic(ArenaState::Idle)),

      _ => Err(fail("Undefined Transition"))
    }
  }

  fn state_init_prestart(&mut self) -> ArenaResult<()> {
    // Need to clone these since there's no guarantee the thread will finish before 
    // the arena is destructed.
    // Note of course that the arena won't be able to exit prestart until this is complete.
    let netw_arc = self.network.clone();
    let stations = self.stations.clone();
    
    let (tx, rx) = channel();

    thread::spawn(move || {
      context!("Arena Prestart Network", {
        let mut net = netw_arc.lock().unwrap();
        let result = net.configure_alliances(&mut stations.iter(), false);
        tx.send(result).unwrap();   // TODO: Better fatal handling
      })
    });

    self.state = ArenaState::PreMatch(rx);

    Ok(())
  }

  pub fn can_change_state_to(&self, desired: ArenaStateVariant) -> bool {
    self.get_state_change(desired).is_ok()
  }

  pub fn queue_state_change(&mut self, desired: ArenaStateVariant) -> ArenaResult<()> {
    context!("Queue State Change", {
      info!("Queuing state transition: {} -> {}", self.state.variant(), desired);

      match self.get_state_change(desired) {
        Err(e) => {
          error!("Could not perform state transition: {}", e);
          Err(e)
        },
        Ok(init) => {
          info!("State transition queued!");
          self.pending_transition = Some(init);
          Ok(())
        }
      }
    })
  }

  // Don't error to the console if a state transition is not possible. Good for automatic transitions
  // like from PreMatch to PreMatchComplete, without evaluating can_change_state_to.
  pub fn maybe_queue_state_change(&mut self, desired: ArenaStateVariant) -> ArenaResult<()> {
   match self.get_state_change(desired) {
     Err(e) => {
       Err(e)
     },
     Ok(init) => {
       info!("State transition queued: {} -> {}", self.state.variant(), desired);
       self.pending_transition = Some(init);
       Ok(())
     }
   } 
  }

  pub fn perform_state_change(&mut self) -> ArenaResult<()> {
    let pending = mem::replace(&mut self.pending_transition, None);
    match pending {
      None => Ok(()),
      Some(init) => {
        init(self)?;
        Ok(())
      }
    }
  }
}
