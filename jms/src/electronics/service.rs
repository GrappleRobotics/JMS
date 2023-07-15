use futures::try_join;
use tokio::net::UdpSocket;

use crate::arena::Arena;

use super::{settings::{ElectronicsSettings, LightingConfig}, lights::FieldLights};

const CLIENT_ADDRESSES: [&'static str; 1] = [
  "10.1.10.155:8283"
];

pub struct FieldElectronics {
  arena: Arena,
  settings: ElectronicsSettings
}

impl FieldElectronics {
  pub fn new(arena: Arena, settings: ElectronicsSettings) -> Self {
    FieldElectronics { arena, settings }
  }

  pub async fn run(self) -> anyhow::Result<()> {
    match self.settings.0 {
      Some(settings) => {
        info!("Starting Field Electronics Server");
        let lights_fut = Self::run_lights(self.arena.clone(), settings.lighting);

        try_join!(lights_fut)?;
        Ok(())
      },
      None => Ok(())
    }
  }

  pub async fn run_lights(arena: Arena, config: LightingConfig) -> anyhow::Result<()> {
    let mut lights = FieldLights::new(config);
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    loop {
      let dmx_msg = lights.update(arena.clone()).await?;
      
      for &addr in CLIENT_ADDRESSES.iter() {
        socket.send_to(&dmx_msg[..], addr).await?;
      }

      tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;
    }
  }
}

// pub struct FieldElectronicsService {
//   arena: Arena,
//   resources: SharedResources,
//   settings: ElectronicsSettings
// }

// impl FieldElectronicsService {
//   pub async fn new(arena: Arena, resources: SharedResources, settings: ElectronicsSettings) -> Self {
//     FieldElectronicsService {
//       arena,
//       resources,
//       settings,
//     }
//   }

//   pub async fn begin(self) -> anyhow::Result<()> {
//     match &self.settings.0 {
//       Some(settings) => {
//         info!("Starting Field Electronics Server");
//         loop {
//           {
//             let mut r = self.resources.lock().await;
//             r.remove(ELECTRONICS_RESOURCE_ID);
//           }

//           match self.start(settings).await {
//             Err(err) => {
//               self.arena.signal(ArenaSignal::Estop).await;
//               error!("Field Electronics Error: {}", err);
//               tokio::time::sleep(Duration::from_millis(250)).await;
//             },
//             Ok(_) => ()
//           }
//         }
//       },
//       None => Ok(()),
//     }
//   }

//   pub async fn start(&self, settings: &InnerElectronicsSettings) -> anyhow::Result<()> {
//     let mut port = tokio_serial::new(&settings.port, settings.baud as u32).open_native_async()?;
//     let mut send_interval = time::interval(Duration::from_millis(250));
//     let mut recv_timeout = time::interval(Duration::from_millis(2000));

//     recv_timeout.reset();

//     info!("Field Electronics Connected!");
//     {
//       let mut r = self.resources.lock().await;
//       r.register(ELECTRONICS_RESOURCE_ID, ResourceRole::FieldElectronics,  &None);
//     }

//     loop {
//       tokio::select! {
//         _ = send_interval.tick() => {
//           // Send an update
//           let msgs = self.create_update().await?;
//           for msg in msgs {
//             let mut buf = bytes::BytesMut::with_capacity(64);
//             msg.pack(&mut buf);
            
//             port.write_u8(buf.len() as u8).await?;
//             port.write_buf(&mut buf).await?;
//           }
//         },
//         _ = recv_timeout.tick() => anyhow::bail!("Receive Timed Out"),
//         result = port.read_u8() => {
//           let n_bytes = result?;

//           if n_bytes > 0 {
//             let mut buf = bytes::BytesMut::zeroed(n_bytes as usize);
//             port.read_exact(&mut buf).await?;

//             recv_timeout.reset();

//             self.handle(AddressedElectronicsMessageIn::unpack(&mut buf)).await?;
//           }
//         }
//       }
//     }
//   }

//   async fn handle(&self, msg: AddressedElectronicsMessageIn) -> anyhow::Result<()> {
//     match msg.msg {
//       ElectronicsMessageIn::Ping => (),   // TODO: Handle timeouts, show in UI
//       ElectronicsMessageIn::Estop(estops) => {
//         if estops.field || estops.blue.iter().any(|&x| x) || estops.red.iter().any(|&x| x) {
//           if estops.field {
//             self.arena.signal(ArenaSignal::Estop).await;
//           }

//           for i in 0..3 {
//             if estops.blue[i] { 
//               arena.estop_station(AllianceStationId { alliance: Alliance::Blue, station: (i + 1) as u32 }).await 
//             }
//             if estops.red[i] {
//               arena.estop_station(AllianceStationId { alliance: Alliance::Red, station: (i + 1) as u32 }).await
//             }
//           }
//         }
        
//       },
//     }
//     Ok(())
//   }

//   async fn create_update(&self) -> anyhow::Result<Vec<AddressedElectronicsMessageOut>> {
//     let lighting = self.arena.lock().await.lighting.clone();

//     Ok(vec![
//       Self::create_update_for_alliance(&lighting, ElectronicsRole::BlueDs)?,
//       Self::create_update_for_alliance(&lighting, ElectronicsRole::RedDs)?,
//       Self::create_update_for_scoring_table(&lighting)?
//     ])
//   }

//   fn create_update_for_alliance(lighting: &ArenaLighting, role: ElectronicsRole) -> anyhow::Result<AddressedElectronicsMessageOut> {
//     let lights = match role {
//       ElectronicsRole::BlueDs => &lighting.teams[&models::Alliance::Blue],
//       ElectronicsRole::RedDs => &lighting.teams[&models::Alliance::Red],
//       role => anyhow::bail!("Role {:?} does not have an alliance!", role),
//     };

//     Ok(AddressedElectronicsMessageOut {
//       role,
//       msg: ElectronicsMessageOut::SetLights(lights.clone()),
//     })
//   }

//   fn create_update_for_scoring_table(lighting: &ArenaLighting) -> anyhow::Result<AddressedElectronicsMessageOut> {
//     let red = &lighting.scoring_table[&models::Alliance::Red];
//     let blue = &lighting.scoring_table[&models::Alliance::Blue];

//     Ok(AddressedElectronicsMessageOut {
//       role: ElectronicsRole::ScoringTable, 
//       msg: ElectronicsMessageOut::SetLights(vec![ red.clone(), blue.clone() ])
//     })
//   }
// }
