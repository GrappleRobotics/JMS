use std::collections::HashMap;

use jms_backup_lib::JMSBackupRPC;
use jms_base::{kv::{self, KVConnection}, mq::{self, MessageQueue}, logging};
use jms_core_lib::{models::JmsComponent, db::Table};
use log::info;
use tokio::try_join;

struct JMSBackups {
  kv: kv::KVConnection,
  mq: mq::MessageQueueChannel
}

impl JMSBackups {
  fn new(kv: kv::KVConnection, mq: mq::MessageQueueChannel) -> Self {
    Self { kv, mq }
  }

  fn dump(&self) -> anyhow::Result<Vec<u8>> {
    let mut data: HashMap<String, serde_json::Value> = HashMap::new();
    for key in self.kv.keys("db:*")? {
      data.insert(key.clone(), self.kv.json_get(&key, "$")?);
    }
    Ok(serde_json::to_vec(&data)?)
  }

  fn load(&self, data: Vec<u8>) -> anyhow::Result<()> {
    let data: HashMap<String, serde_json::Value> = serde_json::from_slice(&data[..])?;
    for (key, value) in data.iter() {
      self.kv.json_set(key, "$", value)?;
    }
    Ok(())
  }
}

#[async_trait::async_trait]
impl JMSBackupRPC for JMSBackups {
  fn mq(&self) -> &mq::MessageQueueChannel {
    &self.mq
  }

  async fn backup_now(&mut self) -> Result<(), String> {
    Ok(())
  }

  async fn backup_to(&mut self) -> Result<Vec<u8>, String> {
    info!("Performing Backup...");
    let data = self.dump().map_err(|e| e.to_string())?;
    info!("Backup Complete!");
    Ok(data)
  }

  async fn restore(&mut self, backup: Vec<u8>) -> Result<(), String> {
    info!("Performing Restore...");
    self.load(backup).map_err(|e| e.to_string())?;
    info!("Restore Complete!");
    Ok(())
  }
}

impl JMSBackups {
  pub async fn run(&mut self) -> anyhow::Result<()> {
    let mut rpc = self.rpc_handle().await?;
    loop {
      tokio::select! {
        msg = rpc.next() => self.rpc_process(msg).await?
      }
    }
  }
}

async fn component_svc(mut component: JmsComponent, kv: kv::KVConnection) -> anyhow::Result<()> {
  let mut interval = tokio::time::interval(std::time::Duration::from_millis(500));

  component.insert(&kv)?;

  loop {
    interval.tick().await;
    component.tick(&kv)?;
  }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  logging::configure(false);
  let kv = KVConnection::new()?;
  let mq = MessageQueue::new("arena-reply").await?;
  info!("Connected!");
  
  let component = JmsComponent::new("jms.backup", "JMS-Backup", "B", 1000);
  let component_fut = component_svc(component, kv.clone()?);

  let mut backups = JMSBackups::new(kv, mq.channel().await?);
  let backups_fut = backups.run();

  try_join!(component_fut, backups_fut)?;

  Ok(())
}
