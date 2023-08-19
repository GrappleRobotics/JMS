use std::time::Duration;

pub mod ds_electronics;

use ds_electronics::DriverStationElectronics;
use jms_arena_lib::ArenaRPCClient;
use jms_base::{kv, mq::{self, MessageQueueChannel}, logging::JMSLogger};
use jms_core_lib::{models::{JmsComponent, Alliance}, db::Table};
use log::{info, warn};
use tokio::{io::{AsyncWriteExt, AsyncReadExt}, try_join};
use tokio_serial::SerialPortBuilderExt;
use futures_util::FutureExt;

async fn component_svc(kv: kv::KVConnection) -> anyhow::Result<()> {
  let mut interval = tokio::time::interval(std::time::Duration::from_millis(500));
  let mut component = JmsComponent::new("jms.electronics", "JMS-Electronics", "E", 1000);

  component.insert(&kv)?;

  loop {
    interval.tick().await;
    component.tick(&kv)?;
  }
}

async fn run_scoring_table(mq: MessageQueueChannel) -> anyhow::Result<()> {
  let tty = std::env::var("SCORING_TABLE_PORT").unwrap_or("/dev/ttyACM0".to_owned());

  info!("Starting Scoring Table Service on Port {}", tty);
  let mut port = tokio_serial::new(tty, 115200).open_native_async()?;
  info!("Connected!");

  let mut interval = tokio::time::interval(Duration::from_millis(100));

  let mut incoming = [0u8; 1];

  loop {
    interval.tick().await;
    port.write("a".as_bytes()).await?;
    port.flush().await?;
    match tokio::time::timeout(Duration::from_millis(100), port.read_exact(&mut incoming[..])).await {
      Ok(result) => {
        result?;
        let char = char::from(incoming[0]);
        if char == 'E' {
          match ArenaRPCClient::signal(&mq, jms_arena_lib::ArenaSignal::Estop, "Field Electronics (Scoring Table)".to_owned()).await? {
            Ok(()) => (),
            Err(e) => warn!("Signal error: {}", e)
          }
        }
      },
      Err(_) => warn!("Field Electronics - No Bytes!")
    };
  }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let _ = JMSLogger::init().await?;

  let kv = kv::KVConnection::new()?;
  let mq = mq::MessageQueue::new("jms.networking-reply").await?;

  let component_fut = component_svc(kv.clone()?);

  let scoring_table_fut = if !std::env::var("NO_SCORING_TABLE").is_ok() {
    run_scoring_table(mq.channel().await?).left_future()
  } else {
    futures::future::ok(()).right_future()
  };

  let ds_elec_futs = if std::env::var("DS_ELECTRONICS").is_ok() {
    let blue = DriverStationElectronics::new(Alliance::Blue, None, kv.clone()?, mq.channel().await?);
    let red = DriverStationElectronics::new(Alliance::Red, None, kv.clone()?, mq.channel().await?);

    (blue.run().left_future(), red.run().left_future())
  } else {
    (futures::future::ok(()).right_future(), futures::future::ok(()).right_future())
  };

  try_join!(scoring_table_fut, component_fut, ds_elec_futs.0, ds_elec_futs.1)?;

  Ok(())
}