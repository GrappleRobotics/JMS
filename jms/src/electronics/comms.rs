use std::io::Cursor;

use prost::Message;
use tokio::{io::AsyncReadExt, net::{TcpListener, TcpStream}};

use crate::{arena::{ArenaSignal, SharedArena, station::AllianceStationId}, models};

use super::protos;

pub struct FieldElectronicsConnection {
  arena: SharedArena,
  socket: TcpStream,
  role: Option<protos::NodeRole>
}

impl FieldElectronicsConnection {
  pub async fn process(&mut self) -> anyhow::Result<()> {
    loop {
      let mut buf = vec![0; 256];
      let n_bytes = self.socket.read(&mut buf).await?;

      if n_bytes > 0 {
        self.process_msg(&buf[0..n_bytes]).await?;
      }
    }
  }

  async fn process_msg(&mut self, buf: &[u8]) -> anyhow::Result<()> {
    let msg = protos::UpdateNode2Field::decode(&mut Cursor::new(buf))?;
    self.role = Some(msg.role());

    let alliance  = Option::<models::Alliance>::from(msg.role());

    match msg.data {
      Some(data) => match (data, alliance) {
        (protos::update_node2_field::Data::Alliance(data), Some(alliance)) => {
          if data.estop1 || data.estop2 || data.estop3 {
            let mut arena = self.arena.lock().await;

            if data.estop1 {
              arena.estop_station(AllianceStationId { alliance, station: 1 });
            }

            if data.estop2 {
              arena.estop_station(AllianceStationId { alliance, station: 2 });
            }

            if data.estop3 {
              arena.estop_station(AllianceStationId { alliance, station: 3 });
            }
          }
        },
        (protos::update_node2_field::Data::ScoringTable(st), None) => {
          if st.abort {
            self.arena.lock().await.signal(ArenaSignal::Estop).await;
          }
        },
        _ => (),
    },
      None => (),
    }

    Ok(())
  }
}

pub struct FieldElectronicsService {
  arena: SharedArena,
  port: usize,
}

impl FieldElectronicsService {
  pub async fn new(arena: SharedArena, port: usize) -> Self {
    FieldElectronicsService { arena, port }
  }

  pub async fn begin(&self) -> anyhow::Result<()> {
    info!("Starting Field Electronics Server");
    let server = TcpListener::bind(format!("0.0.0.0:{}", self.port).as_str()).await?;

    loop {
      let (socket, addr) = server.accept().await?;
      info!("Field Electronics Connected: {}", addr);

      let mut conn = FieldElectronicsConnection { 
        arena: self.arena.clone(), 
        socket,
        role: None
      };
      
      tokio::spawn(async move {
        match conn.process().await {
          Ok(_) => println!("Field Electronics Conn Stopped Gracefully"),
          Err(e) => error!("Field Electronics Conn Stopped: {}", e),
        }
      });
    }
  }
}
