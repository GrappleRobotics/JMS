use std::{sync::Arc, time::Duration, path::Path, fs};

use clap::{App, Arg};
use dotenv::dotenv;
use futures::TryFutureExt;
use jms::{arena::{self, SharedArena}, config::JMSSettings, db, ds::connector::DSConnectionService, electronics::comms::FieldElectronicsService, logging, tba, ui::{self, websocket::{Websockets, WebsocketMessage2UI, WebsocketMessage2JMS, WebsocketParams}}, schedule::{worker::{MatchGenerators, MatchGenerationWorker}, quals::QualsMatchGenerator, playoffs::PlayoffMatchGenerator}};
use log::info;
use tokio::{sync::Mutex, try_join};

#[derive(schemars::JsonSchema)]
struct AllWebsocketMessages {
  #[allow(dead_code)]
  jms2ui: WebsocketMessage2UI,
  #[allow(dead_code)]
  ui2jms: WebsocketMessage2JMS
}

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
    .arg(
      Arg::with_name("gen-schema")
        .long("gen-schema")
        .value_name("SCHEMA_FILE")
        .takes_value(true)
        .help("Only generate the JSON schema for websocket communication")
    )
    .get_matches();

  logging::configure(matches.is_present("debug"));

  let settings = JMSSettings::load_or_create_config(matches.is_present("new-cfg")).await?;

  if let Some(v) = matches.value_of("gen-schema") {
    let file = Path::new(v);
    let schema = schemars::schema_for!(AllWebsocketMessages);

    fs::write(file, serde_json::to_string_pretty(&schema)?)?;
  } else if !matches.is_present("cfg-only") {
    db::database(); // Start connection

    let network = settings.network.create()?;

    let arena: SharedArena = Arc::new(Mutex::new(arena::Arena::new(3, network)));
    let match_workers = Arc::new(Mutex::new(MatchGenerators { 
      quals: MatchGenerationWorker::new(QualsMatchGenerator::new()), 
      playoffs: MatchGenerationWorker::new(PlayoffMatchGenerator::new()) 
    }));

    let a2 = arena.clone();
    let arena_fut = async move {
      let mut interval = tokio::time::interval(Duration::from_millis(50));
      loop {
        interval.tick().await;
        a2.lock().await.update().await;
      }
      #[allow(unreachable_code)]
      Ok(())
    };

    let mut ds_service = DSConnectionService::new(arena.clone()).await;
    let ds_fut = ds_service.run().map_err(|e| anyhow::anyhow!("DS Error: {}", e));

    let electronics_service = FieldElectronicsService::new(arena.clone(), 5333).await;
    let electronics_fut = electronics_service.begin();

    let ws_params = WebsocketParams {
      arena: arena.clone(),
      matches: match_workers.clone()
    };

    let ws = Websockets::new(ws_params, Duration::from_millis(500));
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