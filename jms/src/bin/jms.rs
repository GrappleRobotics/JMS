use std::{sync::Arc, time::Duration};

use clap::{App, Arg};
use dotenv::dotenv;
use futures::TryFutureExt;
use jms::{arena::{self, SharedArena}, config::JMSSettings, db, ds::connector::DSConnectionService, electronics::comms::FieldElectronicsService, logging, tba, ui::{self, websocket::{ArenaWebsocketHandler, DebugWebsocketHandler, EventWebsocketHandler, MatchWebsocketHandler, Websockets}}};
use log::info;
use tokio::{sync::Mutex, try_join};

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

  db::database(); // Start connection

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

    let electronics_service = FieldElectronicsService::new(arena.clone(), 5333).await;
    let electronics_fut = electronics_service.begin();

    let mut ws = Websockets::new(Duration::from_millis(500));
    ws.register("arena", Box::new(ArenaWebsocketHandler::new(arena))).await;
    ws.register("event", Box::new(EventWebsocketHandler::new())).await;
    ws.register("matches", Box::new(MatchWebsocketHandler::new())).await;
    ws.register("debug", Box::new(DebugWebsocketHandler::new())).await;
    let ws_fut = ws.begin();

    let web_fut = ui::web::begin();

    if let Some(tba_client) = settings.tba {
      info!("TBA Enabled");
      let tba_worker = tba::TBAWorker::new(tba_client);
      let tba_fut = tba_worker.begin();

      try_join!(arena_fut, ds_fut, electronics_fut, ws_fut, web_fut, tba_fut)?;
    } else {
      try_join!(arena_fut, ds_fut, electronics_fut, ws_fut, web_fut)?;
    }
  }

  Ok(())
}