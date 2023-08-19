use std::{time::Duration, sync::{Arc, atomic::{AtomicBool, Ordering}}};

use futures::StreamExt;
use jms_base::{kv::KVConnection, logging::JMSLogger};
use jms_core_lib::{models::JmsComponent, db::Table};
use log::{info, error};
use tokio::{sync::broadcast, net::{TcpListener, UdpSocket}, try_join};
use tokio_util::udp::UdpFramed;
use udp_codec::{Ds2FmsUDP, DSUDPCodec};

use crate::connector::DSConnection;

pub mod connector;
pub mod tcp_codec;
pub mod udp_codec;

async fn tcp(kv: KVConnection, udp_tx: &broadcast::Sender<Ds2FmsUDP>, arena_ok: Arc<AtomicBool>) -> anyhow::Result<()> {
  let server = TcpListener::bind("0.0.0.0:1750").await?;
  loop {
    info!("Listening for connections...");
    let (stream, addr) = server.accept().await?;
    info!("Connected: {}", addr);

    let mut conn = DSConnection::new(kv.clone()?, addr, stream, udp_tx.subscribe(), arena_ok.clone()).await;
    tokio::spawn(async move {
      conn.process().await;
      info!(
        "TCP Connection {} disconnected with state {:?}",
        conn.addr_tcp, conn.state
      );
    });
  }
}

async fn udp_recv(udp_tx: &broadcast::Sender<Ds2FmsUDP>) -> anyhow::Result<()> {
  let socket = UdpSocket::bind("0.0.0.0:1160").await?;
  let mut framed = UdpFramed::new(socket, DSUDPCodec::new());
  loop {
    tokio::select! {
      frame = framed.next() => match frame {
        Some(result) => match result {
          Ok((_req, _addr)) => {
            // We send to all DSComms since it simplifies the team number checking, as TCP connections
            // don't communicate their team number until after connection.
            match udp_tx.send(_req) {
              Ok(_) => (),
              Err(e) => {
                error!("UDP Packets received without any DS connections: {}", e);
              },
            }
          },
          Err(e) => {
            error!("UDP Error: {}", e);
          }
        },
        None => ()
      }
    }
  }
}

async fn run_component(kv: KVConnection, mut component: JmsComponent) -> anyhow::Result<()> {
  let mut interval = tokio::time::interval(Duration::from_millis(100));
  loop {
    interval.tick().await;
    component.tick(&kv)?;
  }
}

async fn arena_ok_worker(kv: KVConnection, ok: Arc<AtomicBool>) -> anyhow::Result<()> {
  let mut interval = tokio::time::interval(Duration::from_millis(500));
  loop {
    interval.tick().await;
    ok.store(
      JmsComponent::heartbeat_ok_for("jms.arena", &kv).unwrap_or(false),
      Ordering::Relaxed
    )
  }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let _ = JMSLogger::init().await?;
  let kv = KVConnection::new()?;

  let component = JmsComponent::new("jms.driverstation", "JMS-DriverStation", "D", 500);
  component.insert(&kv)?;

  let arena_ok = Arc::new(AtomicBool::new(false));

  let (udp_tx, _) = broadcast::channel(16);

  let fut_tcp = tcp(kv.clone()?, &udp_tx, arena_ok.clone());
  let fut_udp = udp_recv(&udp_tx);
  let component_fut = run_component(kv.clone()?, component);
  let arena_ok_fut = arena_ok_worker(kv, arena_ok.clone());
  try_join!(fut_tcp, fut_udp, component_fut, arena_ok_fut)?;

  Ok(())
}
