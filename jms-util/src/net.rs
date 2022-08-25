use ipnetwork::Ipv4Network;

#[derive(Debug, Clone)]
pub struct LinkMetadata {
  pub name: String,
  pub vlan: Option<u16>,
  pub addrs: Vec<Ipv4Network>,
  #[allow(dead_code)]
  pub(crate) extra: crate::platform::net::ExtraData
}

impl std::fmt::Display for LinkMetadata {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.name)?;
    if let Some(vlan) = self.vlan {
      write!(f, " VLAN {}", vlan)?;
    }
    if self.addrs.len() > 0 {
      write!(f, " ({})", self.addrs.iter().map(|a| a.to_string()).collect::<Vec<String>>().join(", "))?;
    }
    Ok(())
  }
}

pub use crate::platform::net::*;