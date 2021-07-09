use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};

use ipnetwork::{IpNetwork, Ipv4Network};
use tokio::try_join;

use crate::arena::station::{Alliance, AllianceStationId};
use crate::arena::AllianceStation;

use super::NetworkProvider;

pub mod netlink;
pub mod dhcp;
pub mod firewall;

const ADMIN_IP: &'static str = "10.0.100.5/24";
const ADMIN_ROUTER: &'static str = "10.0.100.1/24";

pub struct OnboardNetwork {
  nl_handle: rtnetlink::Handle,
  wan_iface: String,
  admin_iface: String,
  station_ifaces: HashMap<AllianceStationId, String>,
}

impl OnboardNetwork {

  pub fn new(iface_wan: &str, iface_admin: &str, ifaces_blue: &[&str], ifaces_red: &[&str]) -> super::NetworkResult<OnboardNetwork> {
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
      wan_iface: iface_wan.to_owned(),
      admin_iface: iface_admin.to_owned(),
      station_ifaces,
    })
  }

  async fn configure_ip_addrs(&self, stations: &[AllianceStation]) -> super::NetworkResult<()> {
    netlink::configure_addresses(
      &self.nl_handle,
      self.admin_iface.as_str(),
      vec![
        self.v4_network(ADMIN_IP)?,     // Admin gets both 10.0.100.5 and 10.0.100.1
        self.v4_network(ADMIN_ROUTER)?
      ].into_iter(),
    )
    .await?;

    // TODO: Reverse this. Iterate through the ifaces and lookup into stations
    for &s in stations {
      let iface = self
        .station_ifaces
        .get(&s.station)
        .ok_or_else(|| NoInterfaceError::new(s.station))?;

      let mut addrs = vec![];
      if let Some(team) = s.team {
        addrs.push(self.team_ip(team)?)
      }

      netlink::configure_addresses(&self.nl_handle, iface, addrs).await?;
    }

    Ok(())
  }

  async fn configure_dhcp(&self, stations: &[AllianceStation]) -> super::NetworkResult<()> {
    let admin_cfg = dhcp::DHCPConfig {
      router: self.v4_network(ADMIN_ROUTER)?,
      dhcp: self.dhcp_range(self.v4_network(ADMIN_IP)?)?
    };
    
    let station_dhcps: Vec<dhcp::TeamDHCPConfig> = stations.iter().map(|s| {
      dhcp::TeamDHCPConfig {
        station: s.station,
        team: s.team,
        cfg: s.team.map(|t| {
          let team_net = self.team_ip(t).unwrap();
          dhcp::DHCPConfig {
            router: team_net,
            dhcp: self.dhcp_range(team_net).unwrap()
          }
        })
      }
    }).collect();

    dhcp::configure_dhcp(admin_cfg, &station_dhcps[..]).await?;
    Ok(())
  }

  async fn configure_firewall(&self, stations: &[AllianceStation]) -> super::NetworkResult<()> {
    let admin_cfg = firewall::FirewallConfig {
      iface: self.admin_iface.clone(),
      router: Some(self.v4_network(ADMIN_ROUTER)?),
      server: Some(self.v4_network(ADMIN_IP)?)
    };

    let station_cfgs: Vec<firewall::TeamFirewallConfig> = stations.iter().map(|s| {
      firewall::TeamFirewallConfig {
        station: s.station,
        team: s.team,
        cfg: firewall::FirewallConfig {
          iface: self.station_ifaces[&s.station].clone(),
          router: s.team.map(|t| self.team_ip(t).unwrap()),
          server: None
        }
      }
    }).collect();

    firewall::configure_firewall(self.wan_iface.clone(), admin_cfg, &station_cfgs[..]).await?;

    Ok(())
  }

  fn dhcp_range(&self, network: Ipv4Network) -> super::NetworkResult<(Ipv4Addr, Ipv4Addr)> {
    Ok((network.nth(100).unwrap(), network.nth(150).unwrap()))
  }

  fn team_ip(&self, team: u16) -> super::NetworkResult<Ipv4Network> {
    Ok(format!("10.{}.{}.4/24", team / 100, team % 100).parse()?)
  }

  fn v4_network(&self, ip_str: &str) -> super::NetworkResult<Ipv4Network> {
    Ok(ip_str.parse()?)
  }
}

#[async_trait::async_trait]
impl NetworkProvider for OnboardNetwork {
  async fn configure(&self, stations: &[AllianceStation], _force_reload: bool) -> super::NetworkResult<()> {
    self.configure_ip_addrs(stations).await?;
    let fut_dhcp = self.configure_dhcp(stations);
    let fut_firewall = self.configure_firewall(stations);

    try_join!(fut_dhcp, fut_firewall)?;
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
