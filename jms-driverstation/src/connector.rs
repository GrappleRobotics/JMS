use std::{net::SocketAddr, time::Duration, sync::{atomic::AtomicBool, Arc}};

use chrono::Local;
use futures::{StreamExt, SinkExt};
use jms_arena_lib::{AllianceStation, ARENA_STATE_KEY, ArenaState, SerialisedLoadedMatch, ARENA_MATCH_KEY, MatchPlayState};
use jms_base::kv::KVConnection;
use jms_core_lib::{models::{AllianceStationId, Alliance}, db::Table};
use jms_driverstation_lib::{RobotState, TournamentLevel, DriverStationReport};
use log::error;
use tokio::{net::{TcpStream, UdpSocket}, sync::broadcast, time::{Instant, self}};
use tokio_util::{codec::Framed, udp::UdpFramed};

use crate::{tcp_codec::{DSTCPCodec, Fms2DsStationStatus, Fms2DsTCP, Fms2DsTCPTags, Ds2FmsTCPTags}, udp_codec::{DSUDPCodec, Ds2FmsUDP, Fms2DsUDP, Ds2FmsUDPTags}};

#[derive(Debug, PartialEq, Eq)]
pub enum DSDisconnectionReason {
  // TCPTimeout,
  TCPClosed,
  TCPFault,
  Timeout,
  WrongMatch
}

#[derive(Debug, PartialEq, Eq)]
pub enum DSConnectionState {
  Connected,
  Disconnected(DSDisconnectionReason),
}

pub struct DSConnection {
  pub team: Option<usize>,
  pub state: DSConnectionState,
  pub addr_tcp: SocketAddr,
  addr_udp: SocketAddr, // UDP Outgoing
  framed_tcp: Framed<TcpStream, DSTCPCodec>,
  framed_udp: UdpFramed<DSUDPCodec>, // UDP Outgoing
  udp_rx: broadcast::Receiver<Ds2FmsUDP>,
  last_packet_time: Instant,
  wrong_station_n: usize,
  kv: KVConnection,
  arena_ok: Arc<AtomicBool>
}

impl DSConnection {
  pub async fn new(
    kv: KVConnection,
    addr: SocketAddr,
    stream: TcpStream,
    udp_rx: broadcast::Receiver<Ds2FmsUDP>,
    arena_ok: Arc<AtomicBool>
  ) -> DSConnection {
    let mut addr_udp = addr;
    addr_udp.set_port(1121);

    let udp_socket = UdpSocket::bind("0.0.0.0:0").await.unwrap(); // TODO: Is sending from 0 ok?

    DSConnection {
      team: None,
      addr_tcp: addr,
      addr_udp,
      framed_tcp: Framed::new(stream, DSTCPCodec::new()),
      framed_udp: UdpFramed::new(udp_socket, DSUDPCodec::new()),
      udp_rx,
      state: DSConnectionState::Connected,
      last_packet_time: Instant::now(),
      wrong_station_n: 0,
      kv,
      arena_ok
    }
  }

  // Get the team number according to the source address,
  // as this defines whether the driver station is in the
  // correct station or not.
  // The DS reports the team number according to what's been input,
  // but the IP reflects the DS IP. If the DS is DHCP, there will be
  // a mismatch here. If the IP is static, the DS packets will never
  // make it to the FMS as the interface will not accept the packets
  // (outside the appropriate subnet).
  pub fn team_by_ip(&self) -> Option<usize> {
    match self.addr_tcp {
      SocketAddr::V4(v4) => {
        let ip = v4.ip();
        let [_, hi, lo, _] = ip.octets();
        if hi == 0 && lo == 100 {
          // We're on the admin network / a flat network
          None
        } else {
          // We're on a team network
          Some((hi as usize) * 100 + (lo as usize))
        }
      }
      invalid => {
        error!("Invalid SocketAddr type: {:?}", invalid);
        None
      }
    }
  }

