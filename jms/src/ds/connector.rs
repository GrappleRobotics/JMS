use std::time::Instant;
use std::{error::Error, net::SocketAddr};

use chrono::Local;
use futures::SinkExt;
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::broadcast;
use tokio::{time, time::Duration, try_join};
use tokio_stream::StreamExt;
use tokio_util::codec::Framed;
use tokio_util::udp::UdpFramed;

use crate::arena::matches::MatchPlayState;
use crate::arena::station::AllianceStationId;
use crate::arena::{AllianceStation, AllianceStationDSReport, AllianceStationOccupancy, ArenaState, SharedArena};
use crate::ds::{self, Fms2DsTCP, Fms2DsUDP};
use crate::models;

use super::{DSTCPCodec, DSUDPCodec, Ds2FmsTCPTags, Ds2FmsUDP, Fms2DsStationStatus, Fms2DsTCPTags};

use log::{debug, error, info};

#[derive(Debug, PartialEq, Eq)]
pub enum DSDisconnectionReason {
  // TCPTimeout,
  TCPClosed,
  TCPFault,
  Timeout,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DSConnectionState {
  Connected,
  Disconnected(DSDisconnectionReason),
}

pub struct DSConnection {
  pub team: Option<u16>,
  pub state: DSConnectionState,
  addr_tcp: SocketAddr,
  addr_udp: SocketAddr, // UDP Outgoing
  framed_tcp: Framed<TcpStream, DSTCPCodec>,
  framed_udp: UdpFramed<DSUDPCodec>, // UDP Outgoing
  udp_rx: broadcast::Receiver<Ds2FmsUDP>,
  arena: SharedArena,
  last_packet_time: Instant,
}

impl DSConnection {
  pub async fn new(
    arena: SharedArena,
    addr: SocketAddr,
    stream: TcpStream,
    udp_rx: broadcast::Receiver<Ds2FmsUDP>,
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
      arena,
      last_packet_time: Instant::now(),
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
  pub fn team_by_ip(&self) -> Option<u16> {
    match self.addr_tcp {
      SocketAddr::V4(v4) => {
        let ip = v4.ip();
        let [_, hi, lo, _] = ip.octets();
        Some((hi as u16) * 100 + (lo as u16))
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
            if self.last_packet_time.elapsed() > Duration::from_millis(2000) {
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
            let mut arena = self.arena.lock().await;

            match status {
              Fms2DsStationStatus::Good => {
                if let Some(stn) = arena.station_for_team_mut(self.team) {
                  stn.occupancy = AllianceStationOccupancy::Occupied;
                }
              },
              Fms2DsStationStatus::Bad => {
                if let Some(stn) = arena.station_for_team_mut(self.team_by_ip()) {
                  stn.occupancy = AllianceStationOccupancy::WrongStation;
                } else if let Some(stn) = arena.station_for_team_mut(self.team) {
                  // Fallback to team (not actual occupied station) if not available
                  stn.occupancy = AllianceStationOccupancy::WrongStation;
                }
              },
              Fms2DsStationStatus::Waiting => {
                if let Some(stn) = arena.station_for_team_mut(self.team_by_ip()) {
                  stn.occupancy = AllianceStationOccupancy::WrongMatch;
                }
                // No fallback - Waiting status means that the station for self.team is None
              },
            }
          }
        }

        // UDP Data
        udp_frame = self.udp_rx.recv() => {
          match udp_frame {
            Ok(pkt) if Some(pkt.team) == self.team => {
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

    // Connection closed, notify Arena
    {
      let mut arena = self.arena.lock().await;
      if let Some(stn) = arena.station_for_team_mut(self.team_by_ip()) {
        stn.ds_report = None;
        stn.occupancy = AllianceStationOccupancy::Vacant;
      } else if let Some(stn) = arena.station_for_team_mut(self.team) {
        stn.ds_report = None;
        stn.occupancy = AllianceStationOccupancy::Vacant;
      }
    }
  }

  async fn _encode_udp_update(&self, _team: u16) -> Option<Fms2DsUDP> {
    if let Some(station) = self._get_desired_alliance_station().await {
      let arena = self.arena.lock().await;

      let match_state = arena.current_match.as_ref().map(|m| m.current_state());
      let estop = station.estop || (arena.current_state() == ArenaState::Estop);
      let astop = station.astop;

      let (mode, robots_enabled) = match match_state {
        Some(MatchPlayState::Auto) => (ds::DSMode::Auto, true),
        Some(MatchPlayState::Teleop) => (ds::DSMode::Teleop, true),
        _ => (ds::DSMode::Auto, false),
      };

      let remaining_seconds = arena
        .current_match
        .as_ref()
        .map(|x| x.remaining_time())
        .map(|dt| dt.as_secs_f32())
        .unwrap_or(0f32);

      let match_meta = arena.current_match.as_ref().map(|x| x.metadata());

      let mut pkt = Fms2DsUDP {
        estop: estop,
        enabled: (!station.bypass) && !(estop || astop) && robots_enabled,
        mode,
        station: station.station,
        tournament_level: ds::TournamentLevel::Test,
        match_number: 1,
        play_number: 1,
        time: Local::now(),
        remaining_seconds: f32::max(remaining_seconds, 0f32) as u16,
      };

      if let Some(m) = match_meta {
        pkt.tournament_level = ds::TournamentLevel::from(m.match_type);
        // We use the same encoding as cheesy-arena. For matches with set numbers, the match num is encoded as
        // XYZ, where X = final bracket (Q=4, S=2, F=1), Y = set number, Z = match number
        pkt.match_number = match m.match_type {
          models::MatchType::Playoff => match m.match_subtype.unwrap() {
            models::MatchSubtype::Quarterfinal => (400 + 10 * m.set_number + m.match_number) as u16,
            models::MatchSubtype::Semifinal => (200 + 10 * m.set_number + m.match_number) as u16,
            models::MatchSubtype::Final => (100 + 10 * m.set_number + m.match_number) as u16,
          },
          _ => m.match_number as u16,
        };
      }

      Some(pkt)
    } else {
      None
    }
  }

  async fn _decode_udp_update(&self, pkt: Ds2FmsUDP) {
    let mut arena = self.arena.lock().await;
    let station_mut = arena.station_for_team_mut(self.team);

    match station_mut {
      Some(station_mut) => {
        let mut report = station_mut.ds_report.unwrap_or(AllianceStationDSReport::default());

        report.robot_ping = pkt.robot;
        report.radio_ping = pkt.radio;
        report.rio_ping = pkt.rio;
        report.battery = pkt.battery;

        report.estop = pkt.estop;
        report.mode = if pkt.enabled { Some(pkt.mode) } else { None };

        for tag in pkt.tags {
          match tag {
            ds::Ds2FmsUDPTags::CommsMetrics(lost, sent, rtt) => {
              report.pkts_lost = lost;
              report.pkts_sent = sent;
              report.rtt = rtt;
            }
            _ => (), // Other tags, don't worry about them for now
          }
        }

        station_mut.ds_report = Some(report);
      }
      None => (),
    }
  }

  fn _process_tcp_tag(&mut self, tag: &Ds2FmsTCPTags) {
    match tag {
      Ds2FmsTCPTags::TeamNumber(team) => {
        self.team = Some(*team);
      }
      _ => (), // Other, don't worry about it for now
    }
  }

  async fn _construct_station_tag(&self, status: Fms2DsStationStatus) -> Fms2DsTCPTags {
    let correct_station = self._get_desired_alliance_station().await.map(|x| x.station);

    // if let Some(team) = self.team {
    //   match (&status, correct_station) {
    //       (Fms2DsStationStatus::Good, _) => (),
    //       (Fms2DsStationStatus::Bad, Some(correct)) => {
    //         warn!("WRONG STATION: Team {} is in the wrong station. Move to {}", team, correct);
    //       },
    //       (Fms2DsStationStatus::Bad, None) => error!("Uh oh!"),   // This shouldn't ever trigger
    //       (Fms2DsStationStatus::Waiting, _) => {
    //         warn!("WRONG MATCH: Team {} is not in this match.", team);
    //       }
    //   }
    // }

    Fms2DsTCPTags::StationInfo(
      correct_station.unwrap_or(AllianceStationId::blue1()), // Default to Blue 1 for Waiting
      status,
    )
  }

  async fn _get_station_status(&self) -> Fms2DsStationStatus {
    let desired = self._get_desired_alliance_station().await;
    let actual = self._get_occupied_alliance_station().await;

    match desired {
      // This team isn't in this match
      None => Fms2DsStationStatus::Waiting,
      Some(stn_desired) => {
        match actual {
          // Can't determine actual station, mustn't be in this match. TODO: Delegate these checks to the NetworkProvider
          None => Fms2DsStationStatus::Waiting,
          // Team is in the correct station
          Some(stn_actual) if stn_actual.station == stn_desired.station => Fms2DsStationStatus::Good,
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

  async fn _get_alliance_station(&self, team: Option<u16>) -> Option<AllianceStation> {
    self.arena.lock().await.station_for_team(team)
  }
}

// SERVICE //

pub struct DSConnectionService {
  arena: SharedArena,
  udp_tx: broadcast::Sender<Ds2FmsUDP>,
}

impl DSConnectionService {
  pub async fn new(arena: SharedArena) -> DSConnectionService {
    let (udp_tx, _rx) = broadcast::channel(16);
    DSConnectionService { arena, udp_tx }
  }

  pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
    let fut_tcp = Self::tcp(self.arena.clone(), &self.udp_tx);
    let fut_udp = Self::udp_recv(self.arena.clone(), &self.udp_tx);
    try_join!(fut_tcp, fut_udp)?;
    Ok(())
  }

  async fn tcp(arena: SharedArena, udp_tx: &broadcast::Sender<Ds2FmsUDP>) -> Result<(), Box<dyn Error>> {
    let server = TcpListener::bind("0.0.0.0:1750").await?;
    loop {
      info!("Listening for connections...");
      let (stream, addr) = server.accept().await?;
      debug!("Connected: {}", addr);

      let mut conn = DSConnection::new(arena.clone(), addr, stream, udp_tx.subscribe()).await;
      tokio::spawn(async move {
        conn.process().await;
        info!(
          "TCP Connection {} disconnected with state {:?}",
          conn.addr_tcp, conn.state
        );
      });
    }
  }

  async fn udp_recv(_arena: SharedArena, udp_tx: &broadcast::Sender<Ds2FmsUDP>) -> Result<(), Box<dyn Error>> {
    let socket = UdpSocket::bind("0.0.0.0:1160").await?;
    let mut framed = UdpFramed::new(socket, DSUDPCodec::new());
    loop {
      tokio::select! {
        frame = framed.next() => match frame {
          Some(result) => match result {
            Ok((_req, _addr)) => {
              // We send to all DSComms since it simplifies the team number checking, as TCP connections
              // don't communicate their team number until after connection.
              match udp_tx.send(_req) {
                Ok(_) => (),
                Err(e) => {
                  error!("UDP Packets received without any DS connections: {}", e);
                },
              }
            },
            Err(e) => {
              error!("UDP Error: {}", e);
            }
          },
          None => ()
        }
      }
    }
  }
}
