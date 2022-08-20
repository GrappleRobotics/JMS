use std::net::Ipv4Addr;

use crate::net::LinkMetadata;

use ipnetwork::Ipv4Network;
use windows::{Win32::{NetworkManagement::IpHelper::{GetAdaptersInfo, IP_ADAPTER_INFO, DeleteIPAddress, AddIPAddress}, Foundation::{ERROR_SUCCESS, CHAR, WIN32_ERROR, ERROR_INVALID_PARAMETER, ERROR_ACCESS_DENIED, ERROR_NOT_SUPPORTED}, Networking::WinSock::inet_addr}, core::PCSTR};

pub type Handle = ();

#[derive(Debug, Clone)]
pub struct ExtraData(IP_ADAPTER_INFO);

// Don't ask
unsafe impl Send for ExtraData {}
unsafe impl Sync for ExtraData {}

pub fn handle() -> std::io::Result<Handle> {
  Ok(())
}

fn to_str(chars: &[CHAR]) -> String {
  unsafe { PCSTR::from_raw(chars.as_ptr().cast()).to_string().unwrap() }
}

fn to_pcstr(str: String) -> PCSTR {
  let b = str.as_bytes();
  let raw = &[ &b[..], &[0] ].concat();
  PCSTR::from_raw(raw.as_ptr())
}

pub async fn get_all_ifaces(_: &Handle) -> Result<Vec<LinkMetadata>, anyhow::Error> {
  let mut ifaces = vec![];
  
  unsafe {
    let layout = std::alloc::Layout::array::<IP_ADAPTER_INFO>(16)?;
    let mut size: u32 = layout.size() as u32;
    let info: *mut IP_ADAPTER_INFO = std::alloc::alloc(layout).cast();
    // Box it so we don't have to dealloc manually
    let mut info = Box::from_raw(info);

    let result = GetAdaptersInfo(info.as_mut(), &mut size);

    if result == ERROR_SUCCESS.0 {
      let mut i: *mut IP_ADAPTER_INFO = info.as_mut();
      while !i.is_null() {
        let mut md = LinkMetadata {
          name: to_str(&(*i).Description),
          vlan: None,
          addrs: vec![],
          extra: ExtraData((*i).clone())
        };

        let mut ip = (*i).IpAddressList;
        loop {
          let network = Ipv4Network::with_netmask(to_str(&ip.IpAddress.String).parse()?, to_str(&ip.IpMask.String).parse()?)?;
          if !network.ip().is_unspecified() {
            md.addrs.push(network);
          }
          if ip.Next.is_null() { break; }
          ip = *ip.Next;
        }

        ifaces.push(md);

        i = (*i).Next;
      }
    } else {
      anyhow::bail!("Win32 GetAdaptersInfo Error: {}", result);
    }
  };
  
  Ok(ifaces)
}

pub async fn configure_addresses<I>(handle: &Handle, iface: &str, addresses: I) -> Result<(), anyhow::Error>
where
  I: IntoIterator<Item = Ipv4Network>,
{
  // Fetch again since the addresses may have changed
  let all_ifaces = get_all_ifaces(handle).await?;
  let iface = all_ifaces.iter().find(|i| i.name == iface).ok_or(anyhow::anyhow!("Link does not exist"))?;
 
  // Delete current addresses
  unsafe {
    let mut ip = iface.extra.0.IpAddressList;
    loop {
      let ipaddr: Ipv4Addr = to_str(&ip.IpAddress.String).parse()?;
      if !ipaddr.is_unspecified() {
        let result = DeleteIPAddress(ip.Context);
        if result != ERROR_SUCCESS.0 {
          anyhow::bail!("Could not remove address: {}", result)
        }
      }
      if ip.Next.is_null() { break; }
      ip = *ip.Next;
    }
  }

  // Add new addresses
  unsafe {
    for addr in addresses {
      let mut context = 0;
      let mut instance = 0;

      let result = WIN32_ERROR(AddIPAddress(
        inet_addr( to_pcstr(addr.ip().to_string())),
        inet_addr(to_pcstr(addr.mask().to_string())),
        iface.extra.0.Index,
        &mut context,
        &mut instance
      ));

      match result {
        ERROR_SUCCESS => (),
        ERROR_ACCESS_DENIED => anyhow::bail!("Could not add address: Access denied"),
        ERROR_INVALID_PARAMETER => anyhow::bail!("Could not add address: Invalid Parameter"),
        ERROR_NOT_SUPPORTED => anyhow::bail!("Could not add address: Not Supported"),
        e => anyhow::bail!("Could not add address: {:?}", e)
      }
    }
  }
  Ok(())
}