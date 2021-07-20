mod arena;
mod db;
mod ds;
mod logging;
mod network;
mod ui;
mod utils;
mod schedule;

mod models;
mod schema;

#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate diesel;
extern crate strum;
#[macro_use]
extern crate strum_macros;

use std::{error::Error, sync::Arc, time::Duration};

use arena::SharedArena;
use clap::{App, Arg};
use dotenv::dotenv;
use ds::connector::DSConnectionService;
use futures::TryFutureExt;
use network::onboard::OnboardNetwork;
use network::NetworkProvider;
use tokio::{sync::Mutex, try_join};

use ui::websocket::ArenaWebsocketHandler;
use ui::websocket::Websockets;

use crate::ui::websocket::EventWebsocketHandler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  dotenv().ok();

  let matches = App::new("JMS")
    .about("An Alternative Field-Management-System for FRC Offseason Events.")
    .arg(Arg::with_name("debug").short("d").help("Enable debug logging."))
    .get_matches();

  logging::configure(matches.is_present("debug"));

  db::connection(); // Start connection

  let network = Box::new(
    OnboardNetwork::new(
      "ens18",
      "ens19.100",
      &vec!["ens19.10", "ens19.20", "ens19.30"],
      &vec!["ens19.40", "ens19.50", "ens19.60"],
    )
    .unwrap(),
  );

  network.configure(&vec![], false).await.unwrap();

  let arena: SharedArena = Arc::new(Mutex::new(arena::Arena::new(3, Some(network))));

  // Arena gets its own thread to keep timing strict
  let a2 = arena.clone();
  let arena_fut = async move {
    let mut interval = tokio::time::interval(Duration::from_millis(50));
    loop {
      interval.tick().await;
      a2.lock().await.update().await;
    }
    Ok(())
  };

  let mut ds_service = DSConnectionService::new(arena.clone()).await;
  let ds_fut = ds_service.run();

  let mut ws = Websockets::new(Duration::from_millis(500));
  ws.register("arena", Box::new(ArenaWebsocketHandler::new(arena))).await;
  ws.register("event", Box::new(EventWebsocketHandler::new())).await;
  let ws_fut = ws.begin().map_err(|e| Box::new(e));

  try_join!(arena_fut, ds_fut, ws_fut)?;

  Ok(())
}