  pub async fn process(&mut self) {
    let mut udp_timer = time::interval(Duration::from_millis(250));
    let mut tcp_timer = time::interval(Duration::from_millis(1000));

    while self.state == DSConnectionState::Connected {
      tokio::select! {
        // UDP Update
        _ = udp_timer.tick() => {
          if self._get_station_status().await == Fms2DsStationStatus::Good {
            if let Some(team) = self.team {
              if let Some(msg) = self._encode_udp_update(team).await {
                self.framed_udp.send((msg, self.addr_udp)).await.unwrap();  // TODO: Handle error
              }
            }

            // Check timeout
            if self.last_packet_time.elapsed() > Duration::from_millis(5000) {
              self.state = DSConnectionState::Disconnected(DSDisconnectionReason::Timeout);
              break;
            }
          }
        }

        // TCP Update
        _ = tcp_timer.tick() => {
          // The DS buffers all TCP messages, so if we send TCP messages more often than the DS sends
          // them, the DS thinks it's still connected even if we close the socket.
          // Nice one, NI.

          let status = self._get_station_status().await;

          // Update arena record of station status
          {
            // let mut arena = self.arena.lock().await;
            match status {
              Fms2DsStationStatus::Good => {
                self.wrong_station_n = 0;
              },
              Fms2DsStationStatus::Bad => {
                self.wrong_station_n += 1;
              },
              Fms2DsStationStatus::Waiting => {
                self.wrong_station_n += 1;
              },
            }
          }

          if self.wrong_station_n >= 20 {
            self.state = DSConnectionState::Disconnected(DSDisconnectionReason::WrongMatch);
          }
        }

        // UDP Data
        udp_frame = self.udp_rx.recv() => {
          match udp_frame {
            Ok(pkt) if Some(pkt.team as usize) == self.team => {
              self.last_packet_time = Instant::now();
              self._decode_udp_update(pkt).await;
            },
            Ok(_) => (),  // Ignore it, not for us
            Err(e) => error!("UDP Receive error: {}", e),
          }
        }

        // TCP Data
        frame = self.framed_tcp.next() => {
          match frame {
            Some(req) => match req {
              Ok(pkt) => {
                for tag in pkt.tags.iter() {
                  self._process_tcp_tag(tag);
                }

                // TCP Update
                let status = self._get_station_status().await;

                if let Some(_team) = self.team {
                  let mut tags = vec![];
                  // TODO: Event Code (once implemented)
                  // TODO: Game Data (once implemented)
                  tags.push(self._construct_station_tag(status).await);

                  self.framed_tcp.send(Fms2DsTCP{ tags }).await.unwrap(); // TODO: Handle error
                }
              },
              Err(e) => {
                error!("TCP Error({:?}): {}", self.team, e);
                self.state = DSConnectionState::Disconnected(DSDisconnectionReason::TCPFault);
                break;
              }
            },
            None => {
              self.state = DSConnectionState::Disconnected(DSDisconnectionReason::TCPClosed);
              break;
            },
          }
        }
      }
    }

    // Connection closed, it'll age off.
  }

  async fn _encode_udp_update(&self, _team: usize) -> Option<Fms2DsUDP> {
    if let Some(station) = self._get_desired_alliance_station().await {
      if let Ok(arena_state) = self.kv.json_get::<ArenaState>(ARENA_STATE_KEY, "$") {
        let (mut command_enable, command_state, remaining) = match self.kv.json_get::<SerialisedLoadedMatch>(ARENA_MATCH_KEY, "$") {
          Ok(m) => match m.state {
            MatchPlayState::Auto => (true, RobotState::Auto, m.remaining.0),
            MatchPlayState::Pause => (false, RobotState::Teleop, m.remaining.0),
            MatchPlayState::Teleop => (true, RobotState::Teleop, m.remaining.0),
            _ => (false, RobotState::Auto, m.remaining.0)
          },
          _ => (false, RobotState::Auto, chrono::Duration::milliseconds(0))
        };

        if !self.arena_ok.load(std::sync::atomic::Ordering::Relaxed) {
          // If the arena isn't OK, robots should not be running
          command_enable = false;
        }

        let estop = station.estop || matches!(arena_state, ArenaState::Estop);
        let astop = station.astop && command_state == RobotState::Auto;

        let remaining_seconds = remaining.to_std().unwrap_or(Duration::from_millis(0)).as_secs_f32();

        let pkt = Fms2DsUDP {
          estop: estop,
          enabled: (!station.bypass) && !(estop || astop) && command_enable,
          mode: command_state,
          station: station.id,
          tournament_level: TournamentLevel::Test,      // TODO: Change this
          match_number: 1,
          play_number: 1,
          time: Local::now(),
          remaining_seconds: f32::max(remaining_seconds, 0f32) as u16,
        };

        // TODO: Later
        // if let Some(m) = match_meta {
        //   pkt.tournament_level = ds::TournamentLevel::from(m.match_type);
        //   // We use the same encoding as cheesy-arena. For matches with set numbers, the match num is encoded as
        //   // XYZ, where X = final bracket (Q=4, S=2, F=1), Y = set number, Z = match number
        //   pkt.match_number = match m.match_type {
        //     models::MatchType::Playoff => match m.match_subtype.unwrap() {
        //       models::MatchSubtype::Quarterfinal => (400 + 10 * m.set_number + m.match_number) as u16,
        //       models::MatchSubtype::Semifinal => (200 + 10 * m.set_number + m.match_number) as u16,
        //       models::MatchSubtype::Final => (100 + 10 * m.set_number + m.match_number) as u16,
        //     },
        //     _ => m.match_number as u16,
        //   };
        // }

        Some(pkt)
      } else {
        None
      }
    } else {
      None
    }
  }

