use futures::Future;
use log::{error, info};

use super::exceptions::{ArenaError, ArenaResult};
use crate::{context, models::Team, network::{NetworkProvider, NetworkResult}};

use super::matches::Match;

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum ArenaState {
  Idle,  // Idle state
  Estop, // Arena is emergency stopped and can only be unlocked by FTA

  // Match Pipeline //
  PreMatch,         // Configure network and devices.
  PreMatchComplete, // Pre-Match configuration is complete.
  MatchArmed,       // Arm the match - ensure field crew is off. Can revert to PreMatch.
  MatchPlay,        // Currently running a match - handed off to Match runner
  MatchComplete,    // Match just finished, waiting to commit. Refs can still change scores
  CommitMatch, // Commit the match score - lock ref tablets, publish to TBA and Audience Display
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

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
enum ArenaSignal {
  Estop,
  PreStart,
  StartMatch,
}

pub struct Arena {
  network: Box<dyn NetworkProvider>,
  state: ArenaState,
  last_state: ArenaState,
  entry: ArenaEntryCondition,
  pending_signal: Option<ArenaSignal>,
  current_match: Option<Match>,
  stations: Vec<AllianceStation>,
}

impl Arena {
  pub fn new(num_stations_per_alliance: u32, network: Box<dyn NetworkProvider>) -> Arena {
    let mut a = Arena {
      network,
      state: ArenaState::Idle,
      last_state: ArenaState::Idle,
      entry: ArenaEntryCondition::Any,
      pending_signal: None,
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

  pub fn update(&mut self) -> ArenaResult<()> {
    context!("Arena::Update", {
      let current_state = self.state.clone();

      // Field Emergency Stop
      if self.has_pending_signal(ArenaSignal::Estop) /* E-stop pressed */ && current_state != ArenaState::Estop
      {
        if let Err(e) = self.change_state(ArenaState::Estop) {
          error!("E-STOP PRECONDITION FAILURE: {}", e)
        }
      }

      // Need to start scaffolding the Network, DS Comms, and Field I/O to get the rest of
      // this fleshed out.

      let result = match current_state {
        ArenaState::Idle => {
          if self.has_pending_signal(ArenaSignal::PreStart)
            && self.change_state_or_log(ArenaState::PreMatch)
          {
            // State has changed
          } else {
          }
          Ok(())
        }
        ArenaState::Estop => Err(ArenaError::UnimplementedStateError(current_state)),
        ArenaState::PreMatch => Err(ArenaError::UnimplementedStateError(current_state)),
        ArenaState::PreMatchComplete => Err(ArenaError::UnimplementedStateError(current_state)),
        ArenaState::MatchArmed => Err(ArenaError::UnimplementedStateError(current_state)),
        ArenaState::MatchPlay => Err(ArenaError::UnimplementedStateError(current_state)),
        ArenaState::MatchComplete => Err(ArenaError::UnimplementedStateError(current_state)),
        ArenaState::CommitMatch => Err(ArenaError::UnimplementedStateError(current_state)),
      };

      self.pending_signal = None;

      result
    })
  }

  fn on_state_change(&mut self) -> ArenaResult<()> {
    context!("Arena::OnStateChange", {
      match self.state {
        ArenaState::PreMatch => {
          // let fut = async { self.network.configure_alliances(&mut self.stations.iter(), false) };
          Ok(())
        },
        _ => Ok(())
      }
    })
  }

  fn has_pending_signal(&self, sig: ArenaSignal) -> bool {
    match self.pending_signal {
      Some(s) => s == sig,
      None => false,
    }
  }

  pub fn current_state(&self) -> ArenaState {
    return self.state;
  }

  /**
   * Check preconditions for state change - to be called from triggers
   */
  pub fn can_change_state(&self, desired: ArenaState) -> ArenaResult<()> {
    let err = move |why: &str| ArenaError::IllegalStateChange {
      from: self.state,
      to: desired,
      condition: why.to_owned(),
    };

    if self.state == desired {
      return Err(ArenaError::AlreadyInState(self.state));
    }

    match (self.state, desired) {
      // Estop
      (_, ArenaState::Estop) => Ok(()),
      (ArenaState::Estop, ArenaState::Idle) => Ok(()),

      // Primary match flows
      (ArenaState::Idle, ArenaState::PreMatch) => {
        // TODO: Check if network is ready
        if self.current_match.is_none() {
          Err(err("Cannot Prestart a non-existant match."))
        } else {
          Ok(())
        }
      }
      (ArenaState::PreMatch, ArenaState::PreMatchComplete) => Ok(()), // TODO: If prestart ready
      (ArenaState::PreMatchComplete, ArenaState::MatchArmed) => {
        let ref current_match = self.current_match.expect("Match is null");
        // Match must be ready
        // TODO: Driver stations ready
        if !current_match.ready() {
          Err(err("Match is not ready"))
        } else {
          Ok(())
        }
      }
      (ArenaState::MatchArmed, ArenaState::MatchPlay) => Ok(()),
      (ArenaState::MatchPlay, ArenaState::MatchComplete) => {
        // Match needs to be complete
        let ref current_match = self.current_match.expect("Match is null");
        if current_match.complete() {
          Ok(())
        } else {
          Err(err("Match is not over"))
        }
      }
      (ArenaState::MatchComplete, ArenaState::CommitMatch) => Ok(()), // TODO: If ref tablets ready
      (ArenaState::CommitMatch, ArenaState::Idle) => Ok(()),

      // Alternative match flows
      (ArenaState::PreMatch, ArenaState::Idle) => Ok(()), // Revert match prestart
      (ArenaState::PreMatchComplete, ArenaState::Idle) => Ok(()),
      (ArenaState::MatchArmed, ArenaState::PreMatch) => Ok(()),

      (_, _) => Err(err("Undefined Transition")),
    }
  }

  fn change_state_or_log(&mut self, desired: ArenaState) -> bool {
    match self.change_state(desired) {
      Ok(()) => true,
      Err(e) => {
        error!("{}", e);
        false
      }
    }
  }

  fn change_state(&mut self, desired: ArenaState) -> ArenaResult<()> {
    context!("Arena::ChangeState", {
      let current = self.state.clone();
      info!(
        "Attempting state transition: {:?} -> {:?}",
        current, desired
      );

      if let Err(e) = self.can_change_state(desired) {
        error!("Could not perform state transition: {}", e);
        return Err(e);
      };

      self.last_state = self.state;
      self.state = desired;
      info!("State transition successful!");
      self.on_state_change()
    })
  }
}
