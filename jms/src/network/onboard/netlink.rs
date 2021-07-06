use std::io;

use futures::{future::try_join_all, TryStreamExt};
use ipnetwork::IpNetwork;
use rtnetlink::{new_connection, Error, Handle};

use crate::{log_expect, utils::danger::danger_or_err};

// TODO: Only configure what's changing.
pub async fn configure_addresses<I>(handle: &Handle, iface: &str, addresses: I) -> Result<(), Error>
where
  I: IntoIterator<Item = IpNetwork>,
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
          .add(link.header.index, addr.ip(), addr.prefix())
          .execute(),
      );
    }
    try_join_all(futs).await?;
  }
  Ok(())
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
