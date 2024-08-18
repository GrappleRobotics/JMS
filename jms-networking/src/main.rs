use imaging::ImagingKeyService;
use jms_arena_lib::{ArenaState, AllianceStation, ArenaStateHook};
use jms_base::{kv, mq, logging::JMSLogger};
use jms_core_lib::{models::{JmsComponent, self, AllianceStationId, Alliance}, db::{Table, Singleton}};
use jms_networking_lib::{NetworkingSettings, JMSNetworkingRPC, RadioType};
use tokio::try_join;
use log::{info, error};

pub mod imaging;
pub mod linksys_ap;
pub mod mikrotik;
pub mod pfsense;
pub mod ssh;
pub mod unifi;

#[derive(rust_embed::RustEmbed)]
#[folder = "resources"]
pub struct Resources;

async fn component_svc(mut component: JmsComponent, kv: kv::KVConnection) -> anyhow::Result<()> {
  let mut interval = tokio::time::interval(std::time::Duration::from_millis(500));

  component.insert(&kv)?;

  loop {
    interval.tick().await;
    component.tick(&kv)?;
  }
}

// ( team number, WPA key ). If no WPA key, no wireless network is allocated.
#[derive(Debug, Clone, serde::Serialize)]
pub struct NetworkConfig {
  blue1: ( Option<usize>, Option<String> ),
  blue2: ( Option<usize>, Option<String> ),
  blue3: ( Option<usize>, Option<String> ),
  red1: ( Option<usize>, Option<String> ),
  red2: ( Option<usize>, Option<String> ),
  red3: ( Option<usize>, Option<String> ),
}

async fn do_team_network_update(network: NetworkConfig, settings: NetworkingSettings) -> anyhow::Result<()> {
  info!("Starting Network Update...");
  mikrotik::configure_firewall(&network, &settings).await.map_err(|e| anyhow::anyhow!("Mikrotik Error: {}", e))?;
  match settings.radio_type {
    RadioType::Linksys => linksys_ap::configure_ap_teams(&network, &settings).await.map_err(|e| anyhow::anyhow!("AP Error: {}", e))?,
    RadioType::Unifi => unifi::configure_ap_teams(&network, &settings).await.map_err(|e| anyhow::anyhow!("AP Error: {}", e))?,
  }
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
    let settings = NetworkingSettings::get(&self.kv).map_err(|e| e.to_string())?;
    if settings.radio_type == RadioType::Linksys {
      linksys_ap::configure_ap_admin(&settings).await.map_err(|e| e.to_string())
    } else {
      return Err("Not Supported".to_owned())
    }
  }

  async fn force_reprovision(&mut self) -> Result<(), String> {
    let settings = NetworkingSettings::get(&self.kv).map_err(|e| e.to_string())?;
    if settings.radio_type == RadioType::Unifi {
      unifi::force_reprovision(&settings).await.map_err(|e| e.to_string())
    } else {
      return Err("Not Supported".to_owned())
    }
  }
}

impl NetworkingService {
  pub async fn run(&mut self, mut hook_reset: ArenaStateHook, mut hook_prestart: ArenaStateHook) -> anyhow::Result<()> {
    // let mut event_subscriber: mq::MessageQueueSubscriber<ArenaState> = self.mq.subscribe("arena.state.new", "network-arena-state", "NetworkingService", false).await?;
    let mut rpc = self.rpc_handle().await?;

    loop {
      tokio::select! {
        rpcnext = rpc.next() => self.rpc_process(rpcnext).await?,
        state = hook_reset.next() => {
          state?;
          let config = NetworkConfig {
            blue1: (None, None),
            blue2: (None, None),
            blue3: (None, None),
            red1: (None, None),
            red2: (None, None),
            red3: (None, None),
          };
          match do_team_network_update(config, NetworkingSettings::get(&self.kv)?).await {
            Ok(()) => hook_reset.success(&self.mq).await?,
            Err(e) => { hook_reset.failure(anyhow::anyhow!("Network Update Failure: {}", e), &self.mq).await?; error!("Network Update Failure: {}", e) }
          }
        },
        state = hook_prestart.next() => {
          state?;
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
            blue1: (blue1, blue1.and_then(|t| teams.get(&t).map(|t| t.wpakey.clone()))),
            blue2: (blue2, blue2.and_then(|t| teams.get(&t).map(|t| t.wpakey.clone()))),
            blue3: (blue3, blue3.and_then(|t| teams.get(&t).map(|t| t.wpakey.clone()))),
            red1: (red1, red1.and_then(|t| teams.get(&t).map(|t| t.wpakey.clone()))),
            red2: (red2, red2.and_then(|t| teams.get(&t).map(|t| t.wpakey.clone()))),
            red3: (red3, red3.and_then(|t| teams.get(&t).map(|t| t.wpakey.clone()))),
          };

          match do_team_network_update(config, NetworkingSettings::get(&self.kv)?).await {
            Ok(()) => hook_prestart.success(&self.mq).await?,
            Err(e) => { hook_prestart.failure(anyhow::anyhow!("Network Update Failure: {}", e), &self.mq).await?; error!("Network Update Failure: {}", e) }
          }
        }
      }
      
    }
  }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let _ = JMSLogger::init().await?;

  let kv = kv::KVConnection::new()?;
  let mq = mq::MessageQueue::new("jms.networking-reply").await?;

  let channel = mq.channel().await?;

  let component = JmsComponent::new("jms.networking", "JMS-Networking", "N", 1000);
  let hook_reset = ArenaStateHook::new("jms.networking.reset", &component, ArenaState::Reset { ready: false }, std::time::Duration::from_secs(30), &kv, &channel).await?;
  let hook_prestart = ArenaStateHook::new("jms.networking.prestart", &component, ArenaState::Prestart { ready: false }, std::time::Duration::from_secs(30), &kv, &channel).await?;

  let component_svc = component_svc(component, kv.clone()?);

  let mut networking = NetworkingService { kv: kv.clone()?, mq: mq.channel().await? };
  let imaging = ImagingKeyService::new();
  try_join!(component_svc, networking.run(hook_reset, hook_prestart), imaging.run(kv))?;

  Ok(())
}
