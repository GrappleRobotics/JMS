use std::collections::HashMap;

use ipnetwork::IpNetwork;

use crate::arena::station::{Alliance, AllianceStationId};
use crate::arena::AllianceStation;

use super::NetworkProvider;

pub mod netlink;

pub struct OnboardNetwork {
  nl_handle: rtnetlink::Handle,
  admin_iface: String,
  station_ifaces: HashMap<AllianceStationId, String>,
}

impl OnboardNetwork {
  pub fn new(iface_admin: &str, ifaces_blue: &[&str], ifaces_red: &[&str]) -> super::NetworkResult<OnboardNetwork> {
    let mut station_ifaces = HashMap::new();

    for (i, &iface) in ifaces_red.iter().enumerate() {
      let id = AllianceStationId {
        alliance: Alliance::Red,
        station: (i + 1) as u32,
      };
      station_ifaces.insert(id, iface.to_owned());
    }

    for (i, &iface) in ifaces_blue.iter().enumerate() {
      let id = AllianceStationId {
        alliance: Alliance::Blue,
        station: (i + 1) as u32,
      };
      station_ifaces.insert(id, iface.to_owned());
    }

    Ok(OnboardNetwork {
      nl_handle: netlink::handle()?,
      admin_iface: iface_admin.to_owned(),
      station_ifaces,
    })
  }
}

#[async_trait::async_trait]
impl NetworkProvider for OnboardNetwork {
  async fn configure_admin(&self) -> super::NetworkResult<()> {
    netlink::configure_addresses(
      &self.nl_handle,
      self.admin_iface.as_str(),
      vec![IpNetwork::V4("10.0.100.5/24".parse()?)].into_iter(),
    )
    .await?;
    Ok(())
  }

  async fn configure_alliances(&self, stations: &[AllianceStation], _force_reload: bool) -> super::NetworkResult<()> {
    for &s in stations {
      let iface = self
        .station_ifaces
        .get(&s.station)
        .ok_or_else(|| NoInterfaceError::new(s.station))?;

      let mut addrs = vec![];
      if let Some(team) = s.team {
        addrs.push(IpNetwork::V4(format!("10.{}.{}.4/24", team / 100, team % 100).parse()?))
      }

      netlink::configure_addresses(&self.nl_handle, iface, addrs).await?;
    }
    Ok(())
  }
}

#[derive(Debug, Clone)]
struct NoInterfaceError {
  station: AllianceStationId,
}

impl NoInterfaceError {
  pub fn new(station: AllianceStationId) -> Self {
    Self { station }
  }
}

impl std::fmt::Display for NoInterfaceError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "No such interface for Alliance Station {}", self.station)
  }
}

impl std::error::Error for NoInterfaceError {}
