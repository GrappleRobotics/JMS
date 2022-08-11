use std::{
  mem,
  sync::{
    mpsc::{channel, Receiver, TryRecvError},
    Arc,
  },
};

use anyhow::{anyhow, bail, Result};

use chrono::Duration;
use enum_as_inner::EnumAsInner;
use log::{error, info};
use schemars::JsonSchema;
use tokio::sync::Mutex;

use super::{exceptions::ArenaIllegalStateChange, lighting::{ArenaLighting, LightMode}, matches::MatchPlayState, station::AllianceStationId, resource::SharedResources};

use crate::{arena::{exceptions::CannotLoadMatchError, lighting::ArenaLightingSettings}, ds::DSMode, log_expect, models::{self, Alliance, MatchType, DBResourceRequirements}, network::{NetworkProvider, NetworkResult}, db};

use serde::{Deserialize, Serialize};

use super::matches::LoadedMatch;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display, EnumAsInner, Serialize, JsonSchema)]
#[serde(tag = "state")]
pub enum ArenaState {
  Init,
  Idle { ready: bool }, // Idle state
  Estop,                // Arena is emergency stopped and can only be unlocked by FTA
  EstopReset,           // E-stop resetting...

  // Match Pipeline //
  Prestart { ready: bool },
  MatchArmed,    // Arm the match - ensure field crew is off. Can revert to Prestart.
  MatchPlay,     // Currently running a match - handed off to Match runner
  MatchComplete { ready: bool }, // Match just finished, waiting to commit. Refs can still change scores. Prestart reverts.
  MatchCommit,   // Commit the match score - lock ref tablets, publish to TBA and Audience Display
}

#[derive(EnumAsInner)]
enum StateData {
  Init,
  Idle(Option<Receiver<NetworkResult<()>>>), // recv: network ready receiver
  Estop,
  EstopReset,

  Prestart(Option<Receiver<NetworkResult<()>>>), // recv: network ready receiver
  MatchArmed,
  MatchPlay,
  MatchComplete(Option<Receiver<NetworkResult<()>>>),
  MatchCommit,
}

#[derive(Serialize, JsonSchema)]
#[serde(transparent)]
pub struct BoundState {
  #[serde(skip)]
  first: bool, // First run?
  pub state: ArenaState,
  #[serde(skip)]
  data: StateData,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Display, Deserialize, JsonSchema)]
// #[serde(tag = "signal")]
pub enum ArenaSignal {
  Estop,
  EstopReset,
  Prestart,
  MatchArm(bool),   // bool: force?
  MatchPlay,
  MatchCommit,
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
#[serde(tag = "scene", content = "params")]
pub enum AudienceDisplay {
  Field,
  MatchPreview,
  MatchPlay,
  MatchResults(models::SerializedMatch),
  AllianceSelection,
  Award(models::Award),
  CustomMessage(String),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub enum ArenaAccessRestriction {
  NoRestriction,
  ResetOnly, // Field reset crew (purple lights)
  Teams,     // Teams can collect robots (green lights)
}

#[derive(Debug, Clone, Copy, Serialize, JsonSchema)]
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
      rtt: 0,
    }
  }
}

#[derive(Debug, Clone, Copy, Serialize, JsonSchema, PartialEq, Eq)]
pub enum AllianceStationOccupancy {
  Vacant,
  Occupied,
  WrongStation,
  WrongMatch,
}

#[derive(Debug, Clone, Copy, Serialize, JsonSchema)]
pub struct AllianceStation {
  pub station: AllianceStationId,
  pub team: Option<u16>,
  pub bypass: bool,
  pub estop: bool,
  pub astop: bool,
  pub ds_report: Option<AllianceStationDSReport>,
  pub occupancy: AllianceStationOccupancy,
}

#[derive(Debug, Clone, Copy, Serialize, JsonSchema)]
pub struct SerialisedAllianceStation {
  #[serde(flatten)]
  pub s: AllianceStation,
  pub can_arm: bool
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

