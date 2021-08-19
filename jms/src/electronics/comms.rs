use std::{io::Cursor, time::Duration};

use prost::Message;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream}, sync::broadcast, time};

use crate::{arena::{ArenaSignal, SharedArena, lighting::{ArenaLighting, LightMode}, station::AllianceStationId}, models};

use super::protos;

pub struct FieldElectronicsConnection {
  arena: SharedArena,
  socket: TcpStream,
  role: Option<protos::NodeRole>,
  update_rx: broadcast::Receiver<Vec<protos::UpdateField2Node>>
}

impl FieldElectronicsConnection {
  pub async fn process(&mut self) -> anyhow::Result<()> {
    let mut buf_read = vec![0u8; 256];

    loop {
      tokio::select! {
        result = self.update_rx.recv() => {
          // Send an update
          let msgs = result?;
          for msg in msgs {
            if Some(msg.role()) == self.role {
              let out = msg.encode_to_vec();
              self.socket.write(&out).await?;
            }
          }
        },
        result = self.socket.read(&mut buf_read) => {
          // Process the incoming message
          let n_bytes = result?;
          if n_bytes > 0 {
            self.process_msg(&buf_read[0..n_bytes]).await?;
          }
        }
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
  tx: broadcast::Sender<Vec<protos::UpdateField2Node>>
}

impl FieldElectronicsService {
  pub async fn new(arena: SharedArena, port: usize) -> Self {
    let (tx, _) = broadcast::channel(16);
    FieldElectronicsService {
      arena,
      port,
      tx
    }
  }

  pub async fn begin(&self) -> anyhow::Result<()> {
    info!("Starting Field Electronics Server");
    let server = TcpListener::bind(format!("0.0.0.0:{}", self.port).as_str()).await?;
    let mut send_interval = time::interval(Duration::from_millis(1000));

    loop {
      tokio::select! {
        _ = send_interval.tick() => {
          // Send an update. Issuing updates from here means we only have to lock the arena once
          // instead of N times where N = number of nodes
          let msgs = self.create_update().await?;
          match self.tx.send(msgs) {
            Ok(_) => (),
            Err(e) => debug!("No Field Electronics available! {}", e),
          }
        }
        result = server.accept() => {
          // Accept a connection 
          let (socket, addr) = result?;
          info!("Field Electronics Connected: {}", addr);
          let mut conn = FieldElectronicsConnection { 
            arena: self.arena.clone(), 
            socket,
            role: None,
            update_rx: self.tx.subscribe()
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
  }

  async fn create_update(&self) -> anyhow::Result<Vec<protos::UpdateField2Node>> {
    let lighting = self.arena.lock().await.lighting.clone();

    Ok(vec![
      Self::create_update_for_alliance(&lighting, protos::NodeRole::NodeBlue)?,
      Self::create_update_for_alliance(&lighting, protos::NodeRole::NodeRed)?,
      Self::create_update_for_scoring_table(&lighting)?
    ])
  }

  fn create_update_for_alliance(lighting: &ArenaLighting, role: protos::NodeRole) -> anyhow::Result<protos::UpdateField2Node> {
    let alliance: Option<models::Alliance> = role.into();
    let teams = match alliance {
      Some(models::Alliance::Blue) => &lighting.teams[&models::Alliance::Blue],
      Some(models::Alliance::Red) => &lighting.teams[&models::Alliance::Red],
      None => anyhow::bail!("Role {:?} does not have an alliance!", role),
    };

    Ok(protos::UpdateField2Node {
      role: role.into(),
      data: Some(protos::update_field2_node::Data::Alliance(protos::update_field2_node::Alliance {
        lights1: Some(teams.get(0).unwrap_or(&LightMode::Off).clone().into()),
        lights2: Some(teams.get(1).unwrap_or(&LightMode::Off).clone().into()),
        lights3: Some(teams.get(2).unwrap_or(&LightMode::Off).clone().into()),
      })),
    })
  }

  fn create_update_for_scoring_table(lighting: &ArenaLighting) -> anyhow::Result<protos::UpdateField2Node> {
    let red = &lighting.scoring_table[&models::Alliance::Red];
    let blue = &lighting.scoring_table[&models::Alliance::Blue];

    Ok(protos::UpdateField2Node {
      role: protos::NodeRole::NodeScoringTable.into(),
      data: Some(protos::update_field2_node::Data::ScoringTable(protos::update_field2_node::ScoringTable {
        lights1: Some(red.clone().into()),
        lights2: Some(blue.clone().into()),
      }))
    })
  }
}
