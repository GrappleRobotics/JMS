use anyhow::Result;
use jms_util::WPAKeys;
use tokio::{net::{TcpListener, TcpStream}, io::AsyncWriteExt};

use crate::{models, db::{self, TableType}};

pub struct ImagingKeyService {}

impl ImagingKeyService {
  pub fn new() -> Self {
    Self { }
  }

  pub async fn run(&mut self) -> Result<()> {
    let server = TcpListener::bind(("0.0.0.0", 6789)).await?;
    // let mut rx = self.bcast.subscribe();
    loop {
      let (stream, addr) = server.accept().await?;
      warn!("Imaging Client Connected: {}", addr);
      tokio::spawn(async move {
        match handle_client(stream).await {
          Err(e) => error!("Imaging Client Error: {}", e),
          Ok(()) => ()
        };
      });
      warn!("Imaging Client Disconnected: {}", addr);
    }
  }
}

async fn handle_client(mut stream: TcpStream) -> Result<()> {
  send_keys(&mut stream).await?;
  let mut team_watch = models::Team::table(&db::database())?.watch_all();
  loop {
    let _ = team_watch.get().await?;
    send_keys(&mut stream).await?;
  }
}

pub async fn send_keys(stream: &mut TcpStream) -> Result<()> {
  let keys: WPAKeys = models::Team::all(&db::database())?.iter().filter_map(|t| {
    t.wpakey.as_ref().map(|key| (t.id as u16, key.clone()))
  }).collect();

  let encoded = serde_json::to_vec(&keys)?;
  stream.write_u32(encoded.len() as u32).await?;
  stream.write_all(&encoded).await?;
  Ok(())
}