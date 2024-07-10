pub mod electronics;

use electronics::{JMSElectronics, JMSElectronicsService};
use jms_base::{kv, mq, logging::JMSLogger};
use jms_core_lib::{models::JmsComponent, db::Table};
use tokio::try_join;

async fn component_svc(kv: kv::KVConnection) -> anyhow::Result<()> {
  let mut interval = tokio::time::interval(std::time::Duration::from_millis(500));
  let mut component = JmsComponent::new("jms.electronics", "JMS-Electronics", "E", 1000);

  component.insert(&kv)?;

  loop {
    interval.tick().await;
    component.tick(&kv)?;
  }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let _ = JMSLogger::init().await?;

  let kv = kv::KVConnection::new()?;
  let mq = mq::MessageQueue::new("jms.networking-reply").await?;

  let component_fut = component_svc(kv.clone()?);

  let electronics_recv_fut = JMSElectronics::new(kv.clone()?, mq.channel().await?).run();
  let electronics_service_fut = JMSElectronicsService::new(mq.channel().await?, kv).await?.run();

  try_join!(component_fut, electronics_recv_fut, electronics_service_fut)?;

  Ok(())
}