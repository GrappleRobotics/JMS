mod schedule;
mod scoring;
mod reports;

use std::time::Duration;

use jms_base::{kv::{self}, mq::{MessageQueue, self}};
use jms_core_lib::{models::JmsComponent, db::Table};
use tokio::try_join;

async fn component_svc(kv: &kv::KVConnection, mq: mq::MessageQueueChannel) -> anyhow::Result<()> {
  let mut interval = tokio::time::interval(Duration::from_millis(500));
  let mut component = JmsComponent::new("jms.core", "JMS-Core", "C", 1000);

  component.insert(kv)?;

  loop {
    interval.tick().await;
    component.tick(kv)?;
  }
}

async fn save_db_svc(kv: &kv::KVConnection) -> anyhow::Result<()> {
  let mut interval = tokio::time::interval(Duration::from_millis(1000*60));   // Every 1 minute
  loop {
    interval.tick().await;
    kv.bgsave()?;
  }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  jms_base::logging::configure(false);
  
  let kv = kv::KVConnection::new()?;
  let mq = MessageQueue::new("arena-reply").await?;

  let mut mgsvc = schedule::GeneratorService { kv: kv.clone()?, mq: mq.channel().await? };
  let mgfut = mgsvc.run();

  let mut rgsvc = reports::service::ReportGeneratorService { kv: kv.clone()?, mq: mq.channel().await? };
  let rgfut = rgsvc.run();

  let ssvc = scoring::ScoringService { kv: kv.clone()?, mq: mq.channel().await? };
  let sfut = ssvc.run();

  let dbfut = save_db_svc(&kv);

  try_join!(mgfut, rgfut, sfut, dbfut, component_svc(&kv, mq.channel().await?))?;

  Ok(())
}
