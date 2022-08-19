use jms_util::net::{LinkMetadata, self};
use tokio::{net::TcpStream, io::AsyncWriteExt};
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, LinesCodec};

const IMG_IP: &'static str = "192.168.1.50/24";
const ROUTER_IP: &'static str = "192.168.1.1";

pub async fn image(iface: LinkMetadata, team: u16, key: String) -> anyhow::Result<()> {
  let h = net::handle()?;
  // Set IP to our imaging address
  net::configure_addresses(&h, &iface.name, vec![IMG_IP.parse()?]).await?;

  let result = write_img(team, key).await;

  // Try to reset IP to what it was before
  let _ = net::configure_addresses(&h, &iface.name, iface.addrs.clone()).await;

  result
}

async fn write_img(team: u16, key: String) -> anyhow::Result<()> {
  let msg = format!("B5,{},{},{},N,Y,Y,0,0,,\n\n", team, team, key);

  let mut stream = TcpStream::connect((ROUTER_IP, 8888)).await?;
  stream.write_all(msg.as_bytes()).await?;

  let mut framed = Framed::new(stream, LinesCodec::new());

  let _vers = framed.next().await.ok_or(anyhow::anyhow!("Unexpected Early Return"))??;
  let _opts = framed.next().await.ok_or(anyhow::anyhow!("Unexpected Early Return"))??;
  let conf_text = framed.next().await.ok_or(anyhow::anyhow!("Unexpected Early Return"))??;
  if !conf_text.contains("[CONF] Configuring Radio...") {
    anyhow::bail!("Third line should be [CONF], found {}", conf_text);
  }

  let eot = framed.next().await;
  if let Some(msg) = eot {
    anyhow::bail!("Expected end of stream. Found {}", msg?);
  }

  Ok(())
}