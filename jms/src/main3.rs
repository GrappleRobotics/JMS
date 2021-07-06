use std::{error::Error, net::SocketAddr, sync::{Arc, Mutex}, thread, time::Duration};

use chrono::{Local, Utc};
use ds::{DSUDPCodec, Fms2DsUDP};
use futures::{SinkExt, StreamExt};
use tokio::{join, net::{TcpListener, UdpSocket}, try_join};
use tokio_util::{codec::Framed, udp::UdpFramed};

use crate::{arena::station::AllianceStationId, ds::{DSTCPCodec, Ds2FmsTCP, Ds2FmsTCPTags, Fms2DsStationStatus, Fms2DsTCP, Fms2DsTCPTags, connector::DSConnection}};

mod ds;
mod utils;

extern crate strum;
#[macro_use]
extern crate strum_macros;

mod arena;
mod logging;
mod network;
mod models;
mod db;
mod schema;

#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate diesel;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  // let server = UdpSocket::bind("0.0.0.0:1160").await?;
  // let mut incoming = UdpFramed::new(server, DSUDPCodec::new());
  // println!("Listening for clients...");

  // while let Some(Ok((msg, addr))) = incoming.next().await {
  //   println!("Received from {}: {:#?}", addr, msg);
  // }

  let dses = Arc::new(Mutex::new(vec![] as Vec<SocketAddr>));
  let fut_tcp = tcp(dses.clone());
  let fut_udp = udp(dses.clone());

  try_join!(fut_tcp, fut_udp)?;
  // try_join!(fut_tcp)?;

  Ok(())
}

async fn tcp(dses: Arc<Mutex<Vec<SocketAddr>>>) -> Result<(), Box<dyn Error>> {
  let server = TcpListener::bind("0.0.0.0:1750").await?;
  loop {
    println!("Listening...");
    let (stream, addr) = server.accept().await?;
    println!("Connected: {}", addr);
    dses.lock().unwrap().push(addr);
    
    tokio::spawn(async move {
      // let mut trans = Framed::new(stream, DSTCPCodec::new());

      // while let Some(req) = trans.next().await {
      //   match req {
      //     Ok(req) => {
      //       println!("TCP({}): {:?}", addr, req);
      //       let mut response = Fms2DsTCP { tags: vec![] };
      //       response.tags.push(Fms2DsTCPTags::StationInfo(
      //         AllianceStationId { alliance: arena::station::Alliance::Red, station: 3 },
      //         Fms2DsStationStatus::Good
      //       ));
      //       // response.tags.push(Fms2DsTCPTags::EventCode("2021auwarp".to_owned()));
      //       // response.tags.push(Fms2DsTCPTags::GameData("CAB".to_owned()));

      //       // req.tags.iter().for_each(|x| {
      //       //   match x {
      //       //     Ds2FmsTCPTags::LogData(dat) => {
      //       //       println!("LogData => A: {} T: {} D: {} A: {} D: {}", dat.rtt, dat.ds_teleop, dat.ds_disable, dat.robot_auto, dat.robot_disable);
      //       //     }
      //       //     x => println!("{:?}", x)
      //       //   }
      //       // });
      //       trans.send(response).await.unwrap();
      //     },
      //     Err(e) => println!("Error: {}", e)
      //   }
      // }

      // DSConnection::new()
    });
  }
}

async fn udp(dses: Arc<Mutex<Vec<SocketAddr>>>) -> Result<(), Box<dyn Error>> {
  let mut i = 0;
  let sock = UdpSocket::bind("0.0.0.0:1160").await?;
  let mut framed = UdpFramed::new(sock, DSUDPCodec::new());
  let time = Duration::from_millis(500);
  loop {
    while let Ok(Some(result)) = tokio::time::timeout(time,  framed.next()).await {
      match result {
        Ok((req, _)) => {
          // println!("{:?}", req);
        },
        Err(e) => println!("Error: {}", e)
      }
    }

    for ds in dses.lock().unwrap().iter() {
      let mut addr = *ds;
      addr.set_port(1121);

      let msg = Fms2DsUDP {
        estop: false,
        enabled: (i > 5),
        mode: ds::DSMode::Auto,
        station: AllianceStationId { alliance: arena::station::Alliance::Red, station: 3 },
        tournament_level: ds::TournamentLevel::Qualification,
        match_number: 42,
        play_number: 1,
        time: Local::now(),
        remaining_seconds: 15,
      };

      // println!("UDP {:?} -> {:?}", addr, msg);

      i += 1;
      framed.send((msg, addr)).await?;
    }

    // tokio::time::sleep(time).await;
  }
}