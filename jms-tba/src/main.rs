use jms_base::{kv, logging::JMSLogger};
use jms_core_lib::{models::{JmsComponent, self}, db::Table};
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

async fn tba_svc(kv: kv::KVConnection) -> anyhow::Result<()> {
  let mut tba_update_interval = tokio::time::interval(std::time::Duration::from_secs(5*60)); /* Every 5 minutes */

  loop {
    tba_update_interval.tick().await;
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
  }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let _ = JMSLogger::init().await?;

  let kv = kv::KVConnection::new()?;

  let component_svc = component_svc(kv.clone()?);
  let tba_svc = tba_svc(kv.clone()?);

  try_join!(component_svc, tba_svc)?;

  Ok(())
}
