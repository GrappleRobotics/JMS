pub mod onboard;
pub mod radio;
pub mod snmp;

use crate::{
  arena::station::AllianceStation,
  config::{enum_interactive, Interactive},
};

use async_trait::async_trait;

use self::onboard::{settings::OnboardNetworkSettings, OnboardNetwork};

const ADMIN_IP: &'static str = "10.0.100.5/24";
const ADMIN_ROUTER: &'static str = "10.0.100.1/24";

pub type NetworkResult<T> = anyhow::Result<T>;

#[async_trait]
pub trait NetworkProvider {
  async fn configure(&self, stations: &[AllianceStation]) -> NetworkResult<()>;
}

#[derive(serde::Serialize, serde::Deserialize, EnumDiscriminants, Clone, Debug)]
#[strum_discriminants(derive(EnumIter, EnumString, Display))]
#[serde(tag = "type")]
pub enum InnerNetworkSettings {
  Onboard(OnboardNetworkSettings),
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct NetworkSettings(Option<InnerNetworkSettings>);

impl NetworkSettings {
  pub fn create(&self) -> NetworkResult<Option<Box<dyn NetworkProvider + Send + Sync>>> {
    Ok(match self.0.as_ref() {
      Some(s) => Some(match s.clone() {
        InnerNetworkSettings::Onboard(obs) => Box::new(OnboardNetwork::new(obs)?),
      }),
      None => None,
    })
  }
}

#[async_trait::async_trait]
impl Interactive for InnerNetworkSettings {
  async fn interactive() -> anyhow::Result<Self> {
    let disc = enum_interactive::<InnerNetworkSettingsDiscriminants, _, _>("Network Type?")?;
    match disc {
      InnerNetworkSettingsDiscriminants::Onboard => Ok(InnerNetworkSettings::Onboard(
        OnboardNetworkSettings::interactive().await?,
      )),
    }
  }
}

#[async_trait::async_trait]
impl Interactive for NetworkSettings {
  async fn interactive() -> anyhow::Result<Self> {
    let use_network = inquire::Confirm::new("Configure network?")
      .with_default(true)
      .prompt()?;
    match use_network {
      true => Ok(Self(Some(InnerNetworkSettings::interactive().await?))),
      false => Ok(Self(None)),
    }
  }
}
