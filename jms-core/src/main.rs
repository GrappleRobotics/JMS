mod schedule;

use jms_base::{kv::{self}, mq::MessageQueue};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let kv = kv::KVConnection::new()?;
  let mq = MessageQueue::new("arena-reply").await?;

  let mut mgsvc = schedule::GeneratorService { kv: kv.clone()?, mq: mq.channel().await? };
  mgsvc.run().await?;

  Ok(())
}
