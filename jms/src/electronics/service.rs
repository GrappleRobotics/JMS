use std::time::Duration;

use tokio::{io::{AsyncReadExt, AsyncWriteExt}, time};
use tokio_serial::SerialPortBuilderExt;

use crate::{arena::{ArenaSignal, SharedArena, lighting::ArenaLighting, station::AllianceStationId}, models::{self, Alliance}, electronics::comms::{Packable, Unpackable}};

use super::{ElectronicsMessageIn, ElectronicsMessageOut, AddressedElectronicsMessageOut, ElectronicsRole, AddressedElectronicsMessageIn, settings::{ElectronicsSettings, InnerElectronicsSettings}};

pub struct FieldElectronicsService {
  arena: SharedArena,
  settings: ElectronicsSettings
}

impl FieldElectronicsService {
  pub async fn new(arena: SharedArena, settings: ElectronicsSettings) -> Self {
    FieldElectronicsService {
      arena,
      settings,
    }
  }

  pub async fn begin(&self) -> anyhow::Result<()> {
    match &self.settings.0 {
      Some(settings) => {
        info!("Starting Field Electronics Server");
        loop {
          match self.start(settings).await {
            Err(err) => {
              self.arena.lock().await.signal(ArenaSignal::Estop).await;
              error!("Field Electronics Error: {}", err);
              tokio::time::sleep(Duration::from_millis(250)).await;
            },
            Ok(_) => return Ok(())
          }
        }
      },
      None => Ok(()),
    }
  }

  pub async fn start(&self, settings: &InnerElectronicsSettings) -> anyhow::Result<()> {
    let mut port = tokio_serial::new(&settings.port, settings.baud as u32).open_native_async()?;
    let mut send_interval = time::interval(Duration::from_millis(250));
    let mut recv_timeout = time::interval(Duration::from_millis(500));

    recv_timeout.reset();

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
        },
        _ = recv_timeout.tick() => anyhow::bail!("Receive Timed Out"),
        result = port.read_u8() => {
          let n_bytes = result?;

          if n_bytes > 0 {
            let mut buf = bytes::BytesMut::zeroed(n_bytes as usize);
            port.read_exact(&mut buf).await?;

            recv_timeout.reset();

            self.handle(AddressedElectronicsMessageIn::unpack(&mut buf)).await?;
          }
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