  pub fn connection_ok(&self) -> bool {
    let mut ok = true;
    match &self.ds_report {
      Some(ds) => {
        if !ds.robot_ping || !ds.rio_ping || !ds.radio_ping {
          ok = false;
        }
      },
      None => ok = false
    }
    ok
  }
}

impl From<AllianceStation> for SerialisedAllianceStation {
  fn from(stn: AllianceStation) -> Self {
    SerialisedAllianceStation { 
      can_arm: stn.can_arm_match(),
      s: stn
    }
  }
}

pub struct Arena {
  network: Option<Arc<Mutex<Box<dyn NetworkProvider + Send + Sync>>>>,
  pub state: BoundState,
  pending_state_change: Option<ArenaState>,
  pending_signal: Arc<Mutex<Option<ArenaSignal>>>,
  pub current_match: Option<LoadedMatch>,
  pub stations: Vec<AllianceStation>,
  pub access: ArenaAccessRestriction,
  pub lighting: ArenaLighting,

  pub audience_display: AudienceDisplay,

  pub resources: SharedResources
}

pub type SharedArena = Arc<Mutex<Arena>>;

impl Arena {
  pub fn new(num_stations_per_alliance: u32, network: Option<Box<dyn NetworkProvider + Send + Sync>>, resources: SharedResources) -> Arena {
    let mut a = Arena {
      network: network.map(|x| Arc::new(Mutex::new(x))),
      state: BoundState {
        first: true,
        state: ArenaState::Init,
        data: StateData::Init,
      },
      pending_state_change: None,
      pending_signal: Arc::new(Mutex::new(None)),
      current_match: None,
      stations: vec![],
      access: ArenaAccessRestriction::NoRestriction,
      lighting: ArenaLighting::new(ArenaLightingSettings::default()),
      audience_display: AudienceDisplay::Field,
      resources
    };

    for alliance in vec![Alliance::Blue, Alliance::Red] {
      for i in 1..(num_stations_per_alliance + 1) {
        a.stations
          .push(AllianceStation::new(AllianceStationId { alliance, station: i }));
      }
    }

    a
  }

  pub fn unload_match(&mut self) -> Result<()> {
    match self.state.state {
      ArenaState::Idle { ready: true } => {
        self.current_match = None;
        for stn in self.stations.iter_mut() {
          stn.reset();
        }
        Ok(())
      }
      ref s => bail!(CannotLoadMatchError(
        format!("Can't unload match in state {}", s).into()
      )),
    }
  }

  pub fn load_match(&mut self, m: LoadedMatch) -> Result<()> {
    match self.state.state {
      ArenaState::Idle { ready: true } => {
        self.load_match_teams(m.metadata())?;
        self.current_match = Some(m);
        Ok(())
      }
      ref s => bail!(CannotLoadMatchError(format!("Can't load match in state {}", s))),
    }
  }

