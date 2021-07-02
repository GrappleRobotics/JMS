mod arena;
mod db;
mod logging;
mod network;
mod utils;
mod ds;
mod ui;

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

use ui::websocket::{self, ArenaWebsocketHandler};
use ui::websocket::Websockets;

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

  // let network = Box::new(FakeNetwork {});

  // let arena: SharedArena = Arc::new(Mutex::new(arena::Arena::new(3, Some(network))));
  // {
  //   let mut a = arena.lock().unwrap();
  //   a.load_match(Match::new());
  //   a.stations[1].team = Some(4788);
  //   a.stations[2].team = Some(100);
  //   a.update();
  // }

  // let mut ds_service = DSConnectionService::new(arena).await;
  // ds_service.run().await?;

    let mut ws = Websockets::new();
    ws.register("arena", Box::new(ArenaWebsocketHandler {})).await;
    ws.begin().await?;

  // websocket::begin().await?;

  Ok(())
}