use jms_util::net::{get_all_ifaces, handle, configure_addresses};

#[tokio::test]
async fn test_networking() -> anyhow::Result<()> {
  let h = handle()?;

  // Print interfaces
  let ifaces = get_all_ifaces(&h).await?;
  println!("Interfaces:");
  for iface in ifaces.iter() {
    println!("{}", iface);
  }

  // Try to configure wireless adapter
  configure_addresses(&(), &ifaces[2], vec!["192.168.1.50/24".parse()?]).await?;

  // Print interfaces
  let ifaces = get_all_ifaces(&h).await?;
  println!("Interfaces:");
  for iface in ifaces.iter() {
    println!("{}", iface);
  }

  Ok(())
}