  fn load_match_teams(&mut self, m: &models::Match) -> Result<()> {
    for stn in self.stations.iter_mut() {
      let v = match stn.station.alliance {
        Alliance::Blue => &m.blue_teams,
        Alliance::Red => &m.red_teams,
      };

      stn.reset();

      let i = (stn.station.station - 1) as usize;
      if let Some(&t) = v.get(i) {
        stn.team = t.map(|team| team as u16);
      } else {
        // Test matches are an exception - they start off blank
        if m.match_type != MatchType::Test {
          error!(
            "{} does not have the correct amount of alliance members! Defaulting to None...",
            m.name()
          );
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

  pub fn estop_station(&mut self, station: AllianceStationId) {
    let state = self.current_match.as_ref().map(|m| m.current_state());
    
    if let Some(stn) = self.station_mut(station) {
      match state {
        Some(MatchPlayState::Auto) => stn.astop = true,
        _ => stn.estop = true,
      }
    }
  }

  fn update_match_teams(&mut self) -> Result<()> {
    if let Some(m) = self.current_match.as_mut() {
      m.match_meta.blue_teams.resize(self.stations.len() / 2, None);
      m.match_meta.red_teams.resize(self.stations.len() / 2, None);

      for s in &self.stations {
        match s.station.alliance {
          Alliance::Blue => {
            m.match_meta.blue_teams[(s.station.station - 1) as usize] = s.team.map(|x| x as usize);
          }
          Alliance::Red => {
            m.match_meta.red_teams[(s.station.station - 1) as usize] = s.team.map(|x| x as usize);
          }
        }
      }
    }
    Ok(())
  }

  async fn update_field_estop(&mut self) -> Result<()> {
    if self.state.state != ArenaState::Estop {
      if let Some(ArenaSignal::Estop) = self.current_signal().await {
        self.prepare_state_change(ArenaState::Estop)?;
      }
    }
    Ok(())
  }

  async fn update_states(&mut self) -> Result<()> {
    let first = self.state.first;
    let signal = self.current_signal().await;

    let resource_requirements = DBResourceRequirements::get(&db::database())?.0;
    let resources_ok = match &resource_requirements {
      Some(req) => {
        let res = self.resources.lock().await;
        req.clone().status(&res).ready
      },
      None => true,
    };

    // Need this as self is borrowed as mut below
    match self.state.state {
      ArenaState::Idle { ready: true } if first => self.unload_match()?,
      _ => (),
    }

    match (self.state.state, &mut self.state.data) {
      (ArenaState::Init, _) => {
        if first {
          info!("Init...");
          self.prepare_state_change(ArenaState::Idle { ready: false })?;
        }
      }
      (ArenaState::Idle { ready: false }, StateData::Idle(maybe_recv)) => {
        if first {
          info!("Idle begin...");
        }

        if let Some(recv) = maybe_recv {
          // Check if network is ready
          let recv_result = recv.try_recv();
          match recv_result {
            Err(TryRecvError::Empty) => (), // Not ready yet
            Err(e) => panic!("Network runner fault: {}", e),
            Ok(result) => {
              result.map_err(|e| anyhow!(e))?;
              self.prepare_state_change(ArenaState::Idle { ready: true })?;
            }
          };
        }
      }
      (ArenaState::Idle { ready: true }, _) => {
        if first {
          info!("Idle ready!");
          self.resources.lock().await.reset_all();
        } else if let Some(ArenaSignal::Prestart) = signal {
          self.prepare_state_change(ArenaState::Prestart { ready: false })?;
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
        self.prepare_state_change(ArenaState::Idle { ready: false })?;
      }
      (ArenaState::Prestart { ready: false }, StateData::Prestart(maybe_recv)) => {
        if first {
          info!("Prestart begin...");
          self.resources.lock().await.reset_all();
          for stn in &mut self.stations {
            stn.astop = false;
            stn.estop = false;
          }
        }

        if let Some(recv) = maybe_recv {
          // Check if network is ready
          let recv_result = recv.try_recv();
          match recv_result {
            Err(TryRecvError::Empty) => (), // Not ready yet
            Err(e) => panic!("Network runner fault: {}", e),
            Ok(result) => {
              result.map_err(|e| anyhow!(e))?;
              self.prepare_state_change(ArenaState::Prestart { ready: true })?;
            }
          };
        }
      }
      (ArenaState::Prestart { ready: true }, _) => {
        if first {
          info!("Prestart Ready!");
          if let Some(reqs) = &resource_requirements {
            let mut resources = self.resources.lock().await;
            reqs.request_ready(&mut resources);
          }
        }
        if let Some(ArenaSignal::MatchArm(force)) = signal {
          if force || resources_ok {
            self.prepare_state_change(ArenaState::MatchArmed)?;
          } else {
            bail!("Can't Arm Match unless all resources are ready");
          }
        } else if let Some(ArenaSignal::Prestart) = signal {
          self.prepare_state_change(ArenaState::Prestart { ready: false })?;
        }
      }
      (ArenaState::MatchArmed, _) => {
        if first {
          info!("Match Armed!");
          self.resources.lock().await.reset_all();
        }

        if let Some(ArenaSignal::MatchPlay) = signal {
          self.prepare_state_change(ArenaState::MatchPlay)?;
        }
      }
      (ArenaState::MatchPlay, _) => {
        let m = self.current_match.as_mut().unwrap();
        if first {
          info!("Match play!");
          self.audience_display = AudienceDisplay::MatchPlay;
          m.start()?;
        }

        match m.current_state() {
          MatchPlayState::Pause | MatchPlayState::Teleop => for stn in self.stations.iter_mut() {
            stn.astop = false;
          },
          MatchPlayState::Complete => self.prepare_state_change(ArenaState::MatchComplete { ready: false })?,
          _ => ()
        }
      }
      (ArenaState::MatchComplete { ready: false }, StateData::MatchComplete(maybe_recv)) => {
        if first {
          info!("Match Complete... Resetting network.")
        }

        if let Some(recv) = maybe_recv {
          // Check if network is ready
          let recv_result = recv.try_recv();
          match recv_result {
            Err(TryRecvError::Empty) => (), // Not ready yet
            Err(e) => panic!("Network runner fault: {}", e),
            Ok(result) => {
              result.map_err(|e| anyhow!(e))?;
              self.prepare_state_change(ArenaState::MatchComplete { ready: true })?;
            }
          };
        }
      }
      (ArenaState::MatchComplete { ready: true }, _) => {
        if first {
          info!("Match complete and ready!")
        }
        if let Some(ArenaSignal::MatchCommit) = signal {
          self.prepare_state_change(ArenaState::MatchCommit)?;
        }
      }
      // (ArenaState::MatchComplete { ready }, _) => {
      //   if first {
      //     info!("Match complete!")
      //   }
      //   if let Some(ArenaSignal::MatchCommit) = signal {
      //     self.prepare_state_change(ArenaState::MatchCommit)?;
      //   }
      // }
      (ArenaState::MatchCommit, _) => {
        if first {
          self.update_match_teams()?;
          let m = self.current_match.as_mut().unwrap().commit_score().await?;
          self.prepare_state_change(ArenaState::Idle { ready: false })?;

          self.audience_display = AudienceDisplay::MatchResults(models::SerializedMatch::from(m));
        }
      }
      (state, _) => Err(anyhow!("Unimplemented state: {:?}", state))?,
    };

    self.update_lighting().await;

    Ok(())
  }

  pub async fn update_lighting(&mut self) {
    match self.state.state {
      ArenaState::Init => (),
      ArenaState::Idle { ready: _ } | ArenaState::Prestart { ready: _ } | ArenaState::MatchComplete { ready: _ } | ArenaState::MatchCommit => {
        match self.access {
          ArenaAccessRestriction::NoRestriction => {
            self.lighting.set_all(self.lighting.settings.idle)
          },
          ArenaAccessRestriction::ResetOnly => {
            self.lighting.set_all(self.lighting.settings.field_reset)
          },
          ArenaAccessRestriction::Teams => {
            self.lighting.set_all(self.lighting.settings.field_reset_teams)
          },
        }
      },
      ArenaState::Estop | ArenaState::EstopReset => {
        self.lighting.set_all(self.lighting.settings.field_estop);
      },
      ArenaState::MatchArmed => {
        self.lighting.set_alliance(Alliance::Blue, self.lighting.settings.match_armed_blue);
        self.lighting.set_alliance(Alliance::Red, self.lighting.settings.match_armed_red);
      },
      ArenaState::MatchPlay => {
        // Based on team connection status
        let mut any_red = false;
        let mut any_blue = false;

        for stn in &self.stations {
          let colour = match stn.station.alliance {
            Alliance::Blue => self.lighting.settings.blue,
            Alliance::Red => self.lighting.settings.red,
          };

          if stn.astop || stn.estop || stn.ds_report.map(|ref ds| ds.estop).unwrap_or(false) {
            self.lighting.set_team(stn.station, self.lighting.settings.team_estop);
          } else if stn.bypass {
            self.lighting.set_team(stn.station, LightMode::Off);
          } else if !stn.connection_ok() {
            self.lighting.set_team(stn.station, LightMode::Pulse(
              colour, Duration::seconds(1)
            ));

            match stn.station.alliance {
              Alliance::Blue => any_blue = true,
              Alliance::Red => any_red = true,
            }
          } else {
            self.lighting.set_team(stn.station, LightMode::Constant(colour));
          }
        }

        if any_red {
          self.lighting.set_table(Alliance::Red, LightMode::Pulse(
            self.lighting.settings.red, Duration::seconds(1)
          ))
        } else {
          self.lighting.set_table(Alliance::Red, LightMode::Constant(self.lighting.settings.red))
        }

        if any_blue {
          self.lighting.set_table(Alliance::Blue, LightMode::Pulse(
            self.lighting.settings.blue, Duration::seconds(1)
          ))
        } else {
          self.lighting.set_table(Alliance::Blue, LightMode::Constant(self.lighting.settings.blue))
        }
      },
    }
  }

  pub fn can_change_state_to(&self, desired: ArenaState) -> Result<()> {
    let current = self.state.state;
    let illegal = move |why: &str| ArenaIllegalStateChange {
      from: current,
      to: desired,
      why: why.to_owned(),
    };

    if current == desired {
      bail!(illegal("Can't change state to the current state!"));
    }

    match (&self.state.state, desired, &self.state.data) {
      (ArenaState::Init, ArenaState::Idle { ready: false }, _) => Ok(()),

      // E-Stops
      (_, ArenaState::Estop, _) => Ok(()),
      (ArenaState::Estop, ArenaState::EstopReset, _) => Ok(()),
      (ArenaState::EstopReset, ArenaState::Idle { ready: false }, _) => Ok(()),

      // Primary Flows
      (ArenaState::Idle { ready: false }, ArenaState::Idle { ready: true }, _) => Ok(()),
      (ArenaState::Idle { ready: true }, ArenaState::Prestart { ready: false }, _) => {
        // Prestart must not be ready (false)
        let m = self
          .current_match
          .as_ref()
          .ok_or(illegal("Cannot PreStart without a Match"))?;
        if m.current_state() != MatchPlayState::Waiting {
          bail!(illegal(&format!(
            "Match is not in waiting state! {:?}",
            m.current_state()
          )))
        } else {
          Ok(())
        }
      }
      (ArenaState::Prestart { ready: false }, ArenaState::Prestart { ready: true }, _) => Ok(()),
      (ArenaState::Prestart { ready: true }, ArenaState::Prestart { ready: false }, _) => Ok(()),
      (ArenaState::Prestart { ready: true }, ArenaState::MatchArmed, _) => {
        match self.access {
          ArenaAccessRestriction::NoRestriction => (),
          _ => bail!(illegal(
            "Cannot Arm Match if there is an Arena Access Restriction. Talk to the Head Ref!"
          )),
        }

        if self.stations.iter().all(|x| x.can_arm_match()) {
          Ok(())
        } else {
          bail!(illegal(
            "Cannot Arm Match: Not all teams are ready. Bypass any no-show teams.",
          ))
        }
      }
      (ArenaState::MatchArmed, ArenaState::MatchPlay, _) => Ok(()),
      (ArenaState::MatchPlay, ArenaState::MatchComplete { ready: false }, _) => {
        let m = log_expect!(self.current_match.as_ref().ok_or("No match!"));
        if m.current_state() != MatchPlayState::Complete {
          bail!(illegal("Match is not complete."))
        } else {
          Ok(())
        }
      }
      (ArenaState::MatchComplete { ready: false }, ArenaState::MatchComplete { ready: true }, _) => Ok(()),
      (ArenaState::MatchComplete { ready: true }, ArenaState::MatchCommit, _) => Ok(()),
      (ArenaState::MatchCommit, ArenaState::Idle { ready: false }, _) => Ok(()),

      _ => bail!(illegal("Undefined Transition")),
    }
  }

  fn do_state_init(&mut self, state: ArenaState) -> Result<BoundState> {
    self.can_change_state_to(state)?;

    let current = self.state.state;

    let basic = move |data: StateData| -> Result<BoundState> {
      Ok(BoundState {
        first: true,
        state,
        data,
      })
    };

    match (current, state, &self.state.data) {
      (_, ArenaState::Init, _) => basic(StateData::Init),
      (_, ArenaState::Estop, _) => basic(StateData::Estop),
      (_, ArenaState::EstopReset, _) => basic(StateData::EstopReset),
      (_, ArenaState::Idle { ready: false }, _) => self.state_init_idle(),
      (_, ArenaState::Idle { ready: true }, _) => basic(StateData::Idle(None)),
      (_, ArenaState::Prestart { ready: false }, _) => self.state_init_prestart(),
      (_, ArenaState::Prestart { ready: true }, _) => basic(StateData::Prestart(None)),
      (_, ArenaState::MatchArmed, _) => basic(StateData::MatchArmed),
      (_, ArenaState::MatchPlay, _) => basic(StateData::MatchPlay),
      (_, ArenaState::MatchComplete { ready: false }, _) => self.state_init_match_complete(),
      (_, ArenaState::MatchComplete { ready: true }, _) => basic(StateData::MatchComplete(None)),
      (_, ArenaState::MatchCommit, _) => basic(StateData::MatchCommit),
    }
  }

  fn state_init_with_network(&mut self, stations: Vec<AllianceStation>) -> Result<Option<Receiver<NetworkResult<()>>>> {
    Ok(self.network.clone().map(|nw| {
      let (tx, rx) = channel();

      tokio::task::spawn(async move {
        info!("Configuring Alliances...");
        let mtx = nw.lock().await;
        let result = mtx.configure(&stations[..]).await;
        tx.send(result).unwrap();
        info!("Alliances configured!");
      });

      rx
    }))
  }

  fn state_init_idle(&mut self) -> Result<BoundState> {
    let the_rx = self.state_init_with_network(self.stations.clone())?;

    Ok(BoundState {
      first: true,
      state: ArenaState::Idle {
        ready: the_rx.is_none(),
      }, // Ready if there's no network
      data: StateData::Idle(the_rx),
    })
  }

  fn state_init_prestart(&mut self) -> Result<BoundState> {
    let the_rx = self.state_init_with_network(self.stations.clone())?;

    Ok(BoundState {
      first: true,
      state: ArenaState::Prestart {
        ready: the_rx.is_none(),
      }, // Ready if there's no network
      data: StateData::Prestart(the_rx),
    })
  }

  fn state_init_match_complete(&mut self) -> Result<BoundState> {
    // Match complete should not have teams in the network config, but we don't want to edit the actual stations since they're
    // required by score commit.
    // let mut stations = self.stations.clone();
    let mut stations = vec![];
    for stn in &self.stations {
      let mut stn_copy = stn.clone();
      stn_copy.team = None;
      stations.push(stn_copy);
    }
    let the_rx = self.state_init_with_network(stations)?;

    Ok(BoundState {
      first: true,
      state: ArenaState::MatchComplete {
        ready: the_rx.is_none(),
      }, // Ready if there's no network
      data: StateData::MatchComplete(the_rx),
    })
  }

  pub async fn update(&mut self) {
    // Field Emergency Stop
    let estop_result = self.update_field_estop().await;
    match estop_result {
      Err(ref x) => error!("E-STOP Error {}", x),
      Ok(()) => (),
    }

    // If E-stop state change detected, do the state change ASAP
    if self.pending_state_change.is_some() {
      self.clear_signal().await;
      match self.perform_state_change() {
        Ok(()) => (),
        Err(ref e) => error!("Error during state change: {}", e),
      };
    }

    // General state updates
    let state_result = self.update_states().await;
    match state_result {
      Err(ref e) => error!("Error during state update: {}", e),
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
      Err(ref e) => error!("Error during state change: {}", e),
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

  fn prepare_state_change(&mut self, desired: ArenaState) -> Result<()> {
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

  fn perform_state_change(&mut self) -> Result<()> {
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
