use std::time::Duration;

use tokio::{io::{AsyncReadExt, AsyncWriteExt}, time};
use tokio_serial::SerialPortBuilderExt;

use crate::{arena::{ArenaSignal, SharedArena, lighting::ArenaLighting, station::AllianceStationId}, models::{self, Alliance}, electronics::comms::{Packable, Unpackable}};

use super::{ElectronicsMessageIn, ElectronicsMessageOut, AddressedElectronicsMessageOut, ElectronicsRole, AddressedElectronicsMessageIn};

// pub struct FieldElectronicsConnection {
//   arena: SharedArena,
//   socket: TcpStream,
//   role: Option<protos::NodeRole>,
//   update_rx: broadcast::Receiver<Vec<protos::UpdateField2Node>>
// }

// impl FieldElectronicsConnection {
//   pub async fn process(&mut self) -> anyhow::Result<()> {
    
//     loop {
//       let mut buf_read = vec![0u8; 256];
//       tokio::select! {
//         result = self.update_rx.recv() => {
//           // Send an update
//           let msgs = result?;
//           for msg in msgs {
//             if Some(msg.role()) == self.role {
//               let out = msg.encode_to_vec();
//               self.socket.write(&out).await?;
//             }
//           }
//         },
//         result = self.socket.read(&mut buf_read) => {
//           // Process the incoming message
//           let n_bytes = result?;
//           if n_bytes > 0 {
//             self.process_msg(&buf_read[0..n_bytes]).await?;
//           }
//         }
//       }
//     }
//   }

//   async fn process_msg(&mut self, buf: &[u8]) -> anyhow::Result<()> {
//     let msg = protos::UpdateNode2Field::decode(&mut Cursor::new(buf))?;
//     self.role = Some(msg.role());

//     let alliance  = Option::<models::Alliance>::from(msg.role());

//     match msg.data {
//       Some(data) => match (data, alliance) {
//         (protos::update_node2_field::Data::Alliance(data), Some(alliance)) => {
//           if data.estop1 || data.estop2 || data.estop3 {
//             let mut arena = self.arena.lock().await;

//             if data.estop1 {
//               arena.estop_station(AllianceStationId { alliance, station: 1 });
//             }

//             if data.estop2 {
//               arena.estop_station(AllianceStationId { alliance, station: 2 });
//             }

//             if data.estop3 {
//               arena.estop_station(AllianceStationId { alliance, station: 3 });
//             }
//           }
//         },
//         (protos::update_node2_field::Data::ScoringTable(st), None) => {
//           if st.abort {
//             self.arena.lock().await.signal(ArenaSignal::Estop).await;
//           }
//         },
//         _ => (),
//     },
//       None => (),
//     }

//     Ok(())
//   }
// }

pub struct FieldElectronicsService {
  arena: SharedArena,
  port: String,
  // tx: broadcast::Sender<Vec<ElectronicsMessageOut>>
}

impl FieldElectronicsService {
  pub async fn new(arena: SharedArena, port: String) -> Self {
    // let (tx, _) = broadcast::channel(16);
    FieldElectronicsService {
      arena,
      port,
      // tx
    }
  }

  pub async fn begin(&self) -> anyhow::Result<()> {
    info!("Starting Field Electronics Server");
    // let server = TcpListener::bind(format!("0.0.0.0:{}", self.port).as_str()).await?;
    let mut port = tokio_serial::new(&self.port, 115200).open_native_async()?;
    let mut send_interval = time::interval(Duration::from_millis(250));

    // TODO: Error handling & reconnect

    loop {
      tokio::select! {
        _ = send_interval.tick() => {
          // Send an update
          let msgs = self.create_update().await?;
          for msg in msgs {
            let mut buf = bytes::BytesMut::with_capacity(64);
            msg.pack(&mut buf);
            port.write_u8(buf.len() as u8).await?;
            port.write_buf(&mut buf).await?;
          }
        }
        result = port.read_u8() => {
          let n_bytes = result?;

          let mut buf = bytes::BytesMut::with_capacity(n_bytes as usize);
          port.read_buf(&mut buf).await?;

          // TODO: Handle
          self.handle(AddressedElectronicsMessageIn::unpack(&mut buf)).await?;
        }
      }
    }
  }

  async fn handle(&self, msg: AddressedElectronicsMessageIn) -> anyhow::Result<()> {
    match msg.msg {
      ElectronicsMessageIn::Ping => (),   // TODO: Handle timeouts, show in UI
      ElectronicsMessageIn::Estop(estops) => {
        if estops.field || estops.blue.iter().any(|&x| x) || estops.red.iter().any(|&x| x) {
          let mut arena = self.arena.lock().await;
          if estops.field {
            arena.signal(ArenaSignal::Estop).await;
          }

          for i in 0..3 {
            if estops.blue[i] { 
              arena.estop_station(AllianceStationId { alliance: Alliance::Blue, station: (i + 1) as u32 }) 
            }
            if estops.red[i] {
              arena.estop_station(AllianceStationId { alliance: Alliance::Red, station: (i + 1) as u32 }) 
            }
          }
        }
        
      },
    }
    Ok(())
  }

  async fn create_update(&self) -> anyhow::Result<Vec<AddressedElectronicsMessageOut>> {
    let lighting = self.arena.lock().await.lighting.clone();

    Ok(vec![
      Self::create_update_for_alliance(&lighting, ElectronicsRole::BlueDs)?,
      Self::create_update_for_alliance(&lighting, ElectronicsRole::RedDs)?,
      Self::create_update_for_scoring_table(&lighting)?
    ])
  }

  fn create_update_for_alliance(lighting: &ArenaLighting, role: ElectronicsRole) -> anyhow::Result<AddressedElectronicsMessageOut> {
    let lights = match role {
      ElectronicsRole::BlueDs => &lighting.teams[&models::Alliance::Blue],
      ElectronicsRole::RedDs => &lighting.teams[&models::Alliance::Red],
      role => anyhow::bail!("Role {:?} does not have an alliance!", role),
    };

    // TODO: multiple lights (one per team), etc
    Ok(AddressedElectronicsMessageOut {
      role,
      msg: ElectronicsMessageOut::SetLights(lights.clone()),
    })
  }

  fn create_update_for_scoring_table(lighting: &ArenaLighting) -> anyhow::Result<AddressedElectronicsMessageOut> {
    let red = &lighting.scoring_table[&models::Alliance::Red];
    let blue = &lighting.scoring_table[&models::Alliance::Blue];

    Ok(AddressedElectronicsMessageOut {
      role: ElectronicsRole::ScoringTable, 
      msg: ElectronicsMessageOut::SetLights(vec![ red.clone(), blue.clone() ])
    })
  }
}