  async fn _decode_udp_update(&self, pkt: Ds2FmsUDP) {
    // let mut arena = self.arena.lock().await;
    // let station_mut = arena.station_for_team_mut(self.team);
    // let mut stns = self.stations.lock().await;

    let mut report = DriverStationReport {
      team: pkt.team,
      robot_ping: pkt.robot,
      rio_ping: pkt.rio,
      radio_ping: pkt.radio,
      battery_voltage: pkt.battery,
      estop: pkt.estop,
      mode: pkt.mode,
      pkts_sent: 0,
      pkts_lost: 0,
      rtt: 0,
      actual_station: self._get_occupied_alliance_station().await.map(|x| x.id),
    };

    for tag in pkt.tags {
      match tag {
        Ds2FmsUDPTags::CommsMetrics(lost, sent, rtt) => {
          report.pkts_sent = sent;
          report.pkts_lost = lost;
          report.rtt = rtt;
        },
        _ => ()
      }
    }

    report.insert(&self.kv).ok();
    report.expire(2, &self.kv).ok();
  }

  fn _process_tcp_tag(&mut self, tag: &Ds2FmsTCPTags) {
    match tag {
      Ds2FmsTCPTags::TeamNumber(team) => {
        self.team = Some(*team as usize);
      }
      _ => (), // Other, don't worry about it for now
    }
  }

  async fn _construct_station_tag(&self, status: Fms2DsStationStatus) -> Fms2DsTCPTags {
    let correct_station = self._get_desired_alliance_station().await.map(|x| x.id);

    Fms2DsTCPTags::StationInfo(
      correct_station.unwrap_or(AllianceStationId::new(Alliance::Blue, 1)), // Default to Blue 1 for Waiting
      status,
    )
  }

  async fn _get_station_status(&self) -> Fms2DsStationStatus {
    // If we're on the admin / flat network, automatically assume we're in the right station.
    if self.team_by_ip() == None {
      return Fms2DsStationStatus::Good;
    }

    let desired = self._get_desired_alliance_station().await;
    let actual = self._get_occupied_alliance_station().await;

    match desired {
      // This team isn't in this match
      None => Fms2DsStationStatus::Waiting,
      Some(stn_desired) => {
        match actual {
          // Can't determine actual station, mustn't be in this match.
          None => Fms2DsStationStatus::Waiting,
          // Team is in the correct station
          Some(stn_actual) if stn_actual.id == stn_desired.id => Fms2DsStationStatus::Good,
          // Team's desired station doesn't match their actual station
          Some(_) => Fms2DsStationStatus::Bad,
        }
      }
    }
  }

  async fn _get_desired_alliance_station(&self) -> Option<AllianceStation> {
    self._get_alliance_station(self.team).await
  }

  async fn _get_occupied_alliance_station(&self) -> Option<AllianceStation> {
    self._get_alliance_station(self.team_by_ip()).await
  }

  async fn _get_alliance_station(&self, team: Option<usize>) -> Option<AllianceStation> {
    if team.is_none() {
      return None;
    }

    for stn_id in AllianceStationId::all() {
      let stn = AllianceStation::get(&stn_id, &self.kv);
      match stn {
        Ok(s) if s.team == team => return Some(s),
        _ => ()
      }
    }
    None
  }
}