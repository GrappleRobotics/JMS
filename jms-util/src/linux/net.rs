use std::net::Ipv4Addr;

use futures::{future::try_join_all, TryStreamExt};
use ipnetwork::Ipv4Network;

use crate::net::LinkMetadata;

pub type Handle = rtnetlink::Handle;
pub type ExtraData = ();

pub fn handle() -> std::io::Result<Handle> {
  match rtnetlink::new_connection() {
    Ok((conn, handle, _)) => {
      tokio::spawn(conn);
      Ok(handle)
    }
    Err(e) => Err(e),
  }
}

pub async fn get_all_ifaces(handle: &Handle) -> Result<Vec<LinkMetadata>, rtnetlink::Error> {
  use rtnetlink::packet::link::nlas::{self, Nla};
  
  let mut links = handle.link().get().execute();
  let mut ifaces = vec![];
  while let Some(link) = links.try_next().await? {
    let mut name = None;
    let mut vlan = None;
    let mut addrs = vec![];

    for nla in link.nlas {
      match nla {
        Nla::IfName(n) => name = Some(n),
        Nla::Info(info) => {
          for i in info {
            match i {
              nlas::Info::Data(nlas::InfoData::Vlan(vlans)) => {
                for vl in vlans {
                  match vl {
                    nlas::InfoVlan::Id(id) => vlan = Some(id),
                    _ => (),
                  }
                }
              }
              _ => (),
            }
          }
        }
        _ => (),
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
              if let Ok(net) = Ipv4Network::new( Ipv4Addr::new(a[0], a[1], a[2], a[3]), addr.header.prefix_len ) {
                addrs.push(net);
              }
            }
          }
          _ => (),
        }
      }
    }

    if let Some(link_name) = name {
      ifaces.push(LinkMetadata {
        name: link_name,
        vlan,
        addrs,
        extra: ()
      });
    }
  }
  Ok(ifaces)
}

// TODO: Only configure what's changing.
pub async fn configure_addresses<I>(handle: &Handle, iface: &str, addresses: I) -> Result<(), rtnetlink::Error>
where
  I: IntoIterator<Item = Ipv4Network>,
{
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