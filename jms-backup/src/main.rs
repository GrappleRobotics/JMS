use std::{collections::HashMap, fs::File, path::Path, io::Write};

use jms_backup_lib::{JMSBackupRPC, BackupSettings};
use jms_base::{kv::{self, KVConnection}, mq::{self, MessageQueue}, logging::JMSLogger};
use jms_core_lib::{models::{JmsComponent, EventDetails}, db::{Table, Singleton}};
use log::{info, warn, error};
use s3::Bucket;
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

  async fn backup_filesys(&self, settings: &BackupSettings, backup_filename: String, backup_data: &Vec<u8>) -> anyhow::Result<()> {
    if let Some(target) = &settings.file_target_dir {
      let path = Path::new(target).join(backup_filename);
      std::fs::create_dir_all(path.parent().unwrap().clone())?;
      let mut file = File::create(path)?;
      file.write_all(&backup_data[..])?;
      info!("Filesystem Backup Complete");
    } else {
      info!("Skipping Filesystem Target Backup: No Directory Provided");
    }
    Ok(())
  }

  async fn backup_s3(&self, settings: &BackupSettings, backup_filename: String, backup_data: &Vec<u8>) -> anyhow::Result<()> {
    match (&settings.s3_bucket, &settings.s3_access_key, &settings.s3_secret_access_key) {
      (Some(bucket_name), Some(access_key), Some(secret_access_key)) => {
        let bucket = Bucket::new(
          bucket_name,
          s3::Region::Custom { region: settings.s3_region.clone(), endpoint: settings.s3_endpoint.clone() },
          s3::creds::Credentials::new(Some(&access_key), Some(&secret_access_key), None, None, None)?
        )?.with_path_style();

        let response = bucket.put_object(backup_filename, &backup_data[..]).await?;
        if response.status_code() != 200 {
          anyhow::bail!("S3 Returned Error Code: {}", response.status_code())
        }
        info!("S3 Backup Complete");
      },
      _ => info!("Skipping S3 Backup - One of Bucket, Access Key, Secret Access Key are not provided")
    }
    Ok(())
  }

  async fn do_backup(&self) -> anyhow::Result<()> {
    if let Some(event_code) = EventDetails::get(&self.kv)?.code {
      info!("Starting Backup...");
      let settings = BackupSettings::get(&self.kv)?;
      let data = self.dump()?;
      let filename = format!("{}/jms-backup-{}-{}.json", event_code, event_code, chrono::Local::now().format("%Y-%m-%dT%H%M%S%z"));

      match self.backup_filesys(&settings, filename.clone(), &data).await {
        Ok(()) => (),
        Err(e) => error!("Filesystem Backup Error: {}", e)
      }

      match self.backup_s3(&settings, filename, &data).await {
        Ok(()) => (),
        Err(e) => error!("S3 Backup Error: {}", e)
      }
      info!("Backup Complete!");
    } else {
      warn!("Can't Backup - No Event Code Provided.")
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
    self.do_backup().await.map_err(|e| e.to_string())
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
    let mut backup_interval = tokio::time::interval(std::time::Duration::from_secs(5*60));   // 5 minutes

    loop {
      tokio::select! {
        msg = rpc.next() => self.rpc_process(msg).await?,
        _ = backup_interval.tick() => self.do_backup().await?
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
  let _ = JMSLogger::init().await?;

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
