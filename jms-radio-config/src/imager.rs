use std::time::Duration;

use jms_util::net::{LinkMetadata, self};
use tokio::{net::TcpStream, io::{AsyncWriteExt, AsyncReadExt}};
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, LinesCodec};

const IMG_IP: &'static str = "192.168.1.50/24";
const ROUTER_IP: &'static str = "192.168.1.1";

#[derive(Debug, Clone)]
pub struct ImagingProps {
  pub team: u16,
  pub ssid: String,
  pub key: String,
  pub home: bool
}

pub async fn image(iface: LinkMetadata, props: ImagingProps) -> anyhow::Result<()> {
  let h = net::handle()?;
  // Set IP to our imaging address
  net::configure_addresses(&h, &iface.name, vec![IMG_IP.parse()?]).await?;

  // Just to make sure the network's up and ready
  tokio::time::sleep(Duration::from_millis(5000)).await;

  let result = write_img(props).await;

  result
}

async fn write_img(props: ImagingProps) -> anyhow::Result<()> {
  let mode = if props.home { "AP24" } else { "B5" };
  let msg = format!("{},{},{},{},N,N,Y,0,0,,\n\n", mode, props.team, props.ssid, props.key);

  let mut stream = TcpStream::connect((ROUTER_IP, 8888)).await?;
  stream.write_all(msg.as_bytes()).await?;

  let mut framed = Framed::new(stream, LinesCodec::new());

  let _vers = framed.next().await.ok_or(anyhow::anyhow!("Unexpected Early Return"))??;
  let _opts = framed.next().await.ok_or(anyhow::anyhow!("Unexpected Early Return"))??;
  let conf_text = framed.next().await.ok_or(anyhow::anyhow!("Unexpected Early Return"))??;
  if !conf_text.contains("[CONF] Configuring Radio...") {
    anyhow::bail!("Third line should be [CONF], found {}", conf_text);
  }

  let mut data: Vec<u8> = Vec::new();
  match framed.into_inner().read_to_end(&mut data).await {
    Ok(len) if len > 0 => anyhow::bail!("Extra data was found: {:?}", data),
    Err(e) if e.raw_os_error() == Some(10054) => (),    // End of Stream error on windows - this is normal
    Err(e) => anyhow::bail!("Error: {}", e),
    _ => () // We expect 0 data
  };

  Ok(())
}