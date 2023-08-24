use jms_base::{kv, logging::JMSLogger, mq};
use jms_core_lib::{models::{JmsComponent, self}, db::Table};
use jms_tba_lib::TBARPC;
use log::{info, warn};
use tokio::try_join;

use crate::{alliances::TBAAlliances, eventinfo::TBAEventInfoUpdate, rankings::TBARankings, teams::TBATeams, matches::TBAMatchUpdate};

pub mod alliances;
pub mod client;
pub mod matches;
pub mod eventinfo;
pub mod teams;
pub mod rankings;

async fn component_svc(kv: kv::KVConnection) -> anyhow::Result<()> {
  let mut interval = tokio::time::interval(std::time::Duration::from_millis(500));
  let mut component = JmsComponent::new("jms.tba", "JMS-TBA", "T", 1000);

  component.insert(&kv)?;

  loop {
    interval.tick().await;
    component.tick(&kv)?;
  }
}

async fn do_update(kv: &kv::KVConnection) -> anyhow::Result<()> {
  info!("Starting TBA Update");
  
  // Event Info
  if let Err(e) = TBAEventInfoUpdate::issue(&kv).await { warn!("Could not issue event info update: {}", e) }

  // Teams
  if let Err(e) = TBATeams::issue(&kv).await { warn!("Could not issue teams update: {}", e) }

  // Alliances
  let alliances: TBAAlliances = models::PlayoffAlliance::all(&kv)?.into();
  if let Err(e) = alliances.issue(&kv).await { warn!("Could not issue alliance update: {}", e) }

  // Rankings
  let rankings: TBARankings = models::TeamRanking::all(&kv)?.into();
  if let Err(e) = rankings.issue(&kv).await { warn!("Could not issue ranking update: {}", e) }

  // Matches
  if let Err(e) = TBAMatchUpdate::issue(&kv).await { warn!("Could not issue teams update: {}", e) }

  info!("TBA Update Finished");
  Ok(())
}

pub struct TBAService {
  kv: kv::KVConnection, mq: mq::MessageQueueChannel
}

#[async_trait::async_trait]
impl TBARPC for TBAService {
  fn mq(&self) ->  &jms_base::mq::MessageQueueChannel { &self.mq }

  async fn update_now(&mut self) -> Result<(), String> {
    do_update(&self.kv).await.map_err(|e| e.to_string())
  }
}

impl TBAService {
  pub async fn run(&mut self) -> anyhow::Result<()> {
    let mut rpc = self.rpc_handle().await?;
    let mut tba_update_interval = tokio::time::interval(std::time::Duration::from_secs(5*60)); /* Every 5 minutes */

    loop {
      tokio::select! {
        _ = tba_update_interval.tick() => {
          do_update(&self.kv).await?;
        },
        rpcnext = rpc.next() => self.rpc_process(rpcnext).await?,
      }
    }
  }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let _ = JMSLogger::init().await?;

  let kv = kv::KVConnection::new()?;
  let mq = mq::MessageQueue::new("jms-tba-reply").await?;

  let component_svc = component_svc(kv.clone()?);
  let mut tba_svc = TBAService { kv, mq: mq.channel().await? };

  try_join!(component_svc, tba_svc.run())?;

  Ok(())
}
