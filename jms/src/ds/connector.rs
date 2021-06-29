use std::{error::Error, net::SocketAddr};

use chrono::Local;
use futures::SinkExt;
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::{time, time::Duration, try_join};
use tokio_stream::StreamExt;
use tokio_util::codec::Framed;
use tokio_util::udp::UdpFramed;

use crate::arena::SharedArena;
use crate::arena::station::{AllianceStationId, Alliance};
use crate::ds::{self, Fms2DsTCP, Fms2DsUDP};

use super::{DSTCPCodec, DSUDPCodec, Ds2FmsTCPTags, Fms2DsStationStatus, Fms2DsTCPTags};

use log::{debug, error, info};
use log::warn;

#[derive(Debug)]
pub enum DSDisconnectionReason {
  TCPTimeout,
  TCPClosed,
  TCPFault,
  UDPTimeout,
} 

#[derive(Debug)]
pub enum DSConnectionState {
  Connected,
  Disconnected(DSDisconnectionReason)
}

pub struct DSConnection {
  pub team: Option<u16>,
  pub state: DSConnectionState,
  addr_tcp: SocketAddr,
  addr_udp: SocketAddr,   // UDP Outgoing
  framed_tcp: Framed<TcpStream, DSTCPCodec>,
  framed_udp: UdpFramed<DSUDPCodec>,    // UDP Outgoing
  arena: SharedArena,
  valid: bool
}

impl DSConnection {
  pub async fn new(arena: SharedArena, addr: SocketAddr, stream: TcpStream) -> DSConnection {
    let mut addr_udp = addr;
    addr_udp.set_port(1121);

    let udp_socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();  // TODO: Is sending from 0 ok?

    DSConnection {
      team: None,
      addr_tcp: addr,
      addr_udp,
      framed_tcp: Framed::new(stream, DSTCPCodec::new()),
      framed_udp: UdpFramed::new(udp_socket, DSUDPCodec::new()),
      state: DSConnectionState::Connected,
      arena,
      valid: false
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
      },
      invalid => {
        error!("Invalid SocketAddr type: {:?}", invalid);
        None
      }
    }
  }

  pub async fn process_tcp(&mut self) {
    let mut udp_timer = time::interval(Duration::from_millis(250));
    let mut tcp_timer = time::interval(Duration::from_millis(1000));

    loop {
      tokio::select! {
        _ = udp_timer.tick() => {
          // Time to send UDP Updates
          if self.valid {
            if let Some(team) = self.team {
              // TODO: Use actual values
              let msg = Fms2DsUDP {
                estop: false,
                enabled: false,
                mode: ds::DSMode::Auto,
                station: self._alliance_station_id_or_blue1(team),
                tournament_level: ds::TournamentLevel::Qualification,
                match_number: 1,
                play_number: 1,
                time: Local::now(),
                remaining_seconds: 15,
              };
              self.framed_udp.send((msg, self.addr_udp)).await.unwrap();  // TODO: Handle error
            }
          }
        }

        _ = tcp_timer.tick() => {
          // Time for a TCP Update
          if let Some(team) = self.team {
            let mut tags = vec![];
            // TODO: Event Code (once implemented)
            // TODO: Game Data (once implemented)
            tags.push(self._construct_station_tag(team));

            self.framed_tcp.send(Fms2DsTCP{ tags }).await.unwrap(); // TODO: Handle error
          }
        }

        frame = self.framed_tcp.next() => match frame {
          // Received TCP Data
          Some(req) => match req {
            Ok(pkt) => {
              for tag in pkt.tags.iter() {
                self._process_tcp_tag(tag);
              }
            },
            Err(e) => {
              error!("TCP Error: {}", e);
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

  fn _process_tcp_tag(&mut self, tag: &Ds2FmsTCPTags) {
    match tag {
      Ds2FmsTCPTags::TeamNumber(team) => {
        self.team = Some(*team);
      },
      _ => () // Other, don't worry about it
    }
  }

  fn _construct_station_tag(&mut self, team: u16) -> Fms2DsTCPTags {
    let alliance = self.arena.lock().unwrap().station_for_team(team);
    match alliance {
      Some(a) => {
        // Team is allocated a station in this match
        let status = match self.team_by_ip() {
          Some(t) if t == team => {
            // Teams match
            self.valid = true;
            Fms2DsStationStatus::Good
          },
          Some(t) => {
            warn!("WRONG STATION: Team {} is in {}'s station", team, t);
            self.valid = false;
            Fms2DsStationStatus::Bad
          }
          None => {
            error!("Could not determine driver station validity. Accepting anyway... Team {}", team);
            self.valid = true;
            Fms2DsStationStatus::Good
          }
        };
        Fms2DsTCPTags::StationInfo(a.station, status)
      },
      None => {
        // Team isn't part of this match, or we're not prestarted yet
        warn!("Team {} does not yet belong to an alliance.", team);
        self.valid = false;
        Fms2DsTCPTags::StationInfo(
          AllianceStationId { alliance: Alliance::Blue, station: 1 },
          Fms2DsStationStatus::Waiting
        )
      }
    }
  }

  fn _alliance_station_id_or_blue1(&self, team: u16) -> AllianceStationId {
    let alliance = self.arena.lock().unwrap().station_for_team(team);
    alliance.map(|x| x.station).unwrap_or(AllianceStationId { alliance: Alliance::Blue, station: 1 })
  }
}

pub struct DSConnectionService {
  arena: SharedArena
}

impl DSConnectionService {
  pub fn new(arena: SharedArena) -> DSConnectionService {
    DSConnectionService {
      arena,
    }
  }

  pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
    let fut_tcp = Self::tcp(self.arena.clone());
    let fut_udp = Self::udp_recv(self.arena.clone());
    try_join!(fut_tcp, fut_udp)?;
    Ok(())
  }

  async fn tcp(arena: SharedArena) -> Result<(), Box<dyn Error>> {
    let server = TcpListener::bind("0.0.0.0:1750").await?;
    loop {
      info!("Listening for connections...");
      let (stream, addr) = server.accept().await?;
      debug!("Connected: {}", addr);

      let mut conn = DSConnection::new(arena.clone(), addr, stream).await;
      tokio::spawn(async move {
        conn.process_tcp().await;
        info!("TCP Connection {} disconnected with state {:?}", conn.addr_tcp, conn.state);
      });
    }
  }

  async fn udp_recv(_arena: SharedArena) -> Result<(), Box<dyn Error>> {
    let socket = UdpSocket::bind("0.0.0.0:1160").await?;
    let mut framed = UdpFramed::new(socket, DSUDPCodec::new());
    loop {
      tokio::select! {
        frame = framed.next() => match frame {
          Some(result) => match result {
            Ok((req, _addr)) => {
              // info!("UDP Data: {:?}", req);
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
