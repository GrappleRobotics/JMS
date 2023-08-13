use std::collections::HashMap;

use imaging::ImagingKeyService;
use jms_arena_lib::{ArenaState, AllianceStation};
use jms_base::{kv, mq, logging::configure};
use jms_core_lib::{models::{JmsComponent, self, Team, AllianceStationId, Alliance}, db::{Table, Singleton}};
use jms_networking_lib::{NetworkingSettings, JMSNetworkingRPC};
use tokio::try_join;
use log::{info, error};

pub mod imaging;
pub mod linksys_ap;
pub mod pfsense;
pub mod ssh;

#[derive(rust_embed::RustEmbed)]
#[folder = "resources"]
pub struct Resources;

async fn component_svc(kv: kv::KVConnection) -> anyhow::Result<()> {
  let mut interval = tokio::time::interval(std::time::Duration::from_millis(500));
  let mut component = JmsComponent::new("jms.networking", "JMS-Networking", "N", 1000);

  component.insert(&kv)?;

  loop {
    interval.tick().await;
    component.tick(&kv)?;
  }
}

// ( team number, WPA key ). If no WPA key, no wireless network is allocated.
#[derive(Debug, Clone, serde::Serialize)]
pub struct NetworkConfig {
  blue1: ( usize, Option<String> ),
  blue2: ( usize, Option<String> ),
  blue3: ( usize, Option<String> ),
  red1: ( usize, Option<String> ),
  red2: ( usize, Option<String> ),
  red3: ( usize, Option<String> ),
}

async fn do_team_network_update(network: NetworkConfig, settings: NetworkingSettings) -> anyhow::Result<()> {
  info!("Starting Network Update...");
  pfsense::configure_firewall(&network, &settings).await.map_err(|e| anyhow::anyhow!("PFSense Error: {}", e))?;
  linksys_ap::configure_ap_teams(&network, &settings).await.map_err(|e| anyhow::anyhow!("AP Error: {}", e))?;
  info!("Network Update Complete!");
  Ok(())
}

pub struct NetworkingService {
  kv: kv::KVConnection, mq: mq::MessageQueueChannel
}

#[async_trait::async_trait]
impl JMSNetworkingRPC for NetworkingService {
  fn mq(&self) ->  &jms_base::mq::MessageQueueChannel { &self.mq }

  async fn configure_admin(&mut self) -> Result<(),String> {
    linksys_ap::configure_ap_admin(&NetworkingSettings::get(&self.kv).map_err(|e| e.to_string())?).await.map_err(|e| e.to_string())
  }
}

impl NetworkingService {
  pub async fn run(&mut self) -> anyhow::Result<()> {
    let mut event_subscriber: mq::MessageQueueSubscriber<ArenaState> = self.mq.subscribe("arena.state.new", "network-arena-state", "NetworkingService", false).await?;
    let mut rpc = self.rpc_handle().await?;

    loop {
      tokio::select! {
        rpcnext = rpc.next() => self.rpc_process(rpcnext).await?,
        msg = event_subscriber.next() => match msg {
          Some(Ok(td)) => {
            match td.data {
              ArenaState::Reset => {
                let config = NetworkConfig {
                  blue1: (1, None),
                  blue2: (2, None),
                  blue3: (3, None),
                  red1: (4, None),
                  red2: (5, None),
                  red3: (6, None),
                };
                match do_team_network_update(config, NetworkingSettings::get(&self.kv)?).await {
                  Ok(()) => (),
                  Err(e) => error!("Network Update Failure: {}", e)
                }
              },
              ArenaState::Prestart => {
                let stations = AllianceStation::all_map(&self.kv)?;
                let teams = models::Team::all_map(&self.kv)?;

                // Yes, this is messy but it's only for 6 teams so I don't really care..
                let blue1 = stations.get(&AllianceStationId::new(Alliance::Blue, 1)).map(|x| x.team).unwrap_or(None);
                let blue2 = stations.get(&AllianceStationId::new(Alliance::Blue, 2)).map(|x| x.team).unwrap_or(None);
                let blue3 = stations.get(&AllianceStationId::new(Alliance::Blue, 3)).map(|x| x.team).unwrap_or(None);
                let red1 = stations.get(&AllianceStationId::new(Alliance::Red, 1)).map(|x| x.team).unwrap_or(None);
                let red2 = stations.get(&AllianceStationId::new(Alliance::Red, 2)).map(|x| x.team).unwrap_or(None);
                let red3 = stations.get(&AllianceStationId::new(Alliance::Red, 3)).map(|x| x.team).unwrap_or(None);

                let config = NetworkConfig {
                  blue1: (blue1.unwrap_or(1), blue1.and_then(|t| teams.get(&t).map(|t| t.wpakey.clone()))),
                  blue2: (blue2.unwrap_or(2), blue1.and_then(|t| teams.get(&t).map(|t| t.wpakey.clone()))),
                  blue3: (blue3.unwrap_or(3), blue1.and_then(|t| teams.get(&t).map(|t| t.wpakey.clone()))),
                  red1: (red1.unwrap_or(4), red1.and_then(|t| teams.get(&t).map(|t| t.wpakey.clone()))),
                  red2: (red2.unwrap_or(5), red2.and_then(|t| teams.get(&t).map(|t| t.wpakey.clone()))),
                  red3: (red3.unwrap_or(6), red3.and_then(|t| teams.get(&t).map(|t| t.wpakey.clone()))),
                };

                match do_team_network_update(config, NetworkingSettings::get(&self.kv)?).await {
                  Ok(()) => (),
                  Err(e) => error!("Network Update Failure: {}", e)
                }
              },
              _ => ()
            }
          },
          Some(Err(e)) => anyhow::bail!("Error: {}", e),
          None => ()
        }
      }
      
    }
  }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  jms_base::logging::configure(false);
  let kv = kv::KVConnection::new()?;
  let mq = mq::MessageQueue::new("jms.networking-reply").await?;

  let component_svc = component_svc(kv.clone()?);

  let mut networking = NetworkingService { kv: kv.clone()?, mq: mq.channel().await? };
  let imaging = ImagingKeyService::new();
  try_join!(component_svc, networking.run(), imaging.run(kv))?;

  Ok(())
}
