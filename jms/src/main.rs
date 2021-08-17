mod arena;
mod config;
mod db;
mod ds;
mod logging;
mod network;
mod reports;
mod schedule;
mod scoring;
mod ui;
mod utils;
mod electronics;
mod models;
mod schema;
mod tba;

#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate diesel;
extern crate strum;
#[macro_use]
extern crate strum_macros;
#[macro_use]
extern crate rocket;

use std::{sync::Arc, time::Duration};

use arena::SharedArena;
use clap::{App, Arg};
use dotenv::dotenv;
use ds::connector::DSConnectionService;
use futures::TryFutureExt;
use tokio::{sync::Mutex, try_join};

use ui::websocket::ArenaWebsocketHandler;
use ui::websocket::Websockets;

use crate::config::JMSSettings;
use crate::ui::websocket::EventWebsocketHandler;
use crate::ui::websocket::MatchWebsocketHandler;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  dotenv().ok();

  let matches = App::new("JMS")
    .about("An Alternative Field-Management-System for FRC Offseason Events.")
    .arg(Arg::with_name("debug").short("d").help("Enable debug logging."))
    .arg(
      Arg::with_name("new-cfg")
        .long("new-cfg")
        .help("Forcefully set a new configuration"),
    )
    .arg(
      Arg::with_name("cfg-only")
        .long("cfg-only")
        .help("Only load the configuration, don't start JMS"),
    )
    .get_matches();

  logging::configure(matches.is_present("debug"));

  db::connection(); // Start connection

  let settings = JMSSettings::load_or_create_config(matches.is_present("new-cfg")).await?;
  if !matches.is_present("cfg-only") {
    let network = settings.network.create()?;

    let arena: SharedArena = Arc::new(Mutex::new(arena::Arena::new(3, network)));

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
    let ds_fut = ds_service.run().map_err(|e| anyhow::anyhow!("DS Error: {}", e));

    let mut ws = Websockets::new(Duration::from_millis(500));
    ws.register("arena", Box::new(ArenaWebsocketHandler::new(arena))).await;
    ws.register("event", Box::new(EventWebsocketHandler::new())).await;
    ws.register("matches", Box::new(MatchWebsocketHandler::new())).await;
    let ws_fut = ws.begin();

    let web_fut = ui::web::begin();

    try_join!(arena_fut, ds_fut, ws_fut, web_fut)?;
  }

  Ok(())
}
