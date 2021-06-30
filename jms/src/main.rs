mod arena;
mod db;
mod logging;
mod network;
mod utils;
mod ds;

mod models;
mod schema;

#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate diesel;
extern crate strum;
#[macro_use]
extern crate strum_macros;

use std::{error::Error, sync::{Arc, Mutex}, thread, time::Duration};

use arena::SharedArena;
use clap::{App, Arg};
use dotenv::dotenv;
use ds::connector::DSConnectionService;
use log::info;
use network::NetworkProvider;
use tokio::{net::TcpListener, try_join};

use crate::{arena::{matches::Match, ArenaSignal, ArenaState}, ds::connector::DSConnection};

struct FakeNetwork {}
impl NetworkProvider for FakeNetwork {
  fn configure_admin(&mut self) -> network::NetworkResult<()> {
    info!("Configuring Admin");
    Ok(())
  }

  fn configure_alliances(
    &mut self,
    stations: &mut dyn Iterator<Item = &arena::AllianceStation>,
    force_reload: bool,
  ) -> network::NetworkResult<()> {
    let alls: Vec<&arena::AllianceStation> = stations.collect();
    info!("Configuring Alliances (Force? {}): {:?}", force_reload, alls);
    thread::sleep(Duration::from_millis(1000));
    Ok(())
  }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  dotenv().ok();

  let matches = App::new("JMS")
    .about("An Alternative Field-Management-System for FRC Offseason Events.")
    .arg(Arg::with_name("debug").short("d").help("Enable debug logging."))
    .get_matches();

  logging::configure(matches.is_present("debug"));

  db::connection(); // Start connection

  let network = Box::new(FakeNetwork {});

  // let mut arena = arena::Arena::new(3, Some(network));
  // let mut arena = arena::SharedArena::new(Mutex::new(3, Some(network)));
  let arena: SharedArena = Arc::new(Mutex::new(arena::Arena::new(3, Some(network))));
  {
    let mut a = arena.lock().unwrap();
    a.load_match(Match::new());
    a.stations[1].team = Some(4788);
    a.stations[2].team = Some(100);
    a.update();
  }

  let mut ds_service = DSConnectionService::new(arena).await;
  ds_service.run().await?;

  // let fut_tcp = tcp(arena);
  // try_join!(fut_tcp)?;

  Ok(())
  // assert_eq!(arena.current_state(), ArenaState::Idle);
  // arena.signal(ArenaSignal::Prestart(false));
  // arena.update();
  // assert_eq!(arena.current_state(), ArenaState::Prestart(false, false));
  // let mut s = "".to_owned();
  // while let ArenaState::Prestart(false, _) = arena.current_state() {
  //   arena.update();
  //   s = s + ".";
  //   thread::sleep(Duration::from_millis(10));
  // }
  // assert_eq!(arena.current_state(), ArenaState::Prestart(true, false));
  // arena.update();
  // arena.signal(ArenaSignal::MatchArm);
  // arena.update();
  // assert_eq!(arena.current_state(), ArenaState::MatchArmed);
  // arena.signal(ArenaSignal::MatchPlay);
  // arena.update();
  // assert_eq!(arena.current_state(), ArenaState::MatchPlay);
  // while let ArenaState::MatchPlay = arena.current_state() {
  //   arena.update();
  //   thread::sleep(Duration::from_millis(10));
  // }
  // assert_eq!(arena.current_state(), ArenaState::MatchComplete);
}

// async fn tcp(arena: SharedArena) -> Result<(), Box<dyn Error>> {
//   let server = TcpListener::bind("0.0.0.0:1750").await?;
//   loop {
//     info!("Listening...");
//     let (stream, addr) = server.accept().await?;
//     info!("Connected: {}", addr);
//     let this_arena = arena.clone();
//     tokio::spawn(async move {
//       let mut conn = DSConnection::new(this_arena, addr, stream);
//       conn.process_tcp().await;
//     });
//   }
// }

// async fn udp()