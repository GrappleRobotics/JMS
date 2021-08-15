use std::{io, net::Ipv4Addr};

use futures::{TryStreamExt, future::try_join_all};
use ipnetwork::Ipv4Network;
use rtnetlink::{Error, Handle, new_connection, packet::link::nlas::{self, Nla}};

use crate::{log_expect, utils::danger::danger_or_err};

// TODO: Only configure what's changing.
pub async fn configure_addresses<I>(handle: &Handle, iface: &str, addresses: I) -> Result<(), Error>
where
  I: IntoIterator<Item = Ipv4Network>,
{
  log_expect!(danger_or_err());

  let mut links = handle.link().get().set_name_filter(iface.to_string()).execute();
  if let Some(link) = links.try_next().await? {
    // Flush addresses
    let mut addrs = handle
      .address()
      .get()
      .set_link_index_filter(link.header.index)
      .execute();

    let mut futs = Vec::new();
    while let Some(addr) = addrs.try_next().await? {
      futs.push(handle.address().del(addr).execute());
    }
    try_join_all(futs).await?;

    // Add new addresses
    let mut futs = Vec::new();
    for addr in addresses {
      futs.push(
        handle
          .address()
          .add(link.header.index, addr.ip().into(), addr.prefix())
          .execute(),
      );
    }
    try_join_all(futs).await?;
  }
  Ok(())
}

pub struct LinkMetadata {
  pub name: String,
  pub vlan: Option<u16>,
  pub addrs: Vec<String>
}

impl std::fmt::Display for LinkMetadata {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.name)?;
    if let Some(vlan) = self.vlan {
      write!(f, " VLAN {}", vlan)?;
    }
    if self.addrs.len() > 0 {
      write!(f, " ({})", self.addrs.join(", "))?;
    }
    Ok(())
  }
}

pub async fn get_all_ifaces(handle: &Handle) -> Result<Vec<LinkMetadata>, Error> {
  let mut links = handle.link().get().execute();
  let mut ifaces = vec![];
  while let Some(link) = links.try_next().await? {
    let mut name = None;
    let mut vlan = None;
    let mut addrs = vec![];

    for nla in link.nlas {
      match nla {
        Nla::IfName(n) => name = Some(n),
        Nla::Info(info) => for i in info {
          match i {
            nlas::Info::Data(nlas::InfoData::Vlan(vlans)) => for vl in vlans {
              match vl {
                nlas::InfoVlan::Id(id) => vlan = Some(id),
                _ => ()
              }
            },
            _ => ()
          }
        },
        _ => ()
      }
    }

    let mut link_addrs = handle
      .address()
      .get()
      .set_link_index_filter(link.header.index)
      .execute();
    
    while let Some(addr) = link_addrs.try_next().await? {
      for nla in addr.nlas {
        match nla {
          rtnetlink::packet::address::Nla::Address(a) => {
            if a.len() == 4 {
              addrs.push(Ipv4Addr::new( a[0], a[1], a[2], a[3] ).to_string())
            }
          },
          _ => ()
        }
      }
    }

    if let Some(link_name) = name {
      ifaces.push(LinkMetadata { name: link_name, vlan, addrs });
    }
  }
  Ok(ifaces)
}

pub fn handle() -> io::Result<Handle> {
  match new_connection() {
    Ok((conn, handle, _)) => {
      tokio::spawn(conn);
      Ok(handle)
    }
    Err(e) => Err(e),
  }
}
