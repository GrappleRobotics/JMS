use std::time::Duration;

use jms_arena_lib::{ArenaRPC, ArenaRPCClient};
use jms_base::{kv, mq::{self, MessageQueueChannel}};
use jms_core_lib::{models::JmsComponent, db::Table};
use log::{info, warn};
use tokio::{io::{AsyncWriteExt, AsyncReadExt}, try_join};
use tokio_serial::SerialPortBuilderExt;

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
  jms_base::logging::configure(false);
  let kv = kv::KVConnection::new()?;
  let mq = mq::MessageQueue::new("jms.networking-reply").await?;

  let component_fut = component_svc(kv);
  let scoring_table_fut = run_scoring_table(mq.channel().await?);
  try_join!(scoring_table_fut, component_fut)?;

  Ok(())
